use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlonkError {
    #[error("Proof verification failed")]
    ProofVerificationFailed,
    #[error("G1 addition failed")]
    G1AdditionFailed,
    #[error("G1 scalar multiplication failed")]
    G1MulFailed,
    #[error("G1 decompression failed")]
    G1DecompressionFailed,
    #[error("G1 compression failed")]
    G1CompressionFailed,
    #[error("Pairing check failed")]
    PairingFailed,
    #[error("Invalid number of public inputs")]
    InvalidPublicInputsLength,
    #[error("Lagrange evaluation division by zero")]
    LagrangeDivisionByZero,
    #[error("Keccak256 hash failed")]
    KeccakFailed,
    #[error("Public input greater than field size")]
    PublicInputGreaterThanFieldSize,
}

impl From<PlonkError> for u32 {
    fn from(error: PlonkError) -> Self {
        match error {
            PlonkError::ProofVerificationFailed => 0,
            PlonkError::G1AdditionFailed => 1,
            PlonkError::G1MulFailed => 2,
            PlonkError::G1DecompressionFailed => 3,
            PlonkError::G1CompressionFailed => 4,
            PlonkError::PairingFailed => 5,
            PlonkError::InvalidPublicInputsLength => 6,
            PlonkError::LagrangeDivisionByZero => 7,
            PlonkError::KeccakFailed => 8,
            PlonkError::PublicInputGreaterThanFieldSize => 9,
        }
    }
}
