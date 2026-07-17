# Readiness verification (v0.1.1 candidate)

Generated UTC: 2026-07-17T17:06:50Z  
Engine commit (pre-release commit base): `69a0695c033e27e71e1bd5a115778188bb980d89`  
Corpus SHA-256: `4af661f6597633433abfa26ca7df3add0c847eecebc85ddac7fd72ea8251bbee`

## Status

- `./scripts/verify.sh` **PASSED** (all mandatory gates)
- Table-address: **TABLE_ADDRESS_DETECTION_VERIFIED**
- Upstream patch: **UPSTREAM_PATCH_NOT_JUSTIFIED** (pending focused mutation lab confirmation)

## Required language

No vulnerability in upstream libsecp256k1 has been confirmed by the
published experiments.

Synthetic corrupted and leaky targets are calibration fixtures used
to validate the engine.

A negative bounded trace result is not a universal proof of
constant-time behavior.

## Trace calibration (synthetic)

| Fixture | Result |
|---------|--------|
| secret branch | detected (insn-set jaccard 0.9961) |
| secret table address | detected (insn jaccard 1.0; static_mem 0.9444; cache 0.9000) |
| constant control | clean |

## libsecp baseline (bounded)

- total cases: 148
- policy/boundary mismatches: 41
- arithmetic suspects: 0

## Reproduction

```bash
git clone https://github.com/Yudis-bit/ecc-audit-engine.git
cd ecc-audit-engine
./scripts/bootstrap.sh
./scripts/verify.sh
```
