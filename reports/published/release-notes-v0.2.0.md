# Release notes — v0.2.0

Readiness, reproducibility, and verified upstream ctime coverage study.

## Highlights

- One-command `./scripts/verify.sh` for independent reproduction
- Schema-validated report contracts
- Table-address detection re-proven on synthetic fixtures (not insn equality alone)
- Official libsecp256k1 ctime mutation lab for `schnorrsig_sign_custom`
- Draft upstream test-only PR (no production code change)

## Security language

No vulnerability in upstream libsecp256k1 has been confirmed by the
published experiments.

Synthetic corrupted and leaky targets are calibration fixtures used
to validate the engine.

A negative bounded trace result is not a universal proof of
constant-time behavior.

## Upgrade notes

- Prefer `./scripts/bootstrap.sh` then `./scripts/verify.sh`
- Raw experiment outputs remain gitignored under `reports/**/raw/` and `reports/readiness-run/generated/`
