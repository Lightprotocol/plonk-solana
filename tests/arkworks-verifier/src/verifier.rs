/// PLONK verifier matching snarkjs plonk_verify.js.
/// Reference: https://eprint.iacr.org/2019/953.pdf
use ark_bn254::{Bn254, Fr, G1Affine, G1Projective, G2Affine};
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
use ark_ff::{Field, One, Zero};

use crate::parse::{Proof, VerificationKey};
use crate::transcript::Transcript;

pub struct Challenges {
    pub beta: Fr,
    pub gamma: Fr,
    pub alpha: Fr,
    pub xi: Fr,
    pub xin: Fr,    // xi^n
    pub zh: Fr,     // xi^n - 1
    pub v: [Fr; 6], // v[1]..v[5], v[0] unused
    pub u: Fr,
}

/// Verify a PLONK proof against a verification key and public inputs.
pub fn verify(vk: &VerificationKey, proof: &Proof, public_inputs: &[Fr]) -> bool {
    assert_eq!(
        public_inputs.len(),
        vk.n_public,
        "wrong number of public inputs"
    );

    let challenges = calculate_challenges(vk, proof, public_inputs);
    let lagrange = calculate_lagrange_evaluations(vk, &challenges);
    let pi = calculate_pi(public_inputs, &lagrange);
    let r0 = calculate_r0(proof, &challenges, &pi, &lagrange[1]);
    let d = calculate_d(vk, proof, &challenges, &lagrange[1]);
    let f = calculate_f(vk, proof, &challenges, &d);
    let e = calculate_e(proof, &challenges, &r0);

    is_valid_pairing(vk, proof, &challenges, &e, &f)
}

pub fn calculate_challenges(
    vk: &VerificationKey,
    proof: &Proof,
    public_inputs: &[Fr],
) -> Challenges {
    let mut transcript = Transcript::new();

    // Round 2: beta
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
    let beta = transcript.get_challenge();

    // gamma
    transcript.reset();
    transcript.add_scalar(&beta);
    let gamma = transcript.get_challenge();

    // Round 3: alpha
    transcript.reset();
    transcript.add_scalar(&beta);
    transcript.add_scalar(&gamma);
    transcript.add_point(&proof.z);
    let alpha = transcript.get_challenge();

    // Round 4: xi
    transcript.reset();
    transcript.add_scalar(&alpha);
    transcript.add_point(&proof.t1);
    transcript.add_point(&proof.t2);
    transcript.add_point(&proof.t3);
    let xi = transcript.get_challenge();

    // Round 5: v
    transcript.reset();
    transcript.add_scalar(&xi);
    transcript.add_scalar(&proof.eval_a);
    transcript.add_scalar(&proof.eval_b);
    transcript.add_scalar(&proof.eval_c);
    transcript.add_scalar(&proof.eval_s1);
    transcript.add_scalar(&proof.eval_s2);
    transcript.add_scalar(&proof.eval_zw);
    let v1 = transcript.get_challenge();

    let mut v = [Fr::zero(); 6];
    v[1] = v1;
    for i in 2..6 {
        v[i] = v[i - 1] * v1;
    }

    // u
    transcript.reset();
    transcript.add_point(&proof.wxi);
    transcript.add_point(&proof.wxiw);
    let u = transcript.get_challenge();

    // Compute xin = xi^n and zh = xin - 1
    let mut xin = xi;
    for _ in 0..vk.power {
        xin = xin.square();
    }
    let zh = xin - Fr::one();

    Challenges {
        beta,
        gamma,
        alpha,
        xi,
        xin,
        zh,
        v,
        u,
    }
}

pub fn calculate_lagrange_evaluations(vk: &VerificationKey, challenges: &Challenges) -> Vec<Fr> {
    let domain_size = 1u64 << vk.power;
    let n = Fr::from(domain_size);

    let mut l = vec![Fr::zero()]; // L[0] unused, 1-indexed
    let mut w = Fr::one();

    let count = std::cmp::max(1, vk.n_public);
    for _ in 0..count {
        // L_i(xi) = w * zh / (n * (xi - w))
        let num = w * challenges.zh;
        let den = n * (challenges.xi - w);
        l.push(num * den.inverse().expect("division by zero in Lagrange eval"));
        w *= vk.w;
    }

    l
}

pub fn calculate_pi(public_inputs: &[Fr], lagrange: &[Fr]) -> Fr {
    let mut pi = Fr::zero();
    for (i, input) in public_inputs.iter().enumerate() {
        pi -= *input * lagrange[i + 1];
    }
    pi
}

pub fn calculate_r0(proof: &Proof, challenges: &Challenges, pi: &Fr, l1: &Fr) -> Fr {
    let e1 = *pi;
    let e2 = *l1 * challenges.alpha.square();

    let e3a = proof.eval_a + challenges.beta * proof.eval_s1 + challenges.gamma;
    let e3b = proof.eval_b + challenges.beta * proof.eval_s2 + challenges.gamma;
    let e3c = proof.eval_c + challenges.gamma;
    let e3 = e3a * e3b * e3c * proof.eval_zw * challenges.alpha;

    e1 - e2 - e3
}

pub fn calculate_d(
    vk: &VerificationKey,
    proof: &Proof,
    challenges: &Challenges,
    l1: &Fr,
) -> G1Projective {
    // d1 = eval_a*eval_b*Qm + eval_a*Ql + eval_b*Qr + eval_c*Qo + Qc
    let d1 = (vk.qm * (proof.eval_a * proof.eval_b))
        + (vk.ql * proof.eval_a)
        + (vk.qr * proof.eval_b)
        + (vk.qo * proof.eval_c)
        + vk.qc.into_group();

    // d2: copy constraint contribution
    let betaxi = challenges.beta * challenges.xi;
    let d2a1 = proof.eval_a + betaxi + challenges.gamma;
    let d2a2 = proof.eval_b + betaxi * vk.k1 + challenges.gamma;
    let d2a3 = proof.eval_c + betaxi * vk.k2 + challenges.gamma;
    let d2a = d2a1 * d2a2 * d2a3 * challenges.alpha;
    let d2b = *l1 * challenges.alpha.square();
    let d2 = proof.z * (d2a + d2b + challenges.u);

    // d3: permutation contribution
    let d3a = proof.eval_a + challenges.beta * proof.eval_s1 + challenges.gamma;
    let d3b = proof.eval_b + challenges.beta * proof.eval_s2 + challenges.gamma;
    let d3c = challenges.alpha * challenges.beta * proof.eval_zw;
    let d3 = vk.s3 * (d3a * d3b * d3c);

    // d4: quotient polynomial contribution
    let d4 = ((proof.t1.into_group())
        + (proof.t2 * challenges.xin)
        + (proof.t3 * challenges.xin.square()))
        * challenges.zh;

    d1 + d2 - d3 - d4
}

pub fn calculate_f(
    vk: &VerificationKey,
    proof: &Proof,
    challenges: &Challenges,
    d: &G1Projective,
) -> G1Projective {
    *d + (proof.a * challenges.v[1])
        + (proof.b * challenges.v[2])
        + (proof.c * challenges.v[3])
        + (vk.s1 * challenges.v[4])
        + (vk.s2 * challenges.v[5])
}

pub fn calculate_e(proof: &Proof, challenges: &Challenges, r0: &Fr) -> G1Projective {
    let scalar = -*r0
        + challenges.v[1] * proof.eval_a
        + challenges.v[2] * proof.eval_b
        + challenges.v[3] * proof.eval_c
        + challenges.v[4] * proof.eval_s1
        + challenges.v[5] * proof.eval_s2
        + challenges.u * proof.eval_zw;

    G1Affine::generator() * scalar
}

fn is_valid_pairing(
    vk: &VerificationKey,
    proof: &Proof,
    challenges: &Challenges,
    e: &G1Projective,
    f: &G1Projective,
) -> bool {
    // A1 = Wxi + u * Wxiw
    let a1 = proof.wxi.into_group() + proof.wxiw * challenges.u;

    // B1 = xi * Wxi + u * xi * w * Wxiw + F - E
    let s = challenges.u * challenges.xi * vk.w;
    let b1 = (proof.wxi * challenges.xi) + (proof.wxiw * s) + *f - *e;

    // Check: e(-A1, X_2) * e(B1, G2) == 1
    let neg_a1 = (-a1).into_affine();
    let b1_affine = b1.into_affine();

    let result = Bn254::multi_pairing([neg_a1, b1_affine], [vk.x_2, G2Affine::generator()]);

    result.is_zero()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{ProofJson, VkJson};

    #[test]
    fn test_plonk_verify_valid_proof() {
        let vk_json: VkJson =
            serde_json::from_str(include_str!("../../fixtures/data/verification_key.json"))
                .expect("failed to parse VK");
        let proof_json: ProofJson =
            serde_json::from_str(include_str!("../../fixtures/data/proof.json"))
                .expect("failed to parse proof");
        let public_inputs =
            crate::parse::parse_public_inputs(include_str!("../../fixtures/data/public.json"));

        let vk = vk_json.parse();
        let proof = proof_json.parse();

        assert!(verify(&vk, &proof, &public_inputs), "valid proof rejected");
    }

    #[test]
    fn test_plonk_verify_invalid_public_input() {
        let vk_json: VkJson =
            serde_json::from_str(include_str!("../../fixtures/data/verification_key.json"))
                .expect("failed to parse VK");
        let proof_json: ProofJson =
            serde_json::from_str(include_str!("../../fixtures/data/proof.json"))
                .expect("failed to parse proof");

        let vk = vk_json.parse();
        let proof = proof_json.parse();

        // Wrong public input (34 instead of 33)
        let bad_inputs = vec![Fr::from(34u64)];
        assert!(!verify(&vk, &proof, &bad_inputs), "invalid proof accepted");
    }
}
