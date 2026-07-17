/* Stable C ABI for ecc-audit-engine local targets.
 * Authorized laboratory use only. Synthetic keys.
 */
#ifndef ECC_TARGET_H
#define ECC_TARGET_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define ECC_TARGET_OK 0
#define ECC_TARGET_REJECT 1
#define ECC_TARGET_INTERNAL_ERROR -1

/* Optional calibration hooks for leaky target (may be unimplemented). */
#define ECC_TARGET_LEAK_NONE 0
#define ECC_TARGET_LEAK_BRANCH 1
#define ECC_TARGET_LEAK_TABLE 2

int ecc_target_pubkey_create(
    const uint8_t *secret_key,
    size_t secret_key_len,
    uint8_t *output,
    size_t output_len
);

int ecc_target_point_add(
    const uint8_t *point_a,
    size_t point_a_len,
    const uint8_t *point_b,
    size_t point_b_len,
    uint8_t *output,
    size_t output_len
);

int ecc_target_point_mul(
    const uint8_t *scalar,
    size_t scalar_len,
    const uint8_t *point,
    size_t point_len,
    uint8_t *output,
    size_t output_len
);

int ecc_target_fe_mul(
    const uint8_t *a,
    size_t a_len,
    const uint8_t *b,
    size_t b_len,
    uint8_t *output,
    size_t output_len
);

/* Returns leak mode bitmask for calibration (0 if not a leaky build). */
int ecc_target_leak_mode(void);

/* Clears and returns synthetic leak counter (branch hits). */
unsigned long long ecc_target_leak_counter_swap(unsigned long long new_value);

/* Table sink last index observed (0xff if none). */
unsigned ecc_target_last_table_index(void);

#ifdef __cplusplus
}
#endif

#endif /* ECC_TARGET_H */
