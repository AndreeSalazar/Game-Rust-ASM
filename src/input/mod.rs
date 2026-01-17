//! Input handling module
//! 
//! Rust handles all input logic. No ASM needed here.

use std::collections::HashSet;

/// Keyboard key codes (subset)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Key {
    W, A, S, D,
    Up, Down, Left, Right,
    Space, Enter, Escape,
    Shift, Ctrl, Alt,
    Q, E, R, F,
    Num1, Num2, Num3, Num4, Num5,
    Unknown,
}

/// Mouse button
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Input state for current frame
#[derive(Clone, Debug, Default)]
pub struct InputState {
    keys_down: HashSet<Key>,
    keys_pressed: HashSet<Key>,
    keys_released: HashSet<Key>,
    mouse_buttons: HashSet<MouseButton>,
    mouse_position: (f32, f32),
    mouse_delta: (f32, f32),
    scroll_delta: f32,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Call at start of frame to clear per-frame state
    pub fn begin_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = 0.0;
    }
    
    /// Key pressed this frame
    pub fn key_pressed(&mut self, key: Key) {
        if !self.keys_down.contains(&key) {
            self.keys_pressed.insert(key);
        }
        self.keys_down.insert(key);
    }
    
    /// Key released this frame
    pub fn key_released(&mut self, key: Key) {
        self.keys_down.remove(&key);
        self.keys_released.insert(key);
    }
    
    /// Mouse button pressed
    pub fn mouse_pressed(&mut self, button: MouseButton) {
        self.mouse_buttons.insert(button);
    }
    
    /// Mouse button released
    pub fn mouse_released(&mut self, button: MouseButton) {
        self.mouse_buttons.remove(&button);
    }
    
    /// Mouse moved
    pub fn mouse_moved(&mut self, x: f32, y: f32) {
        self.mouse_delta.0 += x - self.mouse_position.0;
        self.mouse_delta.1 += y - self.mouse_position.1;
        self.mouse_position = (x, y);
    }
    
    /// Mouse scrolled
    pub fn mouse_scrolled(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }
    
    // Query methods
    
    /// Is key currently held down?
    pub fn is_key_down(&self, key: Key) -> bool {
        self.keys_down.contains(&key)
    }
    
    /// Was key just pressed this frame?
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.contains(&key)
    }
    
    /// Was key just released this frame?
    pub fn is_key_released(&self, key: Key) -> bool {
        self.keys_released.contains(&key)
    }
    
    /// Is mouse button down?
    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }
    
    /// Get mouse position
    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }
    
    /// Get mouse delta this frame
    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }
    
    /// Get scroll delta this frame
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }
    
    /// Get horizontal axis (-1, 0, 1) from WASD/Arrows
    pub fn horizontal_axis(&self) -> f32 {
        let mut axis = 0.0;
        if self.is_key_down(Key::A) || self.is_key_down(Key::Left) {
            axis -= 1.0;
        }
        if self.is_key_down(Key::D) || self.is_key_down(Key::Right) {
            axis += 1.0;
        }
        axis
    }
    
    /// Get vertical axis (-1, 0, 1) from WASD/Arrows
    pub fn vertical_axis(&self) -> f32 {
        let mut axis = 0.0;
        if self.is_key_down(Key::W) || self.is_key_down(Key::Up) {
            axis -= 1.0;
        }
        if self.is_key_down(Key::S) || self.is_key_down(Key::Down) {
            axis += 1.0;
        }
        axis
    }
}
