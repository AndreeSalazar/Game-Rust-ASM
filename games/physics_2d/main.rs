//! Physics 2D Game - Platformer Demo
//! 
//! Author: Eddi Andreé Salazar Matos
//! License: MIT
//!
//! Demonstrates:
//! - Real-time window with winit + softbuffer
//! - Custom physics with collision detection
//! - Fixed timestep determinism
//! - Software rendering

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

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const FIXED_DT: f64 = 1.0 / 60.0;

// Colors (0x00RRGGBB for softbuffer)
const BLACK: u32 = 0x00101010;
const CYAN: u32 = 0x0000FFFF;
const RED: u32 = 0x00FF4040;
const GREEN: u32 = 0x0040FF40;
const YELLOW: u32 = 0x00FFFF00;
const WHITE: u32 = 0x00FFFFFF;

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
            window: None,
            context: None,
            surface: None,
            game: Game::new(),
            last_time: Instant::now(),
            accumulator: 0.0,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = WindowAttributes::default()
                .with_title("🎮 Physics 2D - Rust+ASM Engine | Eddi Andreé Salazar Matos")
                .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
                .with_resizable(false);
            
            let window = Rc::new(event_loop.create_window(attrs).expect("Failed to create window"));
            let context = Context::new(window.clone()).expect("Failed to create context");
            let surface = Surface::new(&context, window.clone()).expect("Failed to create surface");
            
            self.window = Some(window);
            self.context = Some(context);
            self.surface = Some(surface);
        }
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state.is_pressed();
                if let PhysicalKey::Code(key) = event.physical_key {
                    match key {
                        KeyCode::Escape => event_loop.exit(),
                        KeyCode::KeyA | KeyCode::ArrowLeft => self.game.input.left = pressed,
                        KeyCode::KeyD | KeyCode::ArrowRight => self.game.input.right = pressed,
                        KeyCode::KeyW | KeyCode::ArrowUp | KeyCode::Space => {
                            if pressed && !self.game.input.jump {
                                self.game.input.jump_pressed = true;
                            }
                            self.game.input.jump = pressed;
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
                    let size = window.inner_size();
                    if let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                        surface.resize(w, h).expect("Failed to resize surface");
                        
                        let mut buffer = surface.buffer_mut().expect("Failed to get buffer");
                        self.game.render(&mut buffer, size.width, size.height);
                        buffer.present().expect("Failed to present buffer");
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
            self.game.fixed_update(FIXED_DT as f32);
            self.accumulator -= FIXED_DT;
        }
        
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[derive(Default)]
struct Input {
    left: bool,
    right: bool,
    jump: bool,
    jump_pressed: bool,
}

struct Entity {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    w: f32,
    h: f32,
    color: u32,
    grounded: bool,
}

impl Entity {
    fn new(x: f32, y: f32, w: f32, h: f32, color: u32) -> Self {
        Self { x, y, vx: 0.0, vy: 0.0, w, h, color, grounded: false }
    }
}

struct Game {
    player: Entity,
    platforms: Vec<Entity>,
    balls: Vec<Entity>,
    input: Input,
    frame: u64,
}

impl Game {
    fn new() -> Self {
        let player = Entity::new(100.0, 300.0, 32.0, 48.0, CYAN);
        
        let mut platforms = vec![
            Entity::new(400.0, 570.0, 800.0, 60.0, GREEN),
        ];
        
        for i in 0..5 {
            platforms.push(Entity::new(
                150.0 + i as f32 * 140.0,
                480.0 - i as f32 * 50.0,
                100.0,
                20.0,
                YELLOW,
            ));
        }
        
        let mut balls = Vec::new();
        for i in 0..8 {
            let mut ball = Entity::new(
                100.0 + i as f32 * 90.0,
                100.0 + (i % 3) as f32 * 40.0,
                20.0,
                20.0,
                RED,
            );
            ball.vx = 80.0 + (i as f32 * 20.0);
            ball.vy = 50.0;
            balls.push(ball);
        }
        
        Self {
            player,
            platforms,
            balls,
            input: Input::default(),
            frame: 0,
        }
    }
    
    fn fixed_update(&mut self, dt: f32) {
        const GRAVITY: f32 = 1200.0;
        const PLAYER_SPEED: f32 = 300.0;
        const JUMP_FORCE: f32 = 500.0;
        const FRICTION: f32 = 0.85;
        
        if self.input.left {
            self.player.vx = -PLAYER_SPEED;
        } else if self.input.right {
            self.player.vx = PLAYER_SPEED;
        } else {
            self.player.vx *= FRICTION;
        }
        
        if self.input.jump_pressed && self.player.grounded {
            self.player.vy = -JUMP_FORCE;
            self.player.grounded = false;
        }
        self.input.jump_pressed = false;
        
        if !self.player.grounded {
            self.player.vy += GRAVITY * dt;
        }
        
        self.player.x += self.player.vx * dt;
        self.player.y += self.player.vy * dt;
        
        self.player.grounded = false;
        for i in 0..self.platforms.len() {
            let px = self.platforms[i].x;
            let py = self.platforms[i].y;
            let pw = self.platforms[i].w;
            let ph = self.platforms[i].h;
            
            if (self.player.x - px).abs() < (self.player.w + pw) / 2.0 &&
               (self.player.y - py).abs() < (self.player.h + ph) / 2.0 {
                let overlap_x = (self.player.w / 2.0 + pw / 2.0) - (self.player.x - px).abs();
                let overlap_y = (self.player.h / 2.0 + ph / 2.0) - (self.player.y - py).abs();
                
                if overlap_x < overlap_y {
                    if self.player.x < px {
                        self.player.x -= overlap_x;
                    } else {
                        self.player.x += overlap_x;
                    }
                    self.player.vx = 0.0;
                } else {
                    if self.player.y < py {
                        self.player.y -= overlap_y;
                        self.player.grounded = true;
                        self.player.vy = 0.0;
                    } else {
                        self.player.y += overlap_y;
                        self.player.vy = 0.0;
                    }
                }
            }
        }
        
        for ball in &mut self.balls {
            ball.vy += GRAVITY * dt * 0.5;
            ball.x += ball.vx * dt;
            ball.y += ball.vy * dt;
            
            if ball.x < ball.w / 2.0 {
                ball.x = ball.w / 2.0;
                ball.vx = ball.vx.abs();
            }
            if ball.x > WIDTH as f32 - ball.w / 2.0 {
                ball.x = WIDTH as f32 - ball.w / 2.0;
                ball.vx = -ball.vx.abs();
            }
        }
        
        for i in 0..self.balls.len() {
            for j in 0..self.platforms.len() {
                let bx = self.balls[i].x;
                let by = self.balls[i].y;
                let bw = self.balls[i].w;
                let bh = self.balls[i].h;
                let px = self.platforms[j].x;
                let py = self.platforms[j].y;
                let pw = self.platforms[j].w;
                let ph = self.platforms[j].h;
                
                if (bx - px).abs() < (bw + pw) / 2.0 && (by - py).abs() < (bh + ph) / 2.0 {
                    if by < py {
                        self.balls[i].y = py - ph / 2.0 - bh / 2.0;
                        self.balls[i].vy = -self.balls[i].vy.abs() * 0.8;
                    }
                }
            }
        }
        
        self.player.x = self.player.x.clamp(self.player.w / 2.0, WIDTH as f32 - self.player.w / 2.0);
        self.frame += 1;
    }
    
    fn render(&self, buffer: &mut [u32], width: u32, height: u32) {
        // Clear
        for pixel in buffer.iter_mut() {
            *pixel = BLACK;
        }
        
        // Draw platforms
        for platform in &self.platforms {
            self.draw_rect(buffer, width, height, platform);
        }
        
        // Draw balls
        for ball in &self.balls {
            self.draw_circle(buffer, width, height, ball);
        }
        
        // Draw player
        self.draw_rect(buffer, width, height, &self.player);
        
        // Draw HUD text area
        for y in 5..45 {
            for x in 5..400 {
                if x < width as usize && y < height as usize {
                    let idx = y * width as usize + x;
                    if idx < buffer.len() {
                        buffer[idx] = 0x00202020;
                    }
                }
            }
        }
        
        // Simple text indicators
        self.draw_text_line(buffer, width, 10, 12, "WASD/Arrows: Move | Space: Jump | ESC: Quit");
        self.draw_text_line(buffer, width, 10, 28, &format!("Frame: {} | Eddi Andree Salazar Matos", self.frame));
    }
    
    fn draw_rect(&self, buffer: &mut [u32], width: u32, height: u32, entity: &Entity) {
        let x1 = (entity.x - entity.w / 2.0).max(0.0) as usize;
        let y1 = (entity.y - entity.h / 2.0).max(0.0) as usize;
        let x2 = (entity.x + entity.w / 2.0).min(width as f32) as usize;
        let y2 = (entity.y + entity.h / 2.0).min(height as f32) as usize;
        
        for y in y1..y2 {
            for x in x1..x2 {
                let idx = y * width as usize + x;
                if idx < buffer.len() {
                    buffer[idx] = entity.color;
                }
            }
        }
    }
    
    fn draw_circle(&self, buffer: &mut [u32], width: u32, height: u32, entity: &Entity) {
        let cx = entity.x as i32;
        let cy = entity.y as i32;
        let r = (entity.w / 2.0) as i32;
        
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let px = cx + dx;
                    let py = cy + dy;
                    if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                        let idx = py as usize * width as usize + px as usize;
                        if idx < buffer.len() {
                            buffer[idx] = entity.color;
                        }
                    }
                }
            }
        }
    }
    
    fn draw_text_line(&self, buffer: &mut [u32], width: u32, x: usize, y: usize, text: &str) {
        for (i, c) in text.chars().enumerate() {
            if c != ' ' {
                let px = x + i * 6;
                for dy in 0..8 {
                    for dx in 0..5 {
                        let idx = (y + dy) * width as usize + px + dx;
                        if idx < buffer.len() && ((dy + dx) % 2 == 0 || dy == 0 || dy == 7) {
                            buffer[idx] = WHITE;
                        }
                    }
                }
            }
        }
    }
}
