//! Rendering module
//! 
//! Rust: Renderer API, draw commands, sprite batching
//! ASM: Raycast inner loop (for raycaster game)

pub mod software;
pub mod raycast;

use crate::math::Vec2;

/// Color in ARGB format
pub type Color = u32;

/// Color constants
pub mod colors {
    use super::Color;
    pub const BLACK: Color = 0xFF000000;
    pub const WHITE: Color = 0xFFFFFFFF;
    pub const RED: Color = 0xFFFF0000;
    pub const GREEN: Color = 0xFF00FF00;
    pub const BLUE: Color = 0xFF0000FF;
    pub const YELLOW: Color = 0xFFFFFF00;
    pub const CYAN: Color = 0xFF00FFFF;
    pub const MAGENTA: Color = 0xFFFF00FF;
}

/// Software renderer
pub struct Renderer {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u32>,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; (width * height) as usize],
        }
    }
    
    /// Clear the buffer
    pub fn clear(&mut self, color: Color) {
        self.buffer.fill(color);
    }
    
    /// Set a pixel
    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.buffer[(y as u32 * self.width + x as u32) as usize] = color;
        }
    }
    
    /// Get a pixel
    #[inline]
    pub fn get_pixel(&self, x: i32, y: i32) -> Color {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.buffer[(y as u32 * self.width + x as u32) as usize]
        } else {
            0
        }
    }
    
    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        let x0 = x.max(0) as u32;
        let y0 = y.max(0) as u32;
        let x1 = ((x + w as i32) as u32).min(self.width);
        let y1 = ((y + h as i32) as u32).min(self.height);
        
        for py in y0..y1 {
            for px in x0..x1 {
                self.buffer[(py * self.width + px) as usize] = color;
            }
        }
    }
    
    /// Draw a circle
    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let r2 = radius * radius;
        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= r2 {
                    self.set_pixel(cx + x, cy + y, color);
                }
            }
        }
    }
    
    /// Draw a line (Bresenham)
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        
        loop {
            self.set_pixel(x, y, color);
            if x == x1 && y == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
    
    /// Draw text (simple 8x8 font placeholder)
    pub fn draw_text(&mut self, _text: &str, _x: i32, _y: i32, _color: Color) {
        // TODO: Implement bitmap font rendering
    }
    
    /// Get buffer for display
    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }
    
    /// Get mutable buffer
    pub fn buffer_mut(&mut self) -> &mut [u32] {
        &mut self.buffer
    }
}

/// Draw command for batching
#[derive(Clone, Debug)]
pub enum DrawCommand {
    Clear(Color),
    Pixel { x: i32, y: i32, color: Color },
    Rect { x: i32, y: i32, w: u32, h: u32, color: Color },
    Circle { x: i32, y: i32, radius: i32, color: Color },
    Line { x0: i32, y0: i32, x1: i32, y1: i32, color: Color },
}

/// Command buffer for deferred rendering
pub struct CommandBuffer {
    commands: Vec<DrawCommand>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }
    
    pub fn clear(&mut self, color: Color) {
        self.commands.push(DrawCommand::Clear(color));
    }
    
    pub fn rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        self.commands.push(DrawCommand::Rect { x, y, w, h, color });
    }
    
    pub fn circle(&mut self, x: i32, y: i32, radius: i32, color: Color) {
        self.commands.push(DrawCommand::Circle { x, y, radius, color });
    }
    
    pub fn execute(&self, renderer: &mut Renderer) {
        for cmd in &self.commands {
            match cmd {
                DrawCommand::Clear(c) => renderer.clear(*c),
                DrawCommand::Pixel { x, y, color } => renderer.set_pixel(*x, *y, *color),
                DrawCommand::Rect { x, y, w, h, color } => renderer.fill_rect(*x, *y, *w, *h, *color),
                DrawCommand::Circle { x, y, radius, color } => renderer.fill_circle(*x, *y, *radius, *color),
                DrawCommand::Line { x0, y0, x1, y1, color } => renderer.draw_line(*x0, *y0, *x1, *y1, *color),
            }
        }
    }
    
    pub fn reset(&mut self) {
        self.commands.clear();
    }
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}
