/*
 * Corrupted laboratory target. Defects behind compile-time flags:
 *   -DCORRUPT_FE_MUL
 *   -DCORRUPT_INFINITY_ADD
 *   -DCORRUPT_NEGATIVE_POINT_ADD  (alias path of infinity)
 *   -DCORRUPT_SCALAR_BOUNDARY
 */
#include "../../crates/target-api/include/ecc_target.h"
#include "../common/secp_mini.h"
#include <string.h>

int ecc_target_pubkey_create(
    const uint8_t *secret_key,
    size_t secret_key_len,
    uint8_t *output,
    size_t output_len
) {
    if (!secret_key || !output || secret_key_len != 32 || output_len < 65)
        return ECC_TARGET_INTERNAL_ERROR;
    u256 sk;
    u256_from_be(&sk, secret_key);
#ifdef CORRUPT_SCALAR_BOUNDARY
    /* CORRUPTION: accept zero scalar and return G instead of reject/infinity */
    if (u256_is_zero(&sk)) {
        point g;
        point_set_generator(&g);
        sec1_serialize_uncompressed(&g, output);
        return ECC_TARGET_OK;
    }
#endif
    if (u256_is_zero(&sk) || u256_cmp(&sk, &SECP_N) >= 0)
        return ECC_TARGET_REJECT;
    point g, q;
    point_set_generator(&g);
    if (point_mul(&q, &sk, &g) != 0)
        return ECC_TARGET_INTERNAL_ERROR;
    if (sec1_serialize_uncompressed(&q, output) != 0)
        return ECC_TARGET_INTERNAL_ERROR;
    return ECC_TARGET_OK;
}

int ecc_target_point_add(
    const uint8_t *point_a,
    size_t point_a_len,
    const uint8_t *point_b,
    size_t point_b_len,
    uint8_t *output,
    size_t output_len
) {
    if (!point_a || !point_b || !output || output_len < 65)
        return ECC_TARGET_INTERNAL_ERROR;
    point a, b, r;
    if (sec1_parse(&a, point_a, point_a_len) != 0)
        return ECC_TARGET_REJECT;
    if (sec1_parse(&b, point_b, point_b_len) != 0)
        return ECC_TARGET_REJECT;

#if defined(CORRUPT_INFINITY_ADD) || defined(CORRUPT_NEGATIVE_POINT_ADD)
    /* CORRUPTION: P + (-P) returns P instead of infinity */
    {
        point negb;
        point_neg(&negb, &b);
        if (!a.infinity && !b.infinity &&
            u256_cmp(&a.x, &b.x) == 0 &&
            u256_cmp(&a.y, &negb.y) != 0) {
            /* same x different y => negatives if both on curve */
            if (u256_cmp(&a.x, &b.x) == 0 && u256_cmp(&a.y, &b.y) != 0) {
                if (sec1_serialize_uncompressed(&a, output) != 0)
                    return ECC_TARGET_INTERNAL_ERROR;
                return ECC_TARGET_OK;
            }
        }
        /* Also explicit: if result would be infinity, return a */
        point tmp;
        if (point_add(&tmp, &a, &b) == 0 && tmp.infinity) {
            if (a.infinity) {
                output[0] = 0x00;
                return ECC_TARGET_OK;
            }
            if (sec1_serialize_uncompressed(&a, output) != 0)
                return ECC_TARGET_INTERNAL_ERROR;
            return ECC_TARGET_OK;
        }
    }
#endif

    if (point_add(&r, &a, &b) != 0)
        return ECC_TARGET_INTERNAL_ERROR;
    if (r.infinity) {
        output[0] = 0x00;
        return ECC_TARGET_OK;
    }
    if (sec1_serialize_uncompressed(&r, output) != 0)
        return ECC_TARGET_INTERNAL_ERROR;
    return ECC_TARGET_OK;
}

int ecc_target_point_mul(
    const uint8_t *scalar,
    size_t scalar_len,
    const uint8_t *point_in,
    size_t point_len,
    uint8_t *output,
    size_t output_len
) {
    if (!scalar || !point_in || !output || scalar_len != 32 || output_len < 65)
        return ECC_TARGET_INTERNAL_ERROR;
    point p, r;
    if (sec1_parse(&p, point_in, point_len) != 0)
        return ECC_TARGET_REJECT;
    u256 k;
    u256_from_be(&k, scalar);
#ifdef CORRUPT_SCALAR_BOUNDARY
    /* CORRUPTION: n*P returns P instead of infinity */
    if (u256_cmp(&k, &SECP_N) == 0) {
        if (p.infinity) {
            output[0] = 0x00;
            return ECC_TARGET_OK;
        }
        if (sec1_serialize_uncompressed(&p, output) != 0)
            return ECC_TARGET_INTERNAL_ERROR;
        return ECC_TARGET_OK;
    }
#endif
    scalar_reduce(&k);
    if (point_mul(&r, &k, &p) != 0)
        return ECC_TARGET_INTERNAL_ERROR;
    if (r.infinity) {
        output[0] = 0x00;
        return ECC_TARGET_OK;
    }
    if (sec1_serialize_uncompressed(&r, output) != 0)
        return ECC_TARGET_INTERNAL_ERROR;
    return ECC_TARGET_OK;
}

int ecc_target_fe_mul(
    const uint8_t *a,
    size_t a_len,
    const uint8_t *b,
    size_t b_len,
    uint8_t *output,
    size_t output_len
) {
    if (!a || !b || !output || a_len != 32 || b_len != 32 || output_len < 32)
        return ECC_TARGET_INTERNAL_ERROR;
    u256 fa, fb, fr;
    u256_from_be(&fa, a);
    u256_from_be(&fb, b);
    if (u256_cmp(&fa, &SECP_P) >= 0 || u256_cmp(&fb, &SECP_P) >= 0)
        return ECC_TARGET_REJECT;
    fe_mul(&fr, &fa, &fb);
    fe_normalize(&fr);
    u256_to_be(&fr, output);
#ifdef CORRUPT_FE_MUL
    /* CORRUPTION: flip LSB of product for carry-heavy class (all-ones low limb patterns)
     * Detect: a and b both have low 8 bytes = 0xff... or product path when a[31]==0xff */
    if (a[31] == 0xff || b[31] == 0xff) {
        output[31] ^= 0x01; /* flip one output bit */
    }
#endif
    return ECC_TARGET_OK;
}

int ecc_target_leak_mode(void) { return ECC_TARGET_LEAK_NONE; }
unsigned long long ecc_target_leak_counter_swap(unsigned long long v) {
    (void)v;
    return 0;
}
unsigned ecc_target_last_table_index(void) { return 0xff; }
