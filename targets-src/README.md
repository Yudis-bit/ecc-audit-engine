# Upstream sources (not vendored in git)

Clone and pin:

```bash
git clone https://github.com/bitcoin-core/secp256k1.git targets-src/secp256k1
cd targets-src/secp256k1
git checkout 11dad6d06c0ea8fd6d9d423d32bddd18b70b8b53
```

Then:

```bash
./scripts/build_libsecp256k1.sh
# link adapter against each build (see scripts or verification docs)
```

Pinned commit used in baseline audit: `11dad6d06c0ea8fd6d9d423d32bddd18b70b8b53` (2026-07-16).
