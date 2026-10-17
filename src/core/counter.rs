use image::{Rgba, RgbaImage};

/// RGBA Color representation.
pub type ColorRgba = [u8; 4];

/// A numbered step badge (e.g., ①, ②, ③).
#[derive(Debug, Clone)]
pub struct StepBadge {
    pub x: u32,
    pub y: u32,
    pub number: usize,
    pub radius: u32,
    pub bg_color: ColorRgba,
    pub text_color: ColorRgba,
}

impl StepBadge {
    pub fn new(x: u32, y: u32, number: usize) -> Self {
        Self {
            x,
            y,
            number,
            radius: 14,
            bg_color: [235, 47, 47, 255], // Vibrant Red (Flameshot style)
            text_color: [255, 255, 255, 255],
        }
    }

    /// Renders the circular badge with step number onto the target image.
    pub fn draw(&self, image: &mut RgbaImage) {
        let r = self.radius as i32;
        let cx = self.x as i32;
        let cy = self.y as i32;
        let img_w = image.width() as i32;
        let img_h = image.height() as i32;

        let bg = Rgba(self.bg_color);
        let border = Rgba([255, 255, 255, 230]);

        // Draw antialiased circle and border
        for dy in -r..=r {
            for dx in -r..=r {
                let px = cx + dx;
                let py = cy + dy;

                if px >= 0 && px < img_w && py >= 0 && py < img_h {
                    let dist_sq = dx * dx + dy * dy;
                    let r_sq = r * r;
                    let inner_r_sq = (r - 2) * (r - 2);

                    if dist_sq <= inner_r_sq {
                        image.put_pixel(px as u32, py as u32, bg);
                    } else if dist_sq <= r_sq {
                        image.put_pixel(px as u32, py as u32, border);
                    }
                }
            }
        }

        // Draw numeric digit using simple 3x5 font matrix
        draw_digit_3x5(image, cx, cy, self.number, Rgba(self.text_color));
    }
}

/// Manages the step sequence state (1, 2, 3...)
#[derive(Debug, Default)]
pub struct CounterManager {
    badges: Vec<StepBadge>,
    current_number: usize,
}

impl CounterManager {
    pub fn new() -> Self {
        Self {
            badges: Vec::new(),
            current_number: 1,
        }
    }

    /// Adds a new step badge at the clicked position and increments counter.
    pub fn add_step(&mut self, x: u32, y: u32) -> &StepBadge {
        let badge = StepBadge::new(x, y, self.current_number);
        self.current_number += 1;
        self.badges.push(badge);
        self.badges.last().unwrap()
    }

    /// Undoes the last step badge.
    #[allow(dead_code)]
    pub fn undo(&mut self) -> Option<StepBadge> {
        let removed = self.badges.pop();
        if removed.is_some() && self.current_number > 1 {
            self.current_number -= 1;
        }
        removed
    }

    /// Resets the counter back to 1.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.badges.clear();
        self.current_number = 1;
    }

    #[allow(dead_code)]
    pub fn badges(&self) -> &[StepBadge] {
        &self.badges
    }

    /// Renders all registered badges onto the canvas.
    pub fn render_all(&self, image: &mut RgbaImage) {
        for badge in &self.badges {
            badge.draw(image);
        }
    }
}

/// Simple 3x5 pixel font for digits 0-9
fn draw_digit_3x5(image: &mut RgbaImage, cx: i32, cy: i32, number: usize, color: Rgba<u8>) {
    // Digits 0-9 represented as 5 rows of 3-bit binary masks
    const DIGIT_GLYPHS: [[u8; 5]; 10] = [
        [0b111, 0b101, 0b101, 0b101, 0b111], // 0
        [0b010, 0b110, 0b010, 0b010, 0b111], // 1
        [0b111, 0b001, 0b111, 0b100, 0b111], // 2
        [0b111, 0b001, 0b111, 0b001, 0b111], // 3
        [0b101, 0b101, 0b111, 0b001, 0b001], // 4
        [0b111, 0b100, 0b111, 0b001, 0b111], // 5
        [0b111, 0b100, 0b111, 0b101, 0b111], // 6
        [0b111, 0b001, 0b010, 0b010, 0b010], // 7
        [0b111, 0b101, 0b111, 0b101, 0b111], // 8
        [0b111, 0b101, 0b111, 0b001, 0b111], // 9
    ];

    let digits_str = number.to_string();
    let num_digits = digits_str.len() as i32;
    let total_width = num_digits * 4 - 1; // 3 width + 1 spacing
    let start_x = cx - total_width / 2;
    let start_y = cy - 2;

    let img_w = image.width() as i32;
    let img_h = image.height() as i32;

    for (i, ch) in digits_str.chars().enumerate() {
        if let Some(d) = ch.to_digit(10) {
            let glyph = DIGIT_GLYPHS[d as usize];
            let ox = start_x + (i as i32) * 4;

            for (row_idx, row_bits) in glyph.iter().enumerate() {
                for col_idx in 0..3 {
                    if (row_bits & (1 << (2 - col_idx))) != 0 {
                        let px = ox + col_idx;
                        let py = start_y + (row_idx as i32);
                        if px >= 0 && px < img_w && py >= 0 && py < img_h {
                            image.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
    }
}
