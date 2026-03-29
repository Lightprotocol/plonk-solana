use integration_program::VerifyInstruction;
use litesvm::LiteSVM;
use plonk_solana::vk_parser;
use plonk_solana::Fr;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

fn program_id() -> Pubkey {
    Pubkey::new_from_array([
        0x0b, 0x56, 0x00, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
        0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89,
        0xab, 0xcd,
    ])
}

fn setup_svm() -> (LiteSVM, Keypair) {
    use solana_compute_budget::compute_budget::ComputeBudget;
    let mut budget = ComputeBudget::new_with_defaults(false, false);
    budget.compute_unit_limit = 1_400_000;
    let mut svm = LiteSVM::new().with_compute_budget(budget);

    let program_bytes = include_bytes!("../../../target/deploy/integration_program.so");
    svm.add_program(program_id(), program_bytes)
        .expect("failed to deploy integration program");

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    (svm, payer)
}

#[allow(clippy::result_large_err)]
fn send_verify_tx(
    svm: &mut LiteSVM,
    payer: &Keypair,
    data: Vec<u8>,
) -> litesvm::types::TransactionResult {
    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![],
        data,
    };
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[payer], svm.latest_blockhash());
    svm.send_transaction(tx)
}

#[test]
fn test_plonk_verify_on_chain_valid_proof() {
    let (mut svm, payer) = setup_svm();

    let proof =
        vk_parser::parse_proof_json(include_str!("../../fixtures/data/proof.json")).unwrap();
    let public_inputs =
        vk_parser::parse_public_inputs_json(include_str!("../../fixtures/data/public.json"))
            .unwrap();

    let ix = VerifyInstruction {
        public_inputs,
        proof,
    };
    let result = send_verify_tx(&mut svm, &payer, borsh::to_vec(&ix).unwrap());
    assert!(
        result.is_ok(),
        "valid proof should verify: {:?}",
        result.err()
    );
}

#[test]
fn test_plonk_verify_on_chain_invalid_proof() {
    let (mut svm, payer) = setup_svm();

    let proof =
        vk_parser::parse_proof_json(include_str!("../../fixtures/data/proof.json")).unwrap();

    let ix = VerifyInstruction {
        public_inputs: vec![Fr::from(99u64)],
        proof,
    };
    let result = send_verify_tx(&mut svm, &payer, borsh::to_vec(&ix).unwrap());
    assert!(result.is_err(), "invalid proof should fail");
}
