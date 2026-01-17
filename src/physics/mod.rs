//! Physics module
//! 
//! Rust: Physics world management, broad phase, constraint solving
//! ASM: Narrow phase collision detection, integration step

pub mod collision;
pub mod integration;
pub mod broad_phase;

pub use collision::*;

use crate::math::Vec2;

/// Physics world configuration
#[derive(Clone, Debug)]
pub struct PhysicsConfig {
    pub gravity: Vec2,
    pub iterations: u32,
    pub substeps: u32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec2::new(0.0, 980.0),
            iterations: 8,
            substeps: 1,
        }
    }
}

/// Physics body
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub mass: f32,
    pub inv_mass: f32,
    pub restitution: f32,
    pub friction: f32,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            mass: 1.0,
            inv_mass: 1.0,
            restitution: 0.5,
            friction: 0.3,
        }
    }
}

impl Body {
    pub fn new(position: Vec2, mass: f32) -> Self {
        Self {
            position,
            mass,
            inv_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            ..Default::default()
        }
    }
    
    pub fn static_body(position: Vec2) -> Self {
        Self {
            position,
            mass: 0.0,
            inv_mass: 0.0,
            ..Default::default()
        }
    }
    
    pub fn is_static(&self) -> bool {
        self.inv_mass == 0.0
    }
}

/// Physics world - manages all bodies and collisions
pub struct PhysicsWorld {
    pub config: PhysicsConfig,
    pub bodies: Vec<Body>,
    contacts: Vec<Contact>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            config: PhysicsConfig::default(),
            bodies: Vec::new(),
            contacts: Vec::new(),
        }
    }
    
    pub fn with_config(config: PhysicsConfig) -> Self {
        Self {
            config,
            bodies: Vec::new(),
            contacts: Vec::new(),
        }
    }
    
    pub fn add_body(&mut self, body: Body) -> usize {
        let id = self.bodies.len();
        self.bodies.push(body);
        id
    }
    
    pub fn step(&mut self) {
        let dt = 1.0 / 60.0 / self.config.substeps as f32;
        
        for _ in 0..self.config.substeps {
            // Apply gravity
            for body in &mut self.bodies {
                if !body.is_static() {
                    body.acceleration = self.config.gravity;
                }
            }
            
            // Integration (ASM accelerated)
            integration::integrate_bodies(&mut self.bodies, dt);
            
            // Collision detection (ASM accelerated narrow phase)
            self.contacts.clear();
            collision::detect_collisions(&self.bodies, &mut self.contacts);
            
            // Resolve collisions
            for _ in 0..self.config.iterations {
                collision::resolve_contacts(&mut self.bodies, &self.contacts);
            }
        }
    }
    
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Contact information
#[derive(Clone, Copy, Debug)]
pub struct Contact {
    pub body_a: usize,
    pub body_b: usize,
    pub normal: Vec2,
    pub penetration: f32,
    pub point: Vec2,
}
