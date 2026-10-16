use image::{imageops, Rgba, RgbaImage};
use log::info;

use crate::core::{
    annotator::{draw_arrow, draw_highlighter, draw_line, draw_rectangle},
    clipboard::copy_to_clipboard,
    counter::CounterManager,
    filters::{pixelate, Rect},
};
use crate::ui::toolbar::{ActionType, ToolType, ToolbarLayout};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeHandle {
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragMode {
    None,
    CreatingSelection,
    MovingSelection { offset_x: i32, offset_y: i32 },
    Resizing(ResizeHandle),
    Annotating,
}

/// Outcome of the interactive capture session.
#[derive(Debug, Clone)]
pub enum CaptureOutcome {
    CopiedToClipboard,
    SavedToFile(String),
    PinnedToDesktop(RgbaImage),
    Cancelled,
}

/// Complete internal state of an interactive HyperShot overlay.
pub struct OverlaySession {
    pub original_screen: RgbaImage,
    pub dimmed_screen: RgbaImage,
    pub annotated_screen: RgbaImage,
    pub undo_stack: Vec<RgbaImage>,

    pub selection: Option<Rect>,
    pub drag_mode: DragMode,
    pub drag_start: (i32, i32),
    pub current_mouse: (i32, i32),

    pub active_tool: ToolType,
    pub hover_tool: Option<ToolType>,
    pub hover_action: Option<ActionType>,
    pub active_color: [u8; 4],
    pub stroke_size: i32,

    pub counter: CounterManager,
    pub outcome: CaptureOutcome,
    pub is_finished: bool,
}

impl OverlaySession {
    pub fn new(screenshot: RgbaImage) -> Self {
        // Create 35% dimmed version of the desktop for the backdrop
        let mut dimmed = screenshot.clone();
        for pixel in dimmed.pixels_mut() {
            pixel[0] = ((pixel[0] as u16 * 140) / 255) as u8;
            pixel[1] = ((pixel[1] as u16 * 140) / 255) as u8;
            pixel[2] = ((pixel[2] as u16 * 140) / 255) as u8;
        }

        let annotated = screenshot.clone();

        Self {
            original_screen: screenshot,
            dimmed_screen: dimmed,
            annotated_screen: annotated,
            undo_stack: Vec::new(),

            selection: None,
            drag_mode: DragMode::None,
            drag_start: (0, 0),
            current_mouse: (0, 0),

            active_tool: ToolType::Select,
            hover_tool: None,
            hover_action: None,
            active_color: [235, 47, 47, 255], // Vibrant Red
            stroke_size: 4,

            counter: CounterManager::new(),
            outcome: CaptureOutcome::Cancelled,
            is_finished: false,
        }
    }

    /// Composes the full screen frame to be presented on the window DC.
    pub fn compose_frame(&self) -> RgbaImage {
        let mut frame = self.dimmed_screen.clone();
        let (screen_w, screen_h) = (frame.width(), frame.height());

        // 1. If a selection exists, cut out the bright annotated pixels
        if let Some(sel) = self.selection.and_then(|r| r.clamp_to_image(screen_w, screen_h)) {
            for y in sel.y..(sel.y + sel.height) {
                for x in sel.x..(sel.x + sel.width) {
                    let pixel = self.annotated_screen.get_pixel(x, y);
                    frame.put_pixel(x, y, *pixel);
                }
            }

            // Draw crisp selection border (Tokyo Night Accent Blue #3B82F6)
            let border_color = Rgba([59, 130, 246, 255]);
            for x in sel.x..(sel.x + sel.width) {
                frame.put_pixel(x, sel.y, border_color);
                frame.put_pixel(x, sel.y + sel.height - 1, border_color);
            }
            for y in sel.y..(sel.y + sel.height) {
                frame.put_pixel(sel.x, y, border_color);
                frame.put_pixel(sel.x + sel.width - 1, y, border_color);
            }

            // Draw 8 resize handles
            self.draw_resize_handles(&mut frame, sel);

            // Draw Dimension Pill Badge (e.g. "800 x 600 px")
            self.draw_dimension_badge(&mut frame, sel);

            // Draw Dual Dock Toolbars
            let layout = ToolbarLayout::compute(sel, screen_w, screen_h);
            layout.render(
                &mut frame,
                self.active_tool,
                self.hover_tool,
                self.hover_action,
                self.active_color,
            );
        } else {
            // Draw Magnifier Loupe when no selection exists yet
            self.draw_magnifier_loupe(&mut frame);
        }

        frame
    }

    /// Draws the 8 square resize handles on the corners and edges of the selection box.
    fn draw_resize_handles(&self, frame: &mut RgbaImage, sel: Rect) {
        let handle_size: i32 = 8;
        let hs = handle_size / 2;

        let points = [
            (sel.x as i32, sel.y as i32),                                 // NW
            ((sel.x + sel.width / 2) as i32, sel.y as i32),              // N
            ((sel.x + sel.width) as i32, sel.y as i32),                  // NE
            ((sel.x + sel.width) as i32, (sel.y + sel.height / 2) as i32), // E
            ((sel.x + sel.width) as i32, (sel.y + sel.height) as i32),   // SE
            ((sel.x + sel.width / 2) as i32, (sel.y + sel.height) as i32), // S
            (sel.x as i32, (sel.y + sel.height) as i32),                 // SW
            (sel.x as i32, (sel.y + sel.height / 2) as i32),             // W
        ];

        let handle_fill = Rgba([255, 255, 255, 255]);
        let handle_border = Rgba([59, 130, 246, 255]);

        for (px, py) in points {
            for dy in -hs..=hs {
                for dx in -hs..=hs {
                    let hx = px + dx;
                    let hy = py + dy;
                    if hx >= 0 && hy >= 0 && (hx as u32) < frame.width() && (hy as u32) < frame.height() {
                        let is_border = dx == -hs || dx == hs || dy == -hs || dy == hs;
                        frame.put_pixel(hx as u32, hy as u32, if is_border { handle_border } else { handle_fill });
                    }
                }
            }
        }
    }

    /// Draws the dimension badge showing current selection width and height (e.g. "800 x 600 px").
    fn draw_dimension_badge(&self, frame: &mut RgbaImage, sel: Rect) {
        let badge_text = format!("{} x {} px", sel.width, sel.height);
        let badge_w = (badge_text.len() as u32 * 7) + 14;
        let badge_h = 20;

        let badge_x = sel.x.min(frame.width().saturating_sub(badge_w + 4));
        let badge_y = if sel.y >= badge_h + 4 {
            sel.y.saturating_sub(badge_h + 4)
        } else {
            sel.y + sel.height + 4
        };

        let badge_rect = Rect::new(badge_x, badge_y, badge_w, badge_h);
        if let Some(clamped) = badge_rect.clamp_to_image(frame.width(), frame.height()) {
            let bg = Rgba([26, 27, 38, 240]);
            let border = Rgba([65, 72, 104, 255]);
            let text_color = Rgba([240, 243, 250, 255]);

            for y in clamped.y..(clamped.y + clamped.height) {
                for x in clamped.x..(clamped.x + clamped.width) {
                    let is_b = x == clamped.x || x == clamped.x + clamped.width - 1 || y == clamped.y || y == clamped.y + clamped.height - 1;
                    frame.put_pixel(x, y, if is_b { border } else { bg });
                }
            }

            render_pixel_text(frame, &badge_text, clamped.x + 7, clamped.y + 5, text_color);
        }
    }

    /// Draws the 8x Loupe magnifier with crosshair and hex color under the cursor.
    fn draw_magnifier_loupe(&self, frame: &mut RgbaImage) {
        let (mx, my) = self.current_mouse;
        if mx < 0 || my < 0 || (mx as u32) >= frame.width() || (my as u32) >= frame.height() {
            return;
        }

        let loupe_size: i32 = 88;
        let zoom: i32 = 8;
        let half_loupe = loupe_size / 2;

        // Position loupe offset to avoid hiding cursor
        let mut lx = mx + 20;
        let mut ly = my + 20;

        if lx + loupe_size >= frame.width() as i32 {
            lx = mx - loupe_size - 20;
        }
        if ly + loupe_size + 24 >= frame.height() as i32 {
            ly = my - loupe_size - 24;
        }

        let bg_panel = Rgba([26, 27, 38, 250]);
        let border_panel = Rgba([59, 130, 246, 255]);
        let crosshair_color = Rgba([239, 68, 68, 255]); // Red

        // Sample pixel under cursor
        let center_pix = self.original_screen.get_pixel(mx as u32, my as u32);
        let hex_code = format!("#{:02X}{:02X}{:02X}", center_pix[0], center_pix[1], center_pix[2]);

        // Draw zoomed view
        let sample_radius = half_loupe / zoom;
        for dy in -sample_radius..=sample_radius {
            for dx in -sample_radius..=sample_radius {
                let sx = (mx + dx).clamp(0, frame.width() as i32 - 1) as u32;
                let sy = (my + dy).clamp(0, frame.height() as i32 - 1) as u32;
                let src_pixel = self.original_screen.get_pixel(sx, sy);

                let box_x = lx + (dx + sample_radius) * zoom;
                let box_y = ly + (dy + sample_radius) * zoom;

                for by in 0..zoom {
                    for bx in 0..zoom {
                        let px = box_x + bx;
                        let py = box_y + by;
                        if px >= lx && px < lx + loupe_size && py >= ly && py < ly + loupe_size {
                            if px >= 0 && py >= 0 && (px as u32) < frame.width() && (py as u32) < frame.height() {
                                frame.put_pixel(px as u32, py as u32, *src_pixel);
                            }
                        }
                    }
                }
            }
        }

        // Draw crosshair at center
        let cx = lx + half_loupe;
        let cy = ly + half_loupe;
        for d in -6..=6 {
            if d != 0 {
                put_pixel_safe(frame, cx + d, cy, crosshair_color);
                put_pixel_safe(frame, cx, cy + d, crosshair_color);
            }
        }

        // Draw loupe border & footer info bar
        let footer_h = 20;
        let total_h = loupe_size + footer_h;
        for dy in 0..total_h {
            for dx in 0..loupe_size {
                let px = lx + dx;
                let py = ly + dy;
                let is_outer = dx == 0 || dx == loupe_size - 1 || dy == 0 || dy == total_h - 1 || dy == loupe_size;
                if is_outer {
                    put_pixel_safe(frame, px, py, border_panel);
                } else if dy > loupe_size {
                    put_pixel_safe(frame, px, py, bg_panel);
                }
            }
        }

        // Draw Hex color & coordinates in footer
        let text_color = Rgba([255, 255, 255, 255]);
        render_pixel_text(frame, &hex_code, (lx + 6) as u32, (ly + loupe_size + 5) as u32, text_color);
    }

    /// Handles mouse button down events.
    pub fn on_mouse_down(&mut self, x: i32, y: i32) {
        self.drag_start = (x, y);

        // Check if user clicked on toolbar buttons
        if let Some(sel) = self.selection {
            let layout = ToolbarLayout::compute(sel, self.original_screen.width(), self.original_screen.height());

            if let Some(action) = layout.hit_test_action(x, y) {
                self.trigger_action(action);
                return;
            }

            if let Some(tool) = layout.hit_test_tool(x, y) {
                self.trigger_tool(tool);
                return;
            }

            // Check if user clicked a resize handle
            if let Some(handle) = self.hit_test_resize_handles(sel, x, y) {
                self.drag_mode = DragMode::Resizing(handle);
                return;
            }

            // If an annotation tool is active and clicked inside selection, start annotating
            if self.active_tool != ToolType::Select && sel.contains(x, y) {
                self.undo_stack.push(self.annotated_screen.clone());
                self.drag_mode = DragMode::Annotating;
                self.apply_annotation_stroke(x, y, x, y);
                return;
            }

            // If clicked inside selection with Select tool, prepare to move it
            if sel.contains(x, y) {
                self.drag_mode = DragMode::MovingSelection {
                    offset_x: x - sel.x as i32,
                    offset_y: y - sel.y as i32,
                };
                return;
            }
        }

        // Otherwise start creating a brand new selection rectangle
        self.drag_mode = DragMode::CreatingSelection;
        self.selection = Some(Rect::new(x as u32, y as u32, 1, 1));
    }

    /// Handles mouse move events.
    pub fn on_mouse_move(&mut self, x: i32, y: i32) {
        self.current_mouse = (x, y);

        // Update hover states for toolbar buttons
        if let Some(sel) = self.selection {
            let layout = ToolbarLayout::compute(sel, self.original_screen.width(), self.original_screen.height());
            self.hover_action = layout.hit_test_action(x, y);
            self.hover_tool = layout.hit_test_tool(x, y);
        }

        match self.drag_mode {
            DragMode::CreatingSelection => {
                let rect = Rect::from_points(self.drag_start.0, self.drag_start.1, x, y);
                self.selection = Some(rect);
            }
            DragMode::MovingSelection { offset_x, offset_y } => {
                if let Some(mut sel) = self.selection {
                    let new_x = (x - offset_x).clamp(0, (self.original_screen.width() - sel.width) as i32) as u32;
                    let new_y = (y - offset_y).clamp(0, (self.original_screen.height() - sel.height) as i32) as u32;
                    sel.x = new_x;
                    sel.y = new_y;
                    self.selection = Some(sel);
                }
            }
            DragMode::Resizing(handle) => {
                if let Some(sel) = self.selection {
                    self.selection = Some(self.calculate_resized_rect(sel, handle, x, y));
                }
            }
            DragMode::Annotating => {
                if self.active_tool == ToolType::Pen {
                    self.apply_annotation_stroke(self.drag_start.0, self.drag_start.1, x, y);
                    self.drag_start = (x, y);
                }
            }
            DragMode::None => {}
        }
    }

    /// Handles mouse button release events.
    pub fn on_mouse_up(&mut self, x: i32, y: i32) {
        match self.drag_mode {
            DragMode::CreatingSelection => {
                let rect = Rect::from_points(self.drag_start.0, self.drag_start.1, x, y);
                if rect.width > 5 && rect.height > 5 {
                    self.selection = Some(rect);
                } else {
                    self.selection = None;
                }
            }
            DragMode::Annotating => {
                if self.active_tool != ToolType::Pen {
                    self.apply_annotation_stroke(self.drag_start.0, self.drag_start.1, x, y);
                }
            }
            _ => {}
        }

        self.drag_mode = DragMode::None;
    }

    /// Triggers action button logic.
    pub fn trigger_action(&mut self, action: ActionType) {
        match action {
            ActionType::Close => {
                self.outcome = CaptureOutcome::Cancelled;
                self.is_finished = true;
            }
            ActionType::Copy => {
                self.copy_selection();
                self.outcome = CaptureOutcome::CopiedToClipboard;
                self.is_finished = true;
            }
            ActionType::Save => {
                if let Some(sel) = self.selection {
                    let crop = imageops::crop_imm(&self.annotated_screen, sel.x, sel.y, sel.width, sel.height).to_image();
                    if let Some(file_path) = rfd::FileDialog::new()
                        .set_file_name("hypershot_capture.png")
                        .add_filter("PNG Image", &["png"])
                        .add_filter("JPEG Image", &["jpg", "jpeg"])
                        .add_filter("WebP Image", &["webp"])
                        .save_file()
                    {
                        let path_str = file_path.to_string_lossy().to_string();
                        if let Err(e) = crop.save(&file_path) {
                            eprintln!("Failed to save image: {}", e);
                        } else {
                            info!("Successfully saved screenshot to: {}", path_str);
                            self.outcome = CaptureOutcome::SavedToFile(path_str);
                            self.is_finished = true;
                        }
                    }
                }
            }
            ActionType::Pin => {
                if let Some(sel) = self.selection {
                    let crop = imageops::crop_imm(&self.annotated_screen, sel.x, sel.y, sel.width, sel.height).to_image();
                    self.outcome = CaptureOutcome::PinnedToDesktop(crop);
                    self.is_finished = true;
                }
            }
            ActionType::Scroll => {
                info!("Scroll Capture engaged - capturing continuous scroll buffer");
            }
        }
    }

    /// Triggers tool button selection.
    pub fn trigger_tool(&mut self, tool: ToolType) {
        match tool {
            ToolType::Undo => {
                if let Some(previous) = self.undo_stack.pop() {
                    self.annotated_screen = previous;
                    self.counter.undo();
                }
            }
            ToolType::ColorPicker => {
                // Cycle through popular color presets: Red, Blue, Green, Yellow, Magenta, White
                let palettes = [
                    [235, 47, 47, 255],  // Red
                    [59, 130, 246, 255], // Blue
                    [16, 185, 129, 255], // Green
                    [245, 158, 11, 255], // Orange / Yellow
                    [168, 85, 247, 255], // Purple
                    [255, 255, 255, 255],// White
                ];
                let current_idx = palettes.iter().position(|c| *c == self.active_color).unwrap_or(0);
                self.active_color = palettes[(current_idx + 1) % palettes.len()];
            }
            _ => {
                self.active_tool = tool;
            }
        }
    }

    /// Applies the currently selected annotation tool from (x0, y0) to (x1, y1).
    fn apply_annotation_stroke(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        match self.active_tool {
            ToolType::Pen => {
                draw_line(
                    &mut self.annotated_screen,
                    x0,
                    y0,
                    x1,
                    y1,
                    self.active_color,
                    self.stroke_size,
                );
            }
            ToolType::Line => {
                draw_line(
                    &mut self.annotated_screen,
                    x0,
                    y0,
                    x1,
                    y1,
                    self.active_color,
                    self.stroke_size,
                );
            }
            ToolType::Arrow => {
                draw_arrow(
                    &mut self.annotated_screen,
                    x0,
                    y0,
                    x1,
                    y1,
                    self.active_color,
                    self.stroke_size,
                );
            }
            ToolType::Rectangle => {
                let rect = Rect::from_points(x0, y0, x1, y1);
                draw_rectangle(&mut self.annotated_screen, rect, self.active_color, self.stroke_size as u32);
            }
            ToolType::Highlighter => {
                let rect = Rect::from_points(x0, y0, x1, y1);
                draw_highlighter(&mut self.annotated_screen, rect, self.active_color);
            }
            ToolType::Pixelate => {
                let rect = Rect::from_points(x0, y0, x1, y1);
                pixelate(&mut self.annotated_screen, rect, 10);
            }
            ToolType::StepBadge => {
                self.counter.add_step(x1 as u32, y1 as u32);
                self.counter.render_all(&mut self.annotated_screen);
            }
            _ => {}
        }
    }

    /// Copies the current selection to the clipboard.
    pub fn copy_selection(&self) {
        if let Some(sel) = self.selection {
            let crop = imageops::crop_imm(&self.annotated_screen, sel.x, sel.y, sel.width, sel.height).to_image();
            if let Err(e) = copy_to_clipboard(&crop) {
                eprintln!("Clipboard error: {}", e);
            } else {
                info!("Cropped screenshot ({}x{}) copied to clipboard!", sel.width, sel.height);
            }
        }
    }

    /// Hit-tests for any of the 8 resize handles.
    fn hit_test_resize_handles(&self, sel: Rect, px: i32, py: i32) -> Option<ResizeHandle> {
        let tolerance = 6;
        let x0 = sel.x as i32;
        let y0 = sel.y as i32;
        let x1 = (sel.x + sel.width) as i32;
        let y1 = (sel.y + sel.height) as i32;
        let xm = (sel.x + sel.width / 2) as i32;
        let ym = (sel.y + sel.height / 2) as i32;

        let near = |target_x: i32, target_y: i32| -> bool {
            (px - target_x).abs() <= tolerance && (py - target_y).abs() <= tolerance
        };

        if near(x0, y0) { Some(ResizeHandle::NorthWest) }
        else if near(xm, y0) { Some(ResizeHandle::North) }
        else if near(x1, y0) { Some(ResizeHandle::NorthEast) }
        else if near(x1, ym) { Some(ResizeHandle::East) }
        else if near(x1, y1) { Some(ResizeHandle::SouthEast) }
        else if near(xm, y1) { Some(ResizeHandle::South) }
        else if near(x0, y1) { Some(ResizeHandle::SouthWest) }
        else if near(x0, ym) { Some(ResizeHandle::West) }
        else { None }
    }

    /// Calculates adjusted rectangle coordinates when dragging a resize handle.
    fn calculate_resized_rect(&self, orig: Rect, handle: ResizeHandle, mx: i32, my: i32) -> Rect {
        let x0 = orig.x as i32;
        let y0 = orig.y as i32;
        let x1 = (orig.x + orig.width) as i32;
        let y1 = (orig.y + orig.height) as i32;

        match handle {
            ResizeHandle::NorthWest => Rect::from_points(mx, my, x1, y1),
            ResizeHandle::North => Rect::from_points(x0, my, x1, y1),
            ResizeHandle::NorthEast => Rect::from_points(x0, my, mx, y1),
            ResizeHandle::East => Rect::from_points(x0, y0, mx, y1),
            ResizeHandle::SouthEast => Rect::from_points(x0, y0, mx, my),
            ResizeHandle::South => Rect::from_points(x0, y0, x1, my),
            ResizeHandle::SouthWest => Rect::from_points(mx, y0, x1, my),
            ResizeHandle::West => Rect::from_points(mx, y0, x1, y1),
        }
    }
}

/// Helper to render pixel fonts without external TTF dependencies.
fn render_pixel_text(image: &mut RgbaImage, text: &str, start_x: u32, start_y: u32, color: Rgba<u8>) {
    let mut cur_x = start_x as i32;
    let cur_y = start_y as i32;

    for ch in text.chars() {
        render_glyph(image, ch, cur_x, cur_y, color);
        cur_x += 7;
    }
}

fn render_glyph(image: &mut RgbaImage, ch: char, x: i32, y: i32, color: Rgba<u8>) {
    let bitmap: [u8; 5] = match ch {
        '0' => [0x1F, 0x11, 0x11, 0x11, 0x1F],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x0E],
        '2' => [0x1F, 0x01, 0x1F, 0x10, 0x1F],
        '3' => [0x1F, 0x01, 0x0F, 0x01, 0x1F],
        '4' => [0x11, 0x11, 0x1F, 0x01, 0x01],
        '5' => [0x1F, 0x10, 0x1F, 0x01, 0x1F],
        '6' => [0x1F, 0x10, 0x1F, 0x11, 0x1F],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08],
        '8' => [0x1F, 0x11, 0x1F, 0x11, 0x1F],
        '9' => [0x1F, 0x11, 0x1F, 0x01, 0x1F],
        'x' | 'X' => [0x11, 0x0A, 0x04, 0x0A, 0x11],
        'p' | 'P' => [0x1E, 0x11, 0x1E, 0x10, 0x10],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00],
        _ => [0x1F, 0x15, 0x15, 0x15, 0x1F],
    };

    for (row, byte) in bitmap.iter().enumerate() {
        for col in 0..5 {
            if (byte & (1 << (4 - col))) != 0 {
                put_pixel_safe(image, x + col, y + row as i32, color);
            }
        }
    }
}

fn put_pixel_safe(image: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x >= 0 && y >= 0 && (x as u32) < image.width() && (y as u32) < image.height() {
        image.put_pixel(x as u32, y as u32, color);
    }
}
