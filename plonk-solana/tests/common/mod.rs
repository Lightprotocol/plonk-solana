#![allow(dead_code)]
use num_bigint::BigUint;
use plonk_solana::{Fr, Proof, VerificationKey, G1};
use serde::Deserialize;

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

pub fn str_to_be32(s: &str) -> [u8; 32] {
    let n = BigUint::parse_bytes(s.as_bytes(), 10).unwrap();
    let bytes = n.to_bytes_be();
    let mut result = [0u8; 32];
    let start = 32usize.saturating_sub(bytes.len());
    result[start..].copy_from_slice(&bytes);
    result
}

pub fn parse_g1_be(coords: &[String]) -> G1 {
    if coords[2] == "0" {
        return G1::ZERO;
    }
    let mut result = [0u8; 64];
    result[..32].copy_from_slice(&str_to_be32(&coords[0]));
    result[32..].copy_from_slice(&str_to_be32(&coords[1]));
    G1(result)
}

pub fn load_test_vk() -> VerificationKey {
    plonk_solana::vk_parser::parse_vk_json(include_str!(
        "../../../tests/fixtures/data/verification_key.json"
    ))
    .unwrap()
}

pub fn load_test_proof() -> Proof {
    let p: ProofJson =
        serde_json::from_str(include_str!("../../../tests/fixtures/data/proof.json")).unwrap();
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

pub fn load_test_public_inputs() -> Vec<Fr> {
    let vals: Vec<String> =
        serde_json::from_str(include_str!("../../../tests/fixtures/data/public.json")).unwrap();
    vals.iter()
        .map(|s| Fr::from_be_bytes(&str_to_be32(s)))
        .collect()
}

pub fn load_test_public_inputs_bytes() -> Vec<[u8; 32]> {
    let vals: Vec<String> =
        serde_json::from_str(include_str!("../../../tests/fixtures/data/public.json")).unwrap();
    vals.iter().map(|s| str_to_be32(s)).collect()
}
