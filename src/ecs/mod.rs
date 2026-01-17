//! Entity Component System
//! 
//! Simple ECS wrapper around hecs for game entities.
//! Rust manages all logic, no ASM here.

pub mod components;
pub mod systems;

use hecs::Entity;

/// Game world containing all entities
pub struct World {
    inner: hecs::World,
    entity_count: u32,
}

impl World {
    pub fn new() -> Self {
        Self {
            inner: hecs::World::new(),
            entity_count: 0,
        }
    }
    
    /// Spawn a new entity with components
    pub fn spawn<T: hecs::DynamicBundle>(&mut self, components: T) -> Entity {
        self.entity_count += 1;
        self.inner.spawn(components)
    }
    
    /// Despawn an entity
    pub fn despawn(&mut self, entity: Entity) -> Result<(), hecs::NoSuchEntity> {
        self.entity_count = self.entity_count.saturating_sub(1);
        self.inner.despawn(entity)
    }
    
    /// Get component from entity
    pub fn get<T: hecs::Component>(&self, entity: Entity) -> Option<hecs::Ref<T>> {
        self.inner.get::<&T>(entity).ok()
    }
    
    /// Get mutable component from entity
    pub fn get_mut<T: hecs::Component>(&mut self, entity: Entity) -> Option<hecs::RefMut<T>> {
        self.inner.get::<&mut T>(entity).ok()
    }
    
    /// Query entities with specific components
    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.inner.query::<Q>()
    }
    
    /// Query entities mutably
    pub fn query_mut<Q: hecs::Query>(&mut self) -> hecs::QueryBorrow<'_, Q> {
        self.inner.query::<Q>()
    }
    
    /// Get inner hecs world
    pub fn inner(&self) -> &hecs::World {
        &self.inner
    }
    
    /// Get inner hecs world mutably
    pub fn inner_mut(&mut self) -> &mut hecs::World {
        &mut self.inner
    }
    
    /// Entity count
    pub fn entity_count(&self) -> u32 {
        self.entity_count
    }
    
    /// Clear all entities
    pub fn clear(&mut self) {
        self.inner.clear();
        self.entity_count = 0;
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
