# Report schemas

Versioned JSON schemas live in `schemas/`:

| Schema | File |
|--------|------|
| Finding | `schemas/finding-v1.schema.json` |
| Experiment | `schemas/experiment-v1.schema.json` |
| Trace summary | `schemas/trace-v1.schema.json` |

Validate with:

```bash
python3 scripts/validate_schemas.py
```

## Finding classifications

- Synthetic calibration findings
- Controlled corrupted-fixture findings
- Real upstream findings
- Negative results
- Inconclusive results
- Unsupported capabilities

## Required metadata fields

Every readiness/finding report should carry:

- schema version
- engine commit
- target commit (when applicable)
- target binary hash
- compiler / build flags
- backend
- corpus hash
- experiment seed
- operation
- sample count
- result
- limitations
- reproduction command

## Legacy reports

Older `schema_version: "1.0.0"` documents under `reports/` remain readable.
New machine-validated readiness output uses `experiment-v1` / `finding-v1` / `trace-v1`.
