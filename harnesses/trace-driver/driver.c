/*
 * Standalone driver for Valgrind Lackey (and similar) dynamic tracing.
 * Loads a target .so implementing ecc_target_pubkey_create, prints metadata
 * for ASLR normalization, then runs a single synthetic pubkey_create.
 *
 * Usage:
 *   ecc-trace-driver <target.so> <32-byte-hex-secret> <case_id>
 */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <link.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include "../../crates/target-api/include/ecc_target.h"
#include "trace_markers.h"

typedef int (*pubkey_fn)(const uint8_t *, size_t, uint8_t *, size_t);

static const char *g_soname_needle;
static uintptr_t g_module_base;
static int g_found;

static int phdr_cb(struct dl_phdr_info *info, size_t size, void *data) {
    (void)size;
    (void)data;
    if (!info->dlpi_name) return 0;
    if (g_soname_needle && strstr(info->dlpi_name, g_soname_needle)) {
        g_module_base = (uintptr_t)info->dlpi_addr;
        g_found = 1;
        return 1;
    }
    return 0;
}

static int hex_nibble(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}

static int parse_hex32(const char *hex, uint8_t out[32]) {
    if (strlen(hex) != 64) return -1;
    for (int i = 0; i < 32; i++) {
        int hi = hex_nibble(hex[i * 2]);
        int lo = hex_nibble(hex[i * 2 + 1]);
        if (hi < 0 || lo < 0) return -1;
        out[i] = (uint8_t)((hi << 4) | lo);
    }
    return 0;
}

/* Simple SHA-256 would need a lib; print hex input hash as FNV-1a 64 for ID only.
 * Full sha256 of input is computed by the Rust harness from the hex string. */
static uint64_t fnv1a64(const uint8_t *p, size_t n) {
    uint64_t h = 14695981039346656037ULL;
    for (size_t i = 0; i < n; i++) {
        h ^= p[i];
        h *= 1099511628211ULL;
    }
    return h;
}

int main(int argc, char **argv) {
    if (argc != 4) {
        fprintf(stderr, "usage: %s <target.so> <64-hex-sk> <case_id>\n", argv[0]);
        return 2;
    }
    const char *path = argv[1];
    const char *hexsk = argv[2];
    const char *case_id = argv[3];

    uint8_t sk[32];
    if (parse_hex32(hexsk, sk) != 0) {
        fprintf(stderr, "bad secret hex\n");
        return 2;
    }

    void *h = dlopen(path, RTLD_NOW | RTLD_LOCAL);
    if (!h) {
        fprintf(stderr, "dlopen: %s\n", dlerror());
        return 3;
    }

    pubkey_fn f = (pubkey_fn)dlsym(h, "ecc_target_pubkey_create");
    if (!f) {
        fprintf(stderr, "dlsym pubkey: %s\n", dlerror());
        return 3;
    }

    /* Prefer markers from the target .so if exported; else use local. */
    void (*begin)(void) = (void (*)(void))dlsym(h, "ecc_trace_region_begin");
    void (*end)(void) = (void (*)(void))dlsym(h, "ecc_trace_region_end");
    if (!begin) begin = ecc_trace_region_begin;
    if (!end) end = ecc_trace_region_end;

    /* Module base of the loaded target */
    const char *base = strrchr(path, '/');
    g_soname_needle = base ? base + 1 : path;
    g_found = 0;
    g_module_base = 0;
    dl_iterate_phdr(phdr_cb, NULL);

    Dl_info di_begin = {0}, di_end = {0}, di_pk = {0};
    dladdr((void *)begin, &di_begin);
    dladdr((void *)end, &di_end);
    dladdr((void *)f, &di_pk);

    uintptr_t base_addr = g_module_base;
    if (!g_found && di_pk.dli_fbase) {
        base_addr = (uintptr_t)di_pk.dli_fbase;
    }

    printf("TRACE_META schema_version=1.0.0\n");
    printf("TRACE_META backend=valgrind-lackey\n");
    printf("TRACE_META target_path=%s\n", path);
    printf("TRACE_META case_id=%s\n", case_id);
    printf("TRACE_META input_hex=%s\n", hexsk);
    printf("TRACE_META input_fnv1a64=0x%016llx\n",
           (unsigned long long)fnv1a64(sk, 32));
    printf("TRACE_META module_base=0x%llx\n", (unsigned long long)base_addr);
    printf("TRACE_META marker_begin_abs=0x%llx\n",
           (unsigned long long)(uintptr_t)begin);
    printf("TRACE_META marker_end_abs=0x%llx\n",
           (unsigned long long)(uintptr_t)end);
    printf("TRACE_META pubkey_create_abs=0x%llx\n",
           (unsigned long long)(uintptr_t)f);
    if (base_addr) {
        printf("TRACE_META marker_begin_off=0x%llx\n",
               (unsigned long long)((uintptr_t)begin - base_addr));
        printf("TRACE_META marker_end_off=0x%llx\n",
               (unsigned long long)((uintptr_t)end - base_addr));
        printf("TRACE_META pubkey_create_off=0x%llx\n",
               (unsigned long long)((uintptr_t)f - base_addr));
    }
    fflush(stdout);

    uint8_t out[65];
    memset(out, 0, sizeof out);
    /* Optional: ECC_TRACE_FAST=1 runs only markers + a tiny target call pattern
     * used when the .so already wraps the crypto in markers (leaky/libsecp adapters).
     * Full path always used by default. */
    begin();
    int rc = f(sk, 32, out, 65);
    end();

    printf("TRACE_META result_rc=%d\n", rc);
    if (rc == 0) {
        printf("TRACE_META result_prefix=%02x\n", out[0]);
    }
    fflush(stdout);
    dlclose(h);
    return rc == 0 ? 0 : 1;
}
