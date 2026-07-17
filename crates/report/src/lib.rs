//! Versioned machine-readable and Markdown reports.

use differential::DiffResult;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use target_api::{Confidence, FindingCategory, FindingLevel, MismatchKind, TargetMetadata};
use timing::{TimingSample, TimingStats};
use trace::LeakFinding;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceValue {
    pub kind: String,
    pub hex: Option<String>,
    pub text: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Finding {
    pub schema_version: String,
    pub id: String,
    pub title: String,
    pub level: FindingLevel,
    pub category: FindingCategory,
    pub target: TargetMetadata,
    pub operation: String,
    pub input_case: String,
    pub expected: EvidenceValue,
    pub observed: EvidenceValue,
    pub raw_evidence: Vec<PathBuf>,
    pub minimized_reproducer: Option<PathBuf>,
    pub reproduction_command: String,
    pub false_positive_analysis: String,
    pub impact_boundary: String,
    pub confidence: Confidence,
    pub mismatch: Option<MismatchKind>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReportDocument {
    pub schema_version: String,
    pub generated_at: String,
    pub findings: Vec<Finding>,
    pub leak_findings: Vec<LeakFinding>,
    pub timing: Vec<TimingSection>,
    pub differential_summary: DiffSummary,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimingSection {
    pub target: String,
    pub stats: TimingStats,
    pub sample_count: usize,
    pub samples_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiffSummary {
    pub target: String,
    pub total: usize,
    pub failures: usize,
    pub results_path: PathBuf,
}

pub fn finding_from_diff(
    target: &TargetMetadata,
    r: &DiffResult,
    repro: Option<PathBuf>,
) -> Finding {
    Finding {
        schema_version: "1.0.0".into(),
        id: format!("DIFF-{}", r.case_id.replace('/', "_")),
        title: format!("Differential mismatch: {}", r.case_id),
        level: FindingLevel::Level2DynamicDivergence,
        category: FindingCategory::Differential,
        target: target.clone(),
        operation: r.category.clone(),
        input_case: r.case_id.clone(),
        expected: EvidenceValue {
            kind: "expected".into(),
            hex: r.expected_hex.clone(),
            text: r.expected_error.clone(),
        },
        observed: EvidenceValue {
            kind: "observed".into(),
            hex: r.observed_hex.clone(),
            text: r.observed_error.clone(),
        },
        raw_evidence: Vec::new(),
        minimized_reproducer: repro,
        reproduction_command: format!(
            "cargo run -p cli -- differential --target {} --case {}",
            target.path.display(),
            r.case_id
        ),
        false_positive_analysis:
            "Compare against reference model; confirm target build flags and ABI lengths.".into(),
        impact_boundary:
            "Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.".into(),
        confidence: Confidence::High,
        mismatch: r.mismatch.clone(),
    }
}

pub fn write_report(out_dir: &Path, doc: &ReportDocument) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    std::fs::create_dir_all(out_dir.join("raw"))?;
    std::fs::create_dir_all(out_dir.join("reproducers"))?;
    let json = serde_json::to_string_pretty(doc).unwrap();
    std::fs::write(out_dir.join("report.json"), &json)?;
    std::fs::write(out_dir.join("report.md"), render_md(doc))?;
    Ok(())
}

fn render_md(doc: &ReportDocument) -> String {
    let mut s = String::new();
    s.push_str("# ecc-audit-engine Report\n\n");
    s.push_str(&format!("Generated: {}\n\n", doc.generated_at));
    s.push_str("## Scope\n\nAuthorized local laboratory. Synthetic keys only.\n\n");
    s.push_str("## Differential summary\n\n");
    s.push_str(&format!(
        "- Target: `{}`\n- Total cases: {}\n- Failures: {}\n- Raw: `{}`\n\n",
        doc.differential_summary.target,
        doc.differential_summary.total,
        doc.differential_summary.failures,
        doc.differential_summary.results_path.display()
    ));
    s.push_str("## Findings\n\n");
    for f in &doc.findings {
        s.push_str(&format!(
            "### {} — {}\n\n- Level: {:?}\n- Category: {:?}\n- Case: `{}`\n- Mismatch: {:?}\n- Expected: {:?}\n- Observed: {:?}\n- Impact: {}\n- Repro: `{}`\n\n",
            f.id,
            f.title,
            f.level,
            f.category,
            f.input_case,
            f.mismatch,
            f.expected,
            f.observed,
            f.impact_boundary,
            f.reproduction_command
        ));
    }
    s.push_str("## Synthetic leak calibration\n\n");
    for l in &doc.leak_findings {
        s.push_str(&format!(
            "### {} — {}\n\n- Sink: {:?}\n- Function: `{}`\n- Source: `{}`\n- Evidence: `{}`\n- Target SHA-256: `{}`\n\n",
            l.id, l.title, l.sink, l.function_name, l.source_location, l.raw_evidence, l.target_sha256
        ));
    }
    s.push_str("## Timing\n\n");
    for t in &doc.timing {
        s.push_str(&format!(
            "### {}\n\n- samples: {}\n- mean_a_ns: {:.2}\n- mean_b_ns: {:.2}\n- welch_t: {:.4}\n- cohens_d: {:.4}\n- CI95 mean_diff: [{:.2}, {:.2}]\n- caveats: {:?}\n- raw: `{}`\n\n",
            t.target,
            t.sample_count,
            t.stats.mean_a_ns,
            t.stats.mean_b_ns,
            t.stats.welch_t,
            t.stats.cohens_d,
            t.stats.ci95_low,
            t.stats.ci95_high,
            t.stats.caveats,
            t.samples_path.display()
        ));
    }
    s.push_str("## Notes\n\n");
    for n in &doc.notes {
        s.push_str(&format!("- {n}\n"));
    }
    s.push_str("\n**No real-world secp256k1 vulnerability was tested or confirmed.**\n");
    s
}

pub fn write_samples(path: &Path, samples: &[TimingSample]) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(path, serde_json::to_string(samples).unwrap())
}

pub fn write_env(path: &Path) -> std::io::Result<()> {
    let env = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "family": std::env::consts::FAMILY,
    });
    std::fs::write(path, serde_json::to_string_pretty(&env).unwrap())
}
