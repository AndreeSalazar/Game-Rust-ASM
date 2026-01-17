//! Math module
//! 
//! Rust handles high-level math API.
//! ASM handles SIMD batch operations and fixed-point math.

pub mod vec2;
pub mod fixed_point;
pub mod simd;

pub use vec2::Vec2;
pub use fixed_point::FixedPoint;
