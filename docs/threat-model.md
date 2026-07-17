# Threat model

## In scope

- Implementation bugs in **local laboratory targets** (mini-C fixtures, adapters)
- Synthetic secret-dependent control flow and static-table address divergence
- Differential mismatches against the reference model
- Coverage gaps in **tests** (not silent production patches)

## Out of scope

- Live wallets, exchanges, nodes, browser extensions, hardware wallets
- Network attacks, social engineering, key theft from third parties
- Claiming universal constant-time proofs from bounded samples
- Treating API rejection-policy differences as arithmetic vulnerabilities

## Trust assumptions

- The host running experiments is controlled by the researcher
- Synthetic private keys have no production value
- Unmodified upstream libsecp256k1 is treated as a high-assurance reference, not an attack target
- Valgrind Lackey is an approximate dynamic observer (not full architectural taint)

## Attacker model for calibration

Calibration fixtures **intentionally** plant:

1. secret-dependent branches
2. secret-dependent static table loads

The engine must detect these under Lackey. Constant-control fixtures must remain clean.

## Disclosure

If unmodified upstream shows a credible secret-dependent behavior with practical impact:

1. stop public disclosure of exploitable detail;
2. preserve private evidence;
3. follow upstream `SECURITY.md`;
4. do not open a public issue/PR that enables exploitation.

Status token: `PRIVATE_DISCLOSURE_REQUIRED`
