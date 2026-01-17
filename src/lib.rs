//! # Game Engine X
//! 
//! Rust-ASM Deterministic 2D Game Engine
//! 
//! **Author:** Eddi Andreé Salazar Matos  
//! **License:** MIT
//! 
//! ## Architecture
//! - **Rust:** Game logic, ECS, scheduler, APIs
//! - **ASM:** Hot paths only (physics, collision, raycast)
//! 
//! ## Modules
//! - `core` - Timing, game loop, profiler
//! - `ecs` - Entity Component System
//! - `math` - Vec2, FixedPoint, SIMD
//! - `physics` - Collision, integration
//! - `render` - Software renderer, raycaster
//! - `input` - Input handling
//! - `audio` - Audio system

pub mod core;
pub mod ecs;
pub mod math;
pub mod physics;
pub mod render;
pub mod input;
pub mod audio;

pub use core::*;
pub use ecs::World;
pub use math::{Vec2, FixedPoint};
pub use physics::PhysicsWorld;
pub use render::Renderer;
pub use input::InputState;

/// Engine configuration
#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fixed_timestep: f64,
    pub max_frame_skip: u32,
    pub vsync: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            title: "Game Engine X".to_string(),
            width: 800,
            height: 600,
            fixed_timestep: 1.0 / 60.0,
            max_frame_skip: 5,
            vsync: true,
        }
    }
}

/// Main engine struct - Rust controls everything
pub struct Engine {
    pub config: EngineConfig,
    pub world: ecs::World,
    pub physics: physics::PhysicsWorld,
    pub input: input::InputState,
    accumulator: f64,
    frame_count: u64,
    pub running: bool,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            world: ecs::World::new(),
            physics: physics::PhysicsWorld::new(),
            input: input::InputState::new(),
            accumulator: 0.0,
            frame_count: 0,
            running: true,
        }
    }
    
    /// Fixed timestep update - deterministic
    pub fn update(&mut self, dt: f64) {
        self.accumulator += dt;
        
        let mut updates = 0;
        while self.accumulator >= self.config.fixed_timestep 
            && updates < self.config.max_frame_skip 
        {
            self.fixed_update(self.config.fixed_timestep);
            self.accumulator -= self.config.fixed_timestep;
            updates += 1;
        }
        
        self.frame_count += 1;
    }
    
    /// Fixed update - called at fixed intervals
    fn fixed_update(&mut self, _dt: f64) {
        // Physics step (calls ASM hot paths internally)
        self.physics.step();
    }
    
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}
