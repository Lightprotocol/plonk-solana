use pinocchio::error::ProgramError;

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum PlonkBenchInstruction {
    Baseline = 0,
    // G1 Operations (1-9)
    G1Add = 1,
    G1Sub = 2,
    G1Neg = 3,
    G1Mul = 4,
    G1Compress = 5,
    G1Decompress = 6,
    // Fr Operations (10-19)
    FrFromBeBytes = 10,
    FrToBeBytes = 11,
    FrSquare = 12,
    FrInverse = 13,
    FrAdd = 14,
    FrSub = 15,
    FrMul = 16,
    IsLessThanFieldSize = 17,
    // Transcript (20-29)
    TranscriptGetChallenge = 20,
    CalculateChallenges = 21,
    // Verification steps (30-39)
    CalculateL1AndPi = 30,
    CalculateR0AndD = 31,
    CalculateF = 33,
    CalculateE = 34,
    IsValidPairing = 35,
    // Top-level (50-59)
    Verify = 50,
    VerifyUnchecked = 51,
    ProofCompress = 52,
    ProofDecompress = 53,
}

impl From<PlonkBenchInstruction> for Vec<u8> {
    fn from(value: PlonkBenchInstruction) -> Self {
        (value as u16).to_le_bytes().to_vec()
    }
}

impl TryFrom<&[u8]> for PlonkBenchInstruction {
    type Error = ProgramError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 2 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let discriminator = u16::from_le_bytes([value[0], value[1]]);
        match discriminator {
            0 => Ok(PlonkBenchInstruction::Baseline),
            1 => Ok(PlonkBenchInstruction::G1Add),
            2 => Ok(PlonkBenchInstruction::G1Sub),
            3 => Ok(PlonkBenchInstruction::G1Neg),
            4 => Ok(PlonkBenchInstruction::G1Mul),
            5 => Ok(PlonkBenchInstruction::G1Compress),
            6 => Ok(PlonkBenchInstruction::G1Decompress),
            10 => Ok(PlonkBenchInstruction::FrFromBeBytes),
            11 => Ok(PlonkBenchInstruction::FrToBeBytes),
            12 => Ok(PlonkBenchInstruction::FrSquare),
            13 => Ok(PlonkBenchInstruction::FrInverse),
            14 => Ok(PlonkBenchInstruction::FrAdd),
            15 => Ok(PlonkBenchInstruction::FrSub),
            16 => Ok(PlonkBenchInstruction::FrMul),
            17 => Ok(PlonkBenchInstruction::IsLessThanFieldSize),
            20 => Ok(PlonkBenchInstruction::TranscriptGetChallenge),
            21 => Ok(PlonkBenchInstruction::CalculateChallenges),
            30 => Ok(PlonkBenchInstruction::CalculateL1AndPi),
            31 => Ok(PlonkBenchInstruction::CalculateR0AndD),
            33 => Ok(PlonkBenchInstruction::CalculateF),
            34 => Ok(PlonkBenchInstruction::CalculateE),
            35 => Ok(PlonkBenchInstruction::IsValidPairing),
            50 => Ok(PlonkBenchInstruction::Verify),
            51 => Ok(PlonkBenchInstruction::VerifyUnchecked),
            52 => Ok(PlonkBenchInstruction::ProofCompress),
            53 => Ok(PlonkBenchInstruction::ProofDecompress),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
