//! Verification key parser for build.rs usage.
//!
//! Parses snarkjs PLONK verification key JSON and generates Rust source code
//! with pre-computed byte constants. This avoids runtime JSON parsing on-chain.
//!
//! # Example
//!
//! ```rust,ignore
//! // In build.rs
//! use plonk_solana::vk_parser::generate_vk_file;
//!
//! fn main() {
//!     let out_dir = std::env::var("OUT_DIR").unwrap();
//!     generate_vk_file(
//!         "verification_key.json",
//!         &out_dir,
//!         "verifying_key.rs",
//!     ).unwrap();
//! }
//! ```
use num_bigint::BigUint;
use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::fr::Fr;
use crate::g1::G1;
use crate::g2::G2;
use crate::plonk::VerificationKey;

#[derive(Debug, thiserror::Error)]
pub enum VkParseError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid verification key data: {0}")]
    InvalidData(String),
}

#[derive(Deserialize)]
struct RawVk {
    #[serde(rename = "nPublic")]
    n_public: usize,
    power: u32,
    k1: String,
    k2: String,
    w: String,
    #[serde(rename = "Qm")]
    qm: Vec<String>,
    #[serde(rename = "Ql")]
    ql: Vec<String>,
    #[serde(rename = "Qr")]
    qr: Vec<String>,
    #[serde(rename = "Qo")]
    qo: Vec<String>,
    #[serde(rename = "Qc")]
    qc: Vec<String>,
    #[serde(rename = "S1")]
    s1: Vec<String>,
    #[serde(rename = "S2")]
    s2: Vec<String>,
    #[serde(rename = "S3")]
    s3: Vec<String>,
    #[serde(rename = "X_2")]
    x_2: Vec<Vec<String>>,
}

fn bigint_to_be32(s: &str) -> Result<[u8; 32], VkParseError> {
    let n = s
        .parse::<BigUint>()
        .map_err(|e| VkParseError::InvalidData(format!("invalid bigint '{}': {}", s, e)))?;
    let bytes = n.to_bytes_be();
    if bytes.len() > 32 {
        return Err(VkParseError::InvalidData(format!(
            "value too large for 32 bytes: {}",
            s
        )));
    }
    let mut result = [0u8; 32];
    let start = 32 - bytes.len();
    result[start..].copy_from_slice(&bytes);
    Ok(result)
}

fn parse_g1(coords: &[String]) -> Result<G1, VkParseError> {
    if coords.len() != 3 {
        return Err(VkParseError::InvalidData(
            "G1 point must have 3 coordinates".into(),
        ));
    }
    if coords[2] == "0" {
        return Ok(G1::ZERO);
    }
    let mut result = [0u8; 64];
    result[..32].copy_from_slice(&bigint_to_be32(&coords[0])?);
    result[32..].copy_from_slice(&bigint_to_be32(&coords[1])?);
    Ok(G1(result))
}

/// Parse G2 in EIP-197 big-endian order: x1, x0, y1, y0
fn parse_g2(coords: &[Vec<String>]) -> Result<G2, VkParseError> {
    if coords.len() != 3 {
        return Err(VkParseError::InvalidData(
            "G2 point must have 3 coordinate pairs".into(),
        ));
    }
    let mut result = [0u8; 128];
    result[..32].copy_from_slice(&bigint_to_be32(&coords[0][1])?);
    result[32..64].copy_from_slice(&bigint_to_be32(&coords[0][0])?);
    result[64..96].copy_from_slice(&bigint_to_be32(&coords[1][1])?);
    result[96..128].copy_from_slice(&bigint_to_be32(&coords[1][0])?);
    Ok(G2(result))
}

/// Parse a snarkjs PLONK verification key JSON string into a `VerificationKey`.
pub fn parse_vk_json(json_content: &str) -> Result<VerificationKey, VkParseError> {
    let raw: RawVk = serde_json::from_str(json_content)?;
    Ok(VerificationKey {
        n_public: raw.n_public,
        power: raw.power,
        k1: Fr::from_be_bytes(&bigint_to_be32(&raw.k1)?),
        k2: Fr::from_be_bytes(&bigint_to_be32(&raw.k2)?),
        w: Fr::from_be_bytes(&bigint_to_be32(&raw.w)?),
        qm: parse_g1(&raw.qm)?,
        ql: parse_g1(&raw.ql)?,
        qr: parse_g1(&raw.qr)?,
        qo: parse_g1(&raw.qo)?,
        qc: parse_g1(&raw.qc)?,
        s1: parse_g1(&raw.s1)?,
        s2: parse_g1(&raw.s2)?,
        s3: parse_g1(&raw.s3)?,
        x_2: parse_g2(&raw.x_2)?,
    })
}

fn format_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("0x{:02x}", b))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_g1(point: &G1) -> String {
    format!("G1([{}])", format_bytes(&point.0))
}

fn format_g2(point: &G2) -> String {
    format!("G2([{}])", format_bytes(&point.0))
}

fn format_fr(bytes: &[u8; 32]) -> String {
    format!("Fr::from_be_bytes(&[{}])", format_bytes(bytes))
}

/// Parse a snarkjs PLONK verification key JSON and generate Rust source code.
///
/// The output is a Rust file defining a `pub fn verifying_key() -> VerificationKey` function
/// with all byte values pre-computed.
pub fn parse_vk_json_to_rust_string(json_content: &str) -> Result<String, VkParseError> {
    let raw: RawVk = serde_json::from_str(json_content)?;

    let k1 = bigint_to_be32(&raw.k1)?;
    let k2 = bigint_to_be32(&raw.k2)?;
    let w = bigint_to_be32(&raw.w)?;
    let qm = parse_g1(&raw.qm)?;
    let ql = parse_g1(&raw.ql)?;
    let qr = parse_g1(&raw.qr)?;
    let qo = parse_g1(&raw.qo)?;
    let qc = parse_g1(&raw.qc)?;
    let s1 = parse_g1(&raw.s1)?;
    let s2 = parse_g1(&raw.s2)?;
    let s3 = parse_g1(&raw.s3)?;
    let x_2 = parse_g2(&raw.x_2)?;

    let mut out = String::new();
    out.push_str("pub fn verifying_key() -> plonk_solana::VerificationKey {\n");
    out.push_str("    use plonk_solana::{VerificationKey, Fr, G1, G2};\n");
    out.push_str("    VerificationKey {\n");
    out.push_str(&format!("        n_public: {},\n", raw.n_public));
    out.push_str(&format!("        power: {},\n", raw.power));
    out.push_str(&format!("        k1: {},\n", format_fr(&k1)));
    out.push_str(&format!("        k2: {},\n", format_fr(&k2)));
    out.push_str(&format!("        w: {},\n", format_fr(&w)));
    out.push_str(&format!("        qm: {},\n", format_g1(&qm)));
    out.push_str(&format!("        ql: {},\n", format_g1(&ql)));
    out.push_str(&format!("        qr: {},\n", format_g1(&qr)));
    out.push_str(&format!("        qo: {},\n", format_g1(&qo)));
    out.push_str(&format!("        qc: {},\n", format_g1(&qc)));
    out.push_str(&format!("        s1: {},\n", format_g1(&s1)));
    out.push_str(&format!("        s2: {},\n", format_g1(&s2)));
    out.push_str(&format!("        s3: {},\n", format_g1(&s3)));
    out.push_str(&format!("        x_2: {},\n", format_g2(&x_2)));
    out.push_str("    }\n");
    out.push_str("}\n");

    Ok(out)
}

/// Parse a snarkjs PLONK proof JSON string into a `Proof`.
pub fn parse_proof_json(json_content: &str) -> Result<crate::plonk::Proof, VkParseError> {
    #[derive(Deserialize)]
    struct RawProof {
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

    let raw: RawProof = serde_json::from_str(json_content)?;
    Ok(crate::plonk::Proof {
        a: parse_g1(&raw.a)?,
        b: parse_g1(&raw.b)?,
        c: parse_g1(&raw.c)?,
        z: parse_g1(&raw.z)?,
        t1: parse_g1(&raw.t1)?,
        t2: parse_g1(&raw.t2)?,
        t3: parse_g1(&raw.t3)?,
        wxi: parse_g1(&raw.wxi)?,
        wxiw: parse_g1(&raw.wxiw)?,

        eval_a: Fr::from_be_bytes(&bigint_to_be32(&raw.eval_a)?),
        eval_b: Fr::from_be_bytes(&bigint_to_be32(&raw.eval_b)?),
        eval_c: Fr::from_be_bytes(&bigint_to_be32(&raw.eval_c)?),
        eval_s1: Fr::from_be_bytes(&bigint_to_be32(&raw.eval_s1)?),
        eval_s2: Fr::from_be_bytes(&bigint_to_be32(&raw.eval_s2)?),
        eval_zw: Fr::from_be_bytes(&bigint_to_be32(&raw.eval_zw)?),
    })
}

/// Parse snarkjs public inputs JSON (array of decimal strings) into `Vec<Fr>`.
pub fn parse_public_inputs_json(json_content: &str) -> Result<Vec<Fr>, VkParseError> {
    let vals: Vec<String> = serde_json::from_str(json_content)?;
    vals.iter()
        .map(|s| Ok(Fr::from_be_bytes(&bigint_to_be32(s)?)))
        .collect()
}

/// Generate a verification key Rust file from a JSON file.
///
/// Reads the JSON, generates Rust source code, and writes it to disk.
/// Intended for use in `build.rs`.
pub fn generate_vk_file(
    json_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
    output_filename: &str,
) -> Result<(), VkParseError> {
    let json_content = fs::read_to_string(json_path.as_ref())?;
    let rust_code = parse_vk_json_to_rust_string(&json_content)?;
    fs::create_dir_all(output_dir.as_ref())?;
    let output_path = output_dir.as_ref().join(output_filename);
    fs::write(output_path, rust_code)?;
    Ok(())
}
