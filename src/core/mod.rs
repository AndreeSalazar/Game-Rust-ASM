//! Core engine systems
//! - Timing (ASM RDTSC)
//! - Game loop
//! - Profiling

pub mod timing;
pub mod game_loop;
pub mod profiler;

pub use timing::*;
pub use game_loop::*;
pub use profiler::*;
