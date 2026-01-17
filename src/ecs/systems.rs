//! ECS Systems - game logic processors

use crate::ecs::World;
use crate::ecs::components::*;

/// Movement system - updates positions based on velocity
pub fn movement_system(world: &mut World, dt: f32) {
    for (_, (transform, velocity)) in world.inner_mut().query_mut::<(&mut Transform, &Velocity)>() {
        transform.position.x += velocity.linear.x * dt;
        transform.position.y += velocity.linear.y * dt;
        transform.rotation += velocity.angular * dt;
    }
}

/// Gravity system - applies gravity to entities with rigid bodies
pub fn gravity_system(world: &mut World, gravity: f32, dt: f32) {
    for (_, (velocity, body)) in world.inner_mut().query_mut::<(&mut Velocity, &RigidBody)>() {
        if body.inv_mass > 0.0 {
            velocity.linear.y += gravity * dt;
        }
    }
}

/// Health system - removes dead entities
pub fn health_system(world: &mut World) -> Vec<hecs::Entity> {
    let dead: Vec<_> = world.inner()
        .query::<&Health>()
        .iter()
        .filter(|(_, health)| health.is_dead())
        .map(|(entity, _)| entity)
        .collect();
    
    for entity in &dead {
        let _ = world.despawn(*entity);
    }
    
    dead
}
