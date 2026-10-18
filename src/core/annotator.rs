use image::{Rgba, RgbaImage};
use crate::core::filters::Rect;

pub type ColorRgba = [u8; 4];

/// Draws an antialiased arrow from (x0, y0) to (x1, y1) with a distinct, solid filled arrowhead.
pub fn draw_arrow(
    image: &mut RgbaImage,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: ColorRgba,
    thickness: i32,
) {
    let rgba = Rgba(color);
    let dx = (x1 - x0) as f64;
    let dy = (y1 - y0) as f64;
    let length = (dx * dx + dy * dy).sqrt();

    if length < 6.0 {
        return;
    }

    let angle = dy.atan2(dx);
    let t = thickness.max(1) as f64;
    let head_len = (t * 3.5 + 14.0).clamp(16.0, 38.0).min(length * 0.65);
    let head_half_width = head_len * 0.42;

    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let base_cx = x1 as f64 - cos_a * head_len;
    let base_cy = y1 as f64 - sin_a * head_len;

    let w_left_x = (base_cx - sin_a * head_half_width).round() as i32;
    let w_left_y = (base_cy + cos_a * head_half_width).round() as i32;
    let w_right_x = (base_cx + sin_a * head_half_width).round() as i32;
    let w_right_y = (base_cy - cos_a * head_half_width).round() as i32;

    let shaft_end_x = (base_cx + cos_a * (head_len * 0.25)).round() as i32;
    let shaft_end_y = (base_cy + sin_a * (head_len * 0.25)).round() as i32;
    draw_line_thick(image, x0, y0, shaft_end_x, shaft_end_y, rgba, thickness);

    // Solid filled arrowhead
    fill_triangle_rgba(image, x1, y1, w_left_x, w_left_y, w_right_x, w_right_y, rgba);

    // Clean edge strokes
    let edge_thick = (thickness / 2).clamp(1, 3);
    draw_line_thick(image, x1, y1, w_left_x, w_left_y, rgba, edge_thick);
    draw_line_thick(image, x1, y1, w_right_x, w_right_y, rgba, edge_thick);
    draw_line_thick(image, w_left_x, w_left_y, w_right_x, w_right_y, rgba, edge_thick);
}

fn fill_triangle_rgba(
    image: &mut RgbaImage,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    color: Rgba<u8>,
) {
    let w = image.width() as i32;
    let h = image.height() as i32;

    let min_y = y0.min(y1).min(y2).max(0);
    let max_y = y0.max(y1).max(y2).min(h - 1);
    if min_y > max_y {
        return;
    }

    let edges = [(x0, y0, x1, y1), (x1, y1, x2, y2), (x2, y2, x0, y0)];

    for y in min_y..=max_y {
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;

        for &(ax, ay, bx, by) in &edges {
            if (ay <= y && y < by) || (by <= y && y < ay) {
                let dy = by - ay;
                if dy != 0 {
                    let x = ax + (y - ay) * (bx - ax) / dy;
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                }
            } else if ay == by && ay == y {
                min_x = min_x.min(ax.min(bx));
                max_x = max_x.max(ax.max(bx));
            }
        }

        if min_x <= max_x {
            let start_x = min_x.clamp(0, w - 1);
            let end_x = max_x.clamp(0, w - 1);
            for x in start_x..=end_x {
                image.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

/// Draws an outline or filled rectangle.
#[allow(dead_code)]
pub fn draw_rectangle(
    image: &mut RgbaImage,
    rect: Rect,
    color: ColorRgba,
    thickness: u32,
) {
    let rgba = Rgba(color);
    let t = thickness.max(1);

    // Top & Bottom
    for h in 0..t {
        for x in rect.x..(rect.x + rect.width) {
            safe_put_pixel(image, x, rect.y + h, rgba);
            safe_put_pixel(image, x, rect.y + rect.height.saturating_sub(1 + h), rgba);
        }
    }

    // Left & Right
    for w in 0..t {
        for y in rect.y..(rect.y + rect.height) {
            safe_put_pixel(image, rect.x + w, y, rgba);
            safe_put_pixel(image, rect.x + rect.width.saturating_sub(1 + w), y, rgba);
        }
    }
}

/// Draws a semi-transparent fluorescent highlighter marker (e.g. yellow, green).
pub fn draw_highlighter(image: &mut RgbaImage, rect: Rect, highlight_color: ColorRgba) {
    let clamped = match rect.clamp_to_image(image.width(), image.height()) {
        Some(r) => r,
        None => return,
    };

    let alpha_f = (highlight_color[3] as f32) / 255.0 * 0.45; // Subtle 45% transparency
    let inv_alpha = 1.0 - alpha_f;

    for y in clamped.y..(clamped.y + clamped.height) {
        for x in clamped.x..(clamped.x + clamped.width) {
            let orig = image.get_pixel(x, y);

            let r = (orig[0] as f32 * inv_alpha + highlight_color[0] as f32 * alpha_f).round() as u8;
            let g = (orig[1] as f32 * inv_alpha + highlight_color[1] as f32 * alpha_f).round() as u8;
            let b = (orig[2] as f32 * inv_alpha + highlight_color[2] as f32 * alpha_f).round() as u8;

            image.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
}

/// Draws a thick antialiased line between two points.
#[allow(dead_code)]
pub fn draw_line(
    image: &mut RgbaImage,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: ColorRgba,
    thickness: i32,
) {
    draw_line_thick(image, x0, y0, x1, y1, Rgba(color), thickness);
}

/// Bresenham thick line algorithm.
pub fn draw_line_thick(
    image: &mut RgbaImage,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: Rgba<u8>,
    thickness: i32,
) {
    let r = (thickness / 2).max(1);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        // Draw brush disk at (x0, y0)
        for dy_brush in -r..=r {
            for dx_brush in -r..=r {
                if dx_brush * dx_brush + dy_brush * dy_brush <= r * r {
                    let px = x0 + dx_brush;
                    let py = y0 + dy_brush;
                    if px >= 0 && py >= 0 {
                        safe_put_pixel(image, px as u32, py as u32, color);
                    }
                }
            }
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn safe_put_pixel(image: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
    if x < image.width() && y < image.height() {
        image.put_pixel(x, y, color);
    }
}
