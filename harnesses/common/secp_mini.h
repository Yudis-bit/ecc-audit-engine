#ifndef SECP_MINI_H
#define SECP_MINI_H

#include <stddef.h>
#include <stdint.h>
#include <string.h>

/* 256-bit LE limbs */
typedef struct { uint64_t d[4]; } u256;

int u256_from_be(u256 *o, const uint8_t *be32);
void u256_to_be(const u256 *a, uint8_t *be32);
int u256_is_zero(const u256 *a);
int u256_cmp(const u256 *a, const u256 *b);
void u256_set_u64(u256 *o, uint64_t v);

/* Field mod p */
void fe_normalize(u256 *a);
void fe_add(u256 *r, const u256 *a, const u256 *b);
void fe_sub(u256 *r, const u256 *a, const u256 *b);
void fe_mul(u256 *r, const u256 *a, const u256 *b);
void fe_sqr(u256 *r, const u256 *a);
void fe_neg(u256 *r, const u256 *a);
int fe_inv(u256 *r, const u256 *a);
int fe_is_zero(const u256 *a);

/* Affine point: infinity flag */
typedef struct {
    u256 x, y;
    int infinity;
} point;

void point_set_infinity(point *p);
void point_set_generator(point *p);
int point_is_on_curve(const point *p);
void point_neg(point *r, const point *p);
int point_double(point *r, const point *p);
int point_add(point *r, const point *p, const point *q);
int point_mul(point *r, const u256 *scalar, const point *p);

int sec1_parse(point *p, const uint8_t *buf, size_t len);
int sec1_serialize_uncompressed(const point *p, uint8_t out[65]);

/* Scalar mod n helpers */
extern const u256 SECP_N;
extern const u256 SECP_P;
void scalar_reduce(u256 *s);
int scalar_is_zero(const u256 *s);

#endif
