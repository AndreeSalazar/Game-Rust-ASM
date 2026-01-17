//! Broad phase collision detection
//! 
//! Rust handles spatial partitioning.
//! ASM not needed here - it's O(n) or O(n log n), not the hot path.

use crate::math::Vec2;
use super::collision::AABB;

/// Simple grid-based spatial hash
pub struct SpatialHash {
    cell_size: f32,
    inv_cell_size: f32,
    cells: std::collections::HashMap<(i32, i32), Vec<usize>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            inv_cell_size: 1.0 / cell_size,
            cells: std::collections::HashMap::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.cells.clear();
    }
    
    fn hash(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x * self.inv_cell_size).floor() as i32,
            (pos.y * self.inv_cell_size).floor() as i32,
        )
    }
    
    pub fn insert(&mut self, id: usize, aabb: &AABB) {
        let min_cell = self.hash(aabb.min);
        let max_cell = self.hash(aabb.max);
        
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                self.cells.entry((x, y)).or_default().push(id);
            }
        }
    }
    
    pub fn query(&self, aabb: &AABB) -> Vec<usize> {
        let mut result = Vec::new();
        let min_cell = self.hash(aabb.min);
        let max_cell = self.hash(aabb.max);
        
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                if let Some(ids) = self.cells.get(&(x, y)) {
                    result.extend(ids);
                }
            }
        }
        
        result.sort_unstable();
        result.dedup();
        result
    }
    
    /// Get all potential collision pairs
    pub fn get_pairs(&self) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        
        for ids in self.cells.values() {
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let a = ids[i].min(ids[j]);
                    let b = ids[i].max(ids[j]);
                    pairs.push((a, b));
                }
            }
        }
        
        pairs.sort_unstable();
        pairs.dedup();
        pairs
    }
}

/// Sweep and prune for 1D broad phase
pub struct SweepAndPrune {
    endpoints: Vec<Endpoint>,
}

#[derive(Clone, Copy)]
struct Endpoint {
    value: f32,
    id: usize,
    is_min: bool,
}

impl SweepAndPrune {
    pub fn new() -> Self {
        Self {
            endpoints: Vec::new(),
        }
    }
    
    pub fn update(&mut self, aabbs: &[(usize, AABB)]) {
        self.endpoints.clear();
        
        for (id, aabb) in aabbs {
            self.endpoints.push(Endpoint { value: aabb.min.x, id: *id, is_min: true });
            self.endpoints.push(Endpoint { value: aabb.max.x, id: *id, is_min: false });
        }
        
        self.endpoints.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    }
    
    pub fn get_pairs(&self) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        let mut active: Vec<usize> = Vec::new();
        
        for endpoint in &self.endpoints {
            if endpoint.is_min {
                for &other in &active {
                    let a = endpoint.id.min(other);
                    let b = endpoint.id.max(other);
                    pairs.push((a, b));
                }
                active.push(endpoint.id);
            } else {
                if let Some(pos) = active.iter().position(|&x| x == endpoint.id) {
                    active.swap_remove(pos);
                }
            }
        }
        
        pairs
    }
}

impl Default for SweepAndPrune {
    fn default() -> Self {
        Self::new()
    }
}
