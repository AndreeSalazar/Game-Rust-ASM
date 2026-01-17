//! Raycasting renderer for DOOM-like games
//! 
//! Rust: Ray setup, wall/floor logic
//! ASM: Inner raycast loop (DDA algorithm)

use crate::math::Vec2;
use super::{Renderer, Color, colors};

#[cfg(not(no_asm))]
extern "C" {
    fn raycast_dda_batch(
        pos_x: f32, pos_y: f32,
        dir_x: *const f32, dir_y: *const f32,
        map: *const u8,
        map_width: u32, map_height: u32,
        distances: *mut f32,
        hit_sides: *mut u8,
        count: usize,
    );
}

/// Raycaster configuration
#[derive(Clone, Debug)]
pub struct RaycastConfig {
    pub fov: f32,
    pub max_distance: f32,
    pub wall_height: f32,
}

impl Default for RaycastConfig {
    fn default() -> Self {
        Self {
            fov: std::f32::consts::PI / 3.0, // 60 degrees
            max_distance: 20.0,
            wall_height: 1.0,
        }
    }
}

/// Raycaster state
pub struct Raycaster {
    pub config: RaycastConfig,
    pub position: Vec2,
    pub direction: Vec2,
    pub plane: Vec2,
    distances: Vec<f32>,
    hit_sides: Vec<u8>,
}

impl Raycaster {
    pub fn new(config: RaycastConfig, width: u32) -> Self {
        let plane_length = (config.fov / 2.0).tan();
        Self {
            config,
            position: Vec2::new(2.0, 2.0),
            direction: Vec2::new(1.0, 0.0),
            plane: Vec2::new(0.0, plane_length),
            distances: vec![0.0; width as usize],
            hit_sides: vec![0; width as usize],
        }
    }
    
    /// Rotate the camera
    pub fn rotate(&mut self, angle: f32) {
        let cos = angle.cos();
        let sin = angle.sin();
        
        let old_dir = self.direction;
        self.direction.x = old_dir.x * cos - old_dir.y * sin;
        self.direction.y = old_dir.x * sin + old_dir.y * cos;
        
        let old_plane = self.plane;
        self.plane.x = old_plane.x * cos - old_plane.y * sin;
        self.plane.y = old_plane.x * sin + old_plane.y * cos;
    }
    
    /// Move forward/backward
    pub fn move_forward(&mut self, distance: f32, map: &[u8], map_width: u32) {
        let new_x = self.position.x + self.direction.x * distance;
        let new_y = self.position.y + self.direction.y * distance;
        
        // Simple collision check
        if map[(new_y as u32 * map_width + self.position.x as u32) as usize] == 0 {
            self.position.y = new_y;
        }
        if map[(self.position.y as u32 * map_width + new_x as u32) as usize] == 0 {
            self.position.x = new_x;
        }
    }
    
    /// Strafe left/right
    pub fn strafe(&mut self, distance: f32, map: &[u8], map_width: u32) {
        let strafe_dir = self.direction.perpendicular();
        let new_x = self.position.x + strafe_dir.x * distance;
        let new_y = self.position.y + strafe_dir.y * distance;
        
        if map[(new_y as u32 * map_width + self.position.x as u32) as usize] == 0 {
            self.position.y = new_y;
        }
        if map[(self.position.y as u32 * map_width + new_x as u32) as usize] == 0 {
            self.position.x = new_x;
        }
    }
    
    /// Cast all rays and render
    pub fn render(&mut self, renderer: &mut Renderer, map: &[u8], map_width: u32, map_height: u32) {
        let width = renderer.width;
        let height = renderer.height;
        
        // Clear with ceiling and floor
        for y in 0..height / 2 {
            for x in 0..width {
                renderer.buffer[(y * width + x) as usize] = 0xFF333333; // Ceiling
            }
        }
        for y in height / 2..height {
            for x in 0..width {
                renderer.buffer[(y * width + x) as usize] = 0xFF666666; // Floor
            }
        }
        
        // Cast rays
        self.cast_rays(map, map_width, map_height, width);
        
        // Draw walls
        for x in 0..width {
            let distance = self.distances[x as usize];
            let side = self.hit_sides[x as usize];
            
            if distance > 0.0 && distance < self.config.max_distance {
                let line_height = ((height as f32 / distance) * self.config.wall_height) as i32;
                let draw_start = (-line_height / 2 + height as i32 / 2).max(0);
                let draw_end = (line_height / 2 + height as i32 / 2).min(height as i32 - 1);
                
                // Color based on side (darker for y-side)
                let color = if side == 0 {
                    0xFFCC0000 // Red for x-side
                } else {
                    0xFF880000 // Darker red for y-side
                };
                
                for y in draw_start..=draw_end {
                    renderer.buffer[(y as u32 * width + x) as usize] = color;
                }
            }
        }
    }
    
    /// Cast rays using DDA algorithm
    fn cast_rays(&mut self, map: &[u8], map_width: u32, map_height: u32, screen_width: u32) {
        for x in 0..screen_width {
            // Calculate ray direction
            let camera_x = 2.0 * x as f32 / screen_width as f32 - 1.0;
            let ray_dir = Vec2::new(
                self.direction.x + self.plane.x * camera_x,
                self.direction.y + self.plane.y * camera_x,
            );
            
            // DDA algorithm
            let (distance, side) = self.dda(ray_dir, map, map_width, map_height);
            self.distances[x as usize] = distance;
            self.hit_sides[x as usize] = side;
        }
    }
    
    /// Digital Differential Analysis for single ray
    fn dda(&self, ray_dir: Vec2, map: &[u8], map_width: u32, map_height: u32) -> (f32, u8) {
        let mut map_x = self.position.x as i32;
        let mut map_y = self.position.y as i32;
        
        let delta_dist_x = if ray_dir.x == 0.0 { f32::MAX } else { (1.0 / ray_dir.x).abs() };
        let delta_dist_y = if ray_dir.y == 0.0 { f32::MAX } else { (1.0 / ray_dir.y).abs() };
        
        let (step_x, mut side_dist_x) = if ray_dir.x < 0.0 {
            (-1, (self.position.x - map_x as f32) * delta_dist_x)
        } else {
            (1, (map_x as f32 + 1.0 - self.position.x) * delta_dist_x)
        };
        
        let (step_y, mut side_dist_y) = if ray_dir.y < 0.0 {
            (-1, (self.position.y - map_y as f32) * delta_dist_y)
        } else {
            (1, (map_y as f32 + 1.0 - self.position.y) * delta_dist_y)
        };
        
        let mut side = 0u8;
        
        // DDA loop
        for _ in 0..64 {
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                map_x += step_x;
                side = 0;
            } else {
                side_dist_y += delta_dist_y;
                map_y += step_y;
                side = 1;
            }
            
            // Check bounds
            if map_x < 0 || map_x >= map_width as i32 || 
               map_y < 0 || map_y >= map_height as i32 {
                return (self.config.max_distance, side);
            }
            
            // Check hit
            if map[(map_y as u32 * map_width + map_x as u32) as usize] > 0 {
                let distance = if side == 0 {
                    side_dist_x - delta_dist_x
                } else {
                    side_dist_y - delta_dist_y
                };
                return (distance, side);
            }
        }
        
        (self.config.max_distance, side)
    }
}
