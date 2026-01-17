//! Common game components

use crate::math::Vec2;

/// Transform component - position, rotation, scale
#[derive(Clone, Copy, Debug, Default)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
        }
    }
}

/// Velocity component
#[derive(Clone, Copy, Debug, Default)]
pub struct Velocity {
    pub linear: Vec2,
    pub angular: f32,
}

/// Rigid body component
#[derive(Clone, Copy, Debug)]
pub struct RigidBody {
    pub mass: f32,
    pub inv_mass: f32,
    pub restitution: f32,
    pub friction: f32,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            mass: 1.0,
            inv_mass: 1.0,
            restitution: 0.5,
            friction: 0.3,
        }
    }
}

impl RigidBody {
    pub fn new(mass: f32) -> Self {
        Self {
            mass,
            inv_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            ..Default::default()
        }
    }
    
    pub fn static_body() -> Self {
        Self {
            mass: 0.0,
            inv_mass: 0.0,
            ..Default::default()
        }
    }
}

/// Collider shapes
#[derive(Clone, Copy, Debug)]
pub enum Collider {
    Circle { radius: f32 },
    AABB { half_extents: Vec2 },
    OBB { half_extents: Vec2 },
}

impl Default for Collider {
    fn default() -> Self {
        Collider::Circle { radius: 1.0 }
    }
}

/// Sprite component for rendering
#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub color: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            color: 0xFFFFFFFF,
            width: 8,
            height: 8,
        }
    }
}

/// Tag for player entity
#[derive(Clone, Copy, Debug, Default)]
pub struct Player;

/// Tag for enemy entities
#[derive(Clone, Copy, Debug, Default)]
pub struct Enemy;

/// Health component
#[derive(Clone, Copy, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
    
    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
    
    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
    
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}
