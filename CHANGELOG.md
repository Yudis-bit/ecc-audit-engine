# Changelog

## 0.1.0

### Verified capabilities

- Auditable BigUint secp256k1 reference model with known generator vectors.
- Deterministic structured corpus (seed 1337, 148 cases).
- Stable C ABI and mini-C correct / corrupted / leaky harnesses.
- Differential testing against the reference model; planted corruption isolation map.
- Input minimizer with replay of minimized reproducers.
- Valgrind Lackey dynamic-trace backend with region markers (non-callback).
- Synthetic branch and table-address calibration via genuine Lackey events.
- Pinned libsecp256k1 public-API adapter differential campaign (negative for arithmetic bugs under public API policy).
- Bounded dynamic-trace campaigns on libsecp256k1 adapter (no class-correlated insn-sequence divergence observed on tested corpus).

### Explicit non-claims

- No upstream production vulnerability reported.
- Dynamic trace equality is not a universal constant-time proof.
- Callback-based counters (legacy `trace` crate) are secondary calibration only.
