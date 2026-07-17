# Bitcoin Core subtree attribution (libsecp256k1)

## Fork status

As of 2026-07-17, **`Yudis-bit/bitcoin` does not exist**. No local `bitcoin/bitcoin` checkout was required for this document; conclusions below are based on public upstream documentation and standard Bitcoin Core practice for `src/secp256k1`.

## How `src/secp256k1` is imported

Bitcoin Core vendors libsecp256k1 under `src/secp256k1` via a **subtree** (or equivalent periodic import) managed by Bitcoin Core maintainers. Individual libsecp256k1 PRs are **not** automatically cherry-picked into Bitcoin Core by contributors.

Typical pattern:

1. Work is reviewed and **merged into `bitcoin-core/secp256k1`**.
2. Maintainers later open a Bitcoin Core PR that updates the vendored subtree to a new libsecp commit/tag.
3. Attribution for the cryptographic change is primarily in **libsecp256k1** history; Bitcoin Core history may show a subtree-update commit authored by the importer, not every individual libsecp author.

## Attribution implications

| Question | Answer |
|----------|--------|
| Does a merged libsecp commit appear as a first-class Bitcoin Core commit author? | Usually **no** — subtree import collapses history. |
| Does `git blame` in Bitcoin Core show original libsecp authors? | Depends on import method; often **not** line-by-line original authors. |
| How is durable credit earned? | Prefer **merged libsecp commit**, release notes, and (if security) advisory acknowledgement. |
| Should a contributor open a Bitcoin Core subtree PR to import their own libsecp patch? | **No** — follow maintainer practice; self-import for attribution is rejected here. |

## Direct Bitcoin Core PR policy (this project)

A direct `bitcoin/bitcoin` PR is justified only for an **independent** integration gap (CI, fuzz, docs mismatch, build, etc.), not to re-land an already-submitted libsecp test.

Current status: **`BITCOIN_CORE_DIRECT_PATCH_NOT_JUSTIFIED`**

## Commands to run after a local Bitcoin Core clone exists

```bash
git clone https://github.com/bitcoin/bitcoin.git
cd bitcoin
git log --oneline -- src/secp256k1 | head
git log --oneline --grep='secp256k1' --merges | head
# Inspect a recent subtree update commit carefully:
# git show <subtree-update-commit> --stat
```

Do not invent file paths such as `src/ecc_impl.h`. Use files present in the checkout.

## Relationship to PR #1893

If #1893 merges into libsecp256k1:

1. Durable attribution: libsecp `git log` / `git blame` on `src/ctime_tests.c`.
2. Bitcoin Core incorporation: wait for normal subtree update.
3. No self-import PR from this project.
