/// PLONK verifier using solana-bn254 syscalls for curve operations.
///
/// All G1 points are 64 bytes big-endian (x || y).
/// All G2 points are 128 bytes big-endian (x1 || x0 || y1 || y0).
/// All scalars are 32 bytes big-endian.
use crate::decompression;
use crate::errors::PlonkError;
use crate::fr::Fr;
use crate::transcript::Transcript;
use solana_bn254::prelude::{
    alt_bn128_g1_addition_be, alt_bn128_g1_multiplication_be, alt_bn128_pairing_be,
};

const G1_ZERO: [u8; 64] = [0u8; 64];

const G1_GENERATOR: [u8; 64] = {
    let mut g = [0u8; 64];
    g[31] = 1;
    g[63] = 2;
    g
};

/// G2 generator in big-endian format (EIP-197 order: x1, x0, y1, y0).
const G2_GENERATOR: [u8; 128] = [
    0x19, 0x8e, 0x93, 0x93, 0x92, 0x0d, 0x48, 0x3a, 0x72, 0x60, 0xbf, 0xb7, 0x31, 0xfb, 0x5d,
    0x25, 0xf1, 0xaa, 0x49, 0x33, 0x35, 0xa9, 0xe7, 0x12, 0x97, 0xe4, 0x85, 0xb7, 0xae, 0xf3,
    0x12, 0xc2, 0x18, 0x00, 0xde, 0xef, 0x12, 0x1f, 0x1e, 0x76, 0x42, 0x6a, 0x00, 0x66, 0x5e,
    0x5c, 0x44, 0x79, 0x67, 0x43, 0x22, 0xd4, 0xf7, 0x5e, 0xda, 0xdd, 0x46, 0xde, 0xbd, 0x5c,
    0xd9, 0x92, 0xf6, 0xed, 0x09, 0x06, 0x89, 0xd0, 0x58, 0x5f, 0xf0, 0x75, 0xec, 0x9e, 0x99,
    0xad, 0x69, 0x0c, 0x33, 0x95, 0xbc, 0x4b, 0x31, 0x33, 0x70, 0xb3, 0x8e, 0xf3, 0x55, 0xac,
    0xda, 0xdc, 0xd1, 0x22, 0x97, 0x5b, 0x12, 0xc8, 0x5e, 0xa5, 0xdb, 0x8c, 0x6d, 0xeb, 0x4a,
    0xab, 0x71, 0x80, 0x8d, 0xcb, 0x40, 0x8f, 0xe3, 0xd1, 0xe7, 0x69, 0x0c, 0x43, 0xd3, 0x7b,
    0x4c, 0xe6, 0xcc, 0x01, 0x66, 0xfa, 0x7d, 0xaa,
];

/// BN254 base field modulus (Fq) in big-endian.
const FQ_MODULUS: [u8; 32] = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58,
    0x5d, 0x97, 0x81, 0x6a, 0x91, 0x68, 0x71, 0xca, 0x8d, 0x3c, 0x20, 0x8c, 0x16, 0xd8, 0x7c,
    0xfd, 0x47,
];

/// Verification key (all points in 64/128-byte big-endian format).
pub struct VerificationKey {
    pub n_public: usize,
    pub power: u32,
    pub k1: Fr,
    pub k2: Fr,
    pub w: Fr,
    pub qm: [u8; 64],
    pub ql: [u8; 64],
    pub qr: [u8; 64],
    pub qo: [u8; 64],
    pub qc: [u8; 64],
    pub s1: [u8; 64],
    pub s2: [u8; 64],
    pub s3: [u8; 64],
    pub x_2: [u8; 128],
}

/// Proof (G1 points in 64-byte big-endian uncompressed format).
pub struct Proof {
    pub a: [u8; 64],
    pub b: [u8; 64],
    pub c: [u8; 64],
    pub z: [u8; 64],
    pub t1: [u8; 64],
    pub t2: [u8; 64],
    pub t3: [u8; 64],
    pub wxi: [u8; 64],
    pub wxiw: [u8; 64],
    pub eval_a: Fr,
    pub eval_b: Fr,
    pub eval_c: Fr,
    pub eval_s1: Fr,
    pub eval_s2: Fr,
    pub eval_zw: Fr,
}

/// Compressed proof (G1 points as 32 bytes each).
/// 9 * 32 + 6 * 32 = 480 bytes vs 768 bytes uncompressed.
pub struct CompressedProof {
    pub a: [u8; 32],
    pub b: [u8; 32],
    pub c: [u8; 32],
    pub z: [u8; 32],
    pub t1: [u8; 32],
    pub t2: [u8; 32],
    pub t3: [u8; 32],
    pub wxi: [u8; 32],
    pub wxiw: [u8; 32],
    pub eval_a: Fr,
    pub eval_b: Fr,
    pub eval_c: Fr,
    pub eval_s1: Fr,
    pub eval_s2: Fr,
    pub eval_zw: Fr,
}

impl CompressedProof {
    pub fn decompress(&self) -> Result<Proof, PlonkError> {
        Ok(Proof {
            a: decompression::decompress_g1(&self.a)?,
            b: decompression::decompress_g1(&self.b)?,
            c: decompression::decompress_g1(&self.c)?,
            z: decompression::decompress_g1(&self.z)?,
            t1: decompression::decompress_g1(&self.t1)?,
            t2: decompression::decompress_g1(&self.t2)?,
            t3: decompression::decompress_g1(&self.t3)?,
            wxi: decompression::decompress_g1(&self.wxi)?,
            wxiw: decompression::decompress_g1(&self.wxiw)?,
            eval_a: self.eval_a,
            eval_b: self.eval_b,
            eval_c: self.eval_c,
            eval_s1: self.eval_s1,
            eval_s2: self.eval_s2,
            eval_zw: self.eval_zw,
        })
    }
}

impl Proof {
    pub fn compress(&self) -> Result<CompressedProof, PlonkError> {
        Ok(CompressedProof {
            a: decompression::compress_g1(&self.a)?,
            b: decompression::compress_g1(&self.b)?,
            c: decompression::compress_g1(&self.c)?,
            z: decompression::compress_g1(&self.z)?,
            t1: decompression::compress_g1(&self.t1)?,
            t2: decompression::compress_g1(&self.t2)?,
            t3: decompression::compress_g1(&self.t3)?,
            wxi: decompression::compress_g1(&self.wxi)?,
            wxiw: decompression::compress_g1(&self.wxiw)?,
            eval_a: self.eval_a,
            eval_b: self.eval_b,
            eval_c: self.eval_c,
            eval_s1: self.eval_s1,
            eval_s2: self.eval_s2,
            eval_zw: self.eval_zw,
        })
    }
}

struct Challenges {
    beta: Fr,
    gamma: Fr,
    alpha: Fr,
    xi: Fr,
    xin: Fr,
    zh: Fr,
    v: [Fr; 6],
    u: Fr,
}

fn g1_add(a: &[u8; 64], b: &[u8; 64]) -> Result<[u8; 64], PlonkError> {
    let input = [a.as_slice(), b.as_slice()].concat();
    let result = alt_bn128_g1_addition_be(&input).map_err(|_| PlonkError::G1AdditionFailed)?;
    result
        .try_into()
        .map_err(|_| PlonkError::G1AdditionFailed)
}

fn g1_sub(a: &[u8; 64], b: &[u8; 64]) -> Result<[u8; 64], PlonkError> {
    let neg_b = g1_neg(b);
    g1_add(a, &neg_b)
}

fn g1_neg(p: &[u8; 64]) -> [u8; 64] {
    if *p == G1_ZERO {
        return G1_ZERO;
    }
    let mut result = [0u8; 64];
    result[..32].copy_from_slice(&p[..32]);
    let y = &p[32..64];
    let mut borrow: u16 = 0;
    for i in (0..32).rev() {
        let diff = (FQ_MODULUS[i] as u16).wrapping_sub(y[i] as u16).wrapping_sub(borrow);
        result[32 + i] = diff as u8;
        borrow = if diff > 255 { 1 } else { 0 };
    }
    result
}

fn g1_mul(point: &[u8; 64], scalar: &Fr) -> Result<[u8; 64], PlonkError> {
    let s = scalar.to_be_bytes();
    let input = [point.as_slice(), s.as_slice()].concat();
    let result =
        alt_bn128_g1_multiplication_be(&input).map_err(|_| PlonkError::G1MulFailed)?;
    result.try_into().map_err(|_| PlonkError::G1MulFailed)
}

/// Verify a PLONK proof against a verification key and public inputs.
pub fn verify(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[Fr],
) -> Result<(), PlonkError> {
    if public_inputs.len() != vk.n_public {
        return Err(PlonkError::InvalidPublicInputsLength);
    }

    let challenges = calculate_challenges(vk, proof, public_inputs)?;
    let lagrange = calculate_lagrange_evaluations(vk, &challenges)?;
    let pi = calculate_pi(public_inputs, &lagrange);
    let r0 = calculate_r0(proof, &challenges, &pi, &lagrange[1]);
    let d = calculate_d(vk, proof, &challenges, &lagrange[1])?;
    let f = calculate_f(vk, proof, &challenges, &d)?;
    let e = calculate_e(proof, &challenges, &r0)?;

    if is_valid_pairing(vk, proof, &challenges, &e, &f)? {
        Ok(())
    } else {
        Err(PlonkError::ProofVerificationFailed)
    }
}

fn calculate_challenges(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[Fr],
) -> Result<Challenges, PlonkError> {
    let mut transcript = Transcript::new();

    transcript.add_point(&vk.qm);
    transcript.add_point(&vk.ql);
    transcript.add_point(&vk.qr);
    transcript.add_point(&vk.qo);
    transcript.add_point(&vk.qc);
    transcript.add_point(&vk.s1);
    transcript.add_point(&vk.s2);
    transcript.add_point(&vk.s3);
    for pi in public_inputs {
        transcript.add_scalar(pi);
    }
    transcript.add_point(&proof.a);
    transcript.add_point(&proof.b);
    transcript.add_point(&proof.c);
    let beta = transcript.get_challenge()?;

    transcript.reset();
    transcript.add_scalar(&beta);
    let gamma = transcript.get_challenge()?;

    transcript.reset();
    transcript.add_scalar(&beta);
    transcript.add_scalar(&gamma);
    transcript.add_point(&proof.z);
    let alpha = transcript.get_challenge()?;

    transcript.reset();
    transcript.add_scalar(&alpha);
    transcript.add_point(&proof.t1);
    transcript.add_point(&proof.t2);
    transcript.add_point(&proof.t3);
    let xi = transcript.get_challenge()?;

    transcript.reset();
    transcript.add_scalar(&xi);
    transcript.add_scalar(&proof.eval_a);
    transcript.add_scalar(&proof.eval_b);
    transcript.add_scalar(&proof.eval_c);
    transcript.add_scalar(&proof.eval_s1);
    transcript.add_scalar(&proof.eval_s2);
    transcript.add_scalar(&proof.eval_zw);
    let v1 = transcript.get_challenge()?;

    let mut v = [Fr::zero(); 6];
    v[1] = v1;
    for i in 2..6 {
        v[i] = v[i - 1] * v1;
    }

    transcript.reset();
    transcript.add_point(&proof.wxi);
    transcript.add_point(&proof.wxiw);
    let u = transcript.get_challenge()?;

    let mut xin = xi;
    for _ in 0..vk.power {
        xin = xin.square();
    }
    let zh = xin - Fr::one();

    Ok(Challenges {
        beta,
        gamma,
        alpha,
        xi,
        xin,
        zh,
        v,
        u,
    })
}

fn calculate_lagrange_evaluations(
    vk: &VerificationKey,
    ch: &Challenges,
) -> Result<Vec<Fr>, PlonkError> {
    let domain_size = 1u64 << vk.power;
    let n = Fr::from(domain_size);

    let mut l = vec![Fr::zero()];
    let mut w = Fr::one();

    let count = std::cmp::max(1, vk.n_public);
    for _ in 0..count {
        let num = w * ch.zh;
        let den = n * (ch.xi - w);
        let inv = den.inverse().ok_or(PlonkError::LagrangeDivisionByZero)?;
        l.push(num * inv);
        w = w * vk.w;
    }

    Ok(l)
}

fn calculate_pi(public_inputs: &[Fr], lagrange: &[Fr]) -> Fr {
    let mut pi = Fr::zero();
    for (i, input) in public_inputs.iter().enumerate() {
        pi = pi - *input * lagrange[i + 1];
    }
    pi
}

fn calculate_r0(proof: &Proof, ch: &Challenges, pi: &Fr, l1: &Fr) -> Fr {
    let e1 = *pi;
    let e2 = *l1 * ch.alpha.square();

    let e3a = proof.eval_a + ch.beta * proof.eval_s1 + ch.gamma;
    let e3b = proof.eval_b + ch.beta * proof.eval_s2 + ch.gamma;
    let e3c = proof.eval_c + ch.gamma;
    let e3 = e3a * e3b * e3c * proof.eval_zw * ch.alpha;

    e1 - e2 - e3
}

fn calculate_d(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    l1: &Fr,
) -> Result<[u8; 64], PlonkError> {
    let ab = proof.eval_a * proof.eval_b;
    let d1 = {
        let t0 = g1_mul(&vk.qm, &ab)?;
        let t1 = g1_mul(&vk.ql, &proof.eval_a)?;
        let t2 = g1_mul(&vk.qr, &proof.eval_b)?;
        let t3 = g1_mul(&vk.qo, &proof.eval_c)?;
        let r = g1_add(&t0, &t1)?;
        let r = g1_add(&r, &t2)?;
        let r = g1_add(&r, &t3)?;
        g1_add(&r, &vk.qc)?
    };

    let betaxi = ch.beta * ch.xi;
    let d2a1 = proof.eval_a + betaxi + ch.gamma;
    let d2a2 = proof.eval_b + betaxi * vk.k1 + ch.gamma;
    let d2a3 = proof.eval_c + betaxi * vk.k2 + ch.gamma;
    let d2a = d2a1 * d2a2 * d2a3 * ch.alpha;
    let d2b = *l1 * ch.alpha.square();
    let d2_scalar = d2a + d2b + ch.u;
    let d2 = g1_mul(&proof.z, &d2_scalar)?;

    let d3a = proof.eval_a + ch.beta * proof.eval_s1 + ch.gamma;
    let d3b = proof.eval_b + ch.beta * proof.eval_s2 + ch.gamma;
    let d3c = ch.alpha * ch.beta * proof.eval_zw;
    let d3_scalar = d3a * d3b * d3c;
    let d3 = g1_mul(&vk.s3, &d3_scalar)?;

    let xin_sq = ch.xin.square();
    let d4_t2 = g1_mul(&proof.t2, &ch.xin)?;
    let d4_t3 = g1_mul(&proof.t3, &xin_sq)?;
    let d4_sum = g1_add(&proof.t1, &g1_add(&d4_t2, &d4_t3)?)?;
    let d4 = g1_mul(&d4_sum, &ch.zh)?;

    let r = g1_add(&d1, &d2)?;
    let r = g1_sub(&r, &d3)?;
    g1_sub(&r, &d4)
}

fn calculate_f(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    d: &[u8; 64],
) -> Result<[u8; 64], PlonkError> {
    let t1 = g1_mul(&proof.a, &ch.v[1])?;
    let t2 = g1_mul(&proof.b, &ch.v[2])?;
    let t3 = g1_mul(&proof.c, &ch.v[3])?;
    let t4 = g1_mul(&vk.s1, &ch.v[4])?;
    let t5 = g1_mul(&vk.s2, &ch.v[5])?;

    let r = g1_add(d, &t1)?;
    let r = g1_add(&r, &t2)?;
    let r = g1_add(&r, &t3)?;
    let r = g1_add(&r, &t4)?;
    g1_add(&r, &t5)
}

fn calculate_e(proof: &Proof, ch: &Challenges, r0: &Fr) -> Result<[u8; 64], PlonkError> {
    let scalar = -*r0
        + ch.v[1] * proof.eval_a
        + ch.v[2] * proof.eval_b
        + ch.v[3] * proof.eval_c
        + ch.v[4] * proof.eval_s1
        + ch.v[5] * proof.eval_s2
        + ch.u * proof.eval_zw;

    g1_mul(&G1_GENERATOR, &scalar)
}

fn is_valid_pairing(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    e: &[u8; 64],
    f: &[u8; 64],
) -> Result<bool, PlonkError> {
    let u_wxiw = g1_mul(&proof.wxiw, &ch.u)?;
    let a1 = g1_add(&proof.wxi, &u_wxiw)?;

    let xi_wxi = g1_mul(&proof.wxi, &ch.xi)?;
    let s = ch.u * ch.xi * vk.w;
    let s_wxiw = g1_mul(&proof.wxiw, &s)?;
    let b1 = g1_add(&xi_wxi, &s_wxiw)?;
    let b1 = g1_add(&b1, f)?;
    let b1 = g1_sub(&b1, e)?;

    let neg_a1 = g1_neg(&a1);

    let pairing_input = [
        neg_a1.as_slice(),
        vk.x_2.as_slice(),
        b1.as_slice(),
        G2_GENERATOR.as_slice(),
    ]
    .concat();

    let result = alt_bn128_pairing_be(&pairing_input).map_err(|_| PlonkError::PairingFailed)?;
    Ok(result[31] == 1)
}

#[cfg(all(test, feature = "vk"))]
mod tests {
    use super::*;
    use serde::Deserialize;

    // ProofJson + proof/public parsing remain here (no proof_parser feature yet)
    #[derive(Deserialize)]
    struct ProofJson {
        #[serde(rename = "A")]
        a: Vec<String>,
        #[serde(rename = "B")]
        b: Vec<String>,
        #[serde(rename = "C")]
        c: Vec<String>,
        #[serde(rename = "Z")]
        z: Vec<String>,
        #[serde(rename = "T1")]
        t1: Vec<String>,
        #[serde(rename = "T2")]
        t2: Vec<String>,
        #[serde(rename = "T3")]
        t3: Vec<String>,
        #[serde(rename = "Wxi")]
        wxi: Vec<String>,
        #[serde(rename = "Wxiw")]
        wxiw: Vec<String>,
        eval_a: String,
        eval_b: String,
        eval_c: String,
        eval_s1: String,
        eval_s2: String,
        eval_zw: String,
    }

    fn str_to_be32(s: &str) -> [u8; 32] {
        let n = num_bigint::BigUint::parse_bytes(s.as_bytes(), 10).unwrap();
        let bytes = n.to_bytes_be();
        let mut result = [0u8; 32];
        let start = 32usize.saturating_sub(bytes.len());
        result[start..].copy_from_slice(&bytes);
        result
    }

    fn parse_g1_be(coords: &[String]) -> [u8; 64] {
        if coords[2] == "0" {
            return [0u8; 64];
        }
        let mut result = [0u8; 64];
        result[..32].copy_from_slice(&str_to_be32(&coords[0]));
        result[32..].copy_from_slice(&str_to_be32(&coords[1]));
        result
    }

    fn test_vk() -> VerificationKey {
        crate::vk_parser::parse_vk_json(include_str!("../test-fixtures/verification_key.json"))
            .unwrap()
    }

    fn test_proof() -> Proof {
        let p: ProofJson =
            serde_json::from_str(include_str!("../test-fixtures/proof.json")).unwrap();
        Proof {
            a: parse_g1_be(&p.a),
            b: parse_g1_be(&p.b),
            c: parse_g1_be(&p.c),
            z: parse_g1_be(&p.z),
            t1: parse_g1_be(&p.t1),
            t2: parse_g1_be(&p.t2),
            t3: parse_g1_be(&p.t3),
            wxi: parse_g1_be(&p.wxi),
            wxiw: parse_g1_be(&p.wxiw),
            eval_a: Fr::from_be_bytes(&str_to_be32(&p.eval_a)),
            eval_b: Fr::from_be_bytes(&str_to_be32(&p.eval_b)),
            eval_c: Fr::from_be_bytes(&str_to_be32(&p.eval_c)),
            eval_s1: Fr::from_be_bytes(&str_to_be32(&p.eval_s1)),
            eval_s2: Fr::from_be_bytes(&str_to_be32(&p.eval_s2)),
            eval_zw: Fr::from_be_bytes(&str_to_be32(&p.eval_zw)),
        }
    }

    fn test_public_inputs() -> Vec<Fr> {
        let vals: Vec<String> =
            serde_json::from_str(include_str!("../test-fixtures/public.json")).unwrap();
        vals.iter()
            .map(|s| Fr::from_be_bytes(&str_to_be32(s)))
            .collect()
    }

    #[test]
    fn test_plonk_verify_valid_proof() {
        let vk = test_vk();
        let proof = test_proof();
        let public_inputs = test_public_inputs();
        verify(&vk, &proof, &public_inputs).unwrap();
    }

    #[test]
    fn test_plonk_verify_invalid_public_input() {
        let vk = test_vk();
        let proof = test_proof();
        let bad_inputs = vec![Fr::from(34u64)];
        assert_eq!(
            verify(&vk, &proof, &bad_inputs),
            Err(PlonkError::ProofVerificationFailed)
        );
    }

    #[test]
    fn test_plonk_verify_compressed_proof() {
        let vk = test_vk();
        let proof = test_proof();
        let public_inputs = test_public_inputs();

        let compressed = proof.compress().unwrap();
        let decompressed = compressed.decompress().unwrap();

        verify(&vk, &decompressed, &public_inputs).unwrap();
    }

    #[test]
    fn test_compression_roundtrip() {
        let proof = test_proof();
        let compressed = proof.compress().unwrap();
        let decompressed = compressed.decompress().unwrap();

        assert_eq!(proof.a, decompressed.a);
        assert_eq!(proof.b, decompressed.b);
        assert_eq!(proof.c, decompressed.c);
        assert_eq!(proof.z, decompressed.z);
        assert_eq!(proof.t1, decompressed.t1);
        assert_eq!(proof.t2, decompressed.t2);
        assert_eq!(proof.t3, decompressed.t3);
        assert_eq!(proof.wxi, decompressed.wxi);
        assert_eq!(proof.wxiw, decompressed.wxiw);
        assert_eq!(proof.eval_a, decompressed.eval_a);
        assert_eq!(proof.eval_b, decompressed.eval_b);
        assert_eq!(proof.eval_c, decompressed.eval_c);
        assert_eq!(proof.eval_s1, decompressed.eval_s1);
        assert_eq!(proof.eval_s2, decompressed.eval_s2);
        assert_eq!(proof.eval_zw, decompressed.eval_zw);
    }
}
