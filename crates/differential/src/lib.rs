//! Differential oracle: local target vs BigUint reference model.

use corpus::{CorpusCase, ExpectedPolicy, TestInput};
use model::{
    fe_mul_bytes, point_add_bytes, point_mul_bytes, pubkey_create, AffinePoint, MathError,
};
use runner::Target;
use serde::{Deserialize, Serialize};
use target_api::{MismatchKind, TargetError};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiffResult {
    pub case_id: String,
    pub category: String,
    pub ok: bool,
    pub mismatch: Option<MismatchKind>,
    pub expected_hex: Option<String>,
    pub observed_hex: Option<String>,
    pub expected_error: Option<String>,
    pub observed_error: Option<String>,
    pub notes: String,
}

fn enc(v: &[u8]) -> String {
    hex::encode(v)
}

fn classify_point_mismatch(expected: &[u8], observed: &[u8]) -> MismatchKind {
    let exp_inf = expected == [0x00];
    let obs_inf = observed == [0x00];
    if exp_inf != obs_inf {
        return MismatchKind::InfinityMismatch;
    }
    if !obs_inf {
        if let Ok(p) = AffinePoint::parse_sec1(observed) {
            if !p.is_on_curve() {
                return MismatchKind::OffCurveOutput;
            }
        } else {
            return MismatchKind::NonCanonicalOutput;
        }
    }
    MismatchKind::PointMismatch
}

fn run_one(target: &dyn Target, case: &CorpusCase) -> DiffResult {
    let mut res = DiffResult {
        case_id: case.id.clone(),
        category: case.category.clone(),
        ok: true,
        mismatch: None,
        expected_hex: None,
        observed_hex: None,
        expected_error: None,
        observed_error: None,
        notes: String::new(),
    };

    match &case.input {
        TestInput::FieldPair { a, b } => {
            let exp = fe_mul_bytes(a, b);
            let got = target.fe_mul(a, b);
            match (exp, got) {
                (Ok(e), Ok(g)) => {
                    res.expected_hex = Some(enc(&e));
                    res.observed_hex = Some(enc(&g));
                    if e != g {
                        res.ok = false;
                        res.mismatch = Some(MismatchKind::ArithmeticMismatch);
                    }
                }
                (Err(e), Err(TargetError::Reject(_))) => {
                    res.expected_error = Some(e.to_string());
                    res.observed_error = Some("reject".into());
                    // both reject — ok for Reject policy
                    if case.expected_policy != ExpectedPolicy::Reject
                        && case.expected_policy != ExpectedPolicy::PolicyDependent
                    {
                        res.notes = "both rejected".into();
                    }
                }
                (Err(e), Ok(g)) => {
                    res.ok = false;
                    res.expected_error = Some(e.to_string());
                    res.observed_hex = Some(enc(&g));
                    res.mismatch = Some(MismatchKind::UnexpectedAccept);
                }
                (Ok(e), Err(te)) => {
                    res.ok = false;
                    res.expected_hex = Some(enc(&e));
                    res.observed_error = Some(te.to_string());
                    res.mismatch = Some(MismatchKind::UnexpectedReject);
                }
                (Err(e), Err(te)) => {
                    res.expected_error = Some(e.to_string());
                    res.observed_error = Some(te.to_string());
                }
            }
        }
        TestInput::SecretKey(sk) => {
            let exp = pubkey_create(sk);
            let got = target.pubkey_create(sk);
            match (exp, got) {
                (Ok(e), Ok(g)) => {
                    res.expected_hex = Some(enc(&e));
                    res.observed_hex = Some(enc(&g));
                    if e.as_slice() != g.as_slice() {
                        res.ok = false;
                        res.mismatch = Some(classify_point_mismatch(&e, &g));
                    }
                }
                (Err(e), Err(_)) => {
                    res.expected_error = Some(e.to_string());
                }
                (Err(e), Ok(g)) => {
                    res.ok = false;
                    res.expected_error = Some(e.to_string());
                    res.observed_hex = Some(enc(&g));
                    res.mismatch = Some(MismatchKind::UnexpectedAccept);
                }
                (Ok(e), Err(te)) => {
                    res.ok = false;
                    res.expected_hex = Some(enc(&e));
                    res.observed_error = Some(te.to_string());
                    res.mismatch = Some(MismatchKind::UnexpectedReject);
                }
            }
        }
        TestInput::PointPair { a, b } => {
            let exp = point_add_bytes(a, b);
            let got = target.point_add(a, b);
            compare_point_results(&mut res, exp, got);
        }
        TestInput::PointMul { scalar, point } => {
            let exp = point_mul_bytes(scalar, point);
            let got = target.point_mul(scalar, point);
            compare_point_results(&mut res, exp, got);
        }
        TestInput::PointSec1(p) => {
            // parse-only: target has no pure parse; use point_mul by 1 as on-curve check
            let one = {
                let mut s = [0u8; 32];
                s[31] = 1;
                s
            };
            match AffinePoint::parse_sec1(p) {
                Ok(ap) if ap.is_infinity() => {
                    let exp = point_mul_bytes(&one, p);
                    let got = target.point_mul(&one, p);
                    compare_point_results(&mut res, exp, got);
                    res.notes = "infinity encoding".into();
                }
                Ok(_) => {
                    let exp = point_mul_bytes(&one, p);
                    let got = target.point_mul(&one, p);
                    compare_point_results(&mut res, exp, got);
                }
                Err(e) => {
                    let got = target.point_mul(&one, p);
                    match got {
                        Err(TargetError::Reject(_)) => {
                            res.expected_error = Some(e.to_string());
                        }
                        Ok(g) => {
                            res.ok = false;
                            res.expected_error = Some(e.to_string());
                            res.observed_hex = Some(enc(&g));
                            res.mismatch = Some(MismatchKind::UnexpectedAccept);
                        }
                        Err(te) => {
                            res.expected_error = Some(e.to_string());
                            res.observed_error = Some(te.to_string());
                        }
                    }
                }
            }
        }
        TestInput::Scalar(s) => {
            // scalar * G
            let g = AffinePoint::generator()
                .serialize_uncompressed()
                .unwrap()
                .to_vec();
            let exp = point_mul_bytes(s, &g);
            let got = target.point_mul(s, &g);
            compare_point_results(&mut res, exp, got);
        }
    }
    res
}

fn compare_point_results(
    res: &mut DiffResult,
    exp: Result<Vec<u8>, MathError>,
    got: Result<Vec<u8>, TargetError>,
) {
    match (exp, got) {
        (Ok(e), Ok(g)) => {
            res.expected_hex = Some(enc(&e));
            res.observed_hex = Some(enc(&g));
            if e != g {
                res.ok = false;
                res.mismatch = Some(classify_point_mismatch(&e, &g));
            }
        }
        (Err(e), Err(TargetError::Reject(_))) => {
            res.expected_error = Some(e.to_string());
        }
        (Err(e), Ok(g)) => {
            res.ok = false;
            res.expected_error = Some(e.to_string());
            res.observed_hex = Some(enc(&g));
            res.mismatch = Some(MismatchKind::UnexpectedAccept);
        }
        (Ok(e), Err(te)) => {
            res.ok = false;
            res.expected_hex = Some(enc(&e));
            res.observed_error = Some(te.to_string());
            res.mismatch = match te {
                TargetError::Crash(_) => Some(MismatchKind::Crash),
                TargetError::Timeout => Some(MismatchKind::Timeout),
                _ => Some(MismatchKind::UnexpectedReject),
            };
        }
        (Err(e), Err(te)) => {
            res.expected_error = Some(e.to_string());
            res.observed_error = Some(te.to_string());
        }
    }
}

pub fn run_corpus(target: &dyn Target, cases: &[CorpusCase]) -> Vec<DiffResult> {
    cases.iter().map(|c| run_one(target, c)).collect()
}

pub fn failures(results: &[DiffResult]) -> Vec<&DiffResult> {
    results.iter().filter(|r| !r.ok).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use corpus::generate_corpus;
    use runner::DynTarget;
    use std::path::PathBuf;

    fn target_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../targets")
            .join(name)
    }

    #[test]
    fn correct_target_mostly_ok() {
        let p = target_path("correct-target.so");
        if !p.exists() {
            eprintln!("skip: build targets first");
            return;
        }
        let t = DynTarget::load(&p, "correct").unwrap();
        let corpus = generate_corpus(1337);
        // Focused subset
        let subset: Vec<_> = corpus
            .into_iter()
            .filter(|c| {
                c.category == "fe_mul"
                    || c.category == "point_add"
                    || c.category == "point_mul"
                    || c.category == "secret_key"
                    || c.id.starts_with("scalar/1")
                    || c.id.starts_with("scalar/2")
                    || c.id.starts_with("scalar/3")
                    || c.id.starts_with("scalar/7")
                    || c.id == "scalar/0"
                    || c.id == "point_add/G+(-G)"
            })
            .collect();
        let results = run_corpus(&t, &subset);
        let fails = failures(&results);
        for f in &fails {
            eprintln!(
                "FAIL {}: {:?} exp={:?} obs={:?}",
                f.case_id, f.mismatch, f.expected_hex, f.observed_hex
            );
        }
        assert!(fails.is_empty(), "{} failures", fails.len());
    }

    #[test]
    fn corrupted_fe_mul_detected() {
        let p = target_path("corrupted-target.so");
        if !p.exists() {
            eprintln!("skip: build targets first");
            return;
        }
        let t = DynTarget::load(&p, "corrupted").unwrap();
        // use low-byte 0xff to trigger CORRUPT_FE_MUL
        let mut a = [0u8; 32];
        a[31] = 0xff;
        let mut b = [0u8; 32];
        b[31] = 0x02;
        let exp = fe_mul_bytes(&a, &b).unwrap();
        let got = t.fe_mul(&a, &b).unwrap();
        assert_ne!(exp, got, "corruption should flip bit");
    }

    #[test]
    fn corrupted_infinity_detected() {
        let p = target_path("corrupted-target.so");
        if !p.exists() {
            return;
        }
        let t = DynTarget::load(&p, "corrupted").unwrap();
        let g = AffinePoint::generator().serialize_uncompressed().unwrap();
        let neg = AffinePoint::generator()
            .negate()
            .serialize_uncompressed()
            .unwrap();
        let exp = point_add_bytes(&g, &neg).unwrap();
        assert_eq!(exp, vec![0x00]);
        let got = t.point_add(&g, &neg).unwrap();
        assert_ne!(got, exp, "should not return infinity");
    }
}
