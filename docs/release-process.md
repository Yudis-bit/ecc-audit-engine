# Release process

## Versioning

| Bump | When |
|------|------|
| patch (`v0.1.x`) | docs, CI, portability, reliability fixes |
| minor (`v0.2.0`) | new detector/target capability |
| major (`v1.0.0`) | incompatible report/schema/CLI changes |

## Checklist

1. `./scripts/verify.sh` passes on host
2. Clean container path documented / run when Docker available
3. `cargo fmt`, `clippy -D warnings`, `cargo test --locked` green
4. CI green on default branch
5. `CHANGELOG.md` updated
6. Tag `vX.Y.Z` and push tag
7. `release.yml` builds source archive + checksums
8. Verify release via `gh release view vX.Y.Z`

## Artifacts

Attach only small safe artifacts (source archive, checksums, compact notes).
Never attach secret inputs, huge traces, or untrusted production `.so` libraries.

## Honest next version after readiness work

Prefer **v0.1.1** when changes are documentation, CI, and portability.
Use **v0.2.0** only when material new detection capability is verified end-to-end.
