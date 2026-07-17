# Contributing

## Rules

1. **Synthetic data only** — no live private keys, wallet dumps, or third-party secrets.
2. **No live-target attacks** — local builds and authorized laboratory targets only.
3. **No fabricated findings** — every claim must link to commands and raw evidence.
4. **Separate finding classes**:
   - synthetic calibration (planted gadgets)
   - controlled fixture defects
   - real-target results
   - negative results
5. **Tests required** for detector and oracle changes (`cargo test`, and where relevant differential / dynamic-trace calibration).
6. Do not describe callback-based calibration as binary taint analysis.
7. Do not claim one Valgrind campaign proves universal constant-time behavior.

## Development

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features -- --nocapture
```

Build C harnesses with `./scripts/build_targets.sh` after installing a C compiler.

Dynamic tracing requires Valgrind Lackey (see `third_party/README.md`).

## Pull requests

- Keep commits atomic and buildable.
- Update `CHANGELOG.md` for user-visible changes.
- Prefer small, reviewable diffs.
