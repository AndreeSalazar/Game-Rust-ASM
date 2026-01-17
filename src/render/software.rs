//! Software rendering utilities

use super::{Color, Renderer};

/// Blend two colors with alpha
#[inline]
pub fn blend_alpha(dst: Color, src: Color) -> Color {
    let sa = (src >> 24) & 0xFF;
    if sa == 0 { return dst; }
    if sa == 255 { return src; }
    
    let da = 255 - sa;
    
    let sr = (src >> 16) & 0xFF;
    let sg = (src >> 8) & 0xFF;
    let sb = src & 0xFF;
    
    let dr = (dst >> 16) & 0xFF;
    let dg = (dst >> 8) & 0xFF;
    let db = dst & 0xFF;
    
    let r = (sr * sa + dr * da) / 255;
    let g = (sg * sa + dg * da) / 255;
    let b = (sb * sa + db * da) / 255;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}

/// Create color from RGB
#[inline]
pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Create color from RGBA
#[inline]
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Extract RGB components
#[inline]
pub const fn to_rgb(color: Color) -> (u8, u8, u8) {
    (
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8,
    )
}

/// Lerp between two colors
pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let (ar, ag, ab) = to_rgb(a);
    let (br, bg, bb) = to_rgb(b);
    
    let r = (ar as f32 + (br as f32 - ar as f32) * t) as u8;
    let g = (ag as f32 + (bg as f32 - ag as f32) * t) as u8;
    let b = (ab as f32 + (bb as f32 - ab as f32) * t) as u8;
    
    rgb(r, g, b)
}

/// Horizontal line (optimized)
pub fn hline(renderer: &mut Renderer, x0: i32, x1: i32, y: i32, color: Color) {
    if y < 0 || y >= renderer.height as i32 { return; }
    
    let start = x0.max(0) as usize;
    let end = (x1.min(renderer.width as i32 - 1) + 1) as usize;
    let y_offset = y as usize * renderer.width as usize;
    
    renderer.buffer[y_offset + start..y_offset + end].fill(color);
}

/// Vertical line (optimized)
pub fn vline(renderer: &mut Renderer, x: i32, y0: i32, y1: i32, color: Color) {
    if x < 0 || x >= renderer.width as i32 { return; }
    
    let start = y0.max(0);
    let end = y1.min(renderer.height as i32 - 1) + 1;
    
    for y in start..end {
        renderer.buffer[(y as u32 * renderer.width + x as u32) as usize] = color;
    }
}
