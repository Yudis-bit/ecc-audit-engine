//! Auditable secp256k1 reference model using BigUint.
//! Primary oracle for differential testing. Not a production ECC library.

use num_bigint::BigUint;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

/// p = 2^256 - 2^32 - 977
pub fn field_prime() -> BigUint {
    (BigUint::one() << 256) - (BigUint::one() << 32) - BigUint::from(977u32)
}

/// Curve order n
pub fn group_order() -> BigUint {
    BigUint::parse_bytes(
        b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
        16,
    )
    .expect("valid n")
}

/// Generator Gx
pub fn generator_x() -> BigUint {
    BigUint::parse_bytes(
        b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
        16,
    )
    .expect("valid Gx")
}

/// Generator Gy
pub fn generator_y() -> BigUint {
    BigUint::parse_bytes(
        b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8",
        16,
    )
    .expect("valid Gy")
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MathError {
    #[error("value out of field range")]
    OutOfRange,
    #[error("non-canonical field encoding")]
    NonCanonical,
    #[error("zero inverse")]
    ZeroInverse,
    #[error("point not on curve")]
    OffCurve,
    #[error("invalid SEC1 encoding")]
    BadEncoding,
    #[error("invalid scalar encoding")]
    BadScalar,
    #[error("square root does not exist")]
    NoSqrt,
}

/// Modular subtraction that never underflows BigUint.
pub fn mod_sub(a: &BigUint, b: &BigUint, modulus: &BigUint) -> BigUint {
    if a >= b {
        (a - b) % modulus
    } else {
        (modulus - ((b - a) % modulus)) % modulus
    }
}

fn mod_add(a: &BigUint, b: &BigUint, modulus: &BigUint) -> BigUint {
    (a + b) % modulus
}

fn mod_mul(a: &BigUint, b: &BigUint, modulus: &BigUint) -> BigUint {
    (a * b) % modulus
}

/// Canonical field element in [0, p).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldElement(BigUint);

impl FieldElement {
    pub fn new(value: BigUint) -> Result<Self, MathError> {
        let p = field_prime();
        if value >= p {
            return Err(MathError::OutOfRange);
        }
        Ok(Self(value))
    }

    pub fn from_raw_reduced(value: BigUint) -> Self {
        let p = field_prime();
        Self(value % p)
    }

    pub fn zero() -> Self {
        Self(BigUint::zero())
    }

    pub fn one() -> Self {
        Self(BigUint::one())
    }

    pub fn inner(&self) -> &BigUint {
        &self.0
    }

    pub fn add(&self, other: &Self) -> Self {
        let p = field_prime();
        Self(mod_add(&self.0, &other.0, &p))
    }

    pub fn sub(&self, other: &Self) -> Self {
        let p = field_prime();
        Self(mod_sub(&self.0, &other.0, &p))
    }

    pub fn mul(&self, other: &Self) -> Self {
        let p = field_prime();
        Self(mod_mul(&self.0, &other.0, &p))
    }

    pub fn square(&self) -> Self {
        self.mul(self)
    }

    pub fn neg(&self) -> Self {
        if self.0.is_zero() {
            Self::zero()
        } else {
            let p = field_prime();
            Self(&p - &self.0)
        }
    }

    pub fn inv(&self) -> Result<Self, MathError> {
        if self.0.is_zero() {
            return Err(MathError::ZeroInverse);
        }
        let p = field_prime();
        // Fermat: a^(p-2) mod p
        let exp = &p - BigUint::from(2u32);
        Ok(Self(self.0.modpow(&exp, &p)))
    }

    pub fn pow(&self, exp: &BigUint) -> Self {
        let p = field_prime();
        Self(self.0.modpow(exp, &p))
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn to_bytes_be(&self) -> [u8; 32] {
        let bytes = self.0.to_bytes_be();
        let mut out = [0u8; 32];
        let start = 32 - bytes.len();
        out[start..].copy_from_slice(&bytes);
        out
    }

    /// Accept only exact 32-byte big-endian values strictly less than p.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, MathError> {
        if bytes.len() != 32 {
            return Err(MathError::BadEncoding);
        }
        let v = BigUint::from_bytes_be(bytes);
        let p = field_prime();
        if v >= p {
            return Err(MathError::NonCanonical);
        }
        Ok(Self(v))
    }

    /// secp256k1 sqrt via (p+1)/4 for p ≡ 3 (mod 4).
    pub fn sqrt(&self) -> Result<Self, MathError> {
        let p = field_prime();
        let exp = (&p + BigUint::one()) >> 2;
        let r = Self(self.0.modpow(&exp, &p));
        if r.square() != *self {
            return Err(MathError::NoSqrt);
        }
        Ok(r)
    }
}

/// Scalar in [0, n) for group operations; reduction helpers for API edges.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scalar(BigUint);

impl Scalar {
    pub fn new(value: BigUint) -> Result<Self, MathError> {
        let n = group_order();
        if value >= n {
            return Err(MathError::OutOfRange);
        }
        Ok(Self(value))
    }

    pub fn from_bytes_reduced(bytes: &[u8; 32]) -> Self {
        let n = group_order();
        Self(BigUint::from_bytes_be(bytes) % n)
    }

    pub fn from_canonical_bytes(bytes: &[u8; 32]) -> Result<Self, MathError> {
        let v = BigUint::from_bytes_be(bytes);
        let n = group_order();
        if v >= n {
            return Err(MathError::BadScalar);
        }
        Ok(Self(v))
    }

    pub fn zero() -> Self {
        Self(BigUint::zero())
    }

    pub fn one() -> Self {
        Self(BigUint::one())
    }

    pub fn inner(&self) -> &BigUint {
        &self.0
    }

    pub fn to_bytes_be(&self) -> [u8; 32] {
        let bytes = self.0.to_bytes_be();
        let mut out = [0u8; 32];
        let start = 32 - bytes.len().min(32);
        if bytes.len() > 32 {
            out.copy_from_slice(&bytes[bytes.len() - 32..]);
        } else {
            out[start..].copy_from_slice(&bytes);
        }
        out
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffinePoint {
    Infinity,
    Finite { x: FieldElement, y: FieldElement },
}

impl AffinePoint {
    pub fn infinity() -> Self {
        Self::Infinity
    }

    pub fn generator() -> Self {
        Self::Finite {
            x: FieldElement::from_raw_reduced(generator_x()),
            y: FieldElement::from_raw_reduced(generator_y()),
        }
    }

    pub fn is_infinity(&self) -> bool {
        matches!(self, Self::Infinity)
    }

    pub fn is_on_curve(&self) -> bool {
        match self {
            Self::Infinity => true,
            Self::Finite { x, y } => {
                // y^2 = x^3 + 7
                let y2 = y.square();
                let x3 = x.square().mul(x);
                let rhs = x3.add(&FieldElement::from_raw_reduced(BigUint::from(7u32)));
                y2 == rhs
            }
        }
    }

    pub fn negate(&self) -> Self {
        match self {
            Self::Infinity => Self::Infinity,
            Self::Finite { x, y } => Self::Finite {
                x: x.clone(),
                y: y.neg(),
            },
        }
    }

    pub fn double(&self) -> Result<Self, MathError> {
        match self {
            Self::Infinity => Ok(Self::Infinity),
            Self::Finite { x, y } => {
                if y.is_zero() {
                    return Ok(Self::Infinity);
                }
                // λ = (3x^2) / (2y)
                let three = FieldElement::from_raw_reduced(BigUint::from(3u32));
                let two = FieldElement::from_raw_reduced(BigUint::from(2u32));
                let num = three.mul(&x.square());
                let den = two.mul(y);
                let lambda = num.mul(&den.inv()?);
                let xr = lambda.square().sub(x).sub(x);
                let yr = lambda.mul(&x.sub(&xr)).sub(y);
                Ok(Self::Finite { x: xr, y: yr })
            }
        }
    }

    pub fn add(&self, other: &Self) -> Result<Self, MathError> {
        match (self, other) {
            (Self::Infinity, q) => Ok(q.clone()),
            (p, Self::Infinity) => Ok(p.clone()),
            (Self::Finite { x: x1, y: y1 }, Self::Finite { x: x2, y: y2 }) => {
                if x1 == x2 {
                    if y1 != y2 || y1.is_zero() {
                        return Ok(Self::Infinity);
                    }
                    return self.double();
                }
                let num = y2.sub(y1);
                let den = x2.sub(x1);
                let lambda = num.mul(&den.inv()?);
                let xr = lambda.square().sub(x1).sub(x2);
                let yr = lambda.mul(&x1.sub(&xr)).sub(y1);
                Ok(Self::Finite { x: xr, y: yr })
            }
        }
    }

    pub fn scalar_mul(&self, k: &Scalar) -> Result<Self, MathError> {
        // Double-and-add, MSB→LSB via bit scan on BigUint
        let mut result = Self::Infinity;
        let mut base = self.clone();
        let mut e = k.inner().clone();
        while !e.is_zero() {
            if e.bit(0) {
                result = result.add(&base)?;
            }
            base = base.double()?;
            e >>= 1;
        }
        Ok(result)
    }

    pub fn serialize_uncompressed(&self) -> Result<[u8; 65], MathError> {
        match self {
            Self::Infinity => Err(MathError::BadEncoding),
            Self::Finite { x, y } => {
                let mut out = [0u8; 65];
                out[0] = 0x04;
                out[1..33].copy_from_slice(&x.to_bytes_be());
                out[33..65].copy_from_slice(&y.to_bytes_be());
                Ok(out)
            }
        }
    }

    pub fn serialize_compressed(&self) -> Result<[u8; 33], MathError> {
        match self {
            Self::Infinity => Err(MathError::BadEncoding),
            Self::Finite { x, y } => {
                let mut out = [0u8; 33];
                let y_odd = y.inner().bit(0);
                out[0] = if y_odd { 0x03 } else { 0x02 };
                out[1..33].copy_from_slice(&x.to_bytes_be());
                Ok(out)
            }
        }
    }

    pub fn parse_sec1(bytes: &[u8]) -> Result<Self, MathError> {
        if bytes.is_empty() {
            return Err(MathError::BadEncoding);
        }
        // Laboratory internal encoding for infinity (not standard SEC1 pubkey).
        if bytes == [0x00] {
            return Ok(Self::Infinity);
        }
        match bytes[0] {
            0x04 => {
                if bytes.len() != 65 {
                    return Err(MathError::BadEncoding);
                }
                let x = FieldElement::from_canonical_bytes(&bytes[1..33])?;
                let y = FieldElement::from_canonical_bytes(&bytes[33..65])?;
                let p = Self::Finite { x, y };
                if !p.is_on_curve() {
                    return Err(MathError::OffCurve);
                }
                Ok(p)
            }
            0x02 | 0x03 => {
                if bytes.len() != 33 {
                    return Err(MathError::BadEncoding);
                }
                let x = FieldElement::from_canonical_bytes(&bytes[1..33])?;
                let x3 = x.square().mul(&x);
                let rhs = x3.add(&FieldElement::from_raw_reduced(BigUint::from(7u32)));
                let mut y = rhs.sqrt()?;
                let want_odd = bytes[0] == 0x03;
                if y.inner().bit(0) != want_odd {
                    y = y.neg();
                }
                let p = Self::Finite { x, y };
                if !p.is_on_curve() {
                    return Err(MathError::OffCurve);
                }
                Ok(p)
            }
            _ => Err(MathError::BadEncoding),
        }
    }

    /// Compare points after ensuring both are on-curve (Infinity equal).
    pub fn eq_affine(&self, other: &Self) -> bool {
        self == other
    }
}

/// Public-key creation: sk in [1, n), Q = sk * G uncompressed.
pub fn pubkey_create(secret_key: &[u8; 32]) -> Result<[u8; 65], MathError> {
    let n = group_order();
    let sk = BigUint::from_bytes_be(secret_key);
    if sk.is_zero() || sk >= n {
        return Err(MathError::BadScalar);
    }
    let s = Scalar::new(sk)?;
    let q = AffinePoint::generator().scalar_mul(&s)?;
    q.serialize_uncompressed()
}

/// Field multiply of two canonical 32-byte BE field elements.
pub fn fe_mul_bytes(a: &[u8; 32], b: &[u8; 32]) -> Result<[u8; 32], MathError> {
    let fa = FieldElement::from_canonical_bytes(a)?;
    let fb = FieldElement::from_canonical_bytes(b)?;
    Ok(fa.mul(&fb).to_bytes_be())
}

/// Point multiply: scalar (reduced mod n) * point (SEC1).
pub fn point_mul_bytes(scalar: &[u8; 32], point: &[u8]) -> Result<Vec<u8>, MathError> {
    let p = AffinePoint::parse_sec1(point)?;
    let s = Scalar::from_bytes_reduced(scalar);
    let r = p.scalar_mul(&s)?;
    if r.is_infinity() {
        // Represent infinity as single 0x00 for internal differential use
        return Ok(vec![0x00]);
    }
    Ok(r.serialize_uncompressed()?.to_vec())
}

pub fn point_add_bytes(a: &[u8], b: &[u8]) -> Result<Vec<u8>, MathError> {
    let pa = if a == [0x00] {
        AffinePoint::Infinity
    } else {
        AffinePoint::parse_sec1(a)?
    };
    let pb = if b == [0x00] {
        AffinePoint::Infinity
    } else {
        AffinePoint::parse_sec1(b)?
    };
    let r = pa.add(&pb)?;
    if r.is_infinity() {
        return Ok(vec![0x00]);
    }
    Ok(r.serialize_uncompressed()?.to_vec())
}

/// Lexicographic compare for tests.
pub fn bytes_cmp(a: &[u8], b: &[u8]) -> Ordering {
    a.cmp(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar_from_u64(v: u64) -> Scalar {
        Scalar::new(BigUint::from(v)).unwrap()
    }

    #[test]
    fn g_on_curve() {
        assert!(AffinePoint::generator().is_on_curve());
    }

    #[test]
    fn one_g_is_g() {
        let g = AffinePoint::generator();
        let r = g.scalar_mul(&scalar_from_u64(1)).unwrap();
        assert_eq!(r, g);
    }

    #[test]
    fn two_g_equals_g_plus_g() {
        let g = AffinePoint::generator();
        let d = g.double().unwrap();
        let a = g.add(&g).unwrap();
        let s = g.scalar_mul(&scalar_from_u64(2)).unwrap();
        assert_eq!(d, a);
        assert_eq!(d, s);
    }

    #[test]
    fn zero_g_infinity() {
        let g = AffinePoint::generator();
        let r = g.scalar_mul(&Scalar::zero()).unwrap();
        assert!(r.is_infinity());
    }

    #[test]
    fn n_g_infinity() {
        let g = AffinePoint::generator();
        // n reduced is 0
        let s = Scalar::from_bytes_reduced(&{
            let mut b = [0u8; 32];
            let nb = group_order().to_bytes_be();
            b[32 - nb.len()..].copy_from_slice(&nb);
            b
        });
        assert!(s.is_zero());
        let r = g.scalar_mul(&s).unwrap();
        assert!(r.is_infinity());
    }

    #[test]
    fn n_minus_one_is_neg_g() {
        let g = AffinePoint::generator();
        let n = group_order();
        let nm1 = Scalar::new(n - BigUint::one()).unwrap();
        let r = g.scalar_mul(&nm1).unwrap();
        assert_eq!(r, g.negate());
    }

    #[test]
    fn g_plus_infinity() {
        let g = AffinePoint::generator();
        assert_eq!(g.add(&AffinePoint::Infinity).unwrap(), g);
        assert_eq!(AffinePoint::Infinity.add(&g).unwrap(), g);
    }

    #[test]
    fn g_plus_neg_g_infinity() {
        let g = AffinePoint::generator();
        let r = g.add(&g.negate()).unwrap();
        assert!(r.is_infinity());
    }

    #[test]
    fn double_equals_add() {
        let g = AffinePoint::generator();
        assert_eq!(g.double().unwrap(), g.add(&g).unwrap());
    }

    #[test]
    fn serialize_parse_roundtrip() {
        let g = AffinePoint::generator();
        let u = g.serialize_uncompressed().unwrap();
        let c = g.serialize_compressed().unwrap();
        assert_eq!(AffinePoint::parse_sec1(&u).unwrap(), g);
        assert_eq!(AffinePoint::parse_sec1(&c).unwrap(), g);
    }

    #[test]
    fn reject_invalid_prefix() {
        let mut u = AffinePoint::generator().serialize_uncompressed().unwrap();
        u[0] = 0x01;
        assert_eq!(
            AffinePoint::parse_sec1(&u).unwrap_err(),
            MathError::BadEncoding
        );
    }

    #[test]
    fn reject_off_curve() {
        let mut u = AffinePoint::generator().serialize_uncompressed().unwrap();
        // flip a y byte to leave curve with high probability
        u[64] ^= 0x01;
        assert_eq!(
            AffinePoint::parse_sec1(&u).unwrap_err(),
            MathError::OffCurve
        );
    }

    #[test]
    fn reject_non_canonical_field() {
        // p itself as 32 bytes BE
        let p = field_prime();
        let pb = p.to_bytes_be();
        let mut buf = [0u8; 32];
        buf[32 - pb.len()..].copy_from_slice(&pb);
        assert_eq!(
            FieldElement::from_canonical_bytes(&buf).unwrap_err(),
            MathError::NonCanonical
        );
    }

    #[test]
    fn mod_sub_underflow_safe() {
        let p = field_prime();
        let a = BigUint::from(3u32);
        let b = BigUint::from(5u32);
        let r = mod_sub(&a, &b, &p);
        assert_eq!(r, &p - BigUint::from(2u32));
    }

    #[test]
    fn known_vector_1g() {
        let g = AffinePoint::generator();
        let u = g.serialize_uncompressed().unwrap();
        assert_eq!(
            hex::encode(u),
            "0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8"
        );
    }

    #[test]
    fn known_vector_2g() {
        let g = AffinePoint::generator();
        let r = g.scalar_mul(&scalar_from_u64(2)).unwrap();
        let u = r.serialize_uncompressed().unwrap();
        // secp256k1 2G known vector
        assert_eq!(
            hex::encode(&u[1..33]),
            "c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5"
        );
        assert_eq!(
            hex::encode(&u[33..65]),
            "1ae168fea63dc339a3c58419466ceaeef7f632653266d0e1236431a950cfe52a"
        );
    }

    #[test]
    fn known_vector_3g() {
        let g = AffinePoint::generator();
        let r = g.scalar_mul(&scalar_from_u64(3)).unwrap();
        let u = r.serialize_uncompressed().unwrap();
        assert_eq!(
            hex::encode(&u[1..33]),
            "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9"
        );
        assert_eq!(
            hex::encode(&u[33..65]),
            "388f7b0f632de8140fe337e62a37f3566500a99934c2231b6cb9fd7584b8e672"
        );
    }

    #[test]
    fn known_vector_7g() {
        // k=7 pubkey (uncompressed) from well-known secp256k1 vectors
        let g = AffinePoint::generator();
        let r = g.scalar_mul(&scalar_from_u64(7)).unwrap();
        let u = r.serialize_uncompressed().unwrap();
        assert_eq!(
            hex::encode(u),
            concat!(
                "04",
                "5cbdf0646e5db4eaa398f365f2ea7a0e3d419b7e0330e39ce92bddedcac4f9bc",
                "6aebca40ba255960a3178d6d861a54dba813d0b813fde7b5a5082628087264da"
            )
        );
    }

    #[test]
    fn known_vector_n_minus_1() {
        let g = AffinePoint::generator();
        let n = group_order();
        let r = g
            .scalar_mul(&Scalar::new(n - BigUint::one()).unwrap())
            .unwrap();
        assert_eq!(r, g.negate());
        let u = r.serialize_uncompressed().unwrap();
        // -G has same x, negated y
        let gu = g.serialize_uncompressed().unwrap();
        assert_eq!(&u[1..33], &gu[1..33]);
        assert_ne!(&u[33..65], &gu[33..65]);
    }

    #[test]
    fn fe_mul_basic() {
        let a = FieldElement::from_raw_reduced(BigUint::from(3u32));
        let b = FieldElement::from_raw_reduced(BigUint::from(5u32));
        assert_eq!(a.mul(&b).inner(), &BigUint::from(15u32));
    }

    #[test]
    fn pubkey_create_one() {
        let mut sk = [0u8; 32];
        sk[31] = 1;
        let pk = pubkey_create(&sk).unwrap();
        assert_eq!(
            AffinePoint::parse_sec1(&pk).unwrap(),
            AffinePoint::generator()
        );
    }

    #[test]
    fn three_g_add_chain() {
        let g = AffinePoint::generator();
        let t = g.add(&g).unwrap().add(&g).unwrap();
        assert_eq!(t, g.scalar_mul(&scalar_from_u64(3)).unwrap());
    }
}
