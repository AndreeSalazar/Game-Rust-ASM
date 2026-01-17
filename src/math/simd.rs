//! SIMD math operations
//! 
//! Rust provides the API, ASM executes batch operations.
//! Falls back to scalar if ASM not available.

use super::Vec2;

#[cfg(not(no_asm))]
extern "C" {
    fn simd_vec2_add_batch(a: *const Vec2, b: *const Vec2, out: *mut Vec2, count: usize);
    fn simd_vec2_mul_scalar_batch(a: *const Vec2, scalar: f32, out: *mut Vec2, count: usize);
    fn simd_dot_product_batch(a: *const Vec2, b: *const Vec2, out: *mut f32, count: usize);
    fn simd_normalize_batch(a: *const Vec2, out: *mut Vec2, count: usize);
}

/// Batch add two arrays of Vec2
pub fn vec2_add_batch(a: &[Vec2], b: &[Vec2], out: &mut [Vec2]) {
    let count = a.len().min(b.len()).min(out.len());
    
    #[cfg(not(no_asm))]
    unsafe {
        simd_vec2_add_batch(a.as_ptr(), b.as_ptr(), out.as_mut_ptr(), count);
    }
    
    #[cfg(no_asm)]
    {
        for i in 0..count {
            out[i] = a[i] + b[i];
        }
    }
}

/// Batch multiply Vec2 array by scalar
pub fn vec2_mul_scalar_batch(a: &[Vec2], scalar: f32, out: &mut [Vec2]) {
    let count = a.len().min(out.len());
    
    #[cfg(not(no_asm))]
    unsafe {
        simd_vec2_mul_scalar_batch(a.as_ptr(), scalar, out.as_mut_ptr(), count);
    }
    
    #[cfg(no_asm)]
    {
        for i in 0..count {
            out[i] = a[i] * scalar;
        }
    }
}

/// Batch dot product
pub fn dot_product_batch(a: &[Vec2], b: &[Vec2], out: &mut [f32]) {
    let count = a.len().min(b.len()).min(out.len());
    
    #[cfg(not(no_asm))]
    unsafe {
        simd_dot_product_batch(a.as_ptr(), b.as_ptr(), out.as_mut_ptr(), count);
    }
    
    #[cfg(no_asm)]
    {
        for i in 0..count {
            out[i] = a[i].dot(b[i]);
        }
    }
}

/// Batch normalize
pub fn normalize_batch(a: &[Vec2], out: &mut [Vec2]) {
    let count = a.len().min(out.len());
    
    #[cfg(not(no_asm))]
    unsafe {
        simd_normalize_batch(a.as_ptr(), out.as_mut_ptr(), count);
    }
    
    #[cfg(no_asm)]
    {
        for i in 0..count {
            out[i] = a[i].normalize();
        }
    }
}

/// SIMD-friendly array of Vec2 (aligned)
#[repr(C, align(32))]
pub struct Vec2Array {
    pub data: Vec<Vec2>,
}

impl Vec2Array {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }
    
    pub fn from_vec(data: Vec<Vec2>) -> Self {
        Self { data }
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn push(&mut self, v: Vec2) {
        self.data.push(v);
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
    }
}
