//! Target ABI types shared across runner/differential/report.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const ECC_TARGET_OK: i32 = 0;
pub const ECC_TARGET_REJECT: i32 = 1;
pub const ECC_TARGET_INTERNAL_ERROR: i32 = -1;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TargetError {
    #[error("reject code {0}")]
    Reject(i32),
    #[error("internal error code {0}")]
    Internal(i32),
    #[error("ABI error: {0}")]
    Abi(String),
    #[error("timeout")]
    Timeout,
    #[error("crash: {0}")]
    Crash(String),
    #[error("IO: {0}")]
    Io(String),
    #[error("load: {0}")]
    Load(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetMetadata {
    pub name: String,
    pub path: PathBuf,
    pub sha256: String,
    pub architecture: String,
    pub compiler: Option<String>,
    pub flags: Vec<String>,
}

impl TargetMetadata {
    pub fn from_path(path: &Path, name: impl Into<String>) -> Result<Self, TargetError> {
        let bytes = std::fs::read(path).map_err(|e| TargetError::Io(e.to_string()))?;
        let hash = Sha256::digest(&bytes);
        Ok(Self {
            name: name.into(),
            path: path.to_path_buf(),
            sha256: hex::encode(hash),
            architecture: std::env::consts::ARCH.to_string(),
            compiler: None,
            flags: Vec::new(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MismatchKind {
    ArithmeticMismatch,
    PointMismatch,
    InfinityMismatch,
    OffCurveOutput,
    NonCanonicalOutput,
    UnexpectedAccept,
    UnexpectedReject,
    Crash,
    Timeout,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingLevel {
    Level0Noise,
    Level1StaticSuspicion,
    Level2DynamicDivergence,
    Level3StatisticalLeakage,
    Level4SyntheticInfoBearing,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingCategory {
    Differential,
    SyntheticLeakBranch,
    SyntheticLeakTable,
    Timing,
    Crash,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Confidence {
    Low,
    Medium,
    High,
}
