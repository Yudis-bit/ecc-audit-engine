//! ecc-audit-engine CLI — authorized laboratory use only.

use clap::{Parser, Subcommand};
use corpus::{generate_corpus, load_corpus_json, write_corpus_json, CorpusCase};
use differential::{failures, run_corpus};
use minimizer::{minimize, write_reproducer};
use report::{
    finding_from_diff, write_env, write_report, write_samples, DiffSummary, ReportDocument,
    TimingSection,
};
use runner::{DynTarget, Target};
use std::path::{Path, PathBuf};
use timing::run_lsb_experiment;
use trace::{control_is_clean, detect_branch_leak, detect_table_leak};

#[derive(Parser)]
#[command(name = "ecc-audit")]
#[command(about = "secp256k1 implementation-security verification prototype")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run reference model self-test summary
    Model {
        #[command(subcommand)]
        action: ModelCmd,
    },
    /// Corpus tools
    Corpus {
        #[command(subcommand)]
        action: CorpusCmd,
    },
    /// Build C targets via scripts/build_targets.sh
    BuildTargets {
        #[arg(long, default_value = "experiments/build-matrix.toml")]
        matrix: PathBuf,
    },
    /// Differential testing
    Differential {
        #[arg(long)]
        target: PathBuf,
        #[arg(long)]
        corpus: PathBuf,
        #[arg(long)]
        minimize: bool,
        #[arg(long)]
        case: Option<String>,
        #[arg(long, default_value = "reports/latest")]
        output: PathBuf,
    },
    /// Synthetic leak trace calibration
    Trace {
        #[arg(long)]
        target: PathBuf,
        #[arg(long, default_value = "experiments/leaky-branch.toml")]
        experiment: PathBuf,
        #[arg(long, default_value = "reports/latest")]
        output: PathBuf,
    },
    /// Timing experiment
    Timing {
        #[arg(long)]
        target: PathBuf,
        #[arg(long, default_value = "experiments/timing-lsb.toml")]
        experiment: PathBuf,
        #[arg(long, default_value = "reports/latest")]
        output: PathBuf,
        #[arg(long, default_value_t = 200)]
        samples: usize,
        #[arg(long, default_value_t = 50)]
        warmup: usize,
    },
    /// Aggregate report helper
    Report {
        #[arg(long, default_value = "experiments/results")]
        input: PathBuf,
        #[arg(long, default_value = "reports/latest")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum ModelCmd {
    SelfTest,
}

#[derive(Subcommand)]
enum CorpusCmd {
    Generate {
        #[arg(long, default_value_t = 1337)]
        seed: u64,
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Commands::Model {
            action: ModelCmd::SelfTest,
        } => {
            let g = model::AffinePoint::generator();
            assert!(g.is_on_curve());
            let one = model::Scalar::one();
            assert_eq!(g.scalar_mul(&one).unwrap(), g);
            println!("model self-test: OK (generator on-curve, 1*G=G)");
        }
        Commands::Corpus {
            action: CorpusCmd::Generate { seed, output },
        } => {
            if let Some(p) = output.parent() {
                std::fs::create_dir_all(p).ok();
            }
            let n = write_corpus_json(&output, seed).expect("write corpus");
            println!("wrote {n} cases to {}", output.display());
        }
        Commands::BuildTargets { matrix } => {
            println!("matrix manifest: {}", matrix.display());
            let status = std::process::Command::new("bash")
                .arg("scripts/build_targets.sh")
                .status()
                .expect("run build_targets");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        Commands::Differential {
            target,
            corpus,
            minimize,
            case,
            output,
        } => {
            run_differential(&target, &corpus, minimize, case.as_deref(), &output);
        }
        Commands::Trace {
            target,
            experiment,
            output,
        } => {
            let _ = experiment;
            run_trace(&target, &output);
        }
        Commands::Timing {
            target,
            experiment,
            output,
            samples,
            warmup,
        } => {
            let _ = experiment;
            run_timing(&target, &output, samples, warmup);
        }
        Commands::Report { input, output } => {
            println!(
                "report: copy/aggregate from {} -> {} (use differential/trace/timing which write reports)",
                input.display(),
                output.display()
            );
            write_env(&output.join("environment.json")).ok();
        }
    }
}

fn run_differential(
    target_path: &Path,
    corpus_path: &Path,
    do_min: bool,
    only: Option<&str>,
    output: &Path,
) {
    std::fs::create_dir_all(output.join("raw")).ok();
    std::fs::create_dir_all(output.join("reproducers")).ok();
    let target = DynTarget::load(
        target_path,
        target_path.file_name().unwrap().to_string_lossy(),
    )
    .expect("load target");
    let mut cases = if corpus_path.exists() {
        load_corpus_json(corpus_path).expect("load corpus")
    } else {
        generate_corpus(1337)
    };
    if let Some(id) = only {
        cases.retain(|c| c.id == id);
    }
    let results = run_corpus(&target as &dyn Target, &cases);
    let fails = failures(&results);
    println!(
        "differential: {} cases, {} failures, target={}",
        results.len(),
        fails.len(),
        target.metadata().sha256
    );
    let results_path = output.join("raw/differential_results.json");
    std::fs::write(
        &results_path,
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();

    let mut findings = Vec::new();
    for f in &fails {
        println!(
            "FAIL {} {:?} exp={:?} obs={:?}",
            f.case_id, f.mismatch, f.expected_hex, f.observed_hex
        );
        let case = cases.iter().find(|c| c.id == f.case_id);
        let mut repro_path = None;
        if do_min {
            if let Some(c) = case {
                if let Some(m) = minimize(&target as &dyn Target, c, f) {
                    let rp = output.join(format!("reproducers/{}.json", m.finding_id));
                    write_reproducer(&rp, &m).ok();
                    println!("minimized -> {}", rp.display());
                    repro_path = Some(rp);
                }
            }
        }
        findings.push(finding_from_diff(target.metadata(), f, repro_path));
    }

    let mut existing = load_existing_report(output).unwrap_or(ReportDocument {
        schema_version: "1.0.0".into(),
        generated_at: chrono_like_now(),
        findings: Vec::new(),
        leak_findings: Vec::new(),
        timing: Vec::new(),
        differential_summary: DiffSummary {
            target: String::new(),
            total: 0,
            failures: 0,
            results_path: PathBuf::from("n/a"),
        },
        notes: Vec::new(),
    });
    existing.generated_at = chrono_like_now();
    existing.findings.extend(findings);
    existing.differential_summary = DiffSummary {
        target: target.metadata().name.clone(),
        total: results.len(),
        failures: fails.len(),
        results_path,
    };
    existing
        .notes
        .push("Authorized laboratory differential run.".into());
    existing
        .notes
        .push("No real-world secp256k1 vulnerability was tested or confirmed.".into());
    write_report(output, &existing).unwrap();
    write_env(&output.join("environment.json")).ok();
    println!("report -> {}/report.json", output.display());
}

fn load_existing_report(output: &Path) -> Option<ReportDocument> {
    let p = output.join("report.json");
    let data = std::fs::read_to_string(p).ok()?;
    serde_json::from_str(&data).ok()
}

fn run_trace(target_path: &Path, output: &Path) {
    std::fs::create_dir_all(output.join("raw")).ok();
    let target = DynTarget::load(
        target_path,
        target_path.file_name().unwrap().to_string_lossy(),
    )
    .expect("load");
    let mut leaks = Vec::new();
    if let Some(b) = detect_branch_leak(&target as &dyn Target) {
        println!("BRANCH LEAK: {}", b.raw_evidence);
        leaks.push(b);
    } else {
        println!("no branch leak calibration hit");
    }
    if let Some(t) = detect_table_leak(&target as &dyn Target) {
        println!("TABLE LEAK: {}", t.raw_evidence);
        leaks.push(t);
    } else {
        println!("no table leak calibration hit");
    }
    if target.leak_mode() == 0 {
        println!("control_clean={}", control_is_clean(&target as &dyn Target));
    }

    let mut existing = load_existing_report(output).unwrap_or(ReportDocument {
        schema_version: "1.0.0".into(),
        generated_at: chrono_like_now(),
        findings: Vec::new(),
        leak_findings: Vec::new(),
        timing: Vec::new(),
        differential_summary: DiffSummary {
            target: target.metadata().name.clone(),
            total: 0,
            failures: 0,
            results_path: PathBuf::from("n/a"),
        },
        notes: vec!["Synthetic leak calibration only.".into()],
    });
    existing.generated_at = chrono_like_now();
    existing.leak_findings.extend(leaks);
    let path = output.join("raw/leak_findings.json");
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&existing.leak_findings).unwrap(),
    )
    .unwrap();
    write_report(output, &existing).unwrap();
}

fn run_timing(target_path: &Path, output: &Path, samples: usize, warmup: usize) {
    std::fs::create_dir_all(output.join("raw")).ok();
    let target = DynTarget::load(
        target_path,
        target_path.file_name().unwrap().to_string_lossy(),
    )
    .expect("load");
    let (samples_v, stats) = run_lsb_experiment(&target as &dyn Target, 1337, warmup, samples);
    let sp = output.join(format!(
        "raw/timing_samples_{}.json",
        target.metadata().name
    ));
    write_samples(&sp, &samples_v).unwrap();
    println!(
        "timing: nA={} nB={} meanA={:.1}ns meanB={:.1}ns t={:.4} d={:.4}",
        stats.class_a_count,
        stats.class_b_count,
        stats.mean_a_ns,
        stats.mean_b_ns,
        stats.welch_t,
        stats.cohens_d
    );
    for c in &stats.caveats {
        println!("caveat: {c}");
    }
    let mut existing = load_existing_report(output).unwrap_or(ReportDocument {
        schema_version: "1.0.0".into(),
        generated_at: chrono_like_now(),
        findings: Vec::new(),
        leak_findings: Vec::new(),
        timing: Vec::new(),
        differential_summary: DiffSummary {
            target: target.metadata().name.clone(),
            total: 0,
            failures: 0,
            results_path: PathBuf::from("n/a"),
        },
        notes: Vec::new(),
    });
    existing.generated_at = chrono_like_now();
    existing.timing.push(TimingSection {
        target: target.metadata().name.clone(),
        stats,
        sample_count: samples_v.len(),
        samples_path: sp,
    });
    existing
        .notes
        .push("Synthetic timing calibration; not key recovery.".into());
    write_report(output, &existing).unwrap();
}

fn chrono_like_now() -> String {
    // Avoid extra dep: use system time seconds
    use std::time::{SystemTime, UNIX_EPOCH};
    let s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{s}")
}

#[allow(dead_code)]
fn _keep_corpus_type(_: &CorpusCase) {}
