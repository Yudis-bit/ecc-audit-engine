//! Dynamic tracing via Valgrind Lackey (genuine instruction/memory events).
//! Not callback-based. Markers only delimit the region.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

pub const CACHE_LINE_SHIFT: u32 = 6; // 64-byte model

#[derive(Debug, Error)]
pub enum DynError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse: {0}")]
    Parse(String),
    #[error("backend: {0}")]
    Backend(String),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceHeader {
    pub schema_version: String,
    pub backend: String,
    pub backend_version: String,
    pub target_path: String,
    pub target_sha256: String,
    pub upstream_commit: Option<String>,
    pub operation: String,
    pub input_case_id: String,
    pub input_sha256: String,
    pub module_base: u64,
    pub marker_begin_abs: u64,
    pub marker_end_abs: u64,
    pub pubkey_create_abs: Option<u64>,
    pub result_rc: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NormEvent {
    Superblock {
        module_offset: Option<u64>,
        abs: u64,
    },
    Instruction {
        module_offset: Option<u64>,
        abs: u64,
        len: u32,
    },
    MemLoad {
        insn_abs: Option<u64>,
        addr_abs: u64,
        module_relative: Option<u64>,
        cache_line: u64,
        size: u32,
        class: MemClass,
    },
    MemStore {
        insn_abs: Option<u64>,
        addr_abs: u64,
        module_relative: Option<u64>,
        cache_line: u64,
        size: u32,
        class: MemClass,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemClass {
    ModuleData,
    StackLikely,
    HeapOrOther,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NormalizedTrace {
    pub header: TraceHeader,
    pub events: Vec<NormEvent>,
    pub insn_count: u64,
    pub sb_count: u64,
    pub load_count: u64,
    pub store_count: u64,
    /// Ordered module-relative instruction offsets (region only)
    pub insn_offsets: Vec<u64>,
    /// Unique module-relative instruction offsets
    pub insn_set: BTreeSet<u64>,
    /// Superblock module offsets sequence
    pub sb_offsets: Vec<u64>,
    pub sb_set: BTreeSet<u64>,
    /// Module-relative data load/store offsets (static data only)
    pub static_mem_offsets: Vec<u64>,
    pub static_mem_set: BTreeSet<u64>,
    /// Cache lines for static data accesses
    pub static_cache_lines: Vec<u64>,
    pub static_cache_line_set: BTreeSet<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompareResult {
    pub a_case: String,
    pub b_case: String,
    pub insn_count_a: u64,
    pub insn_count_b: u64,
    pub insn_count_equal: bool,
    pub sb_seq_equal: bool,
    pub sb_jaccard: f64,
    pub insn_set_jaccard: f64,
    pub static_mem_set_jaccard: f64,
    pub static_cache_jaccard: f64,
    pub first_sb_divergence: Option<(usize, Option<u64>, Option<u64>)>,
    pub first_insn_divergence: Option<(usize, Option<u64>, Option<u64>)>,
    pub longest_common_prefix_sb: usize,
    pub notes: Vec<String>,
}

pub fn sha256_file(path: &Path) -> Result<String, DynError> {
    let data = fs::read(path)?;
    Ok(hex::encode(Sha256::digest(&data)))
}

pub fn sha256_hex_bytes(hex_in: &str) -> Result<String, DynError> {
    let bytes = hex::decode(hex_in).map_err(|e| DynError::Parse(e.to_string()))?;
    Ok(hex::encode(Sha256::digest(&bytes)))
}

fn parse_u64_hex(s: &str) -> Result<u64, DynError> {
    let s = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    u64::from_str_radix(s, 16).map_err(|e| DynError::Parse(format!("hex {s}: {e}")))
}

/// Parse TRACE_META lines from driver stdout.
pub fn parse_meta_stdout(stdout: &str) -> Result<HashMap<String, String>, DynError> {
    let mut m = HashMap::new();
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("TRACE_META ") {
            if let Some((k, v)) = rest.split_once('=') {
                m.insert(k.to_string(), v.to_string());
            }
        }
    }
    if m.is_empty() {
        return Err(DynError::Parse("no TRACE_META lines".into()));
    }
    Ok(m)
}

/// Parse Valgrind Lackey log with --trace-superblocks=yes --trace-mem=yes.
/// Filters to [marker_begin, marker_end] exclusive of marker bodies when possible.
pub fn parse_lackey_region(
    log_path: &Path,
    marker_begin: u64,
    marker_end: u64,
    module_base: u64,
) -> Result<Vec<NormEvent>, DynError> {
    let f = File::open(log_path)?;
    let reader = BufReader::new(f);
    let mut events = Vec::new();
    let mut in_region = false;
    let mut last_insn_abs: Option<u64> = None;
    let mut saw_begin = false;
    let mut saw_end = false;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim_end();
        if line.starts_with("==") || line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("SB ") {
            let abs = parse_u64_hex(rest.trim())?;
            // Enter region when we execute the begin marker superblock/insn
            if abs == marker_begin || (abs <= marker_begin && marker_begin < abs.wrapping_add(16)) {
                // may hit nearby; exact match preferred below on I lines
            }
            if in_region {
                let off = abs.checked_sub(module_base);
                events.push(NormEvent::Superblock {
                    module_offset: off,
                    abs,
                });
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("I  ") {
            // "I  0401f540,3"
            let rest = rest.trim();
            let (addr_s, len_s) = rest
                .split_once(',')
                .ok_or_else(|| DynError::Parse(format!("I line: {line}")))?;
            let abs = parse_u64_hex(addr_s.trim())?;
            let len: u32 = len_s
                .trim()
                .parse()
                .map_err(|e| DynError::Parse(format!("len: {e}")))?;

            if abs == marker_begin {
                in_region = true;
                saw_begin = true;
                last_insn_abs = Some(abs);
                // do not include marker body as crypto content
                continue;
            }
            if abs == marker_end {
                in_region = false;
                saw_end = true;
                last_insn_abs = Some(abs);
                continue;
            }
            // Also treat any insn in [begin, begin+32) as begin, [end, end+32) as end
            if !in_region && abs >= marker_begin && abs < marker_begin.saturating_add(32) {
                in_region = true;
                saw_begin = true;
                continue;
            }
            if in_region && abs >= marker_end && abs < marker_end.saturating_add(32) {
                in_region = false;
                saw_end = true;
                continue;
            }

            if in_region {
                last_insn_abs = Some(abs);
                let off = abs.checked_sub(module_base);
                events.push(NormEvent::Instruction {
                    module_offset: off,
                    abs,
                    len,
                });
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix(" L ") {
            if !in_region {
                continue;
            }
            let (addr_s, size_s) = rest
                .trim()
                .split_once(',')
                .ok_or_else(|| DynError::Parse(format!("L: {line}")))?;
            let addr = parse_u64_hex(addr_s.trim())?;
            let size: u32 = size_s
                .trim()
                .parse()
                .map_err(|e| DynError::Parse(format!("size: {e}")))?;
            events.push(classify_mem(true, last_insn_abs, addr, size, module_base));
            continue;
        }
        if let Some(rest) = line.strip_prefix(" S ") {
            if !in_region {
                continue;
            }
            let (addr_s, size_s) = rest
                .trim()
                .split_once(',')
                .ok_or_else(|| DynError::Parse(format!("S: {line}")))?;
            let addr = parse_u64_hex(addr_s.trim())?;
            let size: u32 = size_s
                .trim()
                .parse()
                .map_err(|e| DynError::Parse(format!("size: {e}")))?;
            events.push(classify_mem(false, last_insn_abs, addr, size, module_base));
            continue;
        }
    }

    if !saw_begin {
        return Err(DynError::Parse(format!(
            "marker begin 0x{marker_begin:x} not observed in lackey log"
        )));
    }
    if !saw_end {
        return Err(DynError::Parse(format!(
            "marker end 0x{marker_end:x} not observed in lackey log"
        )));
    }
    Ok(events)
}

fn classify_mem(
    is_load: bool,
    insn_abs: Option<u64>,
    addr: u64,
    size: u32,
    module_base: u64,
) -> NormEvent {
    // Heuristic: stack typically high user addresses (Linux x86_64 ~0x1ff... or 0x7ff...)
    let class = if module_base != 0 && addr >= module_base && addr < module_base + 0x1000_0000 {
        MemClass::ModuleData
    } else if (addr >> 40) >= 0x7f || (addr >> 32) == 0x1ffe {
        MemClass::StackLikely
    } else {
        MemClass::HeapOrOther
    };
    let module_relative = if matches!(class, MemClass::ModuleData) {
        Some(addr - module_base)
    } else {
        None
    };
    // Cache line: for module data use relative; else use absolute>>6 but mark class
    let cache_line = match module_relative {
        Some(off) => off >> CACHE_LINE_SHIFT,
        None => addr >> CACHE_LINE_SHIFT,
    };
    if is_load {
        NormEvent::MemLoad {
            insn_abs,
            addr_abs: addr,
            module_relative,
            cache_line,
            size,
            class,
        }
    } else {
        NormEvent::MemStore {
            insn_abs,
            addr_abs: addr,
            module_relative,
            cache_line,
            size,
            class,
        }
    }
}

pub fn build_normalized(header: TraceHeader, events: Vec<NormEvent>) -> NormalizedTrace {
    let mut insn_count = 0u64;
    let mut sb_count = 0u64;
    let mut load_count = 0u64;
    let mut store_count = 0u64;
    let mut insn_offsets = Vec::new();
    let mut insn_set = BTreeSet::new();
    let mut sb_offsets = Vec::new();
    let mut sb_set = BTreeSet::new();
    let mut static_mem_offsets = Vec::new();
    let mut static_mem_set = BTreeSet::new();
    let mut static_cache_lines = Vec::new();
    let mut static_cache_line_set = BTreeSet::new();

    for e in &events {
        match e {
            NormEvent::Superblock { module_offset, .. } => {
                sb_count += 1;
                if let Some(o) = module_offset {
                    sb_offsets.push(*o);
                    sb_set.insert(*o);
                }
            }
            NormEvent::Instruction { module_offset, .. } => {
                insn_count += 1;
                if let Some(o) = module_offset {
                    insn_offsets.push(*o);
                    insn_set.insert(*o);
                }
            }
            NormEvent::MemLoad {
                module_relative,
                cache_line,
                class,
                ..
            }
            | NormEvent::MemStore {
                module_relative,
                cache_line,
                class,
                ..
            } => {
                if matches!(e, NormEvent::MemLoad { .. }) {
                    load_count += 1;
                } else {
                    store_count += 1;
                }
                if matches!(class, MemClass::ModuleData) {
                    if let Some(o) = module_relative {
                        static_mem_offsets.push(*o);
                        static_mem_set.insert(*o);
                    }
                    static_cache_lines.push(*cache_line);
                    static_cache_line_set.insert(*cache_line);
                }
            }
        }
    }

    NormalizedTrace {
        header,
        events,
        insn_count,
        sb_count,
        load_count,
        store_count,
        insn_offsets,
        insn_set,
        sb_offsets,
        sb_set,
        static_mem_offsets,
        static_mem_set,
        static_cache_lines,
        static_cache_line_set,
    }
}

fn jaccard(a: &BTreeSet<u64>, b: &BTreeSet<u64>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let inter = a.intersection(b).count() as f64;
    let union = a.union(b).count() as f64;
    if union == 0.0 {
        1.0
    } else {
        inter / union
    }
}

fn lcp(a: &[u64], b: &[u64]) -> usize {
    let n = a.len().min(b.len());
    let mut i = 0;
    while i < n && a[i] == b[i] {
        i += 1;
    }
    i
}

pub fn compare_traces(a: &NormalizedTrace, b: &NormalizedTrace) -> CompareResult {
    let notes = vec![
        "Stack/heap absolute addresses excluded from primary set equality; module-relative only."
            .into(),
        "Backend=Valgrind Lackey; not a timing tool.".into(),
    ];

    let first_sb = {
        let n = a.sb_offsets.len().max(b.sb_offsets.len());
        let mut d = None;
        for i in 0..n {
            let x = a.sb_offsets.get(i).copied();
            let y = b.sb_offsets.get(i).copied();
            if x != y {
                d = Some((i, x, y));
                break;
            }
        }
        d
    };
    let first_insn = {
        let n = a.insn_offsets.len().max(b.insn_offsets.len());
        let mut d = None;
        for i in 0..n {
            let x = a.insn_offsets.get(i).copied();
            let y = b.insn_offsets.get(i).copied();
            if x != y {
                d = Some((i, x, y));
                break;
            }
        }
        d
    };

    CompareResult {
        a_case: a.header.input_case_id.clone(),
        b_case: b.header.input_case_id.clone(),
        insn_count_a: a.insn_count,
        insn_count_b: b.insn_count,
        insn_count_equal: a.insn_count == b.insn_count,
        sb_seq_equal: a.sb_offsets == b.sb_offsets,
        sb_jaccard: jaccard(&a.sb_set, &b.sb_set),
        insn_set_jaccard: jaccard(&a.insn_set, &b.insn_set),
        static_mem_set_jaccard: jaccard(&a.static_mem_set, &b.static_mem_set),
        static_cache_jaccard: jaccard(&a.static_cache_line_set, &b.static_cache_line_set),
        first_sb_divergence: first_sb,
        first_insn_divergence: first_insn,
        longest_common_prefix_sb: lcp(&a.sb_offsets, &b.sb_offsets),
        notes,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendInfo {
    pub name: String,
    pub version: String,
    pub path: String,
    pub tool: String,
}

pub fn detect_valgrind(project_root: &Path) -> Result<BackendInfo, DynError> {
    let local = project_root.join("third_party/valgrind/bin/valgrind");
    let path = if local.exists() {
        local
    } else {
        PathBuf::from("valgrind")
    };
    let out = Command::new(&path)
        .arg("--version")
        .output()
        .map_err(|e| DynError::Backend(format!("valgrind not runnable: {e}")))?;
    if !out.status.success() {
        return Err(DynError::Backend("valgrind --version failed".into()));
    }
    let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(BackendInfo {
        name: "valgrind".into(),
        version: ver,
        path: path.display().to_string(),
        tool: "lackey".into(),
    })
}

/// Run one dynamic trace under Valgrind Lackey + ecc-trace-driver.
///
/// `trace_mem`: when false, only superblocks/instructions are traced (much faster;
/// sufficient for branch divergence). When true, also records loads/stores (needed
/// for secret-dependent table address calibration).
pub fn run_lackey_trace(
    project_root: &Path,
    target_so: &Path,
    secret_hex: &str,
    case_id: &str,
    out_dir: &Path,
) -> Result<NormalizedTrace, DynError> {
    // Lackey emits guest I-lines only when --trace-mem=yes; required for region markers.
    run_lackey_trace_opts(project_root, target_so, secret_hex, case_id, out_dir, true)
}

pub fn run_lackey_trace_opts(
    project_root: &Path,
    target_so: &Path,
    secret_hex: &str,
    case_id: &str,
    out_dir: &Path,
    trace_mem: bool,
) -> Result<NormalizedTrace, DynError> {
    fs::create_dir_all(out_dir)?;
    let backend = detect_valgrind(project_root)?;
    let driver = project_root.join("targets/ecc-trace-driver");
    if !driver.exists() {
        return Err(DynError::Backend(format!(
            "missing driver {}",
            driver.display()
        )));
    }
    if !target_so.exists() {
        return Err(DynError::Backend(format!(
            "missing target {}",
            target_so.display()
        )));
    }

    let log_path = out_dir.join(format!("{case_id}.lackey.log"));
    let stdout_path = out_dir.join(format!("{case_id}.stdout.txt"));

    let mem_flag = if trace_mem {
        "--trace-mem=yes"
    } else {
        "--trace-mem=no"
    };

    let output = Command::new(&backend.path)
        .args([
            "--tool=lackey",
            "--trace-superblocks=yes",
            mem_flag,
            &format!("--log-file={}", log_path.display()),
        ])
        .arg(&driver)
        .arg(target_so)
        .arg(secret_hex)
        .arg(case_id)
        .output()
        .map_err(|e| DynError::Backend(e.to_string()))?;

    fs::write(&stdout_path, &output.stdout)?;
    // lackey writes to log-file; also keep stderr
    fs::write(
        out_dir.join(format!("{case_id}.stderr.txt")),
        &output.stderr,
    )?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let meta = parse_meta_stdout(&stdout)?;
    let module_base = parse_u64_hex(
        meta.get("module_base")
            .ok_or_else(|| DynError::Parse("module_base".into()))?,
    )?;
    let marker_begin = parse_u64_hex(
        meta.get("marker_begin_abs")
            .ok_or_else(|| DynError::Parse("marker_begin_abs".into()))?,
    )?;
    let marker_end = parse_u64_hex(
        meta.get("marker_end_abs")
            .ok_or_else(|| DynError::Parse("marker_end_abs".into()))?,
    )?;
    let pubkey_abs = meta
        .get("pubkey_create_abs")
        .map(|s| parse_u64_hex(s))
        .transpose()?;
    let result_rc = meta.get("result_rc").and_then(|s| s.parse().ok());

    let events = parse_lackey_region(&log_path, marker_begin, marker_end, module_base)?;

    let header = TraceHeader {
        schema_version: "1.0.0".into(),
        backend: format!("{}-{}", backend.name, backend.tool),
        backend_version: backend.version,
        target_path: target_so.display().to_string(),
        target_sha256: sha256_file(target_so)?,
        upstream_commit: None,
        operation: "pubkey_create".into(),
        input_case_id: case_id.into(),
        input_sha256: sha256_hex_bytes(secret_hex)?,
        module_base,
        marker_begin_abs: marker_begin,
        marker_end_abs: marker_end,
        pubkey_create_abs: pubkey_abs,
        result_rc,
    };

    let norm = build_normalized(header, events);
    let norm_path = out_dir.join(format!("{case_id}.normalized.json"));
    let mut f = File::create(&norm_path)?;
    // Compact: store summary without full event dump by default; write full separately if small
    #[derive(Serialize)]
    struct Summary<'a> {
        header: &'a TraceHeader,
        insn_count: u64,
        sb_count: u64,
        load_count: u64,
        store_count: u64,
        insn_offsets: &'a [u64],
        sb_offsets: &'a [u64],
        insn_set: &'a BTreeSet<u64>,
        sb_set: &'a BTreeSet<u64>,
        static_mem_set: &'a BTreeSet<u64>,
        static_cache_line_set: &'a BTreeSet<u64>,
        event_count: usize,
    }
    serde_json::to_writer_pretty(
        &mut f,
        &Summary {
            header: &norm.header,
            insn_count: norm.insn_count,
            sb_count: norm.sb_count,
            load_count: norm.load_count,
            store_count: norm.store_count,
            insn_offsets: &norm.insn_offsets,
            sb_offsets: &norm.sb_offsets,
            insn_set: &norm.insn_set,
            sb_set: &norm.sb_set,
            static_mem_set: &norm.static_mem_set,
            static_cache_line_set: &norm.static_cache_line_set,
            event_count: norm.events.len(),
        },
    )?;
    // full events only if small
    if norm.events.len() <= 50_000 {
        let mut fe = File::create(out_dir.join(format!("{case_id}.events.json")))?;
        serde_json::to_writer(&mut fe, &norm.events)?;
    }
    Ok(norm)
}

pub fn make_valid_key(seed: u64, class_bit: u8) -> String {
    // produce 32-byte BE hex in (1, n)
    let n =
        hex::decode("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141").unwrap();
    let mut sk = [0u8; 32];
    let mut x = seed
        .wrapping_mul(0x9E3779B97F4A7C15)
        .wrapping_add(0x85EBCA77C2B2AE63);
    for i in 0..4 {
        let w = x.to_be_bytes();
        sk[i * 8..(i + 1) * 8].copy_from_slice(&w);
        x = x.wrapping_mul(0xC2B2AE3D27D4EB4F).wrapping_add(1);
    }
    // force in range roughly by zeroing high bits sometimes and setting low byte
    sk[0] = 0;
    if sk.iter().all(|&b| b == 0) {
        sk[31] = 1;
    }
    // ensure < n: if >= n, clear top
    if sk.as_slice() >= n.as_slice() {
        sk[0] = 0;
        sk[1] = 0;
    }
    if class_bit & 1 == 0 {
        sk[31] &= !1;
        if sk[31] == 0 {
            sk[31] = 2;
        }
    } else {
        sk[31] |= 1;
    }
    if sk.iter().all(|&b| b == 0) {
        sk[31] = 1 | (class_bit & 1);
    }
    hex::encode(sk)
}

pub fn hamming_weight_key(seed: u64, high: bool) -> String {
    let mut sk = [0u8; 32];
    if high {
        sk = [0xff; 32];
        sk[0] = 0x7f; // keep < n roughly
                      // force < n
        sk[0] = 0;
        sk[1] = 0;
    } else {
        // low HW: single bit from seed
        let bit = (seed % 250) as usize + 1;
        let byte = bit / 8;
        let b = 7 - (bit % 8);
        sk[byte] |= 1 << b;
        if sk.iter().all(|&x| x == 0) {
            sk[31] = 2;
        }
    }
    hex::encode(sk)
}

/// Campaign: compare two classes of keys.
pub fn campaign_two_class(
    project_root: &Path,
    target_so: &Path,
    out_dir: &Path,
    seed: u64,
    keys_per_class: usize,
    key_fn: impl Fn(u64, u8) -> String,
) -> Result<serde_json::Value, DynError> {
    campaign_two_class_opts(
        project_root,
        target_so,
        out_dir,
        seed,
        keys_per_class,
        key_fn,
        true,
    )
}

pub fn campaign_two_class_opts(
    project_root: &Path,
    target_so: &Path,
    out_dir: &Path,
    seed: u64,
    keys_per_class: usize,
    key_fn: impl Fn(u64, u8) -> String,
    trace_mem: bool,
) -> Result<serde_json::Value, DynError> {
    fs::create_dir_all(out_dir)?;
    let mut plan: Vec<(u8, usize)> = Vec::new();
    for i in 0..keys_per_class {
        plan.push((0, i));
        plan.push((1, i));
    }
    let mut rng = StdRng::seed_from_u64(seed);
    plan.shuffle(&mut rng);

    let mut class0: Vec<NormalizedTrace> = Vec::new();
    let mut class1: Vec<NormalizedTrace> = Vec::new();
    let mut pairs = Vec::new();

    for (cls, i) in plan {
        let sk = key_fn(
            seed.wrapping_add(i as u64).wrapping_mul(17 + cls as u64),
            cls,
        );
        let case = format!("c{cls}_i{i}_s{seed}");
        let t = run_lackey_trace_opts(project_root, target_so, &sk, &case, out_dir, trace_mem)?;
        if cls == 0 {
            class0.push(t);
        } else {
            class1.push(t);
        }
    }

    // Pairwise first-key comparison + aggregate set jaccard stats
    let mut sb_eq = 0usize;
    let mut insn_eq = 0usize;
    let mut comps = Vec::new();
    let n = class0.len().min(class1.len());
    for i in 0..n {
        let c = compare_traces(&class0[i], &class1[i]);
        if c.sb_seq_equal {
            sb_eq += 1;
        }
        if c.insn_count_equal && c.insn_set_jaccard == 1.0 {
            insn_eq += 1;
        }
        if i < 5 {
            comps.push(c);
        }
    }

    // Aggregate unique sets per class
    let mut set0 = BTreeSet::new();
    let mut set1 = BTreeSet::new();
    for t in &class0 {
        set0.extend(t.insn_set.iter().copied());
    }
    for t in &class1 {
        set1.extend(t.insn_set.iter().copied());
    }
    let mut mem0 = BTreeSet::new();
    let mut mem1 = BTreeSet::new();
    for t in &class0 {
        mem0.extend(t.static_mem_set.iter().copied());
    }
    for t in &class1 {
        mem1.extend(t.static_mem_set.iter().copied());
    }
    let mut cl0 = BTreeSet::new();
    let mut cl1 = BTreeSet::new();
    for t in &class0 {
        cl0.extend(t.static_cache_line_set.iter().copied());
    }
    for t in &class1 {
        cl1.extend(t.static_cache_line_set.iter().copied());
    }

    let summary = serde_json::json!({
        "target": target_so.display().to_string(),
        "target_sha256": sha256_file(target_so)?,
        "seed": seed,
        "keys_per_class": keys_per_class,
        "paired_sb_seq_equal": sb_eq,
        "paired_insn_set_equal": insn_eq,
        "pairs": n,
        "aggregate_insn_set_jaccard": jaccard(&set0, &set1),
        "aggregate_static_mem_jaccard": jaccard(&mem0, &mem1),
        "aggregate_static_cache_jaccard": jaccard(&cl0, &cl1),
        "sample_comparisons": comps,
        "notes": [
            "Primary equality uses module-relative code and static-data offsets only.",
            "Stack/heap absolute addresses are classified and excluded from primary sets."
        ]
    });
    let mut f = File::create(out_dir.join("campaign_summary.json"))?;
    serde_json::to_writer_pretty(&mut f, &summary)?;
    pairs.push(summary.clone());
    let _ = pairs;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lackey_sample_region() {
        let sample = "\
==1== Lackey
SB 04001000
I  04001000,3
I  04001010,4
 L 04002000,8
 S 04002008,8
I  04001020,2
I  04001030,1
";
        let dir = std::env::temp_dir().join("dyntrace_test_parse");
        let _ = fs::create_dir_all(&dir);
        let p = dir.join("s.log");
        fs::write(&p, sample).unwrap();
        // begin=0x4001000 end=0x4001020 → region includes 0x4001010 and mem
        let r = parse_lackey_region(&p, 0x4001000, 0x4001020, 0x4000000).unwrap();
        assert!(r
            .iter()
            .any(|e| matches!(e, NormEvent::Instruction { abs: 0x4001010, .. })));
        assert!(r.iter().any(|e| matches!(e, NormEvent::MemLoad { .. })));
        // missing markers → error
        let r2 = parse_lackey_region(&p, 0xdead, 0xbeef, 0x4000000);
        assert!(r2.is_err());
    }

    #[test]
    fn jaccard_basic() {
        let mut a = BTreeSet::new();
        a.insert(1);
        a.insert(2);
        let mut b = BTreeSet::new();
        b.insert(2);
        b.insert(3);
        assert!((jaccard(&a, &b) - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn meta_parse() {
        let s = "TRACE_META module_base=0x1000\nTRACE_META marker_begin_abs=0x10\n";
        let m = parse_meta_stdout(s).unwrap();
        assert_eq!(m.get("module_base").unwrap(), "0x1000");
    }
}
