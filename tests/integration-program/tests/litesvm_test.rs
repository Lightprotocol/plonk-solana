use litesvm::LiteSVM;
use plonk_solana::Fr;
use serde::Deserialize;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

fn program_id() -> Pubkey {
    Pubkey::new_from_array([
        0x0b, 0x56, 0x00, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89,
        0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45,
        0x67, 0x89, 0xab, 0xcd,
    ])
}

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

/// Build instruction data from public inputs and proof JSON.
fn build_instruction_data(public_inputs: &[Fr], proof: &ProofJson) -> Vec<u8> {
    let mut data = Vec::new();

    // Number of public inputs
    data.push(public_inputs.len() as u8);

    // Public inputs
    for pi in public_inputs {
        data.extend_from_slice(&pi.to_be_bytes());
    }

    // 9 G1 proof points
    for g1 in [
        &proof.a, &proof.b, &proof.c, &proof.z, &proof.t1, &proof.t2, &proof.t3, &proof.wxi,
        &proof.wxiw,
    ] {
        data.extend_from_slice(&parse_g1_be(g1));
    }

    // 6 Fr evaluations
    for eval in [
        &proof.eval_a,
        &proof.eval_b,
        &proof.eval_c,
        &proof.eval_s1,
        &proof.eval_s2,
        &proof.eval_zw,
    ] {
        data.extend_from_slice(&str_to_be32(eval));
    }

    data
}

fn setup_svm() -> (LiteSVM, Keypair) {
    use solana_compute_budget::compute_budget::ComputeBudget;
    let mut budget = ComputeBudget::new_with_defaults(false, false);
    budget.compute_unit_limit = 1_400_000;
    let mut svm = LiteSVM::new().with_compute_budget(budget);

    // Deploy the integration program
    let program_bytes = include_bytes!("../../../target/deploy/integration_program.so");
    svm.add_program(program_id(), program_bytes);

    // Fund payer
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    (svm, payer)
}

#[test]
fn test_plonk_verify_on_chain_valid_proof() {
    let (mut svm, payer) = setup_svm();

    // Load proof and public inputs from test fixtures
    let proof: ProofJson =
        serde_json::from_str(include_str!("../test-fixtures/proof.json")).unwrap();
    let public_vals: Vec<String> =
        serde_json::from_str(include_str!("../test-fixtures/public.json")).unwrap();
    let public_inputs: Vec<Fr> = public_vals
        .iter()
        .map(|s| Fr::from_be_bytes(&str_to_be32(s)))
        .collect();

    let instruction_data = build_instruction_data(&public_inputs, &proof);

    let instruction = Instruction {
        program_id: program_id(),
        accounts: vec![],
        data: instruction_data,
    };

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer], svm.latest_blockhash());

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "valid proof should verify on-chain: {:?}", result.err());
}

#[test]
fn test_plonk_verify_on_chain_invalid_proof() {
    let (mut svm, payer) = setup_svm();

    // Load proof but use wrong public input
    let proof: ProofJson =
        serde_json::from_str(include_str!("../test-fixtures/proof.json")).unwrap();
    let bad_inputs = vec![Fr::from(99u64)]; // wrong value

    let instruction_data = build_instruction_data(&bad_inputs, &proof);

    let instruction = Instruction {
        program_id: program_id(),
        accounts: vec![],
        data: instruction_data,
    };

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer], svm.latest_blockhash());

    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "invalid proof should fail on-chain");
}
