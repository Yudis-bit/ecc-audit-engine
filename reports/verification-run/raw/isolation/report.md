# ecc-audit-engine Report

Generated: unix:1784302221

## Scope

Authorized local laboratory. Synthetic keys only.

## Differential summary

- Target: `iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so`
- Total cases: 148
- Failures: 14
- Raw: `reports/verification-run/raw/isolation/raw/differential_results.json`

## Findings

### DIFF-fe_mul_0_x_carry — Differential mismatch: fe_mul/0_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/0_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000000000000"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000000000001"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL.so --case fe_mul/0_x_carry`

### DIFF-fe_mul_1_x_carry — Differential mismatch: fe_mul/1_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/1_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000003ffffff"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000003fffffe"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL.so --case fe_mul/1_x_carry`

### DIFF-fe_mul_2_x_carry — Differential mismatch: fe_mul/2_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/2_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000007fffffe"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000007ffffff"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL.so --case fe_mul/2_x_carry`

### DIFF-fe_mul_p-1_x_carry — Differential mismatch: fe_mul/p-1_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/p-1_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefbfffc30"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefbfffc31"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL.so --case fe_mul/p-1_x_carry`

### DIFF-fe_mul_carry_x_0 — Differential mismatch: fe_mul/carry_x_0

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_0`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000000000000"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000000000001"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL.so --case fe_mul/carry_x_0`

### DIFF-fe_mul_carry_x_1 — Differential mismatch: fe_mul/carry_x_1

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_1`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000003ffffff"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000003fffffe"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL.so --case fe_mul/carry_x_1`

### DIFF-fe_mul_carry_x_2 — Differential mismatch: fe_mul/carry_x_2

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_2`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000007fffffe"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000007ffffff"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL.so --case fe_mul/carry_x_2`

### DIFF-fe_mul_carry_x_p-1 — Differential mismatch: fe_mul/carry_x_p-1

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_p-1`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefbfffc30"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefbfffc31"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL.so --case fe_mul/carry_x_p-1`

### DIFF-fe_mul_carry_x_carry — Differential mismatch: fe_mul/carry_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("000000000000000000000000000000000000000000000000000ffffff8000001"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("000000000000000000000000000000000000000000000000000ffffff8000000"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL.so --case fe_mul/carry_x_carry`

### DIFF-fe_mul_carry_x_near_mid — Differential mismatch: fe_mul/carry_x_near_mid

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_near_mid`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("00000000000000000000000003ffffff00000000000000000000000000000000"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("00000000000000000000000003ffffff00000000000000000000000000000001"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL.so --case fe_mul/carry_x_near_mid`

### DIFF-fe_mul_near_mid_x_carry — Differential mismatch: fe_mul/near_mid_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/near_mid_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("00000000000000000000000003ffffff00000000000000000000000000000000"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("00000000000000000000000003ffffff00000000000000000000000000000001"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL.so --case fe_mul/near_mid_x_carry`

### DIFF-point_add_G+(-G) — Differential mismatch: point_add/G+(-G)

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `point_add/G+(-G)`
- Mismatch: Some(InfinityMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("00"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_INFINITY_A.so --case point_add/G+(-G)`

### DIFF-scalar_n — Differential mismatch: scalar/n

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `scalar/n`
- Mismatch: Some(InfinityMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("00"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_SCALAR_BOUNARY.so --case scalar/n`

### DIFF-point_mul_n*G — Differential mismatch: point_mul/n*G

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `point_mul/n*G`
- Mismatch: Some(InfinityMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("00"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_SCALAR_BOUNARY.so --case point_mul/n*G`

### DIFF-scalar_n — Differential mismatch: scalar/n

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `scalar/n`
- Mismatch: Some(InfinityMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("00"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case scalar/n`

### DIFF-fe_mul_0_x_carry — Differential mismatch: fe_mul/0_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/0_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000000000000"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000000000001"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case fe_mul/0_x_carry`

### DIFF-fe_mul_1_x_carry — Differential mismatch: fe_mul/1_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/1_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000003ffffff"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000003fffffe"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case fe_mul/1_x_carry`

### DIFF-fe_mul_2_x_carry — Differential mismatch: fe_mul/2_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/2_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000007fffffe"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000007ffffff"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case fe_mul/2_x_carry`

### DIFF-fe_mul_p-1_x_carry — Differential mismatch: fe_mul/p-1_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/p-1_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefbfffc30"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefbfffc31"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case fe_mul/p-1_x_carry`

### DIFF-fe_mul_carry_x_0 — Differential mismatch: fe_mul/carry_x_0

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_0`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000000000000"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000000000001"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case fe_mul/carry_x_0`

### DIFF-fe_mul_carry_x_1 — Differential mismatch: fe_mul/carry_x_1

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_1`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000003ffffff"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000003fffffe"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case fe_mul/carry_x_1`

### DIFF-fe_mul_carry_x_2 — Differential mismatch: fe_mul/carry_x_2

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_2`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("0000000000000000000000000000000000000000000000000000000007fffffe"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0000000000000000000000000000000000000000000000000000000007ffffff"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case fe_mul/carry_x_2`

### DIFF-fe_mul_carry_x_p-1 — Differential mismatch: fe_mul/carry_x_p-1

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_p-1`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefbfffc30"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefbfffc31"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case fe_mul/carry_x_p-1`

### DIFF-fe_mul_carry_x_carry — Differential mismatch: fe_mul/carry_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("000000000000000000000000000000000000000000000000000ffffff8000001"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("000000000000000000000000000000000000000000000000000ffffff8000000"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case fe_mul/carry_x_carry`

### DIFF-fe_mul_carry_x_near_mid — Differential mismatch: fe_mul/carry_x_near_mid

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/carry_x_near_mid`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("00000000000000000000000003ffffff00000000000000000000000000000000"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("00000000000000000000000003ffffff00000000000000000000000000000001"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case fe_mul/carry_x_near_mid`

### DIFF-fe_mul_near_mid_x_carry — Differential mismatch: fe_mul/near_mid_x_carry

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `fe_mul/near_mid_x_carry`
- Mismatch: Some(ArithmeticMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("00000000000000000000000003ffffff00000000000000000000000000000000"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("00000000000000000000000003ffffff00000000000000000000000000000001"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case fe_mul/near_mid_x_carry`

### DIFF-point_add_G+(-G) — Differential mismatch: point_add/G+(-G)

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `point_add/G+(-G)`
- Mismatch: Some(InfinityMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("00"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case point_add/G+(-G)`

### DIFF-point_mul_n*G — Differential mismatch: point_mul/n*G

- Level: Level2DynamicDivergence
- Category: Differential
- Case: `point_mul/n*G`
- Mismatch: Some(InfinityMismatch)
- Expected: EvidenceValue { kind: "expected", hex: Some("00"), text: None }
- Observed: EvidenceValue { kind: "observed", hex: Some("0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8"), text: None }
- Impact: Laboratory fixture only. Synthetic keys. Not a production vulnerability claim.
- Repro: `cargo run -p cli -- differential --target targets/iso_iso_CORRUPT_FE_MUL__CORRUPT_INFINITY_A__CORRUPT_SCALAR_BOUNARY.so --case point_mul/n*G`

## Synthetic leak calibration

## Timing

## Notes

- Authorized laboratory differential run.
- No real-world secp256k1 vulnerability was tested or confirmed.
- Authorized laboratory differential run.
- No real-world secp256k1 vulnerability was tested or confirmed.
- Authorized laboratory differential run.
- No real-world secp256k1 vulnerability was tested or confirmed.
- Authorized laboratory differential run.
- No real-world secp256k1 vulnerability was tested or confirmed.
- Authorized laboratory differential run.
- No real-world secp256k1 vulnerability was tested or confirmed.

**No real-world secp256k1 vulnerability was tested or confirmed.**
