use image::{Rgba, RgbaImage};
use log::debug;

/// Result of stitching a newly captured scroll frame.
#[derive(Debug, PartialEq, Eq)]
pub enum StitchResult {
    /// Successfully stitched a new frame, extended canvas by `added_height` pixels.
    Appended { added_height: u32, total_height: u32 },
    /// No scroll detected or reached the bottom of the page.
    EndOfPage,
    /// Frame could not be matched with sufficient confidence.
    MatchFailed,
}

/// Continuous vertical scrolling screenshot stitcher.
///
/// Uses template matching along the vertical axis to find the seam
/// where consecutive frames overlap, expanding the canvas downwards seamlessly.
pub struct ScrollStitcher {
    composite: RgbaImage,
    min_overlap: u32,
    max_overlap: u32,
    threshold: f64,
}

impl ScrollStitcher {
    /// Initializes a new stitcher starting with the initial viewport frame.
    pub fn new(initial_frame: RgbaImage) -> Self {
        let h = initial_frame.height();
        Self {
            composite: initial_frame,
            min_overlap: 40,
            max_overlap: (h * 4) / 5, // Up to 80% overlap
            threshold: 0.15,          // 15% color divergence tolerance
        }
    }

    /// Access the current stitched composite image.
    pub fn composite(&self) -> &RgbaImage {
        &self.composite
    }

    /// Consume the stitcher and retrieve the final full-page image.
    pub fn finish(self) -> RgbaImage {
        self.composite
    }

    /// Attempts to find the overlap and stitch `next_frame` to the bottom of `self.composite`.
    pub fn append_frame(&mut self, next_frame: &RgbaImage) -> StitchResult {
        if next_frame.width() != self.composite.width() {
            return StitchResult::MatchFailed;
        }

        let width = self.composite.width();
        let comp_h = self.composite.height();
        let next_h = next_frame.height();

        let search_min = self.min_overlap.min(comp_h).min(next_h);
        let search_max = self.max_overlap.min(comp_h).min(next_h);

        let mut best_overlap = 0;
        let mut min_error = f64::MAX;

        // Sample horizontal columns for fast matching
        let step_x = (width / 32).max(1);

        for overlap in (search_min..=search_max).step_by(2) {
            let mut diff_sum: u64 = 0;
            let mut samples: u64 = 0;

            let comp_start_y = comp_h - overlap;
            let step_y = (overlap / 16).max(1);

            for y_offset in (0..overlap).step_by(step_y as usize) {
                let cy = comp_start_y + y_offset;
                let ny = y_offset;

                for x in (0..width).step_by(step_x as usize) {
                    let p1 = self.composite.get_pixel(x, cy);
                    let p2 = next_frame.get_pixel(x, ny);

                    let dr = (p1[0] as i32 - p2[0] as i32).abs() as u64;
                    let dg = (p1[1] as i32 - p2[1] as i32).abs() as u64;
                    let db = (p1[2] as i32 - p2[2] as i32).abs() as u64;

                    diff_sum += dr + dg + db;
                    samples += 3 * 255;
                }
            }

            let normalized_error = if samples > 0 {
                diff_sum as f64 / samples as f64
            } else {
                1.0
            };

            if normalized_error < min_error {
                min_error = normalized_error;
                best_overlap = overlap;
            }
        }

        debug!("Stitch search: best_overlap={}, min_error={:.4}", best_overlap, min_error);

        // If error is high, matching failed
        if min_error > self.threshold {
            return StitchResult::MatchFailed;
        }

        // If best overlap covers the entire frame, no scrolling happened (end of page)
        if best_overlap >= next_h - 4 {
            return StitchResult::EndOfPage;
        }

        let added_height = next_h - best_overlap;
        let new_total_h = comp_h + added_height;

        // Create new expanded image canvas
        let mut expanded = RgbaImage::new(width, new_total_h);

        // Copy existing composite
        for y in 0..comp_h {
            for x in 0..width {
                expanded.put_pixel(x, y, *self.composite.get_pixel(x, y));
            }
        }

        // Append non-overlapping part of next_frame
        for y in best_overlap..next_h {
            let target_y = comp_h + (y - best_overlap);
            for x in 0..width {
                expanded.put_pixel(x, target_y, *next_frame.get_pixel(x, y));
            }
        }

        self.composite = expanded;

        StitchResult::Appended {
            added_height,
            total_height: new_total_h,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_stitching_synthetic() {
        let width = 100;
        let height = 80;

        // Create frame 1: gradient from y=0 to 80
        let mut frame1 = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                frame1.put_pixel(x, y, Rgba([x as u8, y as u8, 100, 255]));
            }
        }

        // Create frame 2: scrolled by 30 pixels (so y goes from 30 to 110)
        let scroll_offset = 30;
        let mut frame2 = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let actual_y = (y + scroll_offset) as u8;
                frame2.put_pixel(x, y, Rgba([x as u8, actual_y, 100, 255]));
            }
        }

        let mut stitcher = ScrollStitcher::new(frame1);
        let result = stitcher.append_frame(&frame2);

        match result {
            StitchResult::Appended { added_height, total_height } => {
                assert_eq!(added_height, scroll_offset);
                assert_eq!(total_height, height + scroll_offset);
            }
            _ => panic!("Expected successful stitch, got {:?}", result),
        }
    }
}
