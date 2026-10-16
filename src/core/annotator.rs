use image::{Rgba, RgbaImage};
use crate::core::filters::Rect;

pub type ColorRgba = [u8; 4];

/// Draws an antialiased arrow from (x0, y0) to (x1, y1) with a distinct arrowhead.
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

    if length < 5.0 {
        return;
    }

    let angle = dy.atan2(dx);
    let head_length = (length * 0.25).clamp(14.0, 32.0);
    let head_angle = std::f64::consts::PI / 6.0; // 30 degrees

    // Shaft end is slightly retracted to join arrowhead cleanly
    let shaft_end_x = x1 - (angle.cos() * (head_length * 0.7)) as i32;
    let shaft_end_y = y1 - (angle.sin() * (head_length * 0.7)) as i32;

    draw_line_thick(image, x0, y0, shaft_end_x, shaft_end_y, rgba, thickness);

    // Left and right wings of the arrowhead
    let left_angle = angle + std::f64::consts::PI - head_angle;
    let right_angle = angle + std::f64::consts::PI + head_angle;

    let left_x = x1 + (left_angle.cos() * head_length) as i32;
    let left_y = y1 + (left_angle.sin() * head_length) as i32;

    let right_x = x1 + (right_angle.cos() * head_length) as i32;
    let right_y = y1 + (right_angle.sin() * head_length) as i32;

    draw_line_thick(image, x1, y1, left_x, left_y, rgba, thickness);
    draw_line_thick(image, x1, y1, right_x, right_y, rgba, thickness);
}

/// Draws an outline or filled rectangle.
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
