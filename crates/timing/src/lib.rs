//! Synthetic timing harness. No key-recovery claims.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use runner::Target;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimingSample {
    pub sequence: u64,
    pub class_id: u8,
    pub cycles: Option<u64>,
    pub elapsed_ns: u64,
    pub instructions: Option<u64>,
    pub branches: Option<u64>,
    pub branch_misses: Option<u64>,
    pub cache_references: Option<u64>,
    pub cache_misses: Option<u64>,
    pub cpu_id_before: Option<u32>,
    pub cpu_id_after: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimingStats {
    pub class_a_count: usize,
    pub class_b_count: usize,
    pub mean_a_ns: f64,
    pub mean_b_ns: f64,
    pub var_a_ns: f64,
    pub var_b_ns: f64,
    pub welch_t: f64,
    pub mean_diff_ns: f64,
    pub ci95_low: f64,
    pub ci95_high: f64,
    pub cohens_d: f64,
    pub rejected_samples: usize,
    pub cpu_migration_count: usize,
    pub caveats: Vec<String>,
}

#[cfg(target_arch = "x86_64")]
#[inline]
fn rdtsc() -> u64 {
    unsafe {
        use std::arch::x86_64::{__rdtscp, _mm_lfence};
        let mut aux = 0u32;
        // serialized TSC via RDTSCP + lfence pattern
        _mm_lfence();
        let v = __rdtscp(&mut aux);
        _mm_lfence();
        v
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn rdtsc() -> u64 {
    0
}

fn make_key(class_id: u8, i: u64) -> [u8; 32] {
    let mut sk = [0u8; 32];
    // keep scalar small and valid: use 2..n range, encode i in upper bytes, force LSB
    sk[24..32].copy_from_slice(&(i.wrapping_mul(0x9E3779B97F4A7C15) | 2).to_be_bytes());
    if class_id == 0 {
        sk[31] &= !1; // even
        if sk[31] == 0 {
            sk[31] = 2;
        }
    } else {
        sk[31] |= 1; // odd
    }
    // ensure not zero
    if sk.iter().all(|&b| b == 0) {
        sk[31] = if class_id == 0 { 2 } else { 1 };
    }
    sk
}

pub fn run_lsb_experiment(
    target: &dyn Target,
    seed: u64,
    warmup: usize,
    samples_per_class: usize,
) -> (Vec<TimingSample>, TimingStats) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut plan: Vec<(u8, u64)> = Vec::new();
    for i in 0..samples_per_class as u64 {
        plan.push((0, i));
        plan.push((1, i));
    }
    plan.shuffle(&mut rng);

    // warmup
    for i in 0..warmup {
        let sk = make_key((i % 2) as u8, i as u64);
        let _ = target.pubkey_create(&sk);
    }

    let mut samples = Vec::with_capacity(plan.len());
    let mut rejected = 0usize;
    let mut seq = 0u64;
    let migrations = 0usize;

    for (class_id, i) in plan {
        let sk = make_key(class_id, i);
        let t0 = Instant::now();
        let c0 = if cfg!(target_arch = "x86_64") {
            Some(rdtsc())
        } else {
            None
        };
        let res = target.pubkey_create(&sk);
        let c1 = if cfg!(target_arch = "x86_64") {
            Some(rdtsc())
        } else {
            None
        };
        let elapsed = t0.elapsed().as_nanos() as u64;
        if res.is_err() {
            rejected += 1;
            continue;
        }
        let cycles = match (c0, c1) {
            (Some(a), Some(b)) if b >= a => Some(b - a),
            _ => None,
        };
        samples.push(TimingSample {
            sequence: seq,
            class_id,
            cycles,
            elapsed_ns: elapsed,
            instructions: None, // not silently zero
            branches: None,
            branch_misses: None,
            cache_references: None,
            cache_misses: None,
            cpu_id_before: None,
            cpu_id_after: None,
        });
        seq += 1;
    }

    let stats = analyze(&samples, rejected, migrations);
    (samples, stats)
}

fn mean_var(xs: &[f64]) -> (f64, f64) {
    if xs.is_empty() {
        return (0.0, 0.0);
    }
    let n = xs.len() as f64;
    let m = xs.iter().sum::<f64>() / n;
    let v = xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / n.max(1.0);
    (m, v)
}

pub fn analyze(samples: &[TimingSample], rejected: usize, migrations: usize) -> TimingStats {
    let a: Vec<f64> = samples
        .iter()
        .filter(|s| s.class_id == 0)
        .map(|s| s.elapsed_ns as f64)
        .collect();
    let b: Vec<f64> = samples
        .iter()
        .filter(|s| s.class_id == 1)
        .map(|s| s.elapsed_ns as f64)
        .collect();
    let (mean_a, var_a) = mean_var(&a);
    let (mean_b, var_b) = mean_var(&b);
    let na = a.len() as f64;
    let nb = b.len() as f64;
    let se = ((var_a / na.max(1.0)) + (var_b / nb.max(1.0))).sqrt();
    let welch_t = if se > 0.0 {
        (mean_a - mean_b) / se
    } else {
        0.0
    };
    let mean_diff = mean_a - mean_b;
    // rough normal CI
    let ci = 1.96 * se;
    let pooled = (((na - 1.0) * var_a + (nb - 1.0) * var_b) / (na + nb - 2.0).max(1.0)).sqrt();
    let d = if pooled > 0.0 {
        (mean_a - mean_b) / pooled
    } else {
        0.0
    };

    let mut caveats = vec![
        "Wall-clock and RDTSC are noisy on non-isolated hosts.".into(),
        "Large |t| alone does not demonstrate key recovery.".into(),
        "perf counters unavailable in this first-slice harness (reported as null).".into(),
        "Synthetic LSB classes calibrate detector sensitivity only.".into(),
    ];
    if rejected > 0 {
        caveats.push(format!("{rejected} samples rejected by target policy"));
    }

    TimingStats {
        class_a_count: a.len(),
        class_b_count: b.len(),
        mean_a_ns: mean_a,
        mean_b_ns: mean_b,
        var_a_ns: var_a,
        var_b_ns: var_b,
        welch_t,
        mean_diff_ns: mean_diff,
        ci95_low: mean_diff - ci,
        ci95_high: mean_diff + ci,
        cohens_d: d,
        rejected_samples: rejected,
        cpu_migration_count: migrations,
        caveats,
    }
}
