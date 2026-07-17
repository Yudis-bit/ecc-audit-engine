# Prototype verification (public summary)

| Field | Value |
|-------|--------|
| Engine commits | `a82e8a4`, `f3df00f`, `888075a` |
| Result | Independent clean rebuild confirmed core differential/minimizer claims |
| Corpus | seed 1337, 148 cases, SHA-256 `4af661f6597633433abfa26ca7df3add0c847eecebc85ddac7fd72ea8251bbee` |

## Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
./scripts/build_targets.sh
cargo run -p cli -- differential --target targets/correct-target.so --corpus fixtures/corpus-v1.json
cargo run -p cli -- differential --target targets/corrupted-target.so --corpus fixtures/corpus-v1.json --minimize
```

## Results (representative)

- Correct target: 148 cases, 0 unexpected failures
- Corrupted target: 14 planted failures; isolation map FE_MUL=11, INFINITY=1, SCALAR=2
- Minimizer: 14/14 reproducers replayed

## Limitations

See `methodology-limitations.md`. Full logs are local laboratory artifacts.
