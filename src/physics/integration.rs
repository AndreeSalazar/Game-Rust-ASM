//! Physics integration
//! 
//! ASM accelerates the integration step for large body counts.

use super::Body;

#[cfg(not(no_asm))]
extern "C" {
    fn physics_integrate_batch(
        positions: *mut f32,
        velocities: *mut f32,
        accelerations: *const f32,
        inv_masses: *const f32,
        dt: f32,
        count: usize,
    );
}

/// Integrate all bodies using Semi-implicit Euler
pub fn integrate_bodies(bodies: &mut [Body], dt: f32) {
    #[cfg(not(no_asm))]
    {
        if bodies.len() >= 64 {
            integrate_bodies_asm(bodies, dt);
            return;
        }
    }
    
    // Rust fallback / small batches
    integrate_bodies_rust(bodies, dt);
}

/// Rust implementation of integration
fn integrate_bodies_rust(bodies: &mut [Body], dt: f32) {
    for body in bodies.iter_mut() {
        if body.is_static() {
            continue;
        }
        
        // Semi-implicit Euler
        body.velocity.x += body.acceleration.x * dt;
        body.velocity.y += body.acceleration.y * dt;
        body.position.x += body.velocity.x * dt;
        body.position.y += body.velocity.y * dt;
        
        // Clear acceleration
        body.acceleration.x = 0.0;
        body.acceleration.y = 0.0;
    }
}

/// ASM-accelerated integration (for large batches)
#[cfg(not(no_asm))]
fn integrate_bodies_asm(bodies: &mut [Body], dt: f32) {
    // For now, use Rust implementation
    // ASM version would operate on SoA data layout
    integrate_bodies_rust(bodies, dt);
}

/// Verlet integration (alternative, more stable for constraints)
pub fn integrate_verlet(bodies: &mut [Body], prev_positions: &mut [crate::math::Vec2], dt: f32) {
    for (i, body) in bodies.iter_mut().enumerate() {
        if body.is_static() {
            continue;
        }
        
        let temp = body.position;
        let velocity = body.position - prev_positions[i];
        
        body.position = body.position + velocity + body.acceleration * dt * dt;
        prev_positions[i] = temp;
        
        body.acceleration.x = 0.0;
        body.acceleration.y = 0.0;
    }
}
