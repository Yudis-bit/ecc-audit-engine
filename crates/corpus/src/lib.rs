//! Deterministic structured corpus for secp256k1 differential testing.

use model::{field_prime, group_order, AffinePoint, FieldElement, Scalar};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExpectedPolicy {
    /// Must produce a valid mathematical result
    AcceptValid,
    /// Must reject (parse/policy)
    Reject,
    /// Infinity result expected
    AcceptInfinity,
    /// Implementation-defined (document comparison carefully)
    PolicyDependent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestInput {
    Scalar([u8; 32]),
    FieldPair { a: [u8; 32], b: [u8; 32] },
    PointSec1(Vec<u8>),
    PointPair { a: Vec<u8>, b: Vec<u8> },
    PointMul { scalar: [u8; 32], point: Vec<u8> },
    SecretKey([u8; 32]),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorpusCase {
    pub id: String,
    pub category: String,
    pub expected_policy: ExpectedPolicy,
    pub input: TestInput,
}

fn be32(v: &BigUint) -> [u8; 32] {
    let bytes = v.to_bytes_be();
    let mut out = [0u8; 32];
    if bytes.len() > 32 {
        out.copy_from_slice(&bytes[bytes.len() - 32..]);
    } else {
        out[32 - bytes.len()..].copy_from_slice(&bytes);
    }
    out
}

fn scalar_bytes(v: u64) -> [u8; 32] {
    be32(&BigUint::from(v))
}

/// Generate reproducible corpus from fixed seed.
pub fn generate_corpus(seed: u64) -> Vec<CorpusCase> {
    let mut cases = Vec::new();
    let n = group_order();
    let p = field_prime();
    let g = AffinePoint::generator();
    let g_u = g.serialize_uncompressed().expect("G");
    let neg_g = g.negate().serialize_uncompressed().expect("-G");
    let g2 = g
        .scalar_mul(&Scalar::new(BigUint::from(2u32)).unwrap())
        .unwrap()
        .serialize_uncompressed()
        .unwrap();
    let g3 = g
        .scalar_mul(&Scalar::new(BigUint::from(3u32)).unwrap())
        .unwrap()
        .serialize_uncompressed()
        .unwrap();

    // --- scalars ---
    let scalar_vals: Vec<(&str, BigUint, ExpectedPolicy)> = vec![
        ("scalar/0", BigUint::zero(), ExpectedPolicy::AcceptInfinity),
        ("scalar/1", BigUint::one(), ExpectedPolicy::AcceptValid),
        ("scalar/2", BigUint::from(2u32), ExpectedPolicy::AcceptValid),
        ("scalar/3", BigUint::from(3u32), ExpectedPolicy::AcceptValid),
        ("scalar/7", BigUint::from(7u32), ExpectedPolicy::AcceptValid),
        (
            "scalar/n-1",
            &n - BigUint::one(),
            ExpectedPolicy::AcceptValid,
        ),
        ("scalar/n", n.clone(), ExpectedPolicy::AcceptInfinity), // reduces to 0
        (
            "scalar/n+1",
            &n + BigUint::one(),
            ExpectedPolicy::AcceptValid,
        ),
        (
            "scalar/2^256-1",
            (BigUint::one() << 256) - BigUint::one(),
            ExpectedPolicy::PolicyDependent,
        ),
        (
            "scalar/0xAA..",
            {
                let b = [0xAAu8; 32];
                BigUint::from_bytes_be(&b)
            },
            ExpectedPolicy::PolicyDependent,
        ),
        (
            "scalar/0x55..",
            {
                let b = [0x55u8; 32];
                BigUint::from_bytes_be(&b)
            },
            ExpectedPolicy::PolicyDependent,
        ),
    ];

    for (id, v, pol) in scalar_vals {
        cases.push(CorpusCase {
            id: id.to_string(),
            category: "scalar".into(),
            expected_policy: pol,
            input: TestInput::Scalar(be32(&v)),
        });
        // also as secret key when in [1,n)
        if !v.is_zero() && v < n {
            cases.push(CorpusCase {
                id: format!("{id}/as_sk"),
                category: "secret_key".into(),
                expected_policy: ExpectedPolicy::AcceptValid,
                input: TestInput::SecretKey(be32(&v)),
            });
        }
    }

    for i in 0u32..16 {
        let pow = BigUint::one() << i;
        cases.push(CorpusCase {
            id: format!("scalar/2^{i}"),
            category: "scalar_pow2".into(),
            expected_policy: ExpectedPolicy::AcceptValid,
            input: TestInput::Scalar(be32(&pow)),
        });
        if i > 0 {
            cases.push(CorpusCase {
                id: format!("scalar/2^{i}-1"),
                category: "scalar_pow2".into(),
                expected_policy: ExpectedPolicy::AcceptValid,
                input: TestInput::Scalar(be32(&(&pow - BigUint::one()))),
            });
            cases.push(CorpusCase {
                id: format!("scalar/2^{i}+1"),
                category: "scalar_pow2".into(),
                expected_policy: if &pow + BigUint::one() < n {
                    ExpectedPolicy::AcceptValid
                } else {
                    ExpectedPolicy::PolicyDependent
                },
                input: TestInput::Scalar(be32(&(&pow + BigUint::one()))),
            });
        }
    }

    // low HW
    for bit in [0u32, 1, 7, 31, 63, 127, 200, 255] {
        let v = BigUint::one() << bit;
        if v < n {
            cases.push(CorpusCase {
                id: format!("scalar/low_hw_bit_{bit}"),
                category: "scalar_low_hw".into(),
                expected_policy: ExpectedPolicy::AcceptValid,
                input: TestInput::Scalar(be32(&v)),
            });
        }
    }

    // high HW: n-2, many ones below n
    let high = &n - BigUint::from(2u32);
    cases.push(CorpusCase {
        id: "scalar/high_hw_n-2".into(),
        category: "scalar_high_hw".into(),
        expected_policy: ExpectedPolicy::AcceptValid,
        input: TestInput::Scalar(be32(&high)),
    });

    // repeated windows
    let mut rep4 = [0u8; 32];
    rep4.fill(0x12); // 0001_0010 pattern
    cases.push(CorpusCase {
        id: "scalar/rep_4bit_window".into(),
        category: "scalar_window".into(),
        expected_policy: ExpectedPolicy::PolicyDependent,
        input: TestInput::Scalar(rep4),
    });
    let mut rep5 = [0u8; 32];
    for (i, b) in rep5.iter_mut().enumerate() {
        *b = if i % 2 == 0 { 0xF8 } else { 0x1F };
    }
    cases.push(CorpusCase {
        id: "scalar/rep_5bit_window".into(),
        category: "scalar_window".into(),
        expected_policy: ExpectedPolicy::PolicyDependent,
        input: TestInput::Scalar(rep5),
    });

    // carry-heavy
    cases.push(CorpusCase {
        id: "scalar/all_ff".into(),
        category: "scalar_carry".into(),
        expected_policy: ExpectedPolicy::PolicyDependent,
        input: TestInput::Scalar([0xff; 32]),
    });

    // --- field pairs ---
    let fe_vals = [
        ("0", BigUint::zero()),
        ("1", BigUint::one()),
        ("2", BigUint::from(2u32)),
        ("p-1", &p - BigUint::one()),
        ("carry", (BigUint::one() << 26) - BigUint::one()),
        ("near_mid", BigUint::one() << 128),
    ];
    for (ia, a) in &fe_vals {
        for (ib, b) in &fe_vals {
            cases.push(CorpusCase {
                id: format!("fe_mul/{ia}_x_{ib}"),
                category: "fe_mul".into(),
                expected_policy: ExpectedPolicy::AcceptValid,
                input: TestInput::FieldPair {
                    a: be32(a),
                    b: be32(b),
                },
            });
        }
    }
    // non-canonical p rejected by reference
    cases.push(CorpusCase {
        id: "fe_mul/noncanonical_p".into(),
        category: "fe_mul_reject".into(),
        expected_policy: ExpectedPolicy::Reject,
        input: TestInput::FieldPair {
            a: be32(&p),
            b: be32(&BigUint::one()),
        },
    });

    // --- points ---
    cases.push(CorpusCase {
        id: "point/infinity".into(),
        category: "point".into(),
        expected_policy: ExpectedPolicy::AcceptInfinity,
        input: TestInput::PointSec1(vec![0x00]),
    });
    cases.push(CorpusCase {
        id: "point/G".into(),
        category: "point".into(),
        expected_policy: ExpectedPolicy::AcceptValid,
        input: TestInput::PointSec1(g_u.to_vec()),
    });
    cases.push(CorpusCase {
        id: "point/-G".into(),
        category: "point".into(),
        expected_policy: ExpectedPolicy::AcceptValid,
        input: TestInput::PointSec1(neg_g.to_vec()),
    });
    cases.push(CorpusCase {
        id: "point/2G".into(),
        category: "point".into(),
        expected_policy: ExpectedPolicy::AcceptValid,
        input: TestInput::PointSec1(g2.to_vec()),
    });
    cases.push(CorpusCase {
        id: "point/3G".into(),
        category: "point".into(),
        expected_policy: ExpectedPolicy::AcceptValid,
        input: TestInput::PointSec1(g3.to_vec()),
    });

    // point pairs for add
    cases.push(CorpusCase {
        id: "point_add/G+inf".into(),
        category: "point_add".into(),
        expected_policy: ExpectedPolicy::AcceptValid,
        input: TestInput::PointPair {
            a: g_u.to_vec(),
            b: vec![0x00],
        },
    });
    cases.push(CorpusCase {
        id: "point_add/G+(-G)".into(),
        category: "point_add".into(),
        expected_policy: ExpectedPolicy::AcceptInfinity,
        input: TestInput::PointPair {
            a: g_u.to_vec(),
            b: neg_g.to_vec(),
        },
    });
    cases.push(CorpusCase {
        id: "point_add/G+G".into(),
        category: "point_add".into(),
        expected_policy: ExpectedPolicy::AcceptValid,
        input: TestInput::PointPair {
            a: g_u.to_vec(),
            b: g_u.to_vec(),
        },
    });
    cases.push(CorpusCase {
        id: "point_add/2G+G".into(),
        category: "point_add".into(),
        expected_policy: ExpectedPolicy::AcceptValid,
        input: TestInput::PointPair {
            a: g2.to_vec(),
            b: g_u.to_vec(),
        },
    });

    // point mul
    for k in [0u64, 1, 2, 3, 7] {
        cases.push(CorpusCase {
            id: format!("point_mul/{k}*G"),
            category: "point_mul".into(),
            expected_policy: if k == 0 {
                ExpectedPolicy::AcceptInfinity
            } else {
                ExpectedPolicy::AcceptValid
            },
            input: TestInput::PointMul {
                scalar: scalar_bytes(k),
                point: g_u.to_vec(),
            },
        });
    }
    cases.push(CorpusCase {
        id: "point_mul/(n-1)*G".into(),
        category: "point_mul".into(),
        expected_policy: ExpectedPolicy::AcceptValid,
        input: TestInput::PointMul {
            scalar: be32(&(&n - BigUint::one())),
            point: g_u.to_vec(),
        },
    });
    cases.push(CorpusCase {
        id: "point_mul/n*G".into(),
        category: "point_mul".into(),
        expected_policy: ExpectedPolicy::AcceptInfinity,
        input: TestInput::PointMul {
            scalar: be32(&n),
            point: g_u.to_vec(),
        },
    });

    // malformed
    cases.push(CorpusCase {
        id: "point/bad_prefix".into(),
        category: "point_malformed".into(),
        expected_policy: ExpectedPolicy::Reject,
        input: TestInput::PointSec1({
            let mut v = g_u.to_vec();
            v[0] = 0x01;
            v
        }),
    });
    cases.push(CorpusCase {
        id: "point/truncated".into(),
        category: "point_malformed".into(),
        expected_policy: ExpectedPolicy::Reject,
        input: TestInput::PointSec1(g_u[..10].to_vec()),
    });
    cases.push(CorpusCase {
        id: "point/off_curve".into(),
        category: "point_malformed".into(),
        expected_policy: ExpectedPolicy::Reject,
        input: TestInput::PointSec1({
            let mut v = g_u.to_vec();
            v[64] ^= 0xff;
            v
        }),
    });

    // seed-derived deterministic extras
    let mut rng = StdRng::seed_from_u64(seed);
    for i in 0..8 {
        let mut sk = [0u8; 32];
        rng.fill_bytes(&mut sk);
        // force into range by reducing
        let v = BigUint::from_bytes_be(&sk) % &n;
        if v.is_zero() {
            continue;
        }
        let skb = be32(&v);
        cases.push(CorpusCase {
            id: format!("secret_key/seeded_{i}"),
            category: "secret_key_seeded".into(),
            expected_policy: ExpectedPolicy::AcceptValid,
            input: TestInput::SecretKey(skb),
        });
        let q = AffinePoint::generator()
            .scalar_mul(&Scalar::new(v).unwrap())
            .unwrap()
            .serialize_uncompressed()
            .unwrap();
        cases.push(CorpusCase {
            id: format!("point/seeded_{i}"),
            category: "point_seeded".into(),
            expected_policy: ExpectedPolicy::AcceptValid,
            input: TestInput::PointSec1(q.to_vec()),
        });
    }

    // ensure FieldElement path used
    let _ = FieldElement::one();

    cases
}

pub fn write_corpus_json(path: &std::path::Path, seed: u64) -> std::io::Result<usize> {
    let cases = generate_corpus(seed);
    let n = cases.len();
    let json = serde_json::to_string_pretty(&cases).expect("serialize corpus");
    std::fs::write(path, json)?;
    Ok(n)
}

pub fn load_corpus_json(path: &std::path::Path) -> std::io::Result<Vec<CorpusCase>> {
    let data = std::fs::read_to_string(path)?;
    let cases: Vec<CorpusCase> = serde_json::from_str(&data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(cases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_reproducible() {
        let a = generate_corpus(1337);
        let b = generate_corpus(1337);
        assert_eq!(a, b);
        assert!(a.len() > 50);
    }

    #[test]
    fn corpus_has_required_classes() {
        let c = generate_corpus(1337);
        let ids: Vec<_> = c.iter().map(|x| x.id.as_str()).collect();
        assert!(ids.iter().any(|i| i.contains("scalar/0")));
        assert!(ids.iter().any(|i| i.contains("G+(-G)")));
        assert!(ids.iter().any(|i| i.contains("fe_mul")));
        assert!(ids.iter().any(|i| i.contains("off_curve")));
    }
}
