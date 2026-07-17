/*
 * Adapter: maps ecc_target ABI onto public libsecp256k1 API only.
 * Upstream cryptographic sources are not modified.
 * Synthetic keys / laboratory use only.
 */
#include "../../crates/target-api/include/ecc_target.h"
#include "../trace-driver/trace_markers.h"
#include "secp256k1.h"
#include <string.h>
#include <stdlib.h>

static secp256k1_context *g_ctx;

static void ensure_ctx(void) {
    if (!g_ctx) {
        g_ctx = secp256k1_context_create(SECP256K1_CONTEXT_NONE);
    }
}

/* Constructor for shared object load */
__attribute__((constructor)) static void init_ctx(void) {
    ensure_ctx();
}

__attribute__((destructor)) static void fini_ctx(void) {
    if (g_ctx) {
        secp256k1_context_destroy(g_ctx);
        g_ctx = NULL;
    }
}

int ecc_target_pubkey_create(
    const uint8_t *secret_key,
    size_t secret_key_len,
    uint8_t *output,
    size_t output_len
) {
    ensure_ctx();
    if (!secret_key || !output || secret_key_len != 32 || output_len < 65)
        return ECC_TARGET_INTERNAL_ERROR;
    /* API policy: zero or out-of-range secret → REJECT */
    if (!secp256k1_ec_seckey_verify(g_ctx, secret_key))
        return ECC_TARGET_REJECT;
    secp256k1_pubkey pk;
    /* Trace region: public-key creation only (markers outside upstream). */
    ecc_trace_region_begin();
    int ok = secp256k1_ec_pubkey_create(g_ctx, &pk, secret_key);
    ecc_trace_region_end();
    if (!ok)
        return ECC_TARGET_REJECT;
    size_t len = 65;
    if (!secp256k1_ec_pubkey_serialize(
            g_ctx, output, &len, &pk, SECP256K1_EC_UNCOMPRESSED))
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
    ensure_ctx();
    if (!point_a || !point_b || !output || output_len < 65)
        return ECC_TARGET_INTERNAL_ERROR;
    /* Infinity laboratory encoding */
    if (point_a_len == 1 && point_a[0] == 0x00) {
        if (point_b_len == 1 && point_b[0] == 0x00) {
            output[0] = 0x00;
            return ECC_TARGET_OK;
        }
        /* O + B = B if B valid */
        secp256k1_pubkey pb;
        if (!secp256k1_ec_pubkey_parse(g_ctx, &pb, point_b, point_b_len))
            return ECC_TARGET_REJECT;
        size_t len = 65;
        if (!secp256k1_ec_pubkey_serialize(
                g_ctx, output, &len, &pb, SECP256K1_EC_UNCOMPRESSED))
            return ECC_TARGET_INTERNAL_ERROR;
        return ECC_TARGET_OK;
    }
    if (point_b_len == 1 && point_b[0] == 0x00) {
        secp256k1_pubkey pa;
        if (!secp256k1_ec_pubkey_parse(g_ctx, &pa, point_a, point_a_len))
            return ECC_TARGET_REJECT;
        size_t len = 65;
        if (!secp256k1_ec_pubkey_serialize(
                g_ctx, output, &len, &pa, SECP256K1_EC_UNCOMPRESSED))
            return ECC_TARGET_INTERNAL_ERROR;
        return ECC_TARGET_OK;
    }
    secp256k1_pubkey pa, pb;
    if (!secp256k1_ec_pubkey_parse(g_ctx, &pa, point_a, point_a_len))
        return ECC_TARGET_REJECT;
    if (!secp256k1_ec_pubkey_parse(g_ctx, &pb, point_b, point_b_len))
        return ECC_TARGET_REJECT;
    const secp256k1_pubkey *ins[2] = {&pa, &pb};
    secp256k1_pubkey out;
    if (!secp256k1_ec_pubkey_combine(g_ctx, &out, ins, 2)) {
        /* combine fails if result is infinity */
        output[0] = 0x00;
        return ECC_TARGET_OK;
    }
    size_t len = 65;
    if (!secp256k1_ec_pubkey_serialize(
            g_ctx, output, &len, &out, SECP256K1_EC_UNCOMPRESSED))
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
    ensure_ctx();
    if (!scalar || !point_in || !output || scalar_len != 32 || output_len < 65)
        return ECC_TARGET_INTERNAL_ERROR;
    if (point_len == 1 && point_in[0] == 0x00) {
        output[0] = 0x00;
        return ECC_TARGET_OK;
    }
    secp256k1_pubkey p;
    if (!secp256k1_ec_pubkey_parse(g_ctx, &p, point_in, point_len))
        return ECC_TARGET_REJECT;
    /* Public API: tweak_mul expects tweak in [0,n); zero tweak → infinity policy via API */
    if (!secp256k1_ec_pubkey_tweak_mul(g_ctx, &p, scalar)) {
        /* fails for invalid tweak or infinity result */
        /* distinguish: all-zero scalar → infinity */
        int zero = 1;
        for (size_t i = 0; i < 32; i++)
            if (scalar[i]) {
                zero = 0;
                break;
            }
        if (zero) {
            output[0] = 0x00;
            return ECC_TARGET_OK;
        }
        return ECC_TARGET_REJECT;
    }
    size_t len = 65;
    if (!secp256k1_ec_pubkey_serialize(
            g_ctx, output, &len, &p, SECP256K1_EC_UNCOMPRESSED))
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
    (void)a;
    (void)a_len;
    (void)b;
    (void)b_len;
    (void)output;
    (void)output_len;
    /* Field mul is not a public API surface of libsecp256k1. */
    return ECC_TARGET_REJECT;
}

int ecc_target_leak_mode(void) { return ECC_TARGET_LEAK_NONE; }
unsigned long long ecc_target_leak_counter_swap(unsigned long long v) {
    (void)v;
    return 0;
}
unsigned ecc_target_last_table_index(void) { return 0xff; }
