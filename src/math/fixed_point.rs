//! Fixed-point math for deterministic calculations
//! 
//! ASM can accelerate fixed-point operations with integer SIMD.

use std::ops::{Add, Sub, Mul, Div, Neg};

/// 16.16 Fixed-point number for deterministic math
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct FixedPoint(i32);

impl FixedPoint {
    pub const ZERO: FixedPoint = FixedPoint(0);
    pub const ONE: FixedPoint = FixedPoint(1 << 16);
    pub const HALF: FixedPoint = FixedPoint(1 << 15);
    pub const NEG_ONE: FixedPoint = FixedPoint(-(1 << 16));
    
    const FRAC_BITS: i32 = 16;
    const SCALE: i32 = 1 << 16;
    
    #[inline]
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }
    
    #[inline]
    pub const fn raw(self) -> i32 {
        self.0
    }
    
    #[inline]
    pub const fn from_int(n: i32) -> Self {
        Self(n << Self::FRAC_BITS)
    }
    
    #[inline]
    pub fn from_f32(f: f32) -> Self {
        Self((f * Self::SCALE as f32) as i32)
    }
    
    #[inline]
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / Self::SCALE as f32
    }
    
    #[inline]
    pub const fn to_int(self) -> i32 {
        self.0 >> Self::FRAC_BITS
    }
    
    #[inline]
    pub const fn frac(self) -> Self {
        Self(self.0 & (Self::SCALE - 1))
    }
    
    #[inline]
    pub const fn floor(self) -> Self {
        Self(self.0 & !(Self::SCALE - 1))
    }
    
    #[inline]
    pub fn ceil(self) -> Self {
        Self(((self.0 - 1) | (Self::SCALE - 1)) + 1)
    }
    
    #[inline]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }
    
    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }
    
    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
    
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }
    
    /// Multiply with full precision (uses 64-bit intermediate)
    #[inline]
    pub fn mul_full(self, other: Self) -> Self {
        let result = (self.0 as i64 * other.0 as i64) >> Self::FRAC_BITS;
        Self(result as i32)
    }
    
    /// Divide with full precision
    #[inline]
    pub fn div_full(self, other: Self) -> Self {
        if other.0 == 0 {
            return Self::ZERO;
        }
        let result = ((self.0 as i64) << Self::FRAC_BITS) / other.0 as i64;
        Self(result as i32)
    }
    
    /// Linear interpolation
    #[inline]
    pub fn lerp(self, other: Self, t: Self) -> Self {
        self + (other - self).mul_full(t)
    }
}

impl Add for FixedPoint {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }
}

impl Sub for FixedPoint {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }
}

impl Mul for FixedPoint {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
        self.mul_full(other)
    }
}

impl Div for FixedPoint {
    type Output = Self;
    #[inline]
    fn div(self, other: Self) -> Self {
        self.div_full(other)
    }
}

impl Neg for FixedPoint {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl From<i32> for FixedPoint {
    fn from(n: i32) -> Self {
        Self::from_int(n)
    }
}

impl From<f32> for FixedPoint {
    fn from(f: f32) -> Self {
        Self::from_f32(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_ops() {
        let a = FixedPoint::from_f32(2.5);
        let b = FixedPoint::from_f32(1.5);
        
        assert!((a + b).to_f32() - 4.0 < 0.001);
        assert!((a - b).to_f32() - 1.0 < 0.001);
        assert!((a * b).to_f32() - 3.75 < 0.001);
    }
}
