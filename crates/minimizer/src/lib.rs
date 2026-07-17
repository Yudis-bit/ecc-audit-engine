//! Delta-style input minimizer for differential mismatches.

use corpus::{CorpusCase, TestInput};
use differential::run_corpus;
use differential::DiffResult;
use runner::Target;
use serde::{Deserialize, Serialize};
use std::path::Path;
use target_api::MismatchKind;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MinimizedCase {
    pub finding_id: String,
    pub original_id: String,
    pub mismatch: MismatchKind,
    pub case: CorpusCase,
}

fn still_fails(target: &dyn Target, case: &CorpusCase, kind: &MismatchKind) -> bool {
    let r = run_corpus(target, std::slice::from_ref(case));
    !r[0].ok && r[0].mismatch.as_ref() == Some(kind)
}

fn clear_bit(buf: &mut [u8], bit: usize) {
    let byte = bit / 8;
    let b = 7 - (bit % 8);
    if byte < buf.len() {
        buf[byte] &= !(1u8 << b);
    }
}

/// Minimize field-pair or secret-key style cases preserving mismatch kind.
pub fn minimize(
    target: &dyn Target,
    original: &CorpusCase,
    result: &DiffResult,
) -> Option<MinimizedCase> {
    let kind = result.mismatch.clone()?;
    let mut best = original.clone();
    if !still_fails(target, &best, &kind) {
        return None;
    }

    match best.input.clone() {
        TestInput::FieldPair { mut a, mut b } => {
            for i in 0..32 {
                let mut ta = a;
                ta[i] = 0;
                let trial = CorpusCase {
                    id: best.id.clone(),
                    category: best.category.clone(),
                    expected_policy: best.expected_policy.clone(),
                    input: TestInput::FieldPair { a: ta, b },
                };
                if still_fails(target, &trial, &kind) {
                    a = ta;
                    best = trial;
                }
                let mut tb = b;
                tb[i] = 0;
                let trial = CorpusCase {
                    id: best.id.clone(),
                    category: best.category.clone(),
                    expected_policy: best.expected_policy.clone(),
                    input: TestInput::FieldPair { a, b: tb },
                };
                if still_fails(target, &trial, &kind) {
                    b = tb;
                    best = trial;
                }
            }
            for bit in 0..256 {
                let mut ta = a;
                clear_bit(&mut ta, bit);
                let trial = CorpusCase {
                    id: best.id.clone(),
                    category: best.category.clone(),
                    expected_policy: best.expected_policy.clone(),
                    input: TestInput::FieldPair { a: ta, b },
                };
                if still_fails(target, &trial, &kind) {
                    a = ta;
                    best = trial;
                }
            }
            best.input = TestInput::FieldPair { a, b };
        }
        TestInput::SecretKey(mut sk) => {
            for i in 0..32 {
                let old = sk[i];
                sk[i] = 0;
                let trial = CorpusCase {
                    id: best.id.clone(),
                    category: best.category.clone(),
                    expected_policy: best.expected_policy.clone(),
                    input: TestInput::SecretKey(sk),
                };
                if still_fails(target, &trial, &kind) {
                    best = trial;
                } else {
                    sk[i] = old;
                }
            }
            best.input = TestInput::SecretKey(sk);
        }
        TestInput::Scalar(mut sk) => {
            for i in 0..32 {
                let old = sk[i];
                sk[i] = 0;
                let trial = CorpusCase {
                    id: best.id.clone(),
                    category: best.category.clone(),
                    expected_policy: best.expected_policy.clone(),
                    input: TestInput::Scalar(sk),
                };
                if still_fails(target, &trial, &kind) {
                    best = trial;
                } else {
                    sk[i] = old;
                }
            }
            best.input = TestInput::Scalar(sk);
        }
        TestInput::PointMul { mut scalar, point } => {
            for i in 0..32 {
                let old = scalar[i];
                scalar[i] = 0;
                let trial = CorpusCase {
                    id: best.id.clone(),
                    category: best.category.clone(),
                    expected_policy: best.expected_policy.clone(),
                    input: TestInput::PointMul {
                        scalar,
                        point: point.clone(),
                    },
                };
                if still_fails(target, &trial, &kind) {
                    best = trial;
                } else {
                    scalar[i] = old;
                }
            }
            best.input = TestInput::PointMul { scalar, point };
        }
        TestInput::PointPair { .. } | TestInput::PointSec1(_) => {}
    }

    best.id = format!("{}::minimized", original.id);
    Some(MinimizedCase {
        finding_id: format!("MIN-{}", original.id.replace('/', "_")),
        original_id: original.id.clone(),
        mismatch: kind,
        case: best,
    })
}

pub fn write_reproducer(path: &Path, m: &MinimizedCase) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(m).unwrap();
    std::fs::write(path, json)
}
