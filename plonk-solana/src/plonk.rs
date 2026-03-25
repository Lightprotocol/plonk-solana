/// PLONK verifier for BN254 curve operations.
///
/// On Solana: uses pinocchio alt_bn128 syscalls.
/// Off-chain: uses arkworks fallback.
///
/// All G1 points are 64 bytes big-endian (x || y).
/// All G2 points are 128 bytes big-endian (x1 || x0 || y1 || y0).
/// All scalars are 32 bytes big-endian.
use crate::errors::PlonkError;
use crate::fr::{bigint_to_be_bytes, Fr};
use crate::g1::{CompressedG1, G1};
use crate::g2::G2;
use crate::syscalls::{g1_addition_be, g1_multiplication_be, pairing_be};
use crate::transcript::Transcript;
use ark_bn254::Fq;
use ark_ff::PrimeField as _;

/// Verification key (G1 points + G2 generator + scalar parameters).
#[derive(Debug, PartialEq)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VerificationKey {
    pub n_public: u32,
    pub power: u32,
    pub k1: Fr,
    pub k2: Fr,
    pub w: Fr,
    pub qm: G1,
    pub ql: G1,
    pub qr: G1,
    pub qo: G1,
    pub qc: G1,
    pub s1: G1,
    pub s2: G1,
    pub s3: G1,
    pub x_2: G2,
}

/// Proof (G1 commitments + scalar evaluations).
#[derive(Debug, PartialEq)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Proof {
    pub a: G1,
    pub b: G1,
    pub c: G1,
    pub z: G1,
    pub t1: G1,
    pub t2: G1,
    pub t3: G1,
    pub wxi: G1,
    pub wxiw: G1,
    pub eval_a: Fr,
    pub eval_b: Fr,
    pub eval_c: Fr,
    pub eval_s1: Fr,
    pub eval_s2: Fr,
    pub eval_zw: Fr,
}

/// Compressed proof (G1 points as 32 bytes each).
/// 9 * 32 + 6 * 32 = 480 bytes vs 768 bytes uncompressed.
#[derive(Debug, PartialEq)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompressedProof {
    pub a: CompressedG1,
    pub b: CompressedG1,
    pub c: CompressedG1,
    pub z: CompressedG1,
    pub t1: CompressedG1,
    pub t2: CompressedG1,
    pub t3: CompressedG1,
    pub wxi: CompressedG1,
    pub wxiw: CompressedG1,
    pub eval_a: Fr,
    pub eval_b: Fr,
    pub eval_c: Fr,
    pub eval_s1: Fr,
    pub eval_s2: Fr,
    pub eval_zw: Fr,
}

impl CompressedProof {
    pub fn decompress(&self) -> Result<Proof, PlonkError> {
        Proof::try_from(self)
    }
}

impl Proof {
    pub fn compress(&self) -> Result<CompressedProof, PlonkError> {
        CompressedProof::try_from(self)
    }
}

impl TryFrom<&CompressedProof> for Proof {
    type Error = PlonkError;

    fn try_from(compressed: &CompressedProof) -> Result<Self, PlonkError> {
        Ok(Proof {
            a: compressed.a.decompress()?,
            b: compressed.b.decompress()?,
            c: compressed.c.decompress()?,
            z: compressed.z.decompress()?,
            t1: compressed.t1.decompress()?,
            t2: compressed.t2.decompress()?,
            t3: compressed.t3.decompress()?,
            wxi: compressed.wxi.decompress()?,
            wxiw: compressed.wxiw.decompress()?,
            eval_a: compressed.eval_a,
            eval_b: compressed.eval_b,
            eval_c: compressed.eval_c,
            eval_s1: compressed.eval_s1,
            eval_s2: compressed.eval_s2,
            eval_zw: compressed.eval_zw,
        })
    }
}

impl TryFrom<&Proof> for CompressedProof {
    type Error = PlonkError;

    fn try_from(proof: &Proof) -> Result<Self, PlonkError> {
        Ok(CompressedProof {
            a: proof.a.compress()?,
            b: proof.b.compress()?,
            c: proof.c.compress()?,
            z: proof.z.compress()?,
            t1: proof.t1.compress()?,
            t2: proof.t2.compress()?,
            t3: proof.t3.compress()?,
            wxi: proof.wxi.compress()?,
            wxiw: proof.wxiw.compress()?,
            eval_a: proof.eval_a,
            eval_b: proof.eval_b,
            eval_c: proof.eval_c,
            eval_s1: proof.eval_s1,
            eval_s2: proof.eval_s2,
            eval_zw: proof.eval_zw,
        })
    }
}

pub struct Challenges {
    pub beta: Fr,
    pub gamma: Fr,
    pub alpha: Fr,
    pub xi: Fr,
    pub xin: Fr,
    pub zh: Fr,
    pub v: [Fr; 6],
    pub u: Fr,
}

pub fn g1_add(a: &G1, b: &G1) -> Result<G1, PlonkError> {
    let mut input = [0u8; 128];
    input[..64].copy_from_slice(&a.0);
    input[64..].copy_from_slice(&b.0);
    let result = g1_addition_be(&input)?;
    Ok(G1(result))
}

pub fn g1_sub(a: &G1, b: &G1) -> Result<G1, PlonkError> {
    let neg_b = g1_neg(b);
    g1_add(a, &neg_b)
}

pub fn g1_neg(p: &G1) -> G1 {
    if *p == G1::ZERO {
        return G1::ZERO;
    }
    let mut result = [0u8; 64];
    result[..32].copy_from_slice(&p.0[..32]);
    let y = Fq::from_be_bytes_mod_order(&p.0[32..64]);
    let neg_y = -y;
    let neg_y_bytes = bigint_to_be_bytes(&neg_y.into_bigint());
    result[32..64].copy_from_slice(&neg_y_bytes);
    G1(result)
}

pub fn g1_mul(point: &G1, scalar: &Fr) -> Result<G1, PlonkError> {
    let mut input = [0u8; 96];
    input[..64].copy_from_slice(&point.0);
    input[64..].copy_from_slice(&scalar.to_be_bytes());
    let result = g1_multiplication_be(&input)?;
    Ok(G1(result))
}

/// Verify a PLONK proof, checking that public inputs are less than the field size.
///
/// Public inputs are raw 32-byte big-endian values. Each is validated against
/// the BN254 scalar field modulus before conversion to Fr.
pub fn verify<const N: usize>(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[[u8; 32]; N],
) -> Result<(), PlonkError> {
    if N != vk.n_public as usize {
        return Err(PlonkError::InvalidPublicInputsLength);
    }
    for input in public_inputs {
        if !crate::fr::is_less_than_bn254_field_size_be(input) {
            return Err(PlonkError::PublicInputGreaterThanFieldSize);
        }
    }
    let fr_inputs: [Fr; N] =
        core::array::from_fn(|i| Fr::from_be_bytes_unchecked(&public_inputs[i]));
    verify_unchecked(vk, proof, &fr_inputs)
}

/// Verify a PLONK proof without checking that public inputs are less than the
/// field size. Use this when inputs are already known to be canonical Fr values.
pub fn verify_unchecked<const N: usize>(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[Fr; N],
) -> Result<(), PlonkError> {
    if N != vk.n_public as usize {
        return Err(PlonkError::InvalidPublicInputsLength);
    }

    let challenges = calculate_challenges::<N>(vk, proof, public_inputs)?;
    let (l1, pi) = calculate_l1_and_pi::<N>(vk, &challenges, public_inputs)?;
    let r0 = calculate_r0(proof, &challenges, &pi, &l1);
    let d = calculate_d(vk, proof, &challenges, &l1)?;
    let f = calculate_f(vk, proof, &challenges, &d)?;
    let e = calculate_e(proof, &challenges, &r0)?;

    if is_valid_pairing(vk, proof, &challenges, &e, &f)? {
        Ok(())
    } else {
        Err(PlonkError::ProofVerificationFailed)
    }
}

pub fn calculate_challenges<const N: usize>(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[Fr; N],
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

pub fn calculate_l1_and_pi<const N: usize>(
    vk: &VerificationKey,
    ch: &Challenges,
    public_inputs: &[Fr; N],
) -> Result<(Fr, Fr), PlonkError> {
    let domain_size = 1u64 << vk.power;
    let n = Fr::from(domain_size);
    let mut w = Fr::one();
    let mut l1 = Fr::zero();
    let mut pi = Fr::zero();

    let count = core::cmp::max(1, vk.n_public as usize);
    #[allow(clippy::needless_range_loop)]
    for i in 0..count {
        let num = w * ch.zh;
        let den = n * (ch.xi - w);
        let inv = den.inverse().ok_or(PlonkError::LagrangeDivisionByZero)?;
        let li = num * inv;
        if i == 0 {
            l1 = li;
        }
        if i < N {
            pi = pi - public_inputs[i] * li;
        }
        w = w * vk.w;
    }

    Ok((l1, pi))
}

pub fn calculate_r0(proof: &Proof, ch: &Challenges, pi: &Fr, l1: &Fr) -> Fr {
    let e1 = *pi;
    let e2 = *l1 * ch.alpha.square();

    let e3a = proof.eval_a + ch.beta * proof.eval_s1 + ch.gamma;
    let e3b = proof.eval_b + ch.beta * proof.eval_s2 + ch.gamma;
    let e3c = proof.eval_c + ch.gamma;
    let e3 = e3a * e3b * e3c * proof.eval_zw * ch.alpha;

    e1 - e2 - e3
}

pub fn calculate_d(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    l1: &Fr,
) -> Result<G1, PlonkError> {
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

pub fn calculate_f(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    d: &G1,
) -> Result<G1, PlonkError> {
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

pub fn calculate_e(proof: &Proof, ch: &Challenges, r0: &Fr) -> Result<G1, PlonkError> {
    let scalar = -*r0
        + ch.v[1] * proof.eval_a
        + ch.v[2] * proof.eval_b
        + ch.v[3] * proof.eval_c
        + ch.v[4] * proof.eval_s1
        + ch.v[5] * proof.eval_s2
        + ch.u * proof.eval_zw;

    g1_mul(&G1::GENERATOR, &scalar)
}

pub fn is_valid_pairing(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    e: &G1,
    f: &G1,
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

    // 2 pairing pairs: (neg_a1, x_2) and (b1, G2::GENERATOR)
    // Each pair = 64 (G1) + 128 (G2) = 192 bytes, total = 384 bytes
    let mut pairing_input = [0u8; 384];
    pairing_input[..64].copy_from_slice(&neg_a1.0);
    pairing_input[64..192].copy_from_slice(&vk.x_2.0);
    pairing_input[192..256].copy_from_slice(&b1.0);
    pairing_input[256..384].copy_from_slice(&G2::GENERATOR.0);

    let result = pairing_be(&pairing_input)?;
    Ok(result[31] == 1)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::vec::Vec;

    use super::*;
    use crate::vk_parser;

    fn test_vk() -> VerificationKey {
        vk_parser::parse_vk_json(include_str!(
            "../../tests/fixtures/data/verification_key.json"
        ))
        .unwrap()
    }

    fn test_proof() -> Proof {
        vk_parser::parse_proof_json(include_str!("../../tests/fixtures/data/proof.json")).unwrap()
    }

    fn test_public_inputs_fr() -> Vec<Fr> {
        vk_parser::parse_public_inputs_json(include_str!("../../tests/fixtures/data/public.json"))
            .unwrap()
    }

    fn test_public_input_bytes() -> [u8; 32] {
        test_public_inputs_fr()[0].to_be_bytes()
    }

    #[test]
    fn test_plonk_verify_valid_proof() {
        verify(&test_vk(), &test_proof(), &[test_public_input_bytes()]).unwrap();
    }

    #[test]
    fn test_plonk_verify_unchecked_valid_proof() {
        let inputs = test_public_inputs_fr();
        verify_unchecked(&test_vk(), &test_proof(), &[inputs[0]]).unwrap();
    }

    #[test]
    fn test_plonk_verify_invalid_public_input() {
        let result = verify(&test_vk(), &test_proof(), &[Fr::from(34u64).to_be_bytes()]);
        assert_eq!(
            result,
            Err(PlonkError::ProofVerificationFailed),
            "invalid public input should cause verification failure"
        );
    }

    #[test]
    fn test_plonk_verify_wrong_input_count() {
        let result = verify::<0>(&test_vk(), &test_proof(), &[]);
        assert_eq!(
            result,
            Err(PlonkError::InvalidPublicInputsLength),
            "wrong number of public inputs should be rejected"
        );
    }

    #[test]
    fn test_plonk_verify_public_input_greater_than_field_size() {
        use crate::fr::bigint_to_be_bytes;
        use ark_ff::PrimeField;
        let input = bigint_to_be_bytes(&<ark_bn254::Fr as PrimeField>::MODULUS);

        let result = verify(&test_vk(), &test_proof(), &[input]);
        assert_eq!(
            result,
            Err(PlonkError::PublicInputGreaterThanFieldSize),
            "public input >= field modulus should be rejected by verify"
        );

        // verify_unchecked does not check field size -- the non-canonical value
        // silently reduces and causes a proof verification failure instead.
        let fr_input = Fr::from_be_bytes_unchecked(&input);
        let result = verify_unchecked(&test_vk(), &test_proof(), &[fr_input]);
        assert_eq!(
            result,
            Err(PlonkError::ProofVerificationFailed),
            "verify_unchecked should not catch oversized inputs"
        );
    }

    #[test]
    fn test_plonk_verify_compressed_proof() {
        let proof = test_proof();
        let compressed = proof.compress().unwrap();
        let decompressed = compressed.decompress().unwrap();
        verify(&test_vk(), &decompressed, &[test_public_input_bytes()]).unwrap();
    }

    #[test]
    fn test_compression_roundtrip() {
        let proof = test_proof();
        let decompressed = proof.compress().unwrap().decompress().unwrap();
        assert_eq!(
            proof, decompressed,
            "proof should survive compress/decompress roundtrip"
        );
    }
}
