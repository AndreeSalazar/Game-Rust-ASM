//! Fixed timestep game loop
//! 
//! Rust controls the loop, ensures determinism.
//! ASM only used for timing precision.

use crate::EngineConfig;
use super::timing::Timer;

/// Game loop state
pub struct GameLoop {
    timer: Timer,
    last_time: u64,
    accumulator: f64,
    fixed_dt: f64,
    max_frame_skip: u32,
    frame: u64,
    tick: u64,
}

impl GameLoop {
    pub fn new(config: &EngineConfig) -> Self {
        let mut timer = Timer::new();
        timer.start();
        
        Self {
            timer,
            last_time: 0,
            accumulator: 0.0,
            fixed_dt: config.fixed_timestep,
            max_frame_skip: config.max_frame_skip,
            frame: 0,
            tick: 0,
        }
    }
    
    /// Process one frame, returns number of fixed updates to run
    pub fn tick(&mut self) -> FrameTick {
        let current_time = self.timer.elapsed_ns();
        let delta_ns = current_time.saturating_sub(self.last_time);
        self.last_time = current_time;
        
        let delta_s = delta_ns as f64 / 1_000_000_000.0;
        self.accumulator += delta_s;
        
        // Count fixed updates needed
        let mut updates = 0u32;
        while self.accumulator >= self.fixed_dt && updates < self.max_frame_skip {
            self.accumulator -= self.fixed_dt;
            self.tick += 1;
            updates += 1;
        }
        
        self.frame += 1;
        
        FrameTick {
            frame: self.frame,
            tick: self.tick,
            fixed_updates: updates,
            delta: delta_s,
            fixed_dt: self.fixed_dt,
            interpolation: self.accumulator / self.fixed_dt,
        }
    }
    
    pub fn frame(&self) -> u64 {
        self.frame
    }
    
    pub fn tick_count(&self) -> u64 {
        self.tick
    }
}

/// Result of one game loop iteration
#[derive(Clone, Copy, Debug)]
pub struct FrameTick {
    /// Current render frame number
    pub frame: u64,
    /// Current simulation tick number
    pub tick: u64,
    /// Number of fixed updates to run this frame
    pub fixed_updates: u32,
    /// Actual delta time since last frame (variable)
    pub delta: f64,
    /// Fixed timestep value
    pub fixed_dt: f64,
    /// Interpolation factor for rendering (0.0 - 1.0)
    pub interpolation: f64,
}
