use image::{Rgba, RgbaImage};

/// Simple 2D bounding rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Clamps the rectangle dimensions to stay strictly inside the image bounds.
    pub fn clamp_to_image(&self, img_w: u32, img_h: u32) -> Option<Rect> {
        if self.x >= img_w || self.y >= img_h || self.width == 0 || self.height == 0 {
            return None;
        }

        let clamped_w = self.width.min(img_w - self.x);
        let clamped_h = self.height.min(img_h - self.y);

        Some(Rect {
            x: self.x,
            y: self.y,
            width: clamped_w,
            height: clamped_h,
        })
    }

    /// Creates a Rect from any two diagonal points (x0, y0) and (x1, y1).
    pub fn from_points(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        let left = x0.min(x1).max(0) as u32;
        let top = y0.min(y1).max(0) as u32;
        let right = x0.max(x1).max(0) as u32;
        let bottom = y0.max(y1).max(0) as u32;

        Self {
            x: left,
            y: top,
            width: right.saturating_sub(left).max(1),
            height: bottom.saturating_sub(top).max(1),
        }
    }

    /// Checks if a 2D point lies inside this rectangle.
    pub fn contains(&self, px: i32, py: i32) -> bool {
        if px < 0 || py < 0 {
            return false;
        }
        let px = px as u32;
        let py = py as u32;
        px >= self.x && px < (self.x + self.width) && py >= self.y && py < (self.y + self.height)
    }
}

/// Applies a mosaic pixelation effect over the target rectangle.
///
/// Ideal for instantly censoring sensitive information (passwords, credit cards, emails).
pub fn pixelate(image: &mut RgbaImage, rect: Rect, block_size: u32) {
    let block_size = block_size.max(2);
    let clamped = match rect.clamp_to_image(image.width(), image.height()) {
        Some(r) => r,
        None => return,
    };

    let x_end = clamped.x + clamped.width;
    let y_end = clamped.y + clamped.height;

    let mut by = clamped.y;
    while by < y_end {
        let bh = block_size.min(y_end - by);

        let mut bx = clamped.x;
        while bx < x_end {
            let bw = block_size.min(x_end - bx);

            // Compute average color within the block
            let mut r_sum: u64 = 0;
            let mut g_sum: u64 = 0;
            let mut b_sum: u64 = 0;
            let mut a_sum: u64 = 0;
            let total_pixels = (bw * bh) as u64;

            for py in by..(by + bh) {
                for px in bx..(bx + bw) {
                    let pixel = image.get_pixel(px, py);
                    r_sum += pixel[0] as u64;
                    g_sum += pixel[1] as u64;
                    b_sum += pixel[2] as u64;
                    a_sum += pixel[3] as u64;
                }
            }

            let avg_color = Rgba([
                (r_sum / total_pixels) as u8,
                (g_sum / total_pixels) as u8,
                (b_sum / total_pixels) as u8,
                (a_sum / total_pixels) as u8,
            ]);

            // Fill block with average color
            for py in by..(by + bh) {
                for px in bx..(bx + bw) {
                    image.put_pixel(px, py, avg_color);
                }
            }

            bx += bw;
        }
        by += bh;
    }
}

/// Applies a fast box blur over the target rectangle.
pub fn blur(image: &mut RgbaImage, rect: Rect, radius: u32) {
    if radius == 0 {
        return;
    }
    let clamped = match rect.clamp_to_image(image.width(), image.height()) {
        Some(r) => r,
        None => return,
    };

    let rad = radius as i32;
    let x_start = clamped.x;
    let x_end = clamped.x + clamped.width;
    let y_start = clamped.y;
    let y_end = clamped.y + clamped.height;

    // Temporary buffer for the region
    let mut temp = image.clone();

    // Horizontal pass
    for y in y_start..y_end {
        for x in x_start..x_end {
            let mut r: u64 = 0;
            let mut g: u64 = 0;
            let mut b: u64 = 0;
            let mut a: u64 = 0;
            let mut count: u64 = 0;

            for k in -rad..=rad {
                let px = (x as i32 + k).clamp(x_start as i32, (x_end - 1) as i32) as u32;
                let pixel = image.get_pixel(px, y);
                r += pixel[0] as u64;
                g += pixel[1] as u64;
                b += pixel[2] as u64;
                a += pixel[3] as u64;
                count += 1;
            }

            temp.put_pixel(
                x,
                y,
                Rgba([
                    (r / count) as u8,
                    (g / count) as u8,
                    (b / count) as u8,
                    (a / count) as u8,
                ]),
            );
        }
    }

    // Vertical pass back to main image
    for y in y_start..y_end {
        for x in x_start..x_end {
            let mut r: u64 = 0;
            let mut g: u64 = 0;
            let mut b: u64 = 0;
            let mut a: u64 = 0;
            let mut count: u64 = 0;

            for k in -rad..=rad {
                let py = (y as i32 + k).clamp(y_start as i32, (y_end - 1) as i32) as u32;
                let pixel = temp.get_pixel(x, py);
                r += pixel[0] as u64;
                g += pixel[1] as u64;
                b += pixel[2] as u64;
                a += pixel[3] as u64;
                count += 1;
            }

            image.put_pixel(
                x,
                y,
                Rgba([
                    (r / count) as u8,
                    (g / count) as u8,
                    (b / count) as u8,
                    (a / count) as u8,
                ]),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixelate_in_bounds() {
        let mut img = RgbaImage::new(100, 100);
        for y in 0..100 {
            for x in 0..100 {
                img.put_pixel(x, y, Rgba([x as u8, y as u8, 128, 255]));
            }
        }

        pixelate(&mut img, Rect::new(10, 10, 20, 20), 5);
        // Ensure pixels inside the block are uniform
        let sample = *img.get_pixel(11, 11);
        assert_eq!(sample, *img.get_pixel(12, 12));
    }

    #[test]
    fn test_blur_out_of_bounds() {
        let mut img = RgbaImage::new(50, 50);
        // Applying blur with rect out of bounds should gracefully no-op
        blur(&mut img, Rect::new(60, 60, 20, 20), 3);
    }
}
