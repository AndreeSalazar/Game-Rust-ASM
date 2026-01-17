//! Massive Simulation Game - 10K+ Entities Demo
//! 
//! Author: Eddi Andreé Salazar Matos
//! License: MIT
//!
//! Demonstrates:
//! - SoA (Structure of Arrays) for cache efficiency
//! - SIMD-ready batch processing
//! - Deterministic simulation

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;
use softbuffer::{Context, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const ENTITY_COUNT: usize = 5000;
const FIXED_DT: f64 = 1.0 / 60.0;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}

struct App {
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    game: Game,
    last_time: Instant,
    accumulator: f64,
}

impl App {
    fn new() -> Self {
        Self {
            window: None, context: None, surface: None,
            game: Game::new(ENTITY_COUNT),
            last_time: Instant::now(),
            accumulator: 0.0,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = WindowAttributes::default()
                .with_title("🎮 Massive Sim - 5K Entities | Eddi Andreé Salazar Matos")
                .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
                .with_resizable(false);
            let window = Rc::new(event_loop.create_window(attrs).expect("Failed"));
            let context = Context::new(window.clone()).expect("Failed");
            let surface = Surface::new(&context, window.clone()).expect("Failed");
            self.window = Some(window);
            self.context = Some(context);
            self.surface = Some(surface);
        }
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    if event.state.is_pressed() { event_loop.exit(); }
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
                    let size = window.inner_size();
                    if let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                        surface.resize(w, h).expect("Failed");
                        let mut buffer = surface.buffer_mut().expect("Failed");
                        self.game.render(&mut buffer, size.width, size.height);
                        buffer.present().expect("Failed");
                    }
                }
            }
            _ => {}
        }
    }
    
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_time).as_secs_f64();
        self.last_time = now;
        self.accumulator += dt;
        while self.accumulator >= FIXED_DT {
            self.game.update(FIXED_DT as f32);
            self.accumulator -= FIXED_DT;
        }
        if let Some(window) = &self.window { window.request_redraw(); }
    }
}

struct Game {
    positions_x: Vec<f32>,
    positions_y: Vec<f32>,
    velocities_x: Vec<f32>,
    velocities_y: Vec<f32>,
    colors: Vec<u32>,
    frame: u64,
}

impl Game {
    fn new(count: usize) -> Self {
        let mut positions_x = Vec::with_capacity(count);
        let mut positions_y = Vec::with_capacity(count);
        let mut velocities_x = Vec::with_capacity(count);
        let mut velocities_y = Vec::with_capacity(count);
        let mut colors = Vec::with_capacity(count);
        
        for i in 0..count {
            positions_x.push((i as f32 * 7.3) % WIDTH as f32);
            positions_y.push((i as f32 * 11.7) % HEIGHT as f32);
            velocities_x.push(((i * 13) % 200) as f32 - 100.0);
            velocities_y.push(((i * 17) % 200) as f32 - 100.0);
            colors.push(match i % 5 {
                0 => 0x00FF0000,
                1 => 0x0000FF00,
                2 => 0x000000FF,
                3 => 0x00FFFF00,
                _ => 0x00FF00FF,
            });
        }
        
        Self { positions_x, positions_y, velocities_x, velocities_y, colors, frame: 0 }
    }
    
    fn update(&mut self, dt: f32) {
        let count = self.positions_x.len();
        
        // SIMD-friendly batch update (SoA layout)
        for i in 0..count {
            // Update positions
            self.positions_x[i] += self.velocities_x[i] * dt;
            self.positions_y[i] += self.velocities_y[i] * dt;
            
            // Bounce off walls
            if self.positions_x[i] < 0.0 {
                self.positions_x[i] = 0.0;
                self.velocities_x[i] = self.velocities_x[i].abs();
            }
            if self.positions_x[i] > WIDTH as f32 {
                self.positions_x[i] = WIDTH as f32;
                self.velocities_x[i] = -self.velocities_x[i].abs();
            }
            if self.positions_y[i] < 0.0 {
                self.positions_y[i] = 0.0;
                self.velocities_y[i] = self.velocities_y[i].abs();
            }
            if self.positions_y[i] > HEIGHT as f32 {
                self.positions_y[i] = HEIGHT as f32;
                self.velocities_y[i] = -self.velocities_y[i].abs();
            }
            
            // Simple steering (deterministic)
            let seed = (self.positions_x[i] * 100.0 + self.positions_y[i]) as i32;
            let rand = ((seed.wrapping_mul(1103515245).wrapping_add(12345)) % 1000) as f32 / 1000.0 - 0.5;
            self.velocities_x[i] += rand * 50.0 * dt;
            self.velocities_y[i] += rand * 50.0 * dt;
            
            // Clamp velocity
            let max_vel = 150.0;
            self.velocities_x[i] = self.velocities_x[i].clamp(-max_vel, max_vel);
            self.velocities_y[i] = self.velocities_y[i].clamp(-max_vel, max_vel);
        }
        
        self.frame += 1;
    }
    
    fn render(&self, buffer: &mut [u32], width: u32, height: u32) {
        // Clear
        for pixel in buffer.iter_mut() { *pixel = 0x00101010; }
        
        // Draw entities as pixels
        let count = self.positions_x.len();
        for i in 0..count {
            let x = self.positions_x[i] as usize;
            let y = self.positions_y[i] as usize;
            if x < width as usize && y < height as usize {
                let idx = y * width as usize + x;
                if idx < buffer.len() {
                    buffer[idx] = self.colors[i];
                    // Draw 2x2 for visibility
                    if x + 1 < width as usize { buffer[idx + 1] = self.colors[i]; }
                    if y + 1 < height as usize {
                        buffer[idx + width as usize] = self.colors[i];
                        if x + 1 < width as usize {
                            buffer[idx + width as usize + 1] = self.colors[i];
                        }
                    }
                }
            }
        }
        
        // HUD
        for y in 5..25 { for x in 5..250 {
            let idx = y * width as usize + x;
            if idx < buffer.len() { buffer[idx] = 0x00202020; }
        }}
        // Simple text indicator
        for x in 10..240 {
            let idx = 12 * width as usize + x;
            if idx < buffer.len() && x % 3 == 0 { buffer[idx] = 0x00FFFFFF; }
        }
    }
}
