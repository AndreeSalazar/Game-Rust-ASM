//! Raycaster Game - DOOM-like Engine Demo
//! 
//! Author: Eddi Andreé Salazar Matos
//! License: MIT
//!
//! Demonstrates:
//! - DDA raycasting algorithm
//! - Software 3D rendering
//! - Minimap

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

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const MAP_WIDTH: usize = 16;
const MAP_HEIGHT: usize = 16;
const FIXED_DT: f64 = 1.0 / 60.0;

const MAP: [u8; MAP_WIDTH * MAP_HEIGHT] = [
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,1,1,0,0,0,0,0,1,1,0,0,0,1,
    1,0,0,1,1,0,0,0,0,0,1,1,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,2,2,2,0,0,0,0,0,0,1,
    1,0,0,0,0,0,2,0,2,0,0,0,0,0,0,1,
    1,0,0,0,0,0,2,0,2,0,0,0,0,0,0,1,
    1,0,0,0,0,0,2,2,2,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,3,0,0,0,0,0,0,0,3,0,0,0,1,
    1,0,0,3,0,0,0,0,0,0,0,3,0,0,0,1,
    1,0,0,3,3,3,0,0,0,3,3,3,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
];

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
                .with_title("🎮 Raycaster - DOOM-like | Eddi Andreé Salazar Matos")
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
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state.is_pressed();
                if let PhysicalKey::Code(key) = event.physical_key {
                    match key {
                        KeyCode::Escape => event_loop.exit(),
                        KeyCode::KeyW | KeyCode::ArrowUp => self.game.input.forward = pressed,
                        KeyCode::KeyS | KeyCode::ArrowDown => self.game.input.backward = pressed,
                        KeyCode::KeyA | KeyCode::ArrowLeft => self.game.input.left = pressed,
                        KeyCode::KeyD | KeyCode::ArrowRight => self.game.input.right = pressed,
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
                    let size = window.inner_size();
                    if let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                        surface.resize(w, h).expect("Failed to resize");
                        let mut buffer = surface.buffer_mut().expect("Failed to get buffer");
                        self.game.render(&mut buffer, size.width, size.height);
                        buffer.present().expect("Failed to present");
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

#[derive(Default)]
struct Input { forward: bool, backward: bool, left: bool, right: bool }

struct Game {
    pos_x: f32, pos_y: f32,
    dir_x: f32, dir_y: f32,
    plane_x: f32, plane_y: f32,
    input: Input,
}

impl Game {
    fn new() -> Self {
        Self {
            pos_x: 3.0, pos_y: 3.0,
            dir_x: 1.0, dir_y: 0.0,
            plane_x: 0.0, plane_y: 0.66,
            input: Input::default(),
        }
    }
    
    fn update(&mut self, dt: f32) {
        let move_speed = 3.0 * dt;
        let rot_speed = 2.0 * dt;
        
        if self.input.forward {
            let nx = self.pos_x + self.dir_x * move_speed;
            let ny = self.pos_y + self.dir_y * move_speed;
            if MAP[ny as usize * MAP_WIDTH + nx as usize] == 0 { self.pos_x = nx; self.pos_y = ny; }
        }
        if self.input.backward {
            let nx = self.pos_x - self.dir_x * move_speed;
            let ny = self.pos_y - self.dir_y * move_speed;
            if MAP[ny as usize * MAP_WIDTH + nx as usize] == 0 { self.pos_x = nx; self.pos_y = ny; }
        }
        if self.input.right {
            let old_dir_x = self.dir_x;
            self.dir_x = self.dir_x * (-rot_speed).cos() - self.dir_y * (-rot_speed).sin();
            self.dir_y = old_dir_x * (-rot_speed).sin() + self.dir_y * (-rot_speed).cos();
            let old_plane_x = self.plane_x;
            self.plane_x = self.plane_x * (-rot_speed).cos() - self.plane_y * (-rot_speed).sin();
            self.plane_y = old_plane_x * (-rot_speed).sin() + self.plane_y * (-rot_speed).cos();
        }
        if self.input.left {
            let old_dir_x = self.dir_x;
            self.dir_x = self.dir_x * rot_speed.cos() - self.dir_y * rot_speed.sin();
            self.dir_y = old_dir_x * rot_speed.sin() + self.dir_y * rot_speed.cos();
            let old_plane_x = self.plane_x;
            self.plane_x = self.plane_x * rot_speed.cos() - self.plane_y * rot_speed.sin();
            self.plane_y = old_plane_x * rot_speed.sin() + self.plane_y * rot_speed.cos();
        }
    }
    
    fn render(&self, buffer: &mut [u32], width: u32, height: u32) {
        let half_h = height / 2;
        // Clear - ceiling and floor
        for y in 0..height as usize {
            let color = if y < half_h as usize { 0x00404060 } else { 0x00505050 };
            for x in 0..width as usize {
                buffer[y * width as usize + x] = color;
            }
        }
        
        // Raycasting
        for x in 0..width {
            let camera_x = 2.0 * x as f32 / width as f32 - 1.0;
            let ray_dir_x = self.dir_x + self.plane_x * camera_x;
            let ray_dir_y = self.dir_y + self.plane_y * camera_x;
            
            let mut map_x = self.pos_x as i32;
            let mut map_y = self.pos_y as i32;
            
            let delta_dist_x = if ray_dir_x == 0.0 { 1e30 } else { (1.0 / ray_dir_x).abs() };
            let delta_dist_y = if ray_dir_y == 0.0 { 1e30 } else { (1.0 / ray_dir_y).abs() };
            
            let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
                (-1, (self.pos_x - map_x as f32) * delta_dist_x)
            } else {
                (1, (map_x as f32 + 1.0 - self.pos_x) * delta_dist_x)
            };
            let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
                (-1, (self.pos_y - map_y as f32) * delta_dist_y)
            } else {
                (1, (map_y as f32 + 1.0 - self.pos_y) * delta_dist_y)
            };
            
            let mut side = 0;
            let mut hit = 0;
            
            while hit == 0 {
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    side = 0;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    side = 1;
                }
                if map_x >= 0 && map_x < MAP_WIDTH as i32 && map_y >= 0 && map_y < MAP_HEIGHT as i32 {
                    hit = MAP[map_y as usize * MAP_WIDTH + map_x as usize];
                } else { break; }
            }
            
            let perp_dist = if side == 0 {
                side_dist_x - delta_dist_x
            } else {
                side_dist_y - delta_dist_y
            };
            
            let line_height = if perp_dist > 0.0 { (height as f32 / perp_dist) as i32 } else { height as i32 };
            let draw_start = (-line_height / 2 + half_h as i32).max(0) as usize;
            let draw_end = (line_height / 2 + half_h as i32).min(height as i32 - 1) as usize;
            
            let color = match hit {
                1 => if side == 1 { 0x00AA0000 } else { 0x00FF0000 },
                2 => if side == 1 { 0x0000AA00 } else { 0x0000FF00 },
                3 => if side == 1 { 0x000000AA } else { 0x000000FF },
                _ => 0x00FFFFFF,
            };
            
            for y in draw_start..=draw_end {
                buffer[y * width as usize + x as usize] = color;
            }
        }
        
        // Minimap
        let mm_scale = 6;
        let mm_x = 10;
        let mm_y = 10;
        for my in 0..MAP_HEIGHT {
            for mx in 0..MAP_WIDTH {
                let color = if MAP[my * MAP_WIDTH + mx] > 0 { 0x00FFFFFF } else { 0x00333333 };
                for dy in 0..mm_scale { for dx in 0..mm_scale {
                    let px = mm_x + mx * mm_scale + dx;
                    let py = mm_y + my * mm_scale + dy;
                    if px < width as usize && py < height as usize {
                        buffer[py * width as usize + px] = color;
                    }
                }}
            }
        }
        // Player on minimap
        let px = mm_x + (self.pos_x * mm_scale as f32) as usize;
        let py = mm_y + (self.pos_y * mm_scale as f32) as usize;
        for dy in 0..3 { for dx in 0..3 {
            let idx = (py + dy) * width as usize + px + dx;
            if idx < buffer.len() { buffer[idx] = 0x00FF0000; }
        }}
    }
}
