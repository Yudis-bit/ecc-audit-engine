#!/usr/bin/env python3
"""Lightweight JSON Schema validation for ecc-audit-engine reports."""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMAS = ROOT / "schemas"


def load(p: Path):
    with open(p, encoding="utf-8") as f:
        return json.load(f)


def type_ok(value, expected: str) -> bool:
    if expected == "string":
        return isinstance(value, str)
    if expected == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if expected == "number":
        return isinstance(value, (int, float)) and not isinstance(value, bool)
    if expected == "boolean":
        return isinstance(value, bool)
    if expected == "object":
        return isinstance(value, dict)
    if expected == "array":
        return isinstance(value, list)
    if expected == "null":
        return value is None
    return True


def validate(instance, schema, path="$") -> list[str]:
    errors: list[str] = []
    if "type" in schema:
        t = schema["type"]
        if isinstance(t, list):
            if not any(type_ok(instance, x) for x in t):
                errors.append(f"{path}: expected type {t}, got {type(instance).__name__}")
                return errors
        else:
            if not type_ok(instance, t):
                errors.append(f"{path}: expected type {t}, got {type(instance).__name__}")
                return errors
    if schema.get("type") == "object" or "properties" in schema or "required" in schema:
        if not isinstance(instance, dict):
            return errors
        for key in schema.get("required", []):
            if key not in instance:
                errors.append(f"{path}: missing required field '{key}'")
        props = schema.get("properties", {})
        for key, sub in props.items():
            if key in instance:
                errors.extend(validate(instance[key], sub, f"{path}.{key}"))
        if schema.get("additionalProperties") is False:
            allowed = set(props)
            for key in instance:
                if key not in allowed:
                    errors.append(f"{path}: unexpected field '{key}'")
    if schema.get("type") == "array" and isinstance(instance, list):
        item_schema = schema.get("items")
        if item_schema:
            for i, item in enumerate(instance):
                errors.extend(validate(item, item_schema, f"{path}[{i}]"))
    if "enum" in schema and instance not in schema["enum"]:
        errors.append(f"{path}: value {instance!r} not in enum {schema['enum']}")
    return errors


def main() -> int:
    schemas = {
        "finding": SCHEMAS / "finding-v1.schema.json",
        "experiment": SCHEMAS / "experiment-v1.schema.json",
        "trace": SCHEMAS / "trace-v1.schema.json",
    }
    for name, path in schemas.items():
        if not path.is_file():
            print(f"MISSING schema: {path}", file=sys.stderr)
            return 1
        load(path)
        print(f"ok schema file: {path.name}")

    # Validate any available readiness / published compact JSON that matches experiment schema
    exp_schema = load(schemas["experiment"])
    candidates = [
        ROOT / "reports" / "readiness-run" / "generated" / "readiness-report.json",
        ROOT / "reports" / "readiness-run" / "readiness-report.json",
        ROOT / "reports" / "dynamic-trace" / "calibration-report.json",
        ROOT / "research" / "upstream-coverage-matrix.json",
    ]
    validated = 0
    for c in candidates:
        if not c.is_file():
            continue
        data = load(c)
        # Only enforce experiment schema when document declares experiment-v1 or readiness kind
        if isinstance(data, dict) and (
            data.get("schema_version") in ("experiment-v1", "1.0.0", "pre-readiness-manifest-v1")
            or data.get("kind") == "readiness-verification"
        ):
            # soft: require subset of experiment fields if schema_version is experiment-v1
            if data.get("schema_version") == "experiment-v1":
                errs = validate(data, exp_schema)
                if errs:
                    print(f"schema errors in {c}:", file=sys.stderr)
                    for e in errs:
                        print(f"  {e}", file=sys.stderr)
                    return 1
                print(f"ok experiment schema: {c}")
                validated += 1
            else:
                print(f"ok readable report (legacy/soft): {c}")
                validated += 1
        else:
            print(f"ok json parse: {c}")
            validated += 1

    # Unit-level fixture for finding schema
    finding_schema = load(schemas["finding"])
    sample_finding = {
        "schema_version": "finding-v1",
        "id": "SYNTH-CALIB-001",
        "title": "Synthetic branch calibration",
        "classification": "Synthetic calibration findings",
        "engine_commit": "0" * 40,
        "target_commit": None,
        "target_binary_hash": "a" * 64,
        "compiler": "cc",
        "build_flags": "-O2",
        "backend": "valgrind-lackey",
        "corpus_hash": "b" * 64,
        "experiment_seed": 1337,
        "operation": "pubkey_create",
        "sample_count": 2,
        "result": "detected",
        "limitations": ["Synthetic fixture only."],
        "reproduction_command": "./scripts/run_trace_calibration.sh",
    }
    errs = validate(sample_finding, finding_schema)
    if errs:
        print("sample finding failed:", errs, file=sys.stderr)
        return 1
    print("ok sample finding-v1")

    trace_schema = load(schemas["trace"])
    sample_trace = {
        "schema_version": "trace-v1",
        "engine_commit": "0" * 40,
        "target_binary_hash": "a" * 64,
        "backend": "valgrind-lackey",
        "backend_version": "valgrind-3.22.0",
        "operation": "pubkey_create",
        "experiment_seed": 1337,
        "sample_count": 2,
        "result": "table_address_divergence",
        "instruction_module_relative_offsets_equal": True,
        "static_mem_set_equal": False,
        "static_cache_line_set_equal": False,
        "control_clean": True,
        "limitations": ["Synthetic table fixture."],
        "reproduction_command": "./scripts/run_trace_calibration.sh",
    }
    errs = validate(sample_trace, trace_schema)
    if errs:
        print("sample trace failed:", errs, file=sys.stderr)
        return 1
    print("ok sample trace-v1")
    print(f"validate_schemas complete (documents_seen={validated})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
