//! Collision detection
//! 
//! Rust: Broad phase, collision pairs
//! ASM: AABB tests, circle tests, SAT (narrow phase hot paths)

use crate::math::Vec2;
use super::{Body, Contact};

#[cfg(not(no_asm))]
extern "C" {
    fn collision_aabb_batch(
        positions: *const Vec2,
        half_extents: *const Vec2,
        count: usize,
        pairs: *mut u32,
        max_pairs: usize,
    ) -> usize;
    
    fn collision_circle_batch(
        positions: *const Vec2,
        radii: *const f32,
        count: usize,
        pairs: *mut u32,
        max_pairs: usize,
    ) -> usize;
}

/// AABB for broad phase
#[derive(Clone, Copy, Debug, Default)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABB {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }
    
    pub fn from_center(center: Vec2, half_extents: Vec2) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }
    
    #[inline]
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y
    }
    
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }
    
    pub fn half_extents(&self) -> Vec2 {
        (self.max - self.min) * 0.5
    }
    
    pub fn expand(&self, amount: f32) -> AABB {
        AABB {
            min: self.min - Vec2::splat(amount),
            max: self.max + Vec2::splat(amount),
        }
    }
}

/// Circle collider
#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Circle {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }
    
    #[inline]
    pub fn intersects(&self, other: &Circle) -> bool {
        let dist_sq = self.center.distance_squared(other.center);
        let radii_sum = self.radius + other.radius;
        dist_sq < radii_sum * radii_sum
    }
    
    pub fn to_aabb(&self) -> AABB {
        AABB::from_center(self.center, Vec2::splat(self.radius))
    }
}

/// Circle vs Circle collision
pub fn circle_vs_circle(a: &Circle, b: &Circle) -> Option<Contact> {
    let diff = b.center - a.center;
    let dist_sq = diff.length_squared();
    let radii_sum = a.radius + b.radius;
    
    if dist_sq >= radii_sum * radii_sum {
        return None;
    }
    
    let dist = dist_sq.sqrt();
    let normal = if dist > 0.0 {
        diff / dist
    } else {
        Vec2::UP
    };
    
    Some(Contact {
        body_a: 0,
        body_b: 0,
        normal,
        penetration: radii_sum - dist,
        point: a.center + normal * a.radius,
    })
}

/// AABB vs AABB collision
pub fn aabb_vs_aabb(a: &AABB, b: &AABB) -> Option<Contact> {
    let a_center = a.center();
    let b_center = b.center();
    let a_half = a.half_extents();
    let b_half = b.half_extents();
    
    let diff = b_center - a_center;
    let overlap = a_half + b_half - diff.abs();
    
    if overlap.x <= 0.0 || overlap.y <= 0.0 {
        return None;
    }
    
    let (normal, penetration) = if overlap.x < overlap.y {
        (Vec2::new(diff.x.signum(), 0.0), overlap.x)
    } else {
        (Vec2::new(0.0, diff.y.signum()), overlap.y)
    };
    
    Some(Contact {
        body_a: 0,
        body_b: 0,
        normal,
        penetration,
        point: a_center + normal * a_half.x.min(a_half.y),
    })
}

/// Detect collisions between all bodies (simple O(n²) for now)
pub fn detect_collisions(bodies: &[Body], contacts: &mut Vec<Contact>) {
    let radius = 10.0; // Default radius for now
    
    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            let a = Circle::new(bodies[i].position, radius);
            let b = Circle::new(bodies[j].position, radius);
            
            if let Some(mut contact) = circle_vs_circle(&a, &b) {
                contact.body_a = i;
                contact.body_b = j;
                contacts.push(contact);
            }
        }
    }
}

/// Resolve contact constraints
pub fn resolve_contacts(bodies: &mut [Body], contacts: &[Contact]) {
    for contact in contacts {
        let (a_inv_mass, b_inv_mass, a_restitution, b_restitution, relative_vel);
        {
            let a = &bodies[contact.body_a];
            let b = &bodies[contact.body_b];
            
            if a.is_static() && b.is_static() {
                continue;
            }
            
            a_inv_mass = a.inv_mass;
            b_inv_mass = b.inv_mass;
            a_restitution = a.restitution;
            b_restitution = b.restitution;
            relative_vel = b.velocity - a.velocity;
        }
        
        let vel_along_normal = relative_vel.dot(contact.normal);
        
        if vel_along_normal > 0.0 {
            continue;
        }
        
        let e = a_restitution.min(b_restitution);
        let j = -(1.0 + e) * vel_along_normal / (a_inv_mass + b_inv_mass);
        let impulse = contact.normal * j;
        
        bodies[contact.body_a].velocity -= impulse * a_inv_mass;
        bodies[contact.body_b].velocity += impulse * b_inv_mass;
        
        // Position correction
        let correction = contact.normal * (contact.penetration * 0.8 / (a_inv_mass + b_inv_mass));
        bodies[contact.body_a].position -= correction * a_inv_mass;
        bodies[contact.body_b].position += correction * b_inv_mass;
    }
}
