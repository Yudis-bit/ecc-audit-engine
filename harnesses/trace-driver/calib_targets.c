/*
 * Minimal calibration targets for dynamic tracing (no full scalar mul).
 * Planted branch / table / control only — public output is dummy.
 */
#include "../../crates/target-api/include/ecc_target.h"
#include "trace_markers.h"
#include <string.h>
#include <stdint.h>

#ifndef CALIB_MODE
#define CALIB_MODE 0 /* 0=control 1=branch 2=table */
#endif

static volatile unsigned long long g_counter;
static volatile uint8_t g_sink;
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

    ecc_trace_region_begin();
#if CALIB_MODE == 1
    if (secret_key[31] & 1) {
        g_counter++;
    }
#elif CALIB_MODE == 2
    {
        unsigned idx = (unsigned)(secret_key[31] & 0x0f);
        g_sink = secret_table[idx][0];
        (void)g_sink;
    }
#else
    {
        volatile unsigned long long acc = 0;
        for (unsigned i = 0; i < 16; i++) acc += secret_table[i][0];
        g_counter += (unsigned long long)(secret_key[31] & 1) * 0;
        (void)acc;
    }
#endif
    ecc_trace_region_end();

    /* Dummy valid-looking uncompressed point (not a real pubkey). */
    memset(output, 0, 65);
    output[0] = 0x04;
    output[32] = 1;
    output[64] = 1;
    return ECC_TARGET_OK;
}

int ecc_target_point_add(const uint8_t *a, size_t al, const uint8_t *b, size_t bl,
                         uint8_t *o, size_t ol) {
    (void)a;(void)al;(void)b;(void)bl;(void)o;(void)ol;
    return ECC_TARGET_REJECT;
}
int ecc_target_point_mul(const uint8_t *s, size_t sl, const uint8_t *p, size_t pl,
                         uint8_t *o, size_t ol) {
    (void)s;(void)sl;(void)p;(void)pl;(void)o;(void)ol;
    return ECC_TARGET_REJECT;
}
int ecc_target_fe_mul(const uint8_t *a, size_t al, const uint8_t *b, size_t bl,
                      uint8_t *o, size_t ol) {
    (void)a;(void)al;(void)b;(void)bl;(void)o;(void)ol;
    return ECC_TARGET_REJECT;
}
int ecc_target_leak_mode(void) { return CALIB_MODE; }
unsigned long long ecc_target_leak_counter_swap(unsigned long long v) {
    unsigned long long old = g_counter; g_counter = v; return old;
}
unsigned ecc_target_last_table_index(void) { return 0xff; }
