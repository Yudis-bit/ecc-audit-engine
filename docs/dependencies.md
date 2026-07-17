# Dependencies

## Rust crates (workspace)

| Crate | Why |
|-------|-----|
| `num-bigint` / `num-traits` | Reference field/group math |
| `serde` / `serde_json` | Reports and corpora |
| `toml` | Experiment manifests |
| `sha2` / `hex` | Hashes and encodings |
| `rand` | Deterministic seeded campaigns |
| `thiserror` | Error types |
| `clap` | CLI |
| `libloading` | FFI target loading |

Review with:

```bash
cargo tree
cargo metadata --locked --format-version 1
cargo audit || true
cargo deny check || true
```

## Unsafe / FFI

- `runner` uses `libloading` to call C ABI symbols (`unsafe` call boundaries).
- C harnesses and adapters are external; Rust does not vendor their object code in Git.

## System dependencies

| Tool | Required? | Purpose |
|------|-----------|---------|
| Rust 1.97.0 | yes | build/test |
| GCC/`cc` | yes | lab targets |
| Git | yes | clone pins |
| Python 3 | yes | schema/report helpers |
| Valgrind | optional | dynamic trace |
| CMake | optional | official upstream CMake builds |
| Autotools | optional | official upstream Autotools builds |
| Clang | optional | compiler matrix |

## Valgrind

Preferred: system package matching 3.22+.
Alternate: build under `third_party/valgrind` (gitignored). See `third_party/README.md`.

## libsecp256k1 pin policy

- Record remote + commit in `targets-src/SECP256K1_PIN.txt`
- Clone into `targets-src/secp256k1/` (gitignored)
- Never vendor full upstream trees into engine Git
- Preserve historical pins for published experiment reproducibility
