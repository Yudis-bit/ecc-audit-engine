# Security Policy

## Scope

`ecc-audit-engine` is a **defensive research tool** for local correctness and
side-channel *calibration* of secp256k1-style implementations.

- Use **synthetic keys** only.
- Do not attach live wallet material, production secrets, or third-party data.
- Do not use this tool against systems you do not own or are not authorized to test.

## Reporting issues in this repository

If you find a security issue in **this engine** (for example unsafe FFI handling
or accidental leakage of operator credentials in logs):

1. Prefer a private report via GitHub Security Advisories on this repository when enabled.
2. Do not open a public issue that includes secrets or live keys.

## Upstream libsecp256k1

Suspected issues in **bitcoin-core/secp256k1** must follow upstream’s security
process (see their `SECURITY.md`). Do not file public issues that disclose
exploitable details before coordinated disclosure.

## Explicit baseline result

**No vulnerability in upstream libsecp256k1 has been confirmed by the published
experiments.** Synthetic corrupted and leaky targets are calibration fixtures
used to validate the engine.

A negative bounded trace result is not a universal proof of constant-time
behavior.

## Private disclosure path

If unmodified upstream demonstrates a credible secret-dependent branch or
memory-address behavior with practical impact: stop public disclosure, preserve
raw evidence privately, and follow upstream `SECURITY.md`. Do not open a public
issue or PR that exposes an exploitable flaw.
