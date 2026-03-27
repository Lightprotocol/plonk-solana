use bench_program::PlonkBenchInstruction;
use litesvm::LiteSVM;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;

type BenchResults = BTreeMap<String, BTreeMap<String, Vec<(String, String, String)>>>;

#[test]
fn bench_plonk_operations() {
    let program_path = std::env::var("SBF_OUT_DIR")
        .map(|dir| format!("{}/bench_program.so", dir))
        .unwrap_or_else(|_| "target/deploy/bench_program.so".to_string());

    use solana_compute_budget::compute_budget::ComputeBudget;
    let mut budget = ComputeBudget::new_with_defaults(false, false);
    budget.compute_unit_limit = 1_400_000;
    let mut svm = LiteSVM::new().with_compute_budget(budget);

    let program_id = Pubkey::new_unique();
    svm.add_program_from_file(program_id, &program_path)
        .expect("Failed to load bench program");

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mut results_by_category: BenchResults = BTreeMap::new();

    let instructions = vec![
        PlonkBenchInstruction::Baseline,
        // G1 operations
        PlonkBenchInstruction::G1Add,
        PlonkBenchInstruction::G1Neg,
        PlonkBenchInstruction::G1Mul,
        PlonkBenchInstruction::G1Compress,
        PlonkBenchInstruction::G1Decompress,
        // Fr operations
        PlonkBenchInstruction::FrFromBeBytes,
        PlonkBenchInstruction::FrToBeBytes,
        PlonkBenchInstruction::FrSquare,
        PlonkBenchInstruction::FrInverse,
        PlonkBenchInstruction::FrAdd,
        PlonkBenchInstruction::FrSub,
        PlonkBenchInstruction::FrMul,
        PlonkBenchInstruction::IsLessThanFieldSize,
        // Transcript
        PlonkBenchInstruction::TranscriptGetChallenge,
        PlonkBenchInstruction::CalculateChallenges,
        // Verification steps
        PlonkBenchInstruction::CalculateL1AndPi,
        PlonkBenchInstruction::CalculateR0AndD,
        PlonkBenchInstruction::CalculateF,
        PlonkBenchInstruction::IsValidPairing,
        // Top-level
        PlonkBenchInstruction::Verify,
        PlonkBenchInstruction::VerifyUnchecked,
        PlonkBenchInstruction::ProofCompress,
        PlonkBenchInstruction::ProofDecompress,
    ];

    for instruction_type in instructions.into_iter() {
        let data: Vec<u8> = instruction_type.into();
        let instruction = Instruction {
            program_id,
            accounts: vec![],
            data,
        };

        let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        tx.sign(&[&payer], svm.latest_blockhash());

        let meta = match svm.send_transaction(tx) {
            Ok(meta) => meta,
            Err(e) => {
                eprintln!("Transaction failed for {:?}: {:?}", instruction_type, e);
                continue;
            }
        };

        let logs = meta.pretty_logs();
        println!("{}", logs);

        if let Some((func_name, cu_value, file_location)) = parse_benchmark_log(&meta.logs) {
            let (category, filename) = extract_category_and_file_from_path(&file_location);
            results_by_category
                .entry(category)
                .or_default()
                .entry(filename)
                .or_default()
                .push((func_name, cu_value, file_location));
        }
    }

    write_benchmarks_readme(results_by_category);
    println!("Benchmark results written to BENCHMARKS.md");
}

fn parse_benchmark_log(logs: &[String]) -> Option<(String, String, String)> {
    for log in logs {
        if log.starts_with("Program log:")
            && log.contains("#")
            && log.contains("CU")
            && log.contains("consumed")
        {
            let content = log.strip_prefix("Program log: ").unwrap_or(log);
            let lines: Vec<&str> = content.lines().collect();

            for (i, line) in lines.iter().enumerate() {
                if line.contains("#") && line.contains("    ") {
                    let trimmed = line.trim();
                    if let Some(start) = trimmed.find("    ") {
                        let func_part = trimmed[start..].trim();
                        let parts: Vec<&str> = func_part.split_whitespace().collect();
                        if !parts.is_empty() {
                            let func_name = parts[0].to_string();

                            let mut file_location = String::new();
                            if let Some(location_line) = lines.get(i + 1) {
                                let location_trimmed = location_line.trim();
                                // Profiler emits full path like "tests/bench-program/src/..."
                                // Normalize to "src/..." for category extraction
                                if let Some(pos) = location_trimmed.find("src/") {
                                    file_location = location_trimmed[pos..].to_string();
                                }
                            }

                            let mut cu_value = "N/A".to_string();
                            if let Some(cu_line) = lines.get(i + 2) {
                                if cu_line.contains("CU") && cu_line.contains("consumed") {
                                    if let Some(consumed_pos) = cu_line.find("consumed") {
                                        let after_consumed = cu_line[consumed_pos + 8..].trim();
                                        let parts: Vec<&str> =
                                            after_consumed.split_whitespace().collect();
                                        if !parts.is_empty() {
                                            cu_value = parts[0].to_string();
                                        }
                                    }
                                }
                            }

                            return Some((func_name, cu_value, file_location));
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_category_and_file_from_path(file_location: &str) -> (String, String) {
    if file_location == "src/lib.rs" || file_location.starts_with("src/lib.rs:") {
        return ("baseline".to_string(), "lib".to_string());
    }

    if let Some(without_src) = file_location.strip_prefix("src/") {
        let path_parts: Vec<&str> = without_src.split('/').collect();

        if path_parts.len() >= 2 {
            let folder_name = path_parts[0];
            let file_part = path_parts[1];
            let file_stem = file_part
                .split(':')
                .next()
                .unwrap_or(file_part)
                .trim_end_matches(".rs");
            return (folder_name.to_string(), file_stem.to_string());
        } else if !path_parts.is_empty() {
            let folder_name = path_parts[0];
            let clean_folder = folder_name.split('.').next().unwrap_or(folder_name);
            let clean_folder = clean_folder.split(':').next().unwrap_or(clean_folder);
            return (clean_folder.to_string(), "unknown".to_string());
        }
    }

    ("other".to_string(), "unknown".to_string())
}

fn is_external_api(file_stem: &str) -> bool {
    file_stem == "top_level"
}

fn format_file_display_name(file_stem: &str) -> String {
    file_stem
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn format_cu(cu: u64) -> String {
    let s = cu.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn write_benchmarks_readme(mut results_by_category: BenchResults) {
    let mut readme = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("../../BENCHMARKS.md")
        .expect("Failed to create BENCHMARKS.md");

    writeln!(readme, "# PLONK Verifier CU Benchmarks\n").unwrap();
    writeln!(
        readme,
        "Compute unit benchmarks for PLONK verification on Solana (1 public input).\n"
    )
    .unwrap();

    // Get baseline CU
    let mut baseline_cu: u64 = 0;
    if let Some(baseline_files) = results_by_category.get("baseline") {
        if let Some(first_file_results) = baseline_files.values().next() {
            if let Some((_, cu_str, _)) = first_file_results.first() {
                baseline_cu = cu_str.parse::<u64>().unwrap_or(0);
            }
        }
    }

    writeln!(
        readme,
        "All CU values have baseline profiling overhead ({} CU) subtracted.\n",
        baseline_cu
    )
    .unwrap();

    // Remove baseline from results
    results_by_category.remove("baseline");

    // External API section -- file stems matching is_external_api
    writeln!(readme, "## External API\n").unwrap();
    writeln!(readme, "| Function | CU |").unwrap();
    writeln!(readme, "|----------|-----|").unwrap();
    for files_map in results_by_category.values() {
        for (file_stem, results) in files_map {
            if !is_external_api(file_stem) {
                continue;
            }
            for (func_name, cu_value, file_location) in results {
                let github_link = make_github_link(func_name, file_location);
                let cu = cu_value
                    .parse::<u64>()
                    .unwrap_or(0)
                    .saturating_sub(baseline_cu);
                writeln!(readme, "| {} | {} |", github_link, format_cu(cu)).unwrap();
            }
        }
    }
    writeln!(readme).unwrap();

    // Internal API section -- everything else
    writeln!(readme, "## Internal API\n").unwrap();
    for files_map in results_by_category.values() {
        for (file_stem, results) in files_map {
            if is_external_api(file_stem) {
                continue;
            }
            let display_name = format_file_display_name(file_stem);
            writeln!(readme, "### {}\n", display_name).unwrap();
            writeln!(readme, "| Function | CU |").unwrap();
            writeln!(readme, "|----------|-----|").unwrap();
            for (func_name, cu_value, file_location) in results {
                let github_link = make_github_link(func_name, file_location);
                let cu = cu_value
                    .parse::<u64>()
                    .unwrap_or(0)
                    .saturating_sub(baseline_cu);
                writeln!(readme, "| {} | {} |", github_link, format_cu(cu)).unwrap();
            }
            writeln!(readme).unwrap();
        }
    }
}

fn make_github_link(func_name: &str, file_location: &str) -> String {
    if !file_location.is_empty() {
        let parts: Vec<&str> = file_location.split(':').collect();
        if parts.len() >= 2 {
            let file_path = parts[0];
            let line_num = parts[1].trim().parse::<usize>().unwrap_or(0) + 1;
            return format!(
                "[{}](https://github.com/ananas-block/plonk-solana/blob/main/tests/bench-program/{}#L{})",
                func_name, file_path, line_num
            );
        }
    }
    func_name.to_string()
}
