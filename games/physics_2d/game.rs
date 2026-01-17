//! Physics 2D Game Implementation

use engine::{
    Engine, EngineConfig,
    ecs::World,
    ecs::components::*,
    math::Vec2,
    physics::{PhysicsWorld, Body},
    render::{Renderer, colors},
    input::{InputState, Key},
    core::{GameLoop, Timer},
};

const GRAVITY: f32 = 980.0;
const PLAYER_SPEED: f32 = 200.0;
const JUMP_FORCE: f32 = -400.0;

pub struct Physics2DGame {
    config: EngineConfig,
    world: World,
    physics: PhysicsWorld,
    renderer: Renderer,
    input: InputState,
    game_loop: GameLoop,
    player_id: Option<hecs::Entity>,
    running: bool,
}

impl Physics2DGame {
    pub fn new(config: EngineConfig) -> Self {
        let renderer = Renderer::new(config.width, config.height);
        let game_loop = GameLoop::new(&config);
        
        let mut game = Self {
            config: config.clone(),
            world: World::new(),
            physics: PhysicsWorld::new(),
            renderer,
            input: InputState::new(),
            game_loop,
            player_id: None,
            running: true,
        };
        
        game.setup();
        game
    }
    
    fn setup(&mut self) {
        // Create player
        let player = self.world.spawn((
            Transform::new(100.0, 300.0),
            Velocity::default(),
            RigidBody::new(1.0),
            Collider::Circle { radius: 16.0 },
            Sprite { color: colors::CYAN, width: 32, height: 32 },
            Player,
        ));
        self.player_id = Some(player);
        
        // Add player to physics
        self.physics.add_body(Body::new(Vec2::new(100.0, 300.0), 1.0));
        
        // Create ground
        self.world.spawn((
            Transform::new(400.0, 550.0),
            RigidBody::static_body(),
            Collider::AABB { half_extents: Vec2::new(400.0, 25.0) },
            Sprite { color: colors::GREEN, width: 800, height: 50 },
        ));
        self.physics.add_body(Body::static_body(Vec2::new(400.0, 550.0)));
        
        // Create platforms
        for i in 0..5 {
            let x = 150.0 + i as f32 * 150.0;
            let y = 450.0 - i as f32 * 60.0;
            
            self.world.spawn((
                Transform::new(x, y),
                RigidBody::static_body(),
                Collider::AABB { half_extents: Vec2::new(50.0, 10.0) },
                Sprite { color: colors::YELLOW, width: 100, height: 20 },
            ));
            self.physics.add_body(Body::static_body(Vec2::new(x, y)));
        }
        
        // Create some bouncing balls (enemies/bullets)
        for i in 0..10 {
            let x = 100.0 + i as f32 * 70.0;
            let y = 100.0 + (i % 3) as f32 * 50.0;
            
            self.world.spawn((
                Transform::new(x, y),
                Velocity { linear: Vec2::new(50.0, 0.0), angular: 0.0 },
                RigidBody::new(0.5),
                Collider::Circle { radius: 8.0 },
                Sprite { color: colors::RED, width: 16, height: 16 },
                Enemy,
            ));
            
            let mut body = Body::new(Vec2::new(x, y), 0.5);
            body.velocity = Vec2::new(50.0, 0.0);
            body.restitution = 0.9;
            self.physics.add_body(body);
        }
        
        log::info!("Physics 2D game initialized with {} entities", self.world.entity_count());
    }
    
    pub fn run(&mut self) {
        // Main game loop - in real app, use winit event loop
        log::info!("Starting Physics 2D game...");
        log::info!("Controls: WASD/Arrows to move, Space to jump, ESC to quit");
        
        // Simulate a few frames for demo
        for frame in 0..300 {
            self.input.begin_frame();
            
            // Simulate input
            if frame < 60 {
                self.input.key_pressed(Key::D);
            } else if frame < 120 {
                self.input.key_pressed(Key::Space);
            }
            
            let tick = self.game_loop.tick();
            
            for _ in 0..tick.fixed_updates {
                self.fixed_update(tick.fixed_dt as f32);
            }
            
            self.render(tick.interpolation as f32);
            
            if frame % 60 == 0 {
                log::info!("Frame {}: {} entities, physics bodies: {}", 
                    frame, self.world.entity_count(), self.physics.body_count());
            }
        }
        
        log::info!("Physics 2D demo complete!");
    }
    
    fn fixed_update(&mut self, dt: f32) {
        // Player input
        self.handle_input(dt);
        
        // Physics step (uses ASM for collision/integration)
        self.physics.step();
        
        // Sync physics to ECS
        self.sync_physics_to_ecs();
    }
    
    fn handle_input(&mut self, dt: f32) {
        if let Some(player_id) = self.player_id {
            if let Some(mut vel) = self.world.get_mut::<Velocity>(player_id) {
                // Horizontal movement
                if self.input.is_key_down(Key::A) || self.input.is_key_down(Key::Left) {
                    vel.linear.x = -PLAYER_SPEED;
                } else if self.input.is_key_down(Key::D) || self.input.is_key_down(Key::Right) {
                    vel.linear.x = PLAYER_SPEED;
                } else {
                    vel.linear.x *= 0.9; // Friction
                }
                
                // Jump
                if self.input.is_key_pressed(Key::Space) {
                    vel.linear.y = JUMP_FORCE;
                }
                
                // Gravity
                vel.linear.y += GRAVITY * dt;
            }
        }
    }
    
    fn sync_physics_to_ecs(&mut self) {
        // In a real implementation, sync physics body positions to ECS transforms
        // This is where ASM-accelerated physics results flow back to game logic
    }
    
    fn render(&mut self, _interpolation: f32) {
        self.renderer.clear(colors::BLACK);
        
        // Render all sprites
        for (_, (transform, sprite)) in self.world.query::<(&Transform, &Sprite)>().iter() {
            let x = transform.position.x as i32 - sprite.width as i32 / 2;
            let y = transform.position.y as i32 - sprite.height as i32 / 2;
            self.renderer.fill_rect(x, y, sprite.width, sprite.height, sprite.color);
        }
    }
}
