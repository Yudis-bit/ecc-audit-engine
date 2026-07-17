//! Dynamic library target runner.
//!
//! # Safety
//! All `unsafe` blocks load/call C ABI functions from laboratory-built `.so` files.
//! Buffer lengths are validated before every call. Panics are not allowed to cross FFI.

use libloading::{Library, Symbol};
use std::path::Path;
use target_api::{
    TargetError, TargetMetadata, ECC_TARGET_INTERNAL_ERROR, ECC_TARGET_OK, ECC_TARGET_REJECT,
};

pub trait Target: Send + Sync {
    fn metadata(&self) -> &TargetMetadata;

    fn pubkey_create(&self, secret_key: &[u8; 32]) -> Result<Vec<u8>, TargetError>;

    fn point_add(&self, a: &[u8], b: &[u8]) -> Result<Vec<u8>, TargetError>;

    fn point_mul(&self, scalar: &[u8; 32], point: &[u8]) -> Result<Vec<u8>, TargetError>;

    fn fe_mul(&self, a: &[u8; 32], b: &[u8; 32]) -> Result<[u8; 32], TargetError>;

    fn leak_mode(&self) -> i32 {
        0
    }

    fn leak_counter_swap(&self, new_value: u64) -> u64 {
        let _ = new_value;
        0
    }

    fn last_table_index(&self) -> u32 {
        0xff
    }
}

type PubkeyCreateFn = unsafe extern "C" fn(*const u8, usize, *mut u8, usize) -> i32;
type PointAddFn = unsafe extern "C" fn(*const u8, usize, *const u8, usize, *mut u8, usize) -> i32;
type PointMulFn = unsafe extern "C" fn(*const u8, usize, *const u8, usize, *mut u8, usize) -> i32;
type FeMulFn = unsafe extern "C" fn(*const u8, usize, *const u8, usize, *mut u8, usize) -> i32;
type LeakModeFn = unsafe extern "C" fn() -> i32;
type LeakSwapFn = unsafe extern "C" fn(u64) -> u64;
type LastTableFn = unsafe extern "C" fn() -> u32;

pub struct DynTarget {
    meta: TargetMetadata,
    // Library must outlive symbols; we keep library and re-resolve per call for simplicity.
    lib: Library,
}

fn map_rc(rc: i32) -> Result<(), TargetError> {
    match rc {
        x if x == ECC_TARGET_OK => Ok(()),
        x if x == ECC_TARGET_REJECT => Err(TargetError::Reject(x)),
        x if x == ECC_TARGET_INTERNAL_ERROR => Err(TargetError::Internal(x)),
        x => Err(TargetError::Abi(format!("unknown return code {x}"))),
    }
}

impl DynTarget {
    pub fn load(path: &Path, name: impl Into<String>) -> Result<Self, TargetError> {
        let meta = TargetMetadata::from_path(path, name)?;
        // SAFETY: path points to a local laboratory-built shared object we control.
        let lib = unsafe { Library::new(path) }.map_err(|e| TargetError::Load(e.to_string()))?;
        Ok(Self { meta, lib })
    }
}

impl Target for DynTarget {
    fn metadata(&self) -> &TargetMetadata {
        &self.meta
    }

    fn pubkey_create(&self, secret_key: &[u8; 32]) -> Result<Vec<u8>, TargetError> {
        // SAFETY: symbol must match ecc_target.h; buffers sized correctly.
        let f: Symbol<PubkeyCreateFn> = unsafe {
            self.lib
                .get(b"ecc_target_pubkey_create")
                .map_err(|e| TargetError::Load(e.to_string()))?
        };
        let mut out = vec![0u8; 65];
        let rc = unsafe { f(secret_key.as_ptr(), 32, out.as_mut_ptr(), out.len()) };
        map_rc(rc)?;
        // Infinity not expected for valid sk; keep 65 bytes
        Ok(out)
    }

    fn point_add(&self, a: &[u8], b: &[u8]) -> Result<Vec<u8>, TargetError> {
        if a.is_empty() || b.is_empty() {
            return Err(TargetError::Abi("empty point".into()));
        }
        let f: Symbol<PointAddFn> = unsafe {
            self.lib
                .get(b"ecc_target_point_add")
                .map_err(|e| TargetError::Load(e.to_string()))?
        };
        let mut out = vec![0u8; 65];
        let rc = unsafe {
            f(
                a.as_ptr(),
                a.len(),
                b.as_ptr(),
                b.len(),
                out.as_mut_ptr(),
                out.len(),
            )
        };
        map_rc(rc)?;
        if out[0] == 0x00 {
            return Ok(vec![0x00]);
        }
        Ok(out)
    }

    fn point_mul(&self, scalar: &[u8; 32], point: &[u8]) -> Result<Vec<u8>, TargetError> {
        if point.is_empty() {
            return Err(TargetError::Abi("empty point".into()));
        }
        let f: Symbol<PointMulFn> = unsafe {
            self.lib
                .get(b"ecc_target_point_mul")
                .map_err(|e| TargetError::Load(e.to_string()))?
        };
        let mut out = vec![0u8; 65];
        let rc = unsafe {
            f(
                scalar.as_ptr(),
                32,
                point.as_ptr(),
                point.len(),
                out.as_mut_ptr(),
                out.len(),
            )
        };
        map_rc(rc)?;
        if out[0] == 0x00 {
            return Ok(vec![0x00]);
        }
        Ok(out)
    }

    fn fe_mul(&self, a: &[u8; 32], b: &[u8; 32]) -> Result<[u8; 32], TargetError> {
        let f: Symbol<FeMulFn> = unsafe {
            self.lib
                .get(b"ecc_target_fe_mul")
                .map_err(|e| TargetError::Load(e.to_string()))?
        };
        let mut out = [0u8; 32];
        let rc = unsafe { f(a.as_ptr(), 32, b.as_ptr(), 32, out.as_mut_ptr(), 32) };
        map_rc(rc)?;
        Ok(out)
    }

    fn leak_mode(&self) -> i32 {
        let f: Result<Symbol<LeakModeFn>, _> = unsafe { self.lib.get(b"ecc_target_leak_mode") };
        match f {
            // SAFETY: optional calibration symbol
            Ok(func) => unsafe { func() },
            Err(_) => 0,
        }
    }

    fn leak_counter_swap(&self, new_value: u64) -> u64 {
        let f: Result<Symbol<LeakSwapFn>, _> =
            unsafe { self.lib.get(b"ecc_target_leak_counter_swap") };
        match f {
            Ok(func) => unsafe { func(new_value) },
            Err(_) => 0,
        }
    }

    fn last_table_index(&self) -> u32 {
        let f: Result<Symbol<LastTableFn>, _> =
            unsafe { self.lib.get(b"ecc_target_last_table_index") };
        match f {
            Ok(func) => unsafe { func() },
            Err(_) => 0xff,
        }
    }
}
