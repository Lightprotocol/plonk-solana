use ark_bn254::{Fq, Fq2, Fr, G1Affine, G2Affine};
use num_bigint::BigUint;
use num_traits::Num;
use serde::Deserialize;

/// snarkjs PLONK verification key (JSON format).
#[derive(Deserialize)]
pub struct VkJson {
    #[serde(rename = "nPublic")]
    pub n_public: usize,
    pub power: u32,
    pub k1: String,
    pub k2: String,
    pub w: String,
    #[serde(rename = "Qm")]
    pub qm: Vec<String>,
    #[serde(rename = "Ql")]
    pub ql: Vec<String>,
    #[serde(rename = "Qr")]
    pub qr: Vec<String>,
    #[serde(rename = "Qo")]
    pub qo: Vec<String>,
    #[serde(rename = "Qc")]
    pub qc: Vec<String>,
    #[serde(rename = "S1")]
    pub s1: Vec<String>,
    #[serde(rename = "S2")]
    pub s2: Vec<String>,
    #[serde(rename = "S3")]
    pub s3: Vec<String>,
    #[serde(rename = "X_2")]
    pub x_2: Vec<Vec<String>>,
}

/// snarkjs PLONK proof (JSON format).
#[derive(Deserialize)]
pub struct ProofJson {
    #[serde(rename = "A")]
    pub a: Vec<String>,
    #[serde(rename = "B")]
    pub b: Vec<String>,
    #[serde(rename = "C")]
    pub c: Vec<String>,
    #[serde(rename = "Z")]
    pub z: Vec<String>,
    #[serde(rename = "T1")]
    pub t1: Vec<String>,
    #[serde(rename = "T2")]
    pub t2: Vec<String>,
    #[serde(rename = "T3")]
    pub t3: Vec<String>,
    #[serde(rename = "Wxi")]
    pub wxi: Vec<String>,
    #[serde(rename = "Wxiw")]
    pub wxiw: Vec<String>,
    pub eval_a: String,
    pub eval_b: String,
    pub eval_c: String,
    pub eval_s1: String,
    pub eval_s2: String,
    pub eval_zw: String,
}

/// Parsed verification key.
pub struct VerificationKey {
    pub n_public: usize,
    pub power: u32,
    pub k1: Fr,
    pub k2: Fr,
    pub w: Fr,
    pub qm: G1Affine,
    pub ql: G1Affine,
    pub qr: G1Affine,
    pub qo: G1Affine,
    pub qc: G1Affine,
    pub s1: G1Affine,
    pub s2: G1Affine,
    pub s3: G1Affine,
    pub x_2: G2Affine,
}

/// Parsed proof.
pub struct Proof {
    pub a: G1Affine,
    pub b: G1Affine,
    pub c: G1Affine,
    pub z: G1Affine,
    pub t1: G1Affine,
    pub t2: G1Affine,
    pub t3: G1Affine,
    pub wxi: G1Affine,
    pub wxiw: G1Affine,
    pub eval_a: Fr,
    pub eval_b: Fr,
    pub eval_c: Fr,
    pub eval_s1: Fr,
    pub eval_s2: Fr,
    pub eval_zw: Fr,
}

fn str_to_fr(s: &str) -> Fr {
    let n = BigUint::from_str_radix(s, 10).expect("invalid decimal string");
    Fr::from(n)
}

fn str_to_fq(s: &str) -> Fq {
    let n = BigUint::from_str_radix(s, 10).expect("invalid decimal string");
    Fq::from(n)
}

/// Parse a G1 point from snarkjs JSON format [x, y, z] (projective).
/// snarkjs always outputs z=1 for valid points.
fn parse_g1(coords: &[String]) -> G1Affine {
    assert_eq!(coords.len(), 3, "G1 point must have 3 coordinates");
    let x = str_to_fq(&coords[0]);
    let y = str_to_fq(&coords[1]);
    let z = str_to_fq(&coords[2]);
    if z == Fq::from(0u64) {
        return G1Affine::identity();
    }
    assert_eq!(z, Fq::from(1u64), "expected z=1 for affine point");
    G1Affine::new_unchecked(x, y)
}

/// Parse a G2 point from snarkjs JSON format [[x0,x1],[y0,y1],[z0,z1]].
fn parse_g2(coords: &[Vec<String>]) -> G2Affine {
    assert_eq!(coords.len(), 3, "G2 point must have 3 coordinate pairs");
    let x = Fq2::new(str_to_fq(&coords[0][0]), str_to_fq(&coords[0][1]));
    let y = Fq2::new(str_to_fq(&coords[1][0]), str_to_fq(&coords[1][1]));
    let z0 = str_to_fq(&coords[2][0]);
    let z1 = str_to_fq(&coords[2][1]);
    if z0 == Fq::from(0u64) && z1 == Fq::from(0u64) {
        return G2Affine::identity();
    }
    assert!(
        z0 == Fq::from(1u64) && z1 == Fq::from(0u64),
        "expected z=[1,0] for affine G2 point"
    );
    G2Affine::new_unchecked(x, y)
}

impl VkJson {
    pub fn parse(&self) -> VerificationKey {
        VerificationKey {
            n_public: self.n_public,
            power: self.power,
            k1: str_to_fr(&self.k1),
            k2: str_to_fr(&self.k2),
            w: str_to_fr(&self.w),
            qm: parse_g1(&self.qm),
            ql: parse_g1(&self.ql),
            qr: parse_g1(&self.qr),
            qo: parse_g1(&self.qo),
            qc: parse_g1(&self.qc),
            s1: parse_g1(&self.s1),
            s2: parse_g1(&self.s2),
            s3: parse_g1(&self.s3),
            x_2: parse_g2(&self.x_2),
        }
    }
}

impl ProofJson {
    pub fn parse(&self) -> Proof {
        Proof {
            a: parse_g1(&self.a),
            b: parse_g1(&self.b),
            c: parse_g1(&self.c),
            z: parse_g1(&self.z),
            t1: parse_g1(&self.t1),
            t2: parse_g1(&self.t2),
            t3: parse_g1(&self.t3),
            wxi: parse_g1(&self.wxi),
            wxiw: parse_g1(&self.wxiw),
            eval_a: str_to_fr(&self.eval_a),
            eval_b: str_to_fr(&self.eval_b),
            eval_c: str_to_fr(&self.eval_c),
            eval_s1: str_to_fr(&self.eval_s1),
            eval_s2: str_to_fr(&self.eval_s2),
            eval_zw: str_to_fr(&self.eval_zw),
        }
    }
}

pub fn parse_public_inputs(json: &str) -> Vec<Fr> {
    let vals: Vec<String> = serde_json::from_str(json).expect("invalid public inputs JSON");
    vals.iter().map(|s| str_to_fr(s)).collect()
}
