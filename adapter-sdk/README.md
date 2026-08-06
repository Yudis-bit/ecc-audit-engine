# ECC Target Adapter SDK

Build your own target adapter for the ecc-audit-engine differential testing framework.

## What This Is

The ecc-audit-engine compares a **target** (your C shared library) against a **reference oracle** (the built-in Rust model). If your target implements secp256k1 field/group operations with a specific C ABI, you can integrate it in three steps.

## Quick Start

### 1. Implement the ABI

Your shared library must export these seven functions:

```c
#include "ecc_target.h"

int ecc_target_pubkey_create(
    const uint8_t *secret_key, size_t secret_key_len,
    uint8_t *output, size_t output_len);

int ecc_target_point_add(
    const uint8_t *point_a, size_t point_a_len,
    const uint8_t *point_b, size_t point_b_len,
    uint8_t *output, size_t output_len);

int ecc_target_point_mul(
    const uint8_t *scalar, size_t scalar_len,
    const uint8_t *point_in, size_t point_len,
    uint8_t *output, size_t output_len);

int ecc_target_fe_mul(
    const uint8_t *a, size_t a_len,
    const uint8_t *b, size_t b_len,
    uint8_t *output, size_t output_len);

int ecc_target_leak_mode(void);
unsigned long long ecc_target_leak_counter_swap(unsigned long long new_value);
unsigned ecc_target_last_table_index(void);
```

### Return Codes

| Value | Constant | Meaning |
|--:|:--|:--|
| 0 | `ECC_TARGET_OK` | Operation succeeded |
| 1 | `ECC_TARGET_REJECT` | Input rejected (off-curve, out of range, etc.) |
| -1 | `ECC_TARGET_INTERNAL_ERROR` | Unexpected internal failure |

### 2. Build Your Target

```bash
cc -std=c11 -Wall -Wextra -fPIC -O2 -shared \
  -I crates/target-api/include \
  -I harnesses/common \
  -o targets/your-target.so \
  your-target.c \
  harnesses/common/secp_mini.c
```

If you are using your own arithmetic library instead of `secp_mini.c`, link your implementation instead.

### 3. Run Differential Testing

```bash
cargo run -p cli -- differential \
  --target targets/your-target.so \
  --corpus fixtures/corpus-v1.json \
  --output reports/latest
```

## Example: Minimal Correct Target

See `adapter-sdk/examples/minimal-target.c` for a complete working example using the bundled `secp_mini.c` arithmetic library.

## Determinism Requirements

Your target must be **deterministic**:
- Same input → same output every time
- No random number generators
- No system-time dependencies
- No memory from uninitialized buffers
- No thread-scheduling dependencies

## FFI Safety

- All pointers are caller-owned; do not free them
- Output buffers are pre-allocated; respect `output_len`
- Return `ECC_TARGET_INTERNAL_ERROR` for any invalid pointer or length
- No global mutable state between calls
- Functions must be signal-safe (no `malloc` inside critical paths)

## Corrupted Target Example

The engine ships with a deliberately corrupted target (`harnesses/corrupted-target/target.c`) that introduces intentional errors in field multiplication, infinity handling, and scalar boundary checks. Build it to verify your engine setup:

```bash
cc -std=c11 -fPIC -O2 -shared \
  -DCORRUPT_FE_MUL -DCORRUPT_INFINITY_ADD -DCORRUPT_SCALAR_BOUNDARY \
  -I crates/target-api/include -I harnesses/common \
  -o targets/corrupted-target.so \
  harnesses/corrupted-target/target.c \
  harnesses/common/secp_mini.c
```

## Integration Checklist

- [ ] Clone ecc-audit-engine
- [ ] Verify `cargo test --workspace` passes
- [ ] Implement the 7 ABI functions for your target
- [ ] Build your target as a shared library
- [ ] Run `cargo run -p cli -- differential --target targets/your-target.so --corpus fixtures/corpus-v1.json`
- [ ] Observe output in `reports/latest/`
- [ ] Run the corrupted target to verify detection works
- [ ] Run `cargo run -p cli -- help` to see all commands

## Private Target Integration

For private or proprietary implementations, contact for a scoped integration engagement. The adapter SDK is designed to work offline — your code never leaves your machine.

## Limitations

- Targets must be Linux x86_64 shared libraries
- The reference oracle covers secp256k1 field and group operations
- Only the C ABI is currently supported
- The engine does not perform formal verification
- Timing analysis is bounded and experimental
- Results are differential — they detect discrepancies, not guarantee correctness

## Support

- [Open an issue](https://github.com/Yudis-bit/ecc-audit-engine/issues) for bugs or questions
- Contact for paid integration support, custom adapters, or private deployment