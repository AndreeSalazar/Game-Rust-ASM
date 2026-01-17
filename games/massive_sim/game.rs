//! Massive Simulation Game Implementation
//! 
//! Uses Structure of Arrays (SoA) for cache-friendly updates.
//! ASM SIMD accelerates batch operations.

use engine::{
    EngineConfig,
    math::Vec2,
    render::{Renderer, colors},
    input::{InputState, Key},
    core::{GameLoop, Timer, Profiler},
    physics::broad_phase::SpatialHash,
};

const WORLD_WIDTH: f32 = 1024.0;
const WORLD_HEIGHT: f32 = 768.0;

/// Entity state using Structure of Arrays (SoA) for SIMD
pub struct EntityData {
    pub positions_x: Vec<f32>,
    pub positions_y: Vec<f32>,
    pub velocities_x: Vec<f32>,
    pub velocities_y: Vec<f32>,
    pub colors: Vec<u32>,
    pub radii: Vec<f32>,
    pub states: Vec<EntityState>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EntityState {
    Idle,
    Wandering,
    Seeking,
    Fleeing,
}

impl EntityData {
    pub fn new(capacity: usize) -> Self {
        Self {
            positions_x: Vec::with_capacity(capacity),
            positions_y: Vec::with_capacity(capacity),
            velocities_x: Vec::with_capacity(capacity),
            velocities_y: Vec::with_capacity(capacity),
            colors: Vec::with_capacity(capacity),
            radii: Vec::with_capacity(capacity),
            states: Vec::with_capacity(capacity),
        }
    }
    
    pub fn len(&self) -> usize {
        self.positions_x.len()
    }
    
    pub fn add(&mut self, x: f32, y: f32, color: u32, radius: f32) {
        self.positions_x.push(x);
        self.positions_y.push(y);
        self.velocities_x.push(0.0);
        self.velocities_y.push(0.0);
        self.colors.push(color);
        self.radii.push(radius);
        self.states.push(EntityState::Wandering);
    }
}

pub struct MassiveSimGame {
    config: EngineConfig,
    renderer: Renderer,
    input: InputState,
    game_loop: GameLoop,
    profiler: Profiler,
    entities: EntityData,
    spatial_hash: SpatialHash,
    running: bool,
    frame_count: u64,
}

impl MassiveSimGame {
    pub fn new(config: EngineConfig, entity_count: usize) -> Self {
        let renderer = Renderer::new(config.width, config.height);
        let game_loop = GameLoop::new(&config);
        
        let mut entities = EntityData::new(entity_count);
        // Spawn entities
        for i in 0..entity_count {
            let x = (i as f32 * 7.3) % WORLD_WIDTH;
            let y = (i as f32 * 11.7) % WORLD_HEIGHT;
            let color = match i % 3 {
                0 => colors::RED,
                1 => colors::GREEN,
                _ => colors::BLUE,
            };
            let radius = 2.0 + (i % 3) as f32;
            entities.add(x, y, color, radius);
        }
        
        Self {
            config: config.clone(),
            renderer,
            input: InputState::new(),
            game_loop,
            profiler: Profiler::new(),
            entities,
            spatial_hash: SpatialHash::new(32.0),
            running: true,
            frame_count: 0,
        }
    }
    
    pub fn run(&mut self) {
        log::info!("Starting Massive Sim with {} entities...", self.entities.len());
        
        let mut timer = Timer::new();
        timer.start();
        
        // Simulate frames
        for frame in 0..300 {
            self.input.begin_frame();
            
            let tick = self.game_loop.tick();
            
            // Measure update time
            let mut update_timer = Timer::new();
            update_timer.start();
            
            for _ in 0..tick.fixed_updates {
                self.fixed_update(tick.fixed_dt as f32);
            }
            
            let update_ns = update_timer.elapsed_ns();
            self.profiler.record("update", update_ns);
            
            // Measure render time
            let mut render_timer = Timer::new();
            render_timer.start();
            
            self.render();
            
            let render_ns = render_timer.elapsed_ns();
            self.profiler.record("render", render_ns);
            
            self.frame_count += 1;
            
            if frame % 60 == 0 {
                let update_sample = self.profiler.get("update");
                let render_sample = self.profiler.get("render");
                
                log::info!(
                    "Frame {}: entities={}, update={:.2}ms, render={:.2}ms",
                    frame,
                    self.entities.len(),
                    update_sample.map(|s| s.avg_ms()).unwrap_or(0.0),
                    render_sample.map(|s| s.avg_ms()).unwrap_or(0.0),
                );
            }
        }
        
        let total_ms = timer.elapsed_ms();
        let avg_fps = 300.0 / (total_ms / 1000.0);
        
        log::info!("Massive Sim complete!");
        log::info!("Total time: {:.2}ms, Avg FPS: {:.1}", total_ms, avg_fps);
        self.profiler.print_summary();
    }
    
    fn fixed_update(&mut self, dt: f32) {
        let count = self.entities.len();
        
        // Update velocities (simple wandering behavior)
        // In real implementation, this would use ASM SIMD
        for i in 0..count {
            // Deterministic pseudo-random steering based on position
            let seed = (self.entities.positions_x[i] * 1000.0 + self.entities.positions_y[i]) as i32;
            let rand_x = ((seed.wrapping_mul(1103515245).wrapping_add(12345)) % 1000) as f32 / 1000.0 - 0.5;
            let rand_y = ((seed.wrapping_mul(1103515245).wrapping_add(54321)) % 1000) as f32 / 1000.0 - 0.5;
            self.entities.velocities_x[i] += rand_x * 100.0 * dt;
            self.entities.velocities_y[i] += rand_y * 100.0 * dt;
            
            // Clamp velocity
            let max_speed = 50.0;
            let vx = self.entities.velocities_x[i];
            let vy = self.entities.velocities_y[i];
            let speed = (vx * vx + vy * vy).sqrt();
            
            if speed > max_speed {
                let scale = max_speed / speed;
                self.entities.velocities_x[i] *= scale;
                self.entities.velocities_y[i] *= scale;
            }
        }
        
        // Update positions (SIMD-friendly loop)
        // ASM would process 8 floats at a time with AVX
        for i in 0..count {
            self.entities.positions_x[i] += self.entities.velocities_x[i] * dt;
            self.entities.positions_y[i] += self.entities.velocities_y[i] * dt;
            
            // Wrap around world
            if self.entities.positions_x[i] < 0.0 {
                self.entities.positions_x[i] += WORLD_WIDTH;
            } else if self.entities.positions_x[i] >= WORLD_WIDTH {
                self.entities.positions_x[i] -= WORLD_WIDTH;
            }
            
            if self.entities.positions_y[i] < 0.0 {
                self.entities.positions_y[i] += WORLD_HEIGHT;
            } else if self.entities.positions_y[i] >= WORLD_HEIGHT {
                self.entities.positions_y[i] -= WORLD_HEIGHT;
            }
        }
    }
    
    fn render(&mut self) {
        self.renderer.clear(0xFF111111);
        
        let count = self.entities.len();
        
        // Render all entities as circles
        for i in 0..count {
            let x = self.entities.positions_x[i] as i32;
            let y = self.entities.positions_y[i] as i32;
            let radius = self.entities.radii[i] as i32;
            let color = self.entities.colors[i];
            
            // Simple circle (could be optimized with instanced rendering)
            self.renderer.fill_circle(x, y, radius, color);
        }
        
        // Draw stats
        self.draw_stats();
    }
    
    fn draw_stats(&mut self) {
        // Draw entity count in corner
        let count_text = format!("Entities: {}", self.entities.len());
        // In real implementation, render text
    }
}
