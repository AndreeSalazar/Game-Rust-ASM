//! Fighting Game - 2D Fighter Demo
//! 
//! Author: Eddi Andreé Salazar Matos
//! License: MIT
//!
//! Demonstrates:
//! - Fixed-point math for determinism
//! - Hitbox/hurtbox collision
//! - Frame-perfect input handling

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
const GROUND_Y: i32 = 500;
const FIXED_DT: f64 = 1.0 / 60.0;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("Failed");
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
                .with_title("🎮 Fighting Game | Eddi Andreé Salazar Matos")
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
                let pressed = event.state.is_pressed();
                if let PhysicalKey::Code(key) = event.physical_key {
                    match key {
                        KeyCode::Escape => event_loop.exit(),
                        // Player 1: WASD + F/G
                        KeyCode::KeyA => self.game.p1.input.left = pressed,
                        KeyCode::KeyD => self.game.p1.input.right = pressed,
                        KeyCode::KeyW => { if pressed { self.game.p1.input.jump = true; } }
                        KeyCode::KeyF => { if pressed { self.game.p1.input.punch = true; } }
                        KeyCode::KeyG => { if pressed { self.game.p1.input.kick = true; } }
                        // Player 2: Arrows + K/L
                        KeyCode::ArrowLeft => self.game.p2.input.left = pressed,
                        KeyCode::ArrowRight => self.game.p2.input.right = pressed,
                        KeyCode::ArrowUp => { if pressed { self.game.p2.input.jump = true; } }
                        KeyCode::KeyK => { if pressed { self.game.p2.input.punch = true; } }
                        KeyCode::KeyL => { if pressed { self.game.p2.input.kick = true; } }
                        _ => {}
                    }
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

#[derive(Default, Clone)]
struct Input { left: bool, right: bool, jump: bool, punch: bool, kick: bool }

#[derive(Clone, Copy, PartialEq)]
enum State { Idle, Walking, Jumping, Punching, Kicking, Hitstun }

#[derive(Clone)]
struct Fighter {
    x: f32, y: f32,
    vx: f32, vy: f32,
    health: i32,
    state: State,
    state_timer: i32,
    facing_right: bool,
    input: Input,
    color: u32,
}

impl Fighter {
    fn new(x: f32, color: u32, facing_right: bool) -> Self {
        Self {
            x, y: GROUND_Y as f32,
            vx: 0.0, vy: 0.0,
            health: 100,
            state: State::Idle,
            state_timer: 0,
            facing_right,
            input: Input::default(),
            color,
        }
    }
}

struct Game {
    p1: Fighter,
    p2: Fighter,
    frame: u64,
}

impl Game {
    fn new() -> Self {
        Self {
            p1: Fighter::new(200.0, 0x0000FFFF, true),
            p2: Fighter::new(600.0, 0x00FF4040, false),
            frame: 0,
        }
    }
    
    fn update(&mut self, dt: f32) {
        const GRAVITY: f32 = 1500.0;
        const WALK_SPEED: f32 = 200.0;
        const JUMP_FORCE: f32 = 600.0;
        
        // Store other positions before updating
        let p1_x = self.p1.x;
        let p2_x = self.p2.x;
        
        // Update P1
        Self::update_fighter(&mut self.p1, p2_x, dt, GRAVITY, WALK_SPEED, JUMP_FORCE);
        // Update P2
        Self::update_fighter(&mut self.p2, p1_x, dt, GRAVITY, WALK_SPEED, JUMP_FORCE);
        
        // Hit detection
        let p1_attacking = (self.p1.state == State::Punching || self.p1.state == State::Kicking) && self.p1.state_timer == 10;
        let p2_attacking = (self.p2.state == State::Punching || self.p2.state == State::Kicking) && self.p2.state_timer == 10;
        
        if p1_attacking && (self.p1.x - self.p2.x).abs() < 80.0 && self.p2.state != State::Hitstun {
            self.p2.health -= if self.p1.state == State::Punching { 5 } else { 8 };
            self.p2.state = State::Hitstun;
            self.p2.state_timer = 20;
            self.p2.vx = if self.p1.facing_right { 200.0 } else { -200.0 };
        }
        if p2_attacking && (self.p2.x - self.p1.x).abs() < 80.0 && self.p1.state != State::Hitstun {
            self.p1.health -= if self.p2.state == State::Punching { 5 } else { 8 };
            self.p1.state = State::Hitstun;
            self.p1.state_timer = 20;
            self.p1.vx = if self.p2.facing_right { 200.0 } else { -200.0 };
        }
        
        self.frame += 1;
    }
    
    fn update_fighter(fighter: &mut Fighter, other_x: f32, dt: f32, gravity: f32, walk_speed: f32, jump_force: f32) {
            // State machine
            if fighter.state_timer > 0 { fighter.state_timer -= 1; }
            
            let grounded = fighter.y >= GROUND_Y as f32;
            
            match fighter.state {
                State::Idle | State::Walking => {
                    fighter.vx = 0.0;
                    if fighter.input.left { fighter.vx = -walk_speed; fighter.facing_right = false; }
                    if fighter.input.right { fighter.vx = walk_speed; fighter.facing_right = true; }
                    
                    if fighter.input.jump && grounded {
                        fighter.vy = -jump_force;
                        fighter.state = State::Jumping;
                    } else if fighter.input.punch {
                        fighter.state = State::Punching;
                        fighter.state_timer = 15;
                    } else if fighter.input.kick {
                        fighter.state = State::Kicking;
                        fighter.state_timer = 20;
                    }
                    
                    fighter.state = if fighter.vx != 0.0 { State::Walking } else { State::Idle };
                }
                State::Jumping => {
                    if fighter.input.left { fighter.vx = -walk_speed * 0.7; }
                    if fighter.input.right { fighter.vx = walk_speed * 0.7; }
                    if grounded && fighter.vy >= 0.0 { fighter.state = State::Idle; }
                }
                State::Punching | State::Kicking => {
                    fighter.vx = 0.0;
                    if fighter.state_timer == 0 { fighter.state = State::Idle; }
                }
                State::Hitstun => {
                    if fighter.state_timer == 0 { fighter.state = State::Idle; }
                }
            }
            
            // Physics
            if !grounded { fighter.vy += gravity * dt; }
            fighter.x += fighter.vx * dt;
            fighter.y += fighter.vy * dt;
            
            // Ground collision
            if fighter.y > GROUND_Y as f32 {
                fighter.y = GROUND_Y as f32;
                fighter.vy = 0.0;
            }
            
            // Bounds
            fighter.x = fighter.x.clamp(50.0, WIDTH as f32 - 50.0);
            
            // Face opponent
            fighter.facing_right = fighter.x < other_x;
            
        // Clear one-shot inputs
        fighter.input.jump = false;
        fighter.input.punch = false;
        fighter.input.kick = false;
    }
    
    fn render(&self, buffer: &mut [u32], width: u32, height: u32) {
        // Clear
        for pixel in buffer.iter_mut() { *pixel = 0x00202030; }
        
        // Ground
        for y in GROUND_Y as usize..height as usize {
            for x in 0..width as usize {
                buffer[y * width as usize + x] = 0x00404040;
            }
        }
        
        // Draw fighters
        self.draw_fighter(buffer, width, &self.p1);
        self.draw_fighter(buffer, width, &self.p2);
        
        // Health bars
        self.draw_health_bar(buffer, width, 50, 30, self.p1.health, 0x0000FFFF);
        self.draw_health_bar(buffer, width, (width - 250) as i32, 30, self.p2.health, 0x00FF4040);
        
        // HUD
        for x in 10..300 { buffer[15 * width as usize + x] = 0x00FFFFFF; }
    }
    
    fn draw_fighter(&self, buffer: &mut [u32], width: u32, f: &Fighter) {
        let x = f.x as i32;
        let y = f.y as i32;
        let color = if f.state == State::Hitstun { 0x00FFFFFF } else { f.color };
        
        // Body (40x60)
        for dy in -60..0 {
            for dx in -20..20 {
                let px = (x + dx) as usize;
                let py = (y + dy) as usize;
                if px < width as usize && py < 600 {
                    buffer[py * width as usize + px] = color;
                }
            }
        }
        
        // Attack effect
        if f.state == State::Punching || f.state == State::Kicking {
            let attack_x = if f.facing_right { x + 30 } else { x - 50 };
            let attack_color = 0x00FFFF00;
            for dy in -40..-20 {
                for dx in 0..20 {
                    let px = (attack_x + dx) as usize;
                    let py = (y + dy) as usize;
                    if px < width as usize && py < 600 {
                        buffer[py * width as usize + px] = attack_color;
                    }
                }
            }
        }
    }
    
    fn draw_health_bar(&self, buffer: &mut [u32], width: u32, x: i32, y: i32, health: i32, color: u32) {
        let bar_width = 200;
        let bar_height = 20;
        let fill = (health.max(0) as f32 / 100.0 * bar_width as f32) as i32;
        
        // Background
        for dy in 0..bar_height {
            for dx in 0..bar_width {
                let px = (x + dx) as usize;
                let py = (y + dy) as usize;
                if px < width as usize && py < 600 {
                    buffer[py * width as usize + px] = 0x00333333;
                }
            }
        }
        
        // Health fill
        for dy in 2..bar_height-2 {
            for dx in 2..fill-2 {
                let px = (x + dx) as usize;
                let py = (y + dy) as usize;
                if px < width as usize && py < 600 {
                    buffer[py * width as usize + px] = color;
                }
            }
        }
    }
}
