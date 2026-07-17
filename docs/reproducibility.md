# Reproducibility

## Fresh machine (host)

```bash
git clone https://github.com/Yudis-bit/ecc-audit-engine.git
cd ecc-audit-engine
./scripts/bootstrap.sh
./scripts/verify.sh
```

`verify.sh` returns nonzero if a mandatory gate fails.

## Prerequisites

Required:

- Rust (see `rust-toolchain.toml`, currently 1.97.0)
- `cargo`, `cc`/`gcc`, `git`, `python3`, `pkg-config`

Optional:

- Valgrind 3.22+ (dynamic trace)
- CMake / Autotools (official upstream build paths)
- Clang (compiler matrix)
- Docker (clean environment)

## Clean container (non-trace base)

```bash
docker build -t ecc-audit-engine:readiness .
docker run --rm ecc-audit-engine:readiness ./scripts/verify.sh
```

For Valgrind calibration inside a container:

```bash
docker build -f Dockerfile.trace -t ecc-audit-engine:trace .
docker run --rm ecc-audit-engine:trace ./scripts/verify.sh
```

Note: some hosts restrict ptrace; if Valgrind fails in-container, run trace campaigns on native Linux.

## What must not be required

- Prebuilt `targets/*.so`
- Absolute paths under a developer home directory
- Vendored Valgrind or full upstream source trees in Git
- Manually prepared hidden state

## Pinning

- Rust: `rust-toolchain.toml`
- Crates: `Cargo.lock` (`--locked`)
- libsecp256k1: `targets-src/SECP256K1_PIN.txt`
