/// PLONK verifier for BN254 curve operations.
///
/// On Solana: uses pinocchio alt_bn128 syscalls.
/// Off-chain: uses arkworks fallback.
///
/// All G1 points are 64 bytes big-endian (x || y).
/// All G2 points are 128 bytes big-endian (x1 || x0 || y1 || y0).
/// All scalars are 32 bytes big-endian.
use crate::errors::PlonkError;
use crate::fr::Fr;
use crate::g1::{CompressedG1, G1};
use crate::g2::G2;
use crate::syscalls::{g1_addition_be, g1_multiplication_be, pairing_be};
use crate::transcript::hash_challenge;

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
    pub v1: Fr,
    pub v2: Fr,
    pub v3: Fr,
    pub v4: Fr,
    pub v5: Fr,
    pub u: Fr,
}

pub fn g1_add(a: &G1, b: &G1) -> Result<G1, PlonkError> {
    let mut input = [0u8; 128];
    input[..64].copy_from_slice(&a.0);
    input[64..].copy_from_slice(&b.0);
    let result = g1_addition_be(&input)?;
    Ok(G1(result))
}

pub fn g1_neg(p: &G1) -> G1 {
    if *p == G1::ZERO {
        return G1::ZERO;
    }
    let mut result = [0u8; 64];
    result[..32].copy_from_slice(&p.0[..32]);
    // BN254 Fq modulus (big-endian u64 limbs, most significant first)
    const FQ: [u64; 4] = [
        0x30644e72e131a029,
        0xb85045b68181585d,
        0x97816a916871ca8d,
        0x3c208c16d87cfd47,
    ];
    let y = [
        u64::from_be_bytes([
            p.0[32], p.0[33], p.0[34], p.0[35], p.0[36], p.0[37], p.0[38], p.0[39],
        ]),
        u64::from_be_bytes([
            p.0[40], p.0[41], p.0[42], p.0[43], p.0[44], p.0[45], p.0[46], p.0[47],
        ]),
        u64::from_be_bytes([
            p.0[48], p.0[49], p.0[50], p.0[51], p.0[52], p.0[53], p.0[54], p.0[55],
        ]),
        u64::from_be_bytes([
            p.0[56], p.0[57], p.0[58], p.0[59], p.0[60], p.0[61], p.0[62], p.0[63],
        ]),
    ];
    // Compute p - y with borrow (big-endian: limb[0] is most significant)
    let mut borrow: u64 = 0;
    let mut neg_y = [0u64; 4];
    let mut i = 3;
    loop {
        let (diff, b1) = FQ[i].overflowing_sub(y[i]);
        let (diff, b2) = diff.overflowing_sub(borrow);
        neg_y[i] = diff;
        borrow = (b1 as u64) + (b2 as u64);
        if i == 0 {
            break;
        }
        i -= 1;
    }
    result[32..40].copy_from_slice(&neg_y[0].to_be_bytes());
    result[40..48].copy_from_slice(&neg_y[1].to_be_bytes());
    result[48..56].copy_from_slice(&neg_y[2].to_be_bytes());
    result[56..64].copy_from_slice(&neg_y[3].to_be_bytes());
    G1(result)
}

pub fn g1_mul(point: &G1, scalar: &Fr) -> Result<G1, PlonkError> {
    let mut input = [0u8; 96];
    input[..64].copy_from_slice(&point.0);
    input[64..].copy_from_slice(&scalar.to_be_bytes());
    let result = g1_multiplication_be(&input)?;
    Ok(G1(result))
}

pub fn g1_mul_bytes(point: &G1, scalar_bytes: &[u8; 32]) -> Result<G1, PlonkError> {
    let mut input = [0u8; 96];
    input[..64].copy_from_slice(&point.0);
    input[64..].copy_from_slice(scalar_bytes);
    let result = g1_multiplication_be(&input)?;
    Ok(G1(result))
}

#[inline(never)]
pub fn compute_eval_bytes(proof: &Proof) -> [[u8; 32]; 6] {
    [
        proof.eval_a.to_be_bytes(),
        proof.eval_b.to_be_bytes(),
        proof.eval_c.to_be_bytes(),
        proof.eval_s1.to_be_bytes(),
        proof.eval_s2.to_be_bytes(),
        proof.eval_zw.to_be_bytes(),
    ]
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

    let eval_bytes = compute_eval_bytes(proof);
    let challenges = calculate_challenges::<N>(vk, proof, public_inputs, &eval_bytes)?;
    let (l1, pi) = calculate_l1_and_pi::<N>(vk, &challenges, public_inputs)?;
    let (r0, d) = calculate_r0_and_d(vk, proof, &challenges, &l1, &pi, &eval_bytes)?;
    let f = calculate_f(vk, proof, &challenges, &d)?;

    if is_valid_pairing(vk, proof, &challenges, &r0, &f)? {
        Ok(())
    } else {
        Err(PlonkError::ProofVerificationFailed)
    }
}

pub fn calculate_challenges<const N: usize>(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[Fr; N],
    eval_bytes: &[[u8; 32]; 6],
) -> Result<Challenges, PlonkError> {
    let (beta, gamma, alpha, xi) = challenge_rounds_1_to_4::<N>(vk, proof, public_inputs)?;

    // V1: hash(xi || eval_a || eval_b || eval_c || eval_s1 || eval_s2 || eval_zw)
    let v1 = challenge_v1(&xi, eval_bytes)?;

    let v2 = v1.square();
    let v3 = v2 * v1;
    let v4 = v2.square();
    let v5 = v4 * v1;

    // U: hash(wxi || wxiw)
    let u = hash_challenge(&[&proof.wxi.0, &proof.wxiw.0])?;

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
        v1,
        v2,
        v3,
        v4,
        v5,
        u,
    })
}

#[inline(never)]
fn challenge_rounds_1_to_4<const N: usize>(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[Fr; N],
) -> Result<(Fr, Fr, Fr, Fr), PlonkError> {
    let mut pi_bytes = [[0u8; 32]; N];
    for (i, pi) in public_inputs.iter().enumerate() {
        pi_bytes[i] = pi.to_be_bytes();
    }

    let mut beta_slices: [&[u8]; 12] = [
        &vk.qm.0,
        &vk.ql.0,
        &vk.qr.0,
        &vk.qo.0,
        &vk.qc.0,
        &vk.s1.0,
        &vk.s2.0,
        &vk.s3.0,
        &[],
        &proof.a.0,
        &proof.b.0,
        &proof.c.0,
    ];
    if N == 1 {
        beta_slices[8] = &pi_bytes[0];
    }
    let beta = hash_challenge(&beta_slices)?;

    let beta_bytes = beta.to_be_bytes();
    let gamma = hash_challenge(&[&beta_bytes])?;

    let gamma_bytes = gamma.to_be_bytes();
    let alpha = hash_challenge(&[&beta_bytes, &gamma_bytes, &proof.z.0])?;

    let alpha_bytes = alpha.to_be_bytes();
    let xi = hash_challenge(&[&alpha_bytes, &proof.t1.0, &proof.t2.0, &proof.t3.0])?;

    Ok((beta, gamma, alpha, xi))
}

#[inline(never)]
fn challenge_v1(xi: &Fr, eval_bytes: &[[u8; 32]; 6]) -> Result<Fr, PlonkError> {
    let xi_bytes = xi.to_be_bytes();
    hash_challenge(&[
        &xi_bytes,
        &eval_bytes[0],
        &eval_bytes[1],
        &eval_bytes[2],
        &eval_bytes[3],
        &eval_bytes[4],
        &eval_bytes[5],
    ])
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

pub fn calculate_r0_and_d(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    l1: &Fr,
    pi: &Fr,
    eval_bytes: &[[u8; 32]; 6],
) -> Result<(Fr, G1), PlonkError> {
    // Shared sub-expressions
    let alpha_sq = ch.alpha.square();
    let l1_alpha_sq = *l1 * alpha_sq;
    let beta_s1_gamma = proof.eval_a + ch.beta * proof.eval_s1 + ch.gamma;
    let beta_s2_gamma = proof.eval_b + ch.beta * proof.eval_s2 + ch.gamma;

    // r0 computation
    let e3c = proof.eval_c + ch.gamma;
    let e3 = beta_s1_gamma * beta_s2_gamma * e3c * proof.eval_zw * ch.alpha;
    let r0 = *pi - l1_alpha_sq - e3;

    // d computation (reuses shared values)
    let ab = proof.eval_a * proof.eval_b;
    let d1 = {
        let t0 = g1_mul(&vk.qm, &ab)?;
        let t1 = g1_mul_bytes(&vk.ql, &eval_bytes[0])?;
        let t2 = g1_mul_bytes(&vk.qr, &eval_bytes[1])?;
        let t3 = g1_mul_bytes(&vk.qo, &eval_bytes[2])?;
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
    let d2_scalar = d2a + l1_alpha_sq + ch.u;
    let d2 = g1_mul(&proof.z, &d2_scalar)?;

    let d3c = ch.alpha * ch.beta * proof.eval_zw;
    let d3_scalar = -(beta_s1_gamma * beta_s2_gamma * d3c);
    let d3 = g1_mul(&vk.s3, &d3_scalar)?;

    let xin_sq = ch.xin.square();
    let d4_t2 = g1_mul(&proof.t2, &ch.xin)?;
    let d4_t3 = g1_mul(&proof.t3, &xin_sq)?;
    let d4_sum = g1_add(&proof.t1, &g1_add(&d4_t2, &d4_t3)?)?;
    let d4 = g1_mul(&d4_sum, &(-ch.zh))?;

    let r = g1_add(&d1, &d2)?;
    let r = g1_add(&r, &d3)?;
    let d = g1_add(&r, &d4)?;

    Ok((r0, d))
}

pub fn calculate_f(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    d: &G1,
) -> Result<G1, PlonkError> {
    let t1 = g1_mul(&proof.a, &ch.v1)?;
    let t2 = g1_mul(&proof.b, &ch.v2)?;
    let t3 = g1_mul(&proof.c, &ch.v3)?;
    let t4 = g1_mul(&vk.s1, &ch.v4)?;
    let t5 = g1_mul(&vk.s2, &ch.v5)?;

    let r = g1_add(d, &t1)?;
    let r = g1_add(&r, &t2)?;
    let r = g1_add(&r, &t3)?;
    let r = g1_add(&r, &t4)?;
    g1_add(&r, &t5)
}

pub fn is_valid_pairing(
    vk: &VerificationKey,
    proof: &Proof,
    ch: &Challenges,
    r0: &Fr,
    f: &G1,
) -> Result<bool, PlonkError> {
    let u_wxiw = g1_mul(&proof.wxiw, &ch.u)?;
    let a1 = g1_add(&proof.wxi, &u_wxiw)?;

    // Factor: xi*(wxi + u*w*wxiw) saves 1 g1_mul vs xi*wxi + u*xi*w*wxiw
    let uw = ch.u * vk.w;
    let uw_wxiw = g1_mul(&proof.wxiw, &uw)?;
    let sum = g1_add(&proof.wxi, &uw_wxiw)?;
    let xi_sum = g1_mul(&sum, &ch.xi)?;
    let b1 = g1_add(&xi_sum, f)?;

    // Inline calculate_e with negated scalar to avoid g1_sub
    let neg_e_scalar = *r0
        - ch.v1 * proof.eval_a
        - ch.v2 * proof.eval_b
        - ch.v3 * proof.eval_c
        - ch.v4 * proof.eval_s1
        - ch.v5 * proof.eval_s2
        - ch.u * proof.eval_zw;
    let neg_e = g1_mul(&G1::GENERATOR, &neg_e_scalar)?;
    let b1 = g1_add(&b1, &neg_e)?;

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
