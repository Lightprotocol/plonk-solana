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

fn format_display_name(folder_name: &str) -> String {
    match folder_name {
        "baseline" => "Baseline".to_string(),
        "g1_ops" => "G1 Operations".to_string(),
        "fr_ops" => "Fr Operations".to_string(),
        "transcript_ops" => "Transcript".to_string(),
        "verification_ops" => "Verification Steps".to_string(),
        "top_level" => "Top Level".to_string(),
        _ => {
            let mut chars = folder_name.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => folder_name.to_string(),
            }
        }
    }
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

fn write_benchmarks_readme(mut results_by_category: BenchResults) {
    let mut readme = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("BENCHMARKS.md")
        .expect("Failed to create BENCHMARKS.md");

    writeln!(readme, "# PLONK Verifier CU Benchmarks\n").unwrap();
    writeln!(
        readme,
        "Compute unit benchmarks for PLONK verification operations on Solana.\n"
    )
    .unwrap();

    // Table of contents
    writeln!(readme, "## Table of Contents\n").unwrap();

    let mut section_number = 1;
    let mut baseline_number = 0;
    if results_by_category.contains_key("baseline") {
        writeln!(
            readme,
            "**[{}. Baseline](#{}-baseline)**\n",
            section_number, section_number
        )
        .unwrap();
        baseline_number = section_number;
        section_number += 1;
    }

    let mut category_numbers = BTreeMap::new();
    for category in results_by_category.keys() {
        if category != "baseline" {
            let display_name = format_display_name(category);
            let anchor = format!("{}-{}", section_number, category.replace('_', "-"));
            writeln!(
                readme,
                "**[{}. {}](#{})**\n",
                section_number, display_name, anchor
            )
            .unwrap();

            if let Some(files_map) = results_by_category.get(category) {
                let mut file_number = 1;
                for file_stem in files_map.keys() {
                    let file_display_name = format_file_display_name(file_stem);
                    let anchor = format!(
                        "{}{}-{}",
                        section_number,
                        file_number,
                        file_display_name.to_lowercase().replace(' ', "-")
                    );
                    writeln!(
                        readme,
                        "  - [{}.{} {}](#{})",
                        section_number, file_number, file_display_name, anchor
                    )
                    .unwrap();
                    file_number += 1;
                }
            }
            writeln!(readme).unwrap();

            category_numbers.insert(category.clone(), section_number);
            section_number += 1;
        }
    }

    writeln!(readme).unwrap();

    // Get baseline CU
    let mut baseline_cu: u64 = 0;
    if let Some(baseline_files) = results_by_category.get("baseline") {
        if let Some(first_file_results) = baseline_files.values().next() {
            if let Some((_, cu_str, _)) = first_file_results.first() {
                baseline_cu = cu_str.parse::<u64>().unwrap_or(0);
            }
        }
    }

    // Definitions
    writeln!(readme, "## Definitions\n").unwrap();
    writeln!(
        readme,
        "- **CU**: Compute units consumed by the operation (baseline profiling overhead of {} CU subtracted)\n",
        baseline_cu
    )
    .unwrap();

    // Write Baseline section
    if let Some(baseline_files) = results_by_category.remove("baseline") {
        writeln!(readme, "## {}. Baseline\n", baseline_number).unwrap();

        let mut file_number = 1;
        for (file_stem, results) in baseline_files {
            let file_display_name = format_file_display_name(&file_stem);
            writeln!(
                readme,
                "### {}.{} {}\n",
                baseline_number, file_number, file_display_name
            )
            .unwrap();

            writeln!(readme, "| Function | CU Consumed | CU |").unwrap();
            writeln!(readme, "|----------|-------------|-----|").unwrap();

            for (func_name, cu_value, file_location) in results {
                let github_link = make_github_link(&func_name, &file_location);
                let cu_consumed = cu_value.parse::<u64>().unwrap_or(0);
                let cu_adjusted = cu_consumed.saturating_sub(baseline_cu);
                writeln!(
                    readme,
                    "| {} | {} | {} |",
                    github_link, cu_value, cu_adjusted
                )
                .unwrap();
            }

            writeln!(readme).unwrap();
            file_number += 1;
        }
    }

    // Write remaining categories
    for (category, files_map) in results_by_category {
        let display_name = format_display_name(&category);
        let number = category_numbers.get(&category).unwrap_or(&0);

        writeln!(readme, "## {}. {}\n", number, display_name).unwrap();

        let mut file_number = 1;
        for (file_stem, results) in files_map {
            let file_display_name = format_file_display_name(&file_stem);
            writeln!(
                readme,
                "### {}.{} {}\n",
                number, file_number, file_display_name
            )
            .unwrap();

            writeln!(readme, "| Function | CU |").unwrap();
            writeln!(readme, "|----------|-----|").unwrap();

            for (func_name, cu_value, file_location) in results {
                let github_link = make_github_link(&func_name, &file_location);
                let cu_consumed = cu_value.parse::<u64>().unwrap_or(0);
                let cu_adjusted = if cu_consumed >= baseline_cu {
                    (cu_consumed - baseline_cu).to_string()
                } else {
                    "0".to_string()
                };

                writeln!(readme, "| {} | {} |", github_link, cu_adjusted).unwrap();
            }

            writeln!(readme).unwrap();
            file_number += 1;
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
