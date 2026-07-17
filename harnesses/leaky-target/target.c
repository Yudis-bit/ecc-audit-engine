/*
 * Synthetic leaky laboratory fixtures for calibration only.
 * Modes (compile -DLEAK_MODE=branch|table|control):
 *   branch: secret-dependent branch on sk[31] LSB
 *   table:  secret-dependent table index
 *   control: constant-time control (no secret branch/index)
 */
#include "../../crates/target-api/include/ecc_target.h"
#include "../common/secp_mini.h"
#include <string.h>

#ifndef LEAK_MODE
#define LEAK_MODE 0 /* 0=control, 1=branch, 2=table */
#endif

static volatile unsigned long long g_leak_counter = 0;
static volatile unsigned g_last_table_index = 0xff;
static volatile uint8_t g_sink;

/* Synthetic table (16 lines) */
static const uint8_t secret_table[16][64] = {
    {0x00}, {0x01}, {0x02}, {0x03}, {0x04}, {0x05}, {0x06}, {0x07},
    {0x08}, {0x09}, {0x0a}, {0x0b}, {0x0c}, {0x0d}, {0x0e}, {0x0f},
};

int ecc_target_pubkey_create(
    const uint8_t *secret_key,
    size_t secret_key_len,
    uint8_t *output,
    size_t output_len
) {
    if (!secret_key || !output || secret_key_len != 32 || output_len < 65)
        return ECC_TARGET_INTERNAL_ERROR;

#if LEAK_MODE == 1
    /* PLANTED: secret-dependent branch */
    if (secret_key[31] & 1) {
        g_leak_counter++;
    }
#elif LEAK_MODE == 2
    /* PLANTED: secret-dependent memory lookup */
    {
        unsigned idx = (unsigned)(secret_key[31] & 0x0f);
        g_last_table_index = idx;
        g_sink = secret_table[idx][0];
        (void)g_sink;
    }
#elif LEAK_MODE == 0
    /* CONTROL: touch all table entries and both branch sides equivalently */
    {
        volatile unsigned long long acc = 0;
        for (unsigned i = 0; i < 16; i++) {
            acc += secret_table[i][0];
        }
        /* branchless counter update using mask */
        unsigned long long lsb = (unsigned long long)(secret_key[31] & 1);
        g_leak_counter += lsb * 0; /* no-op but data-dependent arithmetic only */
        (void)acc;
        g_last_table_index = 0xff;
    }
#endif

    u256 sk;
    u256_from_be(&sk, secret_key);
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
    const uint8_t *point_a, size_t point_a_len,
    const uint8_t *point_b, size_t point_b_len,
    uint8_t *output, size_t output_len
) {
    if (!point_a || !point_b || !output || output_len < 65)
        return ECC_TARGET_INTERNAL_ERROR;
    point a, b, r;
    if (sec1_parse(&a, point_a, point_a_len) != 0) return ECC_TARGET_REJECT;
    if (sec1_parse(&b, point_b, point_b_len) != 0) return ECC_TARGET_REJECT;
    if (point_add(&r, &a, &b) != 0) return ECC_TARGET_INTERNAL_ERROR;
    if (r.infinity) { output[0] = 0x00; return ECC_TARGET_OK; }
    if (sec1_serialize_uncompressed(&r, output) != 0) return ECC_TARGET_INTERNAL_ERROR;
    return ECC_TARGET_OK;
}

int ecc_target_point_mul(
    const uint8_t *scalar, size_t scalar_len,
    const uint8_t *point_in, size_t point_len,
    uint8_t *output, size_t output_len
) {
    if (!scalar || !point_in || !output || scalar_len != 32 || output_len < 65)
        return ECC_TARGET_INTERNAL_ERROR;
    point p, r;
    if (sec1_parse(&p, point_in, point_len) != 0) return ECC_TARGET_REJECT;
    u256 k;
    u256_from_be(&k, scalar);
    scalar_reduce(&k);
    if (point_mul(&r, &k, &p) != 0) return ECC_TARGET_INTERNAL_ERROR;
    if (r.infinity) { output[0] = 0x00; return ECC_TARGET_OK; }
    if (sec1_serialize_uncompressed(&r, output) != 0) return ECC_TARGET_INTERNAL_ERROR;
    return ECC_TARGET_OK;
}

int ecc_target_fe_mul(
    const uint8_t *a, size_t a_len,
    const uint8_t *b, size_t b_len,
    uint8_t *output, size_t output_len
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
    return ECC_TARGET_OK;
}

int ecc_target_leak_mode(void) {
#if LEAK_MODE == 1
    return ECC_TARGET_LEAK_BRANCH;
#elif LEAK_MODE == 2
    return ECC_TARGET_LEAK_TABLE;
#else
    return ECC_TARGET_LEAK_NONE;
#endif
}

unsigned long long ecc_target_leak_counter_swap(unsigned long long new_value) {
    unsigned long long old = g_leak_counter;
    g_leak_counter = new_value;
    return old;
}

unsigned ecc_target_last_table_index(void) {
    return g_last_table_index;
}
