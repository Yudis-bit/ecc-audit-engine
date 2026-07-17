# libsecp256k1 baseline (public summary)

| Field | Value |
|-------|--------|
| Upstream | https://github.com/bitcoin-core/secp256k1.git |
| Commit | `11dad6d06c0ea8fd6d9d423d32bddd18b70b8b53` |
| Adapter | public API only (`harnesses/libsecp256k1-adapter/`) |
| Compiler | GCC 13.3 (host) |

## Differential (public API)

- 10 000 valid `pubkey_create` vs mini-C: 0 mismatches
- Metamorphic `pubkey(n−k)=−pubkey(k)`: pass
- Multi-build matrix sample: no output disagreement
- Full structured corpus vs model includes **expected policy rejects** (no public `fe_mul`; seckey_verify rejects ≥ n) — not arithmetic vulnerabilities

## Explicit statement

**No vulnerability in upstream libsecp256k1 was found by the published baseline experiments.**

## Commands

```bash
./scripts/build_libsecp256k1.sh
# link adapter; then:
cargo run -p cli -- differential --target targets/libsecp256k1-adapter.so --corpus fixtures/corpus-v1.json
```
