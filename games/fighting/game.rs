//! Fighting Game Implementation
//! 
//! Uses fixed-point math for determinism.
//! Frame-perfect input and hitbox detection.

use engine::{
    EngineConfig,
    math::{Vec2, FixedPoint},
    render::{Renderer, colors},
    input::{InputState, Key},
    core::{GameLoop, Timer},
};

const GROUND_Y: i32 = 500;
const GRAVITY: FixedPoint = FixedPoint::from_raw(0x0000_6000); // ~0.375
const WALK_SPEED: FixedPoint = FixedPoint::from_raw(0x0003_0000); // 3.0
const JUMP_FORCE: FixedPoint = FixedPoint::from_raw(-786432); // -12.0 in 16.16 fixed point

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FighterState {
    Idle,
    Walking,
    Jumping,
    Attacking,
    Hitstun,
    Blockstun,
}

#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    pub x: FixedPoint,
    pub y: FixedPoint,
    pub width: FixedPoint,
    pub height: FixedPoint,
    pub active: bool,
}

impl Hitbox {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x: FixedPoint::from_int(x),
            y: FixedPoint::from_int(y),
            width: FixedPoint::from_int(w),
            height: FixedPoint::from_int(h),
            active: false,
        }
    }
    
    pub fn intersects(&self, other: &Hitbox) -> bool {
        if !self.active || !other.active {
            return false;
        }
        
        let self_right = self.x + self.width;
        let self_bottom = self.y + self.height;
        let other_right = other.x + other.width;
        let other_bottom = other.y + other.height;
        
        self.x < other_right && self_right > other.x &&
        self.y < other_bottom && self_bottom > other.y
    }
}

#[derive(Clone)]
pub struct Fighter {
    pub x: FixedPoint,
    pub y: FixedPoint,
    pub vel_x: FixedPoint,
    pub vel_y: FixedPoint,
    pub facing_right: bool,
    pub state: FighterState,
    pub state_frame: u8,
    pub health: u8,
    pub hitbox: Hitbox,
    pub hurtbox: Hitbox,
    pub grounded: bool,
}

impl Fighter {
    pub fn new(x: i32, facing_right: bool) -> Self {
        Self {
            x: FixedPoint::from_int(x),
            y: FixedPoint::from_int(GROUND_Y),
            vel_x: FixedPoint::ZERO,
            vel_y: FixedPoint::ZERO,
            facing_right,
            state: FighterState::Idle,
            state_frame: 0,
            health: 100,
            hitbox: Hitbox::new(0, 0, 40, 20),
            hurtbox: Hitbox::new(0, 0, 50, 100),
            grounded: true,
        }
    }
    
    pub fn update(&mut self) {
        // Apply gravity
        if !self.grounded {
            self.vel_y = self.vel_y + GRAVITY;
        }
        
        // Apply velocity
        self.x = self.x + self.vel_x;
        self.y = self.y + self.vel_y;
        
        // Ground check
        let ground = FixedPoint::from_int(GROUND_Y);
        if self.y >= ground {
            self.y = ground;
            self.vel_y = FixedPoint::ZERO;
            self.grounded = true;
            
            if self.state == FighterState::Jumping {
                self.state = FighterState::Idle;
                self.state_frame = 0;
            }
        }
        
        // Update hurtbox position
        self.hurtbox.x = self.x - FixedPoint::from_int(25);
        self.hurtbox.y = self.y - FixedPoint::from_int(100);
        self.hurtbox.active = true;
        
        // Update state
        self.state_frame = self.state_frame.saturating_add(1);
        
        // Handle attack frames
        if self.state == FighterState::Attacking {
            // Active frames 3-6
            if self.state_frame >= 3 && self.state_frame <= 6 {
                self.hitbox.active = true;
                let offset = if self.facing_right { 30 } else { -70 };
                self.hitbox.x = self.x + FixedPoint::from_int(offset);
                self.hitbox.y = self.y - FixedPoint::from_int(60);
            } else {
                self.hitbox.active = false;
            }
            
            // Recovery at frame 15
            if self.state_frame >= 15 {
                self.state = FighterState::Idle;
                self.state_frame = 0;
            }
        }
        
        // Handle hitstun
        if self.state == FighterState::Hitstun {
            if self.state_frame >= 20 {
                self.state = FighterState::Idle;
                self.state_frame = 0;
            }
        }
    }
    
    pub fn walk(&mut self, direction: i32) {
        if self.state != FighterState::Idle && self.state != FighterState::Walking {
            return;
        }
        
        self.state = FighterState::Walking;
        self.facing_right = direction > 0;
        
        if direction > 0 {
            self.vel_x = WALK_SPEED;
        } else if direction < 0 {
            self.vel_x = FixedPoint::ZERO - WALK_SPEED;
        }
    }
    
    pub fn stop(&mut self) {
        if self.state == FighterState::Walking {
            self.state = FighterState::Idle;
            self.vel_x = FixedPoint::ZERO;
        }
    }
    
    pub fn jump(&mut self) {
        if !self.grounded || self.state == FighterState::Attacking {
            return;
        }
        
        self.state = FighterState::Jumping;
        self.state_frame = 0;
        self.vel_y = JUMP_FORCE;
        self.grounded = false;
    }
    
    pub fn attack(&mut self) {
        if self.state == FighterState::Attacking || !self.grounded {
            return;
        }
        
        self.state = FighterState::Attacking;
        self.state_frame = 0;
        self.vel_x = FixedPoint::ZERO;
    }
    
    pub fn take_hit(&mut self, damage: u8) {
        self.health = self.health.saturating_sub(damage);
        self.state = FighterState::Hitstun;
        self.state_frame = 0;
        self.hitbox.active = false;
        
        // Knockback
        let knockback = if self.facing_right {
            FixedPoint::from_int(-5)
        } else {
            FixedPoint::from_int(5)
        };
        self.vel_x = knockback;
    }
}

pub struct FightingGame {
    config: EngineConfig,
    renderer: Renderer,
    input: InputState,
    game_loop: GameLoop,
    player1: Fighter,
    player2: Fighter,
    frame: u64,
    running: bool,
}

impl FightingGame {
    pub fn new(config: EngineConfig) -> Self {
        let renderer = Renderer::new(config.width, config.height);
        let game_loop = GameLoop::new(&config);
        
        Self {
            config: config.clone(),
            renderer,
            input: InputState::new(),
            game_loop,
            player1: Fighter::new(200, true),
            player2: Fighter::new(600, false),
            frame: 0,
            running: true,
        }
    }
    
    pub fn run(&mut self) {
        log::info!("Starting Fighting Game...");
        log::info!("Controls: WASD to move P1, Space to attack, Arrow keys for P2, Enter to attack");
        
        // Simulate a fight
        for frame in 0..300 {
            self.input.begin_frame();
            
            // Simulate inputs
            match frame {
                0..=30 => self.input.key_pressed(Key::D),
                40..=45 => self.input.key_pressed(Key::Space),
                60..=90 => self.input.key_pressed(Key::A),
                100..=105 => {
                    self.input.key_pressed(Key::W);
                    self.input.key_pressed(Key::Space);
                }
                _ => {}
            }
            
            let tick = self.game_loop.tick();
            
            for _ in 0..tick.fixed_updates {
                self.fixed_update();
            }
            
            self.render();
            self.frame += 1;
            
            if frame % 60 == 0 {
                log::info!(
                    "Frame {}: P1 HP={}, state={:?} | P2 HP={}, state={:?}",
                    frame,
                    self.player1.health, self.player1.state,
                    self.player2.health, self.player2.state,
                );
            }
        }
        
        log::info!("Fighting Game demo complete!");
        log::info!("Final: P1 HP={}, P2 HP={}", self.player1.health, self.player2.health);
    }
    
    fn fixed_update(&mut self) {
        // Handle P1 input
        if self.input.is_key_down(Key::A) {
            self.player1.walk(-1);
        } else if self.input.is_key_down(Key::D) {
            self.player1.walk(1);
        } else {
            self.player1.stop();
        }
        
        if self.input.is_key_pressed(Key::W) {
            self.player1.jump();
        }
        
        if self.input.is_key_pressed(Key::Space) {
            self.player1.attack();
        }
        
        // Update fighters
        self.player1.update();
        self.player2.update();
        
        // Check hit detection (ASM would accelerate this)
        if self.player1.hitbox.intersects(&self.player2.hurtbox) {
            self.player2.take_hit(10);
            self.player1.hitbox.active = false;
        }
        
        if self.player2.hitbox.intersects(&self.player1.hurtbox) {
            self.player1.take_hit(10);
            self.player2.hitbox.active = false;
        }
    }
    
    fn render(&mut self) {
        self.renderer.clear(0xFF202020);
        
        // Draw ground
        self.renderer.fill_rect(0, GROUND_Y, self.config.width, 100, 0xFF404040);
        
        // Draw fighters - clone to avoid borrow issues
        let p1 = self.player1.clone();
        let p2 = self.player2.clone();
        self.draw_fighter(&p1, colors::CYAN);
        self.draw_fighter(&p2, colors::RED);
        
        // Draw health bars
        self.draw_health_bar(50, 30, p1.health, colors::CYAN);
        self.draw_health_bar(self.config.width as i32 - 250, 30, p2.health, colors::RED);
    }
    
    fn draw_fighter(&mut self, fighter: &Fighter, color: u32) {
        let x = fighter.x.to_int();
        let y = fighter.y.to_int();
        
        // Body
        self.renderer.fill_rect(x - 25, y - 100, 50, 100, color);
        
        // Hurtbox (debug)
        if fighter.hurtbox.active {
            let hx = fighter.hurtbox.x.to_int();
            let hy = fighter.hurtbox.y.to_int();
            let hw = fighter.hurtbox.width.to_int() as u32;
            let hh = fighter.hurtbox.height.to_int() as u32;
            self.renderer.fill_rect(hx, hy, hw, hh, 0x4000FF00);
        }
        
        // Hitbox (debug)
        if fighter.hitbox.active {
            let hx = fighter.hitbox.x.to_int();
            let hy = fighter.hitbox.y.to_int();
            let hw = fighter.hitbox.width.to_int() as u32;
            let hh = fighter.hitbox.height.to_int() as u32;
            self.renderer.fill_rect(hx, hy, hw, hh, 0x40FF0000);
        }
    }
    
    fn draw_health_bar(&mut self, x: i32, y: i32, health: u8, color: u32) {
        // Background
        self.renderer.fill_rect(x, y, 200, 20, 0xFF333333);
        
        // Health
        let health_width = (health as u32 * 2).min(200);
        self.renderer.fill_rect(x, y, health_width, 20, color);
    }
}
