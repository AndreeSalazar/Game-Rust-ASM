//! Raycaster Game Implementation

use engine::{
    EngineConfig,
    math::Vec2,
    render::{Renderer, colors, raycast::{Raycaster, RaycastConfig}},
    input::{InputState, Key},
    core::{GameLoop, Timer},
};

const MOVE_SPEED: f32 = 3.0;
const ROT_SPEED: f32 = 2.0;

pub struct RaycasterGame {
    config: EngineConfig,
    renderer: Renderer,
    raycaster: Raycaster,
    input: InputState,
    game_loop: GameLoop,
    map: Vec<u8>,
    map_width: u32,
    map_height: u32,
    running: bool,
}

impl RaycasterGame {
    pub fn new(config: EngineConfig) -> Self {
        let renderer = Renderer::new(config.width, config.height);
        let game_loop = GameLoop::new(&config);
        
        let raycast_config = RaycastConfig {
            fov: std::f32::consts::PI / 3.0,
            max_distance: 16.0,
            wall_height: 1.0,
        };
        let raycaster = Raycaster::new(raycast_config, config.width);
        
        // Create map (1 = wall, 0 = empty)
        #[rustfmt::skip]
        let map = vec![
            1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
            1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
            1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
            1,0,0,1,1,0,0,0,0,0,1,1,0,0,0,1,
            1,0,0,1,1,0,0,0,0,0,1,1,0,0,0,1,
            1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
            1,0,0,0,0,0,1,1,1,0,0,0,0,0,0,1,
            1,0,0,0,0,0,1,0,1,0,0,0,0,0,0,1,
            1,0,0,0,0,0,1,1,1,0,0,0,0,0,0,1,
            1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
            1,0,0,1,0,0,0,0,0,0,0,1,0,0,0,1,
            1,0,0,1,0,0,0,0,0,0,0,1,0,0,0,1,
            1,0,0,1,0,0,0,0,0,0,0,1,0,0,0,1,
            1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
            1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
            1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
        ];
        
        Self {
            config: config.clone(),
            renderer,
            raycaster,
            input: InputState::new(),
            game_loop,
            map,
            map_width: 16,
            map_height: 16,
            running: true,
        }
    }
    
    pub fn run(&mut self) {
        log::info!("Starting Raycaster game...");
        log::info!("Controls: WASD to move, Left/Right arrows to rotate, ESC to quit");
        
        // Set initial position
        self.raycaster.position = Vec2::new(8.0, 8.0);
        self.raycaster.direction = Vec2::new(-1.0, 0.0);
        
        // Simulate frames
        for frame in 0..300 {
            self.input.begin_frame();
            
            // Simulate movement
            if frame < 100 {
                self.input.key_pressed(Key::W);
            } else if frame < 150 {
                self.input.key_pressed(Key::Right);
            } else if frame < 250 {
                self.input.key_pressed(Key::W);
            }
            
            let tick = self.game_loop.tick();
            
            for _ in 0..tick.fixed_updates {
                self.fixed_update(tick.fixed_dt as f32);
            }
            
            self.render();
            
            if frame % 60 == 0 {
                log::info!("Frame {}: pos=({:.2}, {:.2}), dir=({:.2}, {:.2})", 
                    frame, 
                    self.raycaster.position.x, self.raycaster.position.y,
                    self.raycaster.direction.x, self.raycaster.direction.y);
            }
        }
        
        log::info!("Raycaster demo complete!");
    }
    
    fn fixed_update(&mut self, dt: f32) {
        // Rotation
        if self.input.is_key_down(Key::Left) {
            self.raycaster.rotate(-ROT_SPEED * dt);
        }
        if self.input.is_key_down(Key::Right) {
            self.raycaster.rotate(ROT_SPEED * dt);
        }
        
        // Movement
        if self.input.is_key_down(Key::W) || self.input.is_key_down(Key::Up) {
            self.raycaster.move_forward(MOVE_SPEED * dt, &self.map, self.map_width);
        }
        if self.input.is_key_down(Key::S) || self.input.is_key_down(Key::Down) {
            self.raycaster.move_forward(-MOVE_SPEED * dt, &self.map, self.map_width);
        }
        if self.input.is_key_down(Key::A) {
            self.raycaster.strafe(-MOVE_SPEED * dt, &self.map, self.map_width);
        }
        if self.input.is_key_down(Key::D) {
            self.raycaster.strafe(MOVE_SPEED * dt, &self.map, self.map_width);
        }
    }
    
    fn render(&mut self) {
        // Raycaster renders directly to buffer (ASM-accelerated DDA)
        self.raycaster.render(&mut self.renderer, &self.map, self.map_width, self.map_height);
        
        // Draw minimap
        self.draw_minimap();
    }
    
    fn draw_minimap(&mut self) {
        let scale = 4;
        let offset_x = 10;
        let offset_y = 10;
        
        for y in 0..self.map_height {
            for x in 0..self.map_width {
                let idx = (y * self.map_width + x) as usize;
                let color = if self.map[idx] > 0 { colors::WHITE } else { 0xFF333333 };
                
                self.renderer.fill_rect(
                    offset_x + (x * scale) as i32,
                    offset_y + (y * scale) as i32,
                    scale,
                    scale,
                    color,
                );
            }
        }
        
        // Draw player position
        let px = offset_x + (self.raycaster.position.x * scale as f32) as i32;
        let py = offset_y + (self.raycaster.position.y * scale as f32) as i32;
        self.renderer.fill_circle(px, py, 2, colors::RED);
    }
}
