#include "secp_mini.h"

/* p = 2^256 - 2^32 - 977 */
const u256 SECP_P = {{
    0xFFFFFFFEFFFFFC2FULL,
    0xFFFFFFFFFFFFFFFFULL,
    0xFFFFFFFFFFFFFFFFULL,
    0xFFFFFFFFFFFFFFFFULL
}};

/* n */
const u256 SECP_N = {{
    0xBFD25E8CD0364141ULL,
    0xBAAEDCE6AF48A03BULL,
    0xFFFFFFFFFFFFFFFEULL,
    0xFFFFFFFFFFFFFFFFULL
}};

static const u256 SECP_GX = {{
    0x59F2815B16F81798ULL,
    0x029BFCDB2DCE28D9ULL,
    0x55A06295CE870B07ULL,
    0x79BE667EF9DCBBACULL
}};

static const u256 SECP_GY = {{
    0x9C47D08FFB10D4B8ULL,
    0xFD17B448A6855419ULL,
    0x5DA4FBFC0E1108A8ULL,
    0x483ADA7726A3C465ULL
}};

static const u256 SECP_B = {{7, 0, 0, 0}};

void u256_set_u64(u256 *o, uint64_t v) {
    o->d[0] = v;
    o->d[1] = o->d[2] = o->d[3] = 0;
}

int u256_is_zero(const u256 *a) {
    return (a->d[0] | a->d[1] | a->d[2] | a->d[3]) == 0;
}

int u256_cmp(const u256 *a, const u256 *b) {
    for (int i = 3; i >= 0; i--) {
        if (a->d[i] < b->d[i]) return -1;
        if (a->d[i] > b->d[i]) return 1;
    }
    return 0;
}

int u256_from_be(u256 *o, const uint8_t *be32) {
    o->d[3] = ((uint64_t)be32[0] << 56) | ((uint64_t)be32[1] << 48) |
              ((uint64_t)be32[2] << 40) | ((uint64_t)be32[3] << 32) |
              ((uint64_t)be32[4] << 24) | ((uint64_t)be32[5] << 16) |
              ((uint64_t)be32[6] << 8) | (uint64_t)be32[7];
    o->d[2] = ((uint64_t)be32[8] << 56) | ((uint64_t)be32[9] << 48) |
              ((uint64_t)be32[10] << 40) | ((uint64_t)be32[11] << 32) |
              ((uint64_t)be32[12] << 24) | ((uint64_t)be32[13] << 16) |
              ((uint64_t)be32[14] << 8) | (uint64_t)be32[15];
    o->d[1] = ((uint64_t)be32[16] << 56) | ((uint64_t)be32[17] << 48) |
              ((uint64_t)be32[18] << 40) | ((uint64_t)be32[19] << 32) |
              ((uint64_t)be32[20] << 24) | ((uint64_t)be32[21] << 16) |
              ((uint64_t)be32[22] << 8) | (uint64_t)be32[23];
    o->d[0] = ((uint64_t)be32[24] << 56) | ((uint64_t)be32[25] << 48) |
              ((uint64_t)be32[26] << 40) | ((uint64_t)be32[27] << 32) |
              ((uint64_t)be32[28] << 24) | ((uint64_t)be32[29] << 16) |
              ((uint64_t)be32[30] << 8) | (uint64_t)be32[31];
    return 0;
}

void u256_to_be(const u256 *a, uint8_t *be32) {
    for (int limb = 3; limb >= 0; limb--) {
        uint64_t v = a->d[limb];
        int off = (3 - limb) * 8;
        be32[off + 0] = (uint8_t)(v >> 56);
        be32[off + 1] = (uint8_t)(v >> 48);
        be32[off + 2] = (uint8_t)(v >> 40);
        be32[off + 3] = (uint8_t)(v >> 32);
        be32[off + 4] = (uint8_t)(v >> 24);
        be32[off + 5] = (uint8_t)(v >> 16);
        be32[off + 6] = (uint8_t)(v >> 8);
        be32[off + 7] = (uint8_t)(v);
    }
}

/* r = a + b (256-bit, returns carry) */
static uint64_t add256(u256 *r, const u256 *a, const u256 *b) {
    unsigned __int128 c = 0;
    for (int i = 0; i < 4; i++) {
        c += (unsigned __int128)a->d[i] + b->d[i];
        r->d[i] = (uint64_t)c;
        c >>= 64;
    }
    return (uint64_t)c;
}

static uint64_t sub256(u256 *r, const u256 *a, const u256 *b) {
    unsigned __int128 c = 0;
    for (int i = 0; i < 4; i++) {
        c = (unsigned __int128)a->d[i] - b->d[i] - c;
        r->d[i] = (uint64_t)c;
        c = (c >> 127) & 1; /* borrow */
    }
    return (uint64_t)c;
}

void fe_normalize(u256 *a) {
    if (u256_cmp(a, &SECP_P) >= 0) {
        u256 t;
        sub256(&t, a, &SECP_P);
        *a = t;
    }
}

void fe_add(u256 *r, const u256 *a, const u256 *b) {
    u256 t;
    uint64_t c = add256(&t, a, b);
    if (c || u256_cmp(&t, &SECP_P) >= 0) {
        sub256(r, &t, &SECP_P);
    } else {
        *r = t;
    }
}

void fe_sub(u256 *r, const u256 *a, const u256 *b) {
    u256 t;
    if (sub256(&t, a, b)) {
        /* borrowed: a < b → a + p - b */
        add256(r, a, &SECP_P);
        sub256(r, r, b);
    } else {
        *r = t;
    }
}

void fe_neg(u256 *r, const u256 *a) {
    if (u256_is_zero(a)) {
        u256_set_u64(r, 0);
    } else {
        sub256(r, &SECP_P, a);
    }
}

/* 512-bit product then reduce mod p = 2^256 - 2^32 - 977
 * Identity: 2^256 ≡ R (mod p) where R = 2^32 + 977 = 0x1000003D1
 * Carry is propagated during schoolbook multiply (no __int128 cell overflow).
 */
void fe_mul(u256 *r, const u256 *a, const u256 *b) {
    const uint64_t R = 0x1000003D1ULL;
    uint64_t w[8] = {0};
    for (int i = 0; i < 4; i++) {
        unsigned __int128 carry = 0;
        for (int j = 0; j < 4; j++) {
            unsigned __int128 t = (unsigned __int128)w[i + j]
                + (unsigned __int128)a->d[i] * b->d[j]
                + carry;
            w[i + j] = (uint64_t)t;
            carry = t >> 64;
        }
        int k = i + 4;
        while (carry) {
            unsigned __int128 t = (unsigned __int128)w[k] + carry;
            w[k] = (uint64_t)t;
            carry = t >> 64;
            k++;
        }
    }

    uint64_t z[6] = {w[0], w[1], w[2], w[3], 0, 0};
    for (int i = 0; i < 4; i++) {
        unsigned __int128 m = (unsigned __int128)w[4 + i] * R;
        unsigned __int128 c = (unsigned __int128)z[i] + (uint64_t)m;
        z[i] = (uint64_t)c;
        c >>= 64;
        c += (unsigned __int128)z[i + 1] + (uint64_t)(m >> 64);
        z[i + 1] = (uint64_t)c;
        c >>= 64;
        int k = i + 2;
        while (c) {
            c += z[k];
            z[k] = (uint64_t)c;
            c >>= 64;
            k++;
        }
    }
    for (int pass = 0; pass < 3; pass++) {
        if (z[4] == 0 && z[5] == 0) {
            break;
        }
        unsigned __int128 m0 = (unsigned __int128)z[4] * R;
        unsigned __int128 m1 = (unsigned __int128)z[5] * R;
        z[4] = z[5] = 0;
        unsigned __int128 c = (unsigned __int128)z[0] + (uint64_t)m0;
        z[0] = (uint64_t)c;
        c >>= 64;
        c += (unsigned __int128)z[1] + (uint64_t)(m0 >> 64) + (uint64_t)m1;
        z[1] = (uint64_t)c;
        c >>= 64;
        c += (unsigned __int128)z[2] + (uint64_t)(m1 >> 64);
        z[2] = (uint64_t)c;
        c >>= 64;
        c += z[3];
        z[3] = (uint64_t)c;
        c >>= 64;
        c += z[4];
        z[4] = (uint64_t)c;
        c >>= 64;
        z[5] = (uint64_t)c;
    }

    r->d[0] = z[0];
    r->d[1] = z[1];
    r->d[2] = z[2];
    r->d[3] = z[3];
    while (u256_cmp(r, &SECP_P) >= 0) {
        u256 t;
        sub256(&t, r, &SECP_P);
        *r = t;
    }
}

void fe_sqr(u256 *r, const u256 *a) {
    fe_mul(r, a, a);
}

/* Fermat inv a^(p-2) */
int fe_inv(u256 *r, const u256 *a) {
    if (u256_is_zero(a)) return -1;
    /* exp = p-2 */
    u256 e = SECP_P;
    /* p-2: subtract 2 */
    u256 two;
    u256_set_u64(&two, 2);
    sub256(&e, &SECP_P, &two);

    u256 base = *a;
    u256_set_u64(r, 1);
    for (int limb = 0; limb < 4; limb++) {
        uint64_t v = e.d[limb];
        for (int bit = 0; bit < 64; bit++) {
            if (v & 1ULL) {
                fe_mul(r, r, &base);
            }
            fe_sqr(&base, &base);
            v >>= 1;
        }
    }
    return 0;
}

int fe_is_zero(const u256 *a) {
    u256 t = *a;
    fe_normalize(&t);
    return u256_is_zero(&t);
}

void point_set_infinity(point *p) {
    memset(p, 0, sizeof(*p));
    p->infinity = 1;
}

void point_set_generator(point *p) {
    p->x = SECP_GX;
    p->y = SECP_GY;
    p->infinity = 0;
}

int point_is_on_curve(const point *p) {
    if (p->infinity) return 1;
    u256 y2, x2, x3, rhs;
    fe_sqr(&y2, &p->y);
    fe_sqr(&x2, &p->x);
    fe_mul(&x3, &x2, &p->x);
    fe_add(&rhs, &x3, &SECP_B);
    return u256_cmp(&y2, &rhs) == 0;
}

void point_neg(point *r, const point *p) {
    if (p->infinity) {
        point_set_infinity(r);
        return;
    }
    r->x = p->x;
    fe_neg(&r->y, &p->y);
    r->infinity = 0;
}

int point_double(point *r, const point *p) {
    if (p->infinity || fe_is_zero(&p->y)) {
        point_set_infinity(r);
        return 0;
    }
    u256 three, two, num, den, lam, xr, yr, t;
    u256_set_u64(&three, 3);
    u256_set_u64(&two, 2);
    fe_sqr(&num, &p->x);
    fe_mul(&num, &num, &three);
    fe_mul(&den, &p->y, &two);
    if (fe_inv(&den, &den) != 0) return -1;
    fe_mul(&lam, &num, &den);
    fe_sqr(&xr, &lam);
    fe_sub(&xr, &xr, &p->x);
    fe_sub(&xr, &xr, &p->x);
    fe_sub(&t, &p->x, &xr);
    fe_mul(&yr, &lam, &t);
    fe_sub(&yr, &yr, &p->y);
    r->x = xr;
    r->y = yr;
    r->infinity = 0;
    return 0;
}

int point_add(point *r, const point *p, const point *q) {
    if (p->infinity) {
        *r = *q;
        return 0;
    }
    if (q->infinity) {
        *r = *p;
        return 0;
    }
    if (u256_cmp(&p->x, &q->x) == 0) {
        u256 ysum;
        fe_add(&ysum, &p->y, &q->y);
        if (fe_is_zero(&ysum) || u256_cmp(&p->y, &q->y) != 0) {
            /* P + (-P) or y mismatch same x */
            if (u256_cmp(&p->y, &q->y) != 0 || fe_is_zero(&p->y)) {
                point_set_infinity(r);
                return 0;
            }
        }
        return point_double(r, p);
    }
    u256 num, den, lam, xr, yr, t;
    fe_sub(&num, &q->y, &p->y);
    fe_sub(&den, &q->x, &p->x);
    if (fe_inv(&den, &den) != 0) return -1;
    fe_mul(&lam, &num, &den);
    fe_sqr(&xr, &lam);
    fe_sub(&xr, &xr, &p->x);
    fe_sub(&xr, &xr, &q->x);
    fe_sub(&t, &p->x, &xr);
    fe_mul(&yr, &lam, &t);
    fe_sub(&yr, &yr, &p->y);
    r->x = xr;
    r->y = yr;
    r->infinity = 0;
    return 0;
}

int point_mul(point *r, const u256 *scalar, const point *p) {
    point_set_infinity(r);
    point base = *p;
    u256 e = *scalar;
    for (int limb = 0; limb < 4; limb++) {
        uint64_t v = e.d[limb];
        for (int bit = 0; bit < 64; bit++) {
            if (v & 1ULL) {
                point tmp;
                if (point_add(&tmp, r, &base) != 0) return -1;
                *r = tmp;
            }
            point tmp;
            if (point_double(&tmp, &base) != 0) return -1;
            base = tmp;
            v >>= 1;
        }
    }
    return 0;
}

void scalar_reduce(u256 *s) {
    while (u256_cmp(s, &SECP_N) >= 0) {
        u256 t;
        sub256(&t, s, &SECP_N);
        *s = t;
    }
}

int scalar_is_zero(const u256 *s) {
    return u256_is_zero(s);
}

/* sqrt mod p: a^((p+1)/4) */
static int fe_sqrt(u256 *r, const u256 *a) {
    u256 e = SECP_P;
    /* (p+1)/4: p = 2^256 - 2^32 - 977 → (p+1)/4 */
    /* compute a^((p+1)/4) via modpow */
    /* e = (p+1) >> 2 */
    u256 one;
    u256_set_u64(&one, 1);
    add256(&e, &SECP_P, &one);
    /* >> 2 */
    uint64_t carry = 0;
    for (int i = 3; i >= 0; i--) {
        uint64_t new_carry = e.d[i] << 62;
        e.d[i] = (e.d[i] >> 2) | carry;
        carry = new_carry;
    }
    u256 base = *a;
    u256_set_u64(r, 1);
    for (int limb = 0; limb < 4; limb++) {
        uint64_t v = e.d[limb];
        for (int bit = 0; bit < 64; bit++) {
            if (v & 1ULL) fe_mul(r, r, &base);
            fe_sqr(&base, &base);
            v >>= 1;
        }
    }
    u256 check;
    fe_sqr(&check, r);
    return u256_cmp(&check, a) == 0 ? 0 : -1;
}

int sec1_parse(point *p, const uint8_t *buf, size_t len) {
    if (len == 1 && buf[0] == 0x00) {
        point_set_infinity(p);
        return 0;
    }
    if (len == 65 && buf[0] == 0x04) {
        u256_from_be(&p->x, buf + 1);
        u256_from_be(&p->y, buf + 33);
        p->infinity = 0;
        if (u256_cmp(&p->x, &SECP_P) >= 0 || u256_cmp(&p->y, &SECP_P) >= 0)
            return 1;
        if (!point_is_on_curve(p)) return 1;
        return 0;
    }
    if (len == 33 && (buf[0] == 0x02 || buf[0] == 0x03)) {
        u256_from_be(&p->x, buf + 1);
        if (u256_cmp(&p->x, &SECP_P) >= 0) return 1;
        u256 x2, x3, rhs;
        fe_sqr(&x2, &p->x);
        fe_mul(&x3, &x2, &p->x);
        fe_add(&rhs, &x3, &SECP_B);
        if (fe_sqrt(&p->y, &rhs) != 0) return 1;
        int odd = (int)(p->y.d[0] & 1ULL);
        int want = (buf[0] == 0x03);
        if (odd != want) fe_neg(&p->y, &p->y);
        p->infinity = 0;
        if (!point_is_on_curve(p)) return 1;
        return 0;
    }
    return 1;
}

int sec1_serialize_uncompressed(const point *p, uint8_t out[65]) {
    if (p->infinity) return -1;
    out[0] = 0x04;
    u256_to_be(&p->x, out + 1);
    u256_to_be(&p->y, out + 33);
    return 0;
}
