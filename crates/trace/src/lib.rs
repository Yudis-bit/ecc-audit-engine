//! Narrow synthetic leak detection via target calibration callbacks.
//! Not a full binary taint engine — identifies planted branch/table gadgets.

use runner::Target;
use serde::{Deserialize, Serialize};
use target_api::{Confidence, FindingCategory, FindingLevel};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LeakSink {
    SecretDependentBranch,
    SecretDependentMemoryAddress,
    SecretDependentLoopBound,
    SecretDependentIndirectCall,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SourceSet {
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Taint {
    pub value_sources: SourceSet,
    pub address_sources: SourceSet,
    pub control_sources: SourceSet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeakFinding {
    pub id: String,
    pub title: String,
    pub level: FindingLevel,
    pub category: FindingCategory,
    pub sink: LeakSink,
    pub function_name: String,
    pub source_location: String,
    pub instruction_address: Option<String>,
    pub secret_source: String,
    pub calibration_input_hex: String,
    pub raw_evidence: String,
    pub confidence: Confidence,
    pub target_sha256: String,
}

/// Detect planted branch leak: counter increments only when LSB=1.
pub fn detect_branch_leak(target: &dyn Target) -> Option<LeakFinding> {
    let mode = target.leak_mode();
    target.leak_counter_swap(0);

    let mut sk0 = [0u8; 32];
    sk0[31] = 2; // even, valid small key may be rejected if 0; use 2
    let mut sk1 = [0u8; 32];
    sk1[31] = 1; // odd

    let _ = target.pubkey_create(&sk0);
    let c0 = target.leak_counter_swap(0);
    let _ = target.pubkey_create(&sk1);
    let c1 = target.leak_counter_swap(0);

    let evidence = format!("leak_mode={mode}; counter_after_lsb0={c0}; counter_after_lsb1={c1}");

    if c1 > c0 && c1 >= 1 {
        return Some(LeakFinding {
            id: "SYN-BRANCH-001".into(),
            title: "Synthetic secret-dependent branch calibration hit".into(),
            level: FindingLevel::Level2DynamicDivergence,
            category: FindingCategory::SyntheticLeakBranch,
            sink: LeakSink::SecretDependentBranch,
            function_name: "ecc_target_pubkey_create".into(),
            source_location: "harnesses/leaky-target/target.c:LEAK_MODE==1".into(),
            instruction_address: None,
            secret_source: "secret_key[31] LSB".into(),
            calibration_input_hex: format!("sk0={} sk1={}", hex::encode(sk0), hex::encode(sk1)),
            raw_evidence: evidence,
            confidence: Confidence::High,
            target_sha256: target.metadata().sha256.clone(),
        });
    }
    None
}

/// Detect planted table leak: last_table_index equals secret low nibble.
pub fn detect_table_leak(target: &dyn Target) -> Option<LeakFinding> {
    let mode = target.leak_mode();
    let mut sk = [0u8; 32];
    sk[31] = 0x0b;

    let _ = target.pubkey_create(&sk);
    let idx = target.last_table_index();
    let evidence = format!("leak_mode={mode}; last_table_index=0x{idx:x}; expected=0xb");

    if idx == 0x0b {
        return Some(LeakFinding {
            id: "SYN-TABLE-001".into(),
            title: "Synthetic secret-dependent table index calibration hit".into(),
            level: FindingLevel::Level2DynamicDivergence,
            category: FindingCategory::SyntheticLeakTable,
            sink: LeakSink::SecretDependentMemoryAddress,
            function_name: "ecc_target_pubkey_create".into(),
            source_location: "harnesses/leaky-target/target.c:LEAK_MODE==2".into(),
            instruction_address: None,
            secret_source: "secret_key[31] & 0x0f".into(),
            calibration_input_hex: hex::encode(sk),
            raw_evidence: evidence,
            confidence: Confidence::High,
            target_sha256: target.metadata().sha256.clone(),
        });
    }
    None
}

/// Control should not report branch counter discrimination with mode 0.
pub fn control_is_clean(target: &dyn Target) -> bool {
    if target.leak_mode() != 0 {
        return false;
    }
    detect_branch_leak(target).is_none() && {
        // table index should stay 0xff for control
        let mut sk = [0u8; 32];
        sk[31] = 0x0b;
        let _ = target.pubkey_create(&sk);
        target.last_table_index() == 0xff
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runner::DynTarget;
    use std::path::PathBuf;

    fn p(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../targets")
            .join(name)
    }

    #[test]
    fn branch_gadget_detected() {
        let path = p("leaky-branch.so");
        if !path.exists() {
            return;
        }
        let t = DynTarget::load(&path, "leaky-branch").unwrap();
        assert!(detect_branch_leak(&t).is_some());
    }

    #[test]
    fn table_gadget_detected() {
        let path = p("leaky-table.so");
        if !path.exists() {
            return;
        }
        let t = DynTarget::load(&path, "leaky-table").unwrap();
        assert!(detect_table_leak(&t).is_some());
    }

    #[test]
    fn control_clean() {
        let path = p("leaky-control.so");
        if !path.exists() {
            return;
        }
        let t = DynTarget::load(&path, "control").unwrap();
        assert!(control_is_clean(&t));
    }
}
