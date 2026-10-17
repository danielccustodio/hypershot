use image::{imageops, Rgba, RgbaImage};
use log::info;
use std::fs;
use std::path::Path;

use crate::core::capture::CaptureEngine;
use crate::core::config::AppConfig;
use crate::core::filters::Rect;
use crate::core::stitching::{ScrollStitcher, StitchResult};
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

#[derive(Debug, Clone)]
pub enum CaptureOutcome {
    CopiedToClipboard,
    SavedToFile(String),
    PinnedToDesktop(RgbaImage),
    Cancelled,
}

/// Ultra-high performance, zero-allocation overlay session.
pub struct OverlaySession {
    pub width: u32,
    pub height: u32,
    pub config: AppConfig,

    pub original_bgra: Vec<u32>,
    pub dimmed_bgra: Vec<u32>,
    pub annotated_bgra: Vec<u32>,
    pub render_buffer: Vec<u32>,
    pub undo_stack: Vec<Vec<u32>>,

    pub selection: Option<Rect>,
    pub drag_mode: DragMode,
    pub drag_start: (i32, i32),
    pub current_mouse: (i32, i32),

    pub active_tool: ToolType,
    pub hover_tool: Option<ToolType>,
    pub hover_action: Option<ActionType>,
    pub active_color: u32,
    pub stroke_size: i32,
    pub step_counter: u32,

    pub scroll_stitcher: Option<ScrollStitcher>,
    pub is_scroll_mode: bool,
    pub scroll_status: Option<String>,

    pub outcome: CaptureOutcome,
    pub is_finished: bool,
}

impl OverlaySession {
    pub fn new(screenshot: RgbaImage) -> Self {
        let width = screenshot.width();
        let height = screenshot.height();
        let total_pixels = (width * height) as usize;

        let config = AppConfig::load();

        let mut original_bgra = vec![0u32; total_pixels];
        let mut dimmed_bgra = vec![0u32; total_pixels];

        let dim_factor = config.dark_backdrop_level as u32;

        let raw = screenshot.as_raw();
        for (i, chunk) in raw.chunks_exact(4).enumerate() {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;

            let pix = 0xFF000000 | (r << 16) | (g << 8) | b;
            original_bgra[i] = pix;

            // Configurable backdrop dimming
            let dr = (r * dim_factor) / 255;
            let dg = (g * dim_factor) / 255;
            let db = (b * dim_factor) / 255;
            dimmed_bgra[i] = 0xFF000000 | (dr << 16) | (dg << 8) | db;
        }

        let annotated_bgra = original_bgra.clone();
        let render_buffer = dimmed_bgra.clone();

        let active_color = 0xFF000000
            | ((config.default_color[0] as u32) << 16)
            | ((config.default_color[1] as u32) << 8)
            | (config.default_color[2] as u32);

        Self {
            width,
            height,
            config: config.clone(),
            original_bgra,
            dimmed_bgra,
            annotated_bgra,
            render_buffer,
            undo_stack: Vec::new(),

            selection: None,
            drag_mode: DragMode::None,
            drag_start: (0, 0),
            current_mouse: (0, 0),

            active_tool: ToolType::Select,
            hover_tool: None,
            hover_action: None,
            active_color,
            stroke_size: config.default_stroke_size,
            step_counter: 1,

            scroll_stitcher: None,
            is_scroll_mode: false,
            scroll_status: None,

            outcome: CaptureOutcome::Cancelled,
            is_finished: false,
        }
    }

    /// Blazing fast SIMD-friendly compose into the pre-allocated render buffer (Zero allocations).
    pub fn render_frame(&mut self) -> &[u32] {
        self.render_buffer.copy_from_slice(&self.dimmed_bgra);

        let (w, h) = (self.width, self.height);

        if let Some(sel) = self.selection.and_then(|r| r.clamp_to_image(w, h)) {
            for y in sel.y..(sel.y + sel.height) {
                let row_start = (y * w + sel.x) as usize;
                let row_end = row_start + sel.width as usize;
                self.render_buffer[row_start..row_end].copy_from_slice(&self.annotated_bgra[row_start..row_end]);
            }

            let border_color: u32 = if self.is_scroll_mode { 0xFF10B981 } else { 0xFF3B82F6 };
            for x in sel.x..(sel.x + sel.width) {
                put_pixel(&mut self.render_buffer, w, h, x as i32, sel.y as i32, border_color);
                put_pixel(&mut self.render_buffer, w, h, x as i32, (sel.y + sel.height - 1) as i32, border_color);
            }
            for y in sel.y..(sel.y + sel.height) {
                put_pixel(&mut self.render_buffer, w, h, sel.x as i32, y as i32, border_color);
                put_pixel(&mut self.render_buffer, w, h, (sel.x + sel.width - 1) as i32, y as i32, border_color);
            }

            self.draw_resize_handles(sel);
            self.draw_dimension_badge(sel);

            let layout = ToolbarLayout::compute(sel, w, h);
            layout.render_to_buffer(
                &mut self.render_buffer,
                w,
                h,
                self.active_tool,
                self.hover_tool,
                self.hover_action,
                self.active_color,
            );

            // Draw Scroll Banner if in Scroll Mode
            if self.is_scroll_mode {
                self.draw_scroll_banner();
            }
        } else if self.config.enable_loupe {
            self.draw_magnifier_loupe();
        }

        &self.render_buffer
    }

    fn draw_scroll_banner(&mut self) {
        let (w, h) = (self.width, self.height);
        let status = self.scroll_status.as_deref().unwrap_or("Scroll Capture Ativo: Gire o scroll para costurar a pagina (Enter para concluir)");
        let banner_w = ((status.len() as u32 * 7) + 24).min(w - 20);
        let banner_h = 28;
        let banner_x = (w.saturating_sub(banner_w)) / 2;
        let banner_y = 20;

        let bg: u32 = 0xFF1A1B26;
        let border: u32 = 0xFF10B981; // Green
        let text_color: u32 = 0xFFFFFFFF;

        for dy in 0..banner_h {
            for dx in 0..banner_w {
                let is_b = dx == 0 || dx == banner_w - 1 || dy == 0 || dy == banner_h - 1;
                put_pixel(&mut self.render_buffer, w, h, (banner_x + dx) as i32, (banner_y + dy) as i32, if is_b { border } else { bg });
            }
        }

        render_text_to_buffer(&mut self.render_buffer, w, h, status, (banner_x + 12) as i32, (banner_y + 9) as i32, text_color);
    }

    fn draw_resize_handles(&mut self, sel: Rect) {
        let (w, h) = (self.width, self.height);
        let hs = 3;

        let points = [
            (sel.x as i32, sel.y as i32),
            ((sel.x + sel.width / 2) as i32, sel.y as i32),
            ((sel.x + sel.width) as i32, sel.y as i32),
            ((sel.x + sel.width) as i32, (sel.y + sel.height / 2) as i32),
            ((sel.x + sel.width) as i32, (sel.y + sel.height) as i32),
            ((sel.x + sel.width / 2) as i32, (sel.y + sel.height) as i32),
            (sel.x as i32, (sel.y + sel.height) as i32),
            (sel.x as i32, (sel.y + sel.height / 2) as i32),
        ];

        let handle_fill: u32 = 0xFFFFFFFF;
        let handle_border: u32 = if self.is_scroll_mode { 0xFF10B981 } else { 0xFF3B82F6 };

        for (px, py) in points {
            for dy in -hs..=hs {
                for dx in -hs..=hs {
                    let is_border = dx == -hs || dx == hs || dy == -hs || dy == hs;
                    put_pixel(&mut self.render_buffer, w, h, px + dx, py + dy, if is_border { handle_border } else { handle_fill });
                }
            }
        }
    }

    fn draw_dimension_badge(&mut self, sel: Rect) {
        let (w, h) = (self.width, self.height);
        let total_h = if let Some(ref stitcher) = self.scroll_stitcher {
            stitcher.composite().height()
        } else {
            sel.height
        };

        let badge_text = if self.is_scroll_mode {
            format!("{} x {} px [SCROLL]", sel.width, total_h)
        } else {
            format!("{} x {} px", sel.width, sel.height)
        };

        let badge_w = (badge_text.len() as u32 * 7) + 14;
        let badge_h = 20;

        let badge_x = sel.x.min(w.saturating_sub(badge_w + 4));
        let badge_y = if sel.y >= badge_h + 4 {
            sel.y.saturating_sub(badge_h + 4)
        } else {
            sel.y + sel.height + 4
        };

        let bg: u32 = 0xFF1A1B26;
        let border: u32 = 0xFF414868;
        let text_color: u32 = 0xFFF0F3FA;

        for dy in 0..badge_h {
            for dx in 0..badge_w {
                let is_b = dx == 0 || dx == badge_w - 1 || dy == 0 || dy == badge_h - 1;
                put_pixel(&mut self.render_buffer, w, h, (badge_x + dx) as i32, (badge_y + dy) as i32, if is_b { border } else { bg });
            }
        }

        render_text_to_buffer(&mut self.render_buffer, w, h, &badge_text, (badge_x + 7) as i32, (badge_y + 5) as i32, text_color);
    }

    fn draw_magnifier_loupe(&mut self) {
        let (mx, my) = self.current_mouse;
        let (w, h) = (self.width, self.height);

        if mx < 0 || my < 0 || (mx as u32) >= w || (my as u32) >= h {
            return;
        }

        let loupe_size: i32 = 88;
        let zoom: i32 = 8;
        let half = loupe_size / 2;

        let mut lx = mx + 20;
        let mut ly = my + 20;
        if lx + loupe_size >= w as i32 { lx = mx - loupe_size - 20; }
        if ly + loupe_size + 24 >= h as i32 { ly = my - loupe_size - 24; }

        let sample_radius = half / zoom;
        for dy in -sample_radius..=sample_radius {
            for dx in -sample_radius..=sample_radius {
                let sx = (mx + dx).clamp(0, w as i32 - 1) as usize;
                let sy = (my + dy).clamp(0, h as i32 - 1) as usize;
                let pix = self.original_bgra[sy * w as usize + sx];

                let box_x = lx + (dx + sample_radius) * zoom;
                let box_y = ly + (dy + sample_radius) * zoom;

                for by in 0..zoom {
                    for bx in 0..zoom {
                        put_pixel(&mut self.render_buffer, w, h, box_x + bx, box_y + by, pix);
                    }
                }
            }
        }

        let cx = lx + half;
        let cy = ly + half;
        let crosshair: u32 = 0xFFEF4444; // Red
        for d in -6..=6 {
            if d != 0 {
                put_pixel(&mut self.render_buffer, w, h, cx + d, cy, crosshair);
                put_pixel(&mut self.render_buffer, w, h, cx, cy + d, crosshair);
            }
        }

        let footer_h = 20;
        let total_h = loupe_size + footer_h;
        let border: u32 = 0xFF3B82F6;
        let footer_bg: u32 = 0xFF1A1B26;

        for dy in 0..total_h {
            for dx in 0..loupe_size {
                let is_outer = dx == 0 || dx == loupe_size - 1 || dy == 0 || dy == total_h - 1 || dy == loupe_size;
                if is_outer {
                    put_pixel(&mut self.render_buffer, w, h, lx + dx, ly + dy, border);
                } else if dy > loupe_size {
                    put_pixel(&mut self.render_buffer, w, h, lx + dx, ly + dy, footer_bg);
                }
            }
        }

        let center_pix = self.original_bgra[(my as u32 * w + mx as u32) as usize];
        let r = (center_pix >> 16) & 0xFF;
        let g = (center_pix >> 8) & 0xFF;
        let b = center_pix & 0xFF;
        let hex_code = format!("#{:02X}{:02X}{:02X}", r, g, b);
        render_text_to_buffer(&mut self.render_buffer, w, h, &hex_code, lx + 6, ly + loupe_size + 5, 0xFFFFFFFF);
    }

    pub fn on_mouse_down(&mut self, x: i32, y: i32) {
        self.drag_start = (x, y);

        if let Some(sel) = self.selection {
            let layout = ToolbarLayout::compute(sel, self.width, self.height);

            if let Some(action) = layout.hit_test_action(x, y) {
                self.trigger_action(action);
                return;
            }

            if let Some(tool) = layout.hit_test_tool(x, y) {
                self.trigger_tool(tool);
                return;
            }

            if let Some(handle) = self.hit_test_resize_handles(sel, x, y) {
                self.drag_mode = DragMode::Resizing(handle);
                return;
            }

            if self.active_tool != ToolType::Select && sel.contains(x, y) {
                if self.scroll_stitcher.is_some() {
                    // The export is the stitched composite, whose pixels were all taken before
                    // this click: a stroke drawn now would show in the preview but not in the output.
                    self.drag_mode = DragMode::None;
                    self.scroll_status = Some("Anotacoes desativadas no Scroll Mode: anote antes de clicar em Scroll".to_string());
                    return;
                }
                self.undo_stack.push(self.annotated_bgra.clone());
                self.drag_mode = DragMode::Annotating;
                self.apply_annotation_stroke(x, y, x, y);
                return;
            }

            if sel.contains(x, y) {
                self.drag_mode = DragMode::MovingSelection {
                    offset_x: x - sel.x as i32,
                    offset_y: y - sel.y as i32,
                };
                return;
            }
        }

        self.drag_mode = DragMode::CreatingSelection;
        self.selection = Some(Rect::new(x as u32, y as u32, 1, 1));
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32) {
        self.current_mouse = (x, y);

        if let Some(sel) = self.selection {
            let layout = ToolbarLayout::compute(sel, self.width, self.height);
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
                    let new_x = (x - offset_x).clamp(0, (self.width - sel.width) as i32) as u32;
                    let new_y = (y - offset_y).clamp(0, (self.height - sel.height) as i32) as u32;
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

    /// Handles mouse wheel scrolling (adjusts stroke size or stitches scroll capture).
    pub fn on_mouse_wheel(&mut self, delta: i32) {
        if self.is_scroll_mode {
            if let Some(sel) = self.selection {
                // Re-capture desktop to get newly scrolled content
                if let Ok(new_desktop) = CaptureEngine::capture_primary() {
                    let clamped = sel.clamp_to_image(new_desktop.width(), new_desktop.height()).unwrap_or(sel);
                    let new_frame = imageops::crop_imm(&new_desktop, clamped.x, clamped.y, clamped.width, clamped.height).to_image();

                    if let Some(ref mut stitcher) = self.scroll_stitcher {
                        match stitcher.append_frame(&new_frame) {
                            StitchResult::Appended { added_height, total_height } => {
                                self.scroll_status = Some(format!("Costurando: +{}px (Altura Total: {}px) - Enter para salvar", added_height, total_height));
                            }
                            StitchResult::EndOfPage => {
                                self.scroll_status = Some("Fim da pagina detectado - Pressione Enter para salvar".to_string());
                            }
                            StitchResult::MatchFailed => {
                                self.scroll_status = Some("Role a janela mais devagar para sobrepor".to_string());
                            }
                        }
                    }
                }
            }
        } else {
            // Adjust stroke thickness
            let change = if delta > 0 { 1 } else { -1 };
            self.stroke_size = (self.stroke_size + change).clamp(1, 24);
        }
    }

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
                    let crop = self.get_final_image(sel);

                    let save_dir = Path::new(&self.config.save_directory);
                    let _ = fs::create_dir_all(save_dir);

                    let ext = self.config.default_format.to_lowercase();
                    let default_file_name = format!("hypershot_{}.{}", chrono::Local::now().format("%Y%m%d_%H%M%S"), ext);

                    let dialog = rfd::FileDialog::new()
                        .set_directory(save_dir)
                        .set_file_name(&default_file_name)
                        .add_filter("PNG Image", &["png"])
                        .add_filter("JPEG Image", &["jpg", "jpeg"])
                        .add_filter("WebP Image", &["webp"]);

                    if let Some(file_path) = dialog.save_file() {
                        let path_str = file_path.to_string_lossy().to_string();
                        if let Err(e) = crop.save(&file_path) {
                            eprintln!("Failed to save image: {}", e);
                        } else {
                            info!("Successfully saved screenshot to: {}", path_str);

                            if self.config.copy_to_clipboard_on_save {
                                let _ = crate::core::clipboard::copy_to_clipboard(&crop);
                            }

                            self.outcome = CaptureOutcome::SavedToFile(path_str);
                            self.is_finished = true;
                        }
                    }
                }
            }
            ActionType::Pin => {
                if let Some(sel) = self.selection {
                    let crop = self.get_final_image(sel);
                    self.outcome = CaptureOutcome::PinnedToDesktop(crop);
                    self.is_finished = true;
                }
            }
            ActionType::Scroll => {
                if let Some(sel) = self.selection {
                    let initial_crop = self.extract_crop(sel);
                    self.scroll_stitcher = Some(ScrollStitcher::new(initial_crop));
                    self.is_scroll_mode = true;
                    self.scroll_status = Some("Scroll Mode Ativo: Gire o scroll sobre a pagina ou aperte Enter para finalizar".to_string());
                    info!("Scroll capture initialized for region {}x{}", sel.width, sel.height);
                }
            }
        }
    }

    pub fn trigger_tool(&mut self, tool: ToolType) {
        match tool {
            ToolType::Undo => {
                if let Some(prev) = self.undo_stack.pop() {
                    self.annotated_bgra = prev;
                    self.step_counter = self.step_counter.saturating_sub(1).max(1);
                }
            }
            ToolType::ColorPicker => {
                let palettes = [
                    0xFFEB2F2F, // Red
                    0xFF3B82F6, // Blue
                    0xFF10B981, // Green
                    0xFFF59E0B, // Orange
                    0xFFA855F7, // Purple
                    0xFFFFFFFF, // White
                ];
                let current_idx = palettes.iter().position(|c| *c == self.active_color).unwrap_or(0);
                self.active_color = palettes[(current_idx + 1) % palettes.len()];
            }
            _ => {
                self.active_tool = tool;
            }
        }
    }

    fn apply_annotation_stroke(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        // get_final_image exports the stitched composite whenever a stitcher exists, so a
        // stroke made now would only alter the preview. Also covers a drag left in progress.
        if self.scroll_stitcher.is_some() {
            return;
        }

        let (w, h) = (self.width, self.height);
        let color = self.active_color;
        let thickness = self.stroke_size;

        match self.active_tool {
            ToolType::Pen | ToolType::Line => {
                draw_line_fast(&mut self.annotated_bgra, w, h, x0, y0, x1, y1, color, thickness);
            }
            ToolType::Arrow => {
                draw_arrow_fast(&mut self.annotated_bgra, w, h, x0, y0, x1, y1, color, thickness);
            }
            ToolType::Rectangle => {
                let rect = Rect::from_points(x0, y0, x1, y1);
                draw_rect_outline(&mut self.annotated_bgra, w, h, rect, color, thickness as u32);
            }
            ToolType::Highlighter => {
                let rect = Rect::from_points(x0, y0, x1, y1);
                draw_highlight_fast(&mut self.annotated_bgra, w, h, rect, color);
            }
            ToolType::Pixelate => {
                let rect = Rect::from_points(x0, y0, x1, y1);
                apply_pixelate_fast(&mut self.annotated_bgra, w, h, rect, 10);
            }
            ToolType::StepBadge => {
                draw_step_badge_fast(&mut self.annotated_bgra, w, h, x1, y1, self.step_counter, color);
                self.step_counter += 1;
            }
            _ => {}
        }
    }

    pub fn copy_selection(&self) {
        if let Some(sel) = self.selection {
            let final_img = self.get_final_image(sel);
            if let Err(e) = crate::core::clipboard::copy_to_clipboard(&final_img) {
                eprintln!("Clipboard error: {}", e);
            } else {
                info!("Screenshot ({}x{}) copied to clipboard!", final_img.width(), final_img.height());
            }
        }
    }

    /// Retrieves either the full-page stitched composite image or the annotated rectangle crop.
    pub fn get_final_image(&self, sel: Rect) -> RgbaImage {
        if let Some(ref stitcher) = self.scroll_stitcher {
            stitcher.composite().clone()
        } else {
            self.extract_crop(sel)
        }
    }

    pub fn extract_crop(&self, sel: Rect) -> RgbaImage {
        let (w, h) = (self.width, self.height);
        let clamped = sel.clamp_to_image(w, h).unwrap_or(Rect::new(0, 0, 1, 1));

        let mut img = RgbaImage::new(clamped.width, clamped.height);
        for y in 0..clamped.height {
            let src_row = ((clamped.y + y) * w + clamped.x) as usize;
            for x in 0..clamped.width {
                let pix = self.annotated_bgra[src_row + x as usize];
                let r = ((pix >> 16) & 0xFF) as u8;
                let g = ((pix >> 8) & 0xFF) as u8;
                let b = (pix & 0xFF) as u8;
                img.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }
        img
    }

    fn hit_test_resize_handles(&self, sel: Rect, px: i32, py: i32) -> Option<ResizeHandle> {
        let tol = 6;
        let x0 = sel.x as i32;
        let y0 = sel.y as i32;
        let x1 = (sel.x + sel.width) as i32;
        let y1 = (sel.y + sel.height) as i32;
        let xm = (sel.x + sel.width / 2) as i32;
        let ym = (sel.y + sel.height / 2) as i32;

        let near = |tx: i32, ty: i32| (px - tx).abs() <= tol && (py - ty).abs() <= tol;

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

// ----------------- FAST DRAWING PRIMITIVES (Zero allocation) -----------------

#[inline(always)]
fn put_pixel(buffer: &mut [u32], stride: u32, max_h: u32, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 && (x as u32) < stride && (y as u32) < max_h {
        buffer[(y as u32 * stride + x as u32) as usize] = color;
    }
}

fn draw_line_fast(buffer: &mut [u32], stride: u32, max_h: u32, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u32, thickness: i32) {
    let r = (thickness / 2).max(1);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        for dy_b in -r..=r {
            for dx_b in -r..=r {
                if dx_b * dx_b + dy_b * dy_b <= r * r {
                    put_pixel(buffer, stride, max_h, x0 + dx_b, y0 + dy_b, color);
                }
            }
        }

        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}

fn draw_arrow_fast(buffer: &mut [u32], stride: u32, max_h: u32, x0: i32, y0: i32, x1: i32, y1: i32, color: u32, thickness: i32) {
    let dx = (x1 - x0) as f64;
    let dy = (y1 - y0) as f64;
    let length = (dx * dx + dy * dy).sqrt();
    if length < 5.0 { return; }

    let angle = dy.atan2(dx);
    let head_length = (length * 0.25).clamp(14.0, 32.0);
    let head_angle = std::f64::consts::PI / 6.0;

    let shaft_end_x = x1 - (angle.cos() * (head_length * 0.7)) as i32;
    let shaft_end_y = y1 - (angle.sin() * (head_length * 0.7)) as i32;
    draw_line_fast(buffer, stride, max_h, x0, y0, shaft_end_x, shaft_end_y, color, thickness);

    let left_angle = angle + std::f64::consts::PI - head_angle;
    let right_angle = angle + std::f64::consts::PI + head_angle;

    let left_x = x1 + (left_angle.cos() * head_length) as i32;
    let left_y = y1 + (left_angle.sin() * head_length) as i32;
    let right_x = x1 + (right_angle.cos() * head_length) as i32;
    let right_y = y1 + (right_angle.sin() * head_length) as i32;

    draw_line_fast(buffer, stride, max_h, x1, y1, left_x, left_y, color, thickness);
    draw_line_fast(buffer, stride, max_h, x1, y1, right_x, right_y, color, thickness);
}

fn draw_rect_outline(buffer: &mut [u32], stride: u32, max_h: u32, rect: Rect, color: u32, thickness: u32) {
    let t = thickness.max(1);
    let x_end = (rect.x + rect.width).min(stride);
    let y_end = (rect.y + rect.height).min(max_h);

    for h in 0..t {
        for x in rect.x..x_end {
            put_pixel(buffer, stride, max_h, x as i32, (rect.y + h) as i32, color);
            put_pixel(buffer, stride, max_h, x as i32, (y_end.saturating_sub(1 + h)) as i32, color);
        }
    }
    for w in 0..t {
        for y in rect.y..y_end {
            put_pixel(buffer, stride, max_h, (rect.x + w) as i32, y as i32, color);
            put_pixel(buffer, stride, max_h, (x_end.saturating_sub(1 + w)) as i32, y as i32, color);
        }
    }
}

fn draw_highlight_fast(buffer: &mut [u32], stride: u32, max_h: u32, rect: Rect, highlight_color: u32) {
    let hr = (highlight_color >> 16) & 0xFF;
    let hg = (highlight_color >> 8) & 0xFF;
    let hb = highlight_color & 0xFF;

    let x_end = (rect.x + rect.width).min(stride);
    let y_end = (rect.y + rect.height).min(max_h);

    for y in rect.y..y_end {
        let row_offset = (y * stride) as usize;
        for x in rect.x..x_end {
            let orig = buffer[row_offset + x as usize];
            let or = (orig >> 16) & 0xFF;
            let og = (orig >> 8) & 0xFF;
            let ob = orig & 0xFF;

            let nr = (or * 55 + hr * 45) / 100;
            let ng = (og * 55 + hg * 45) / 100;
            let nb = (ob * 55 + hb * 45) / 100;
            buffer[row_offset + x as usize] = 0xFF000000 | (nr << 16) | (ng << 8) | nb;
        }
    }
}

fn apply_pixelate_fast(buffer: &mut [u32], stride: u32, max_h: u32, rect: Rect, block_size: u32) {
    let bs = block_size.max(2);
    let x_end = (rect.x + rect.width).min(stride);
    let y_end = (rect.y + rect.height).min(max_h);

    let mut by = rect.y;
    while by < y_end {
        let bh = bs.min(y_end - by);
        let mut bx = rect.x;
        while bx < x_end {
            let bw = bs.min(x_end - bx);

            let mut r_sum = 0u64;
            let mut g_sum = 0u64;
            let mut b_sum = 0u64;
            let count = (bw * bh) as u64;

            for py in by..(by + bh) {
                let row = (py * stride) as usize;
                for px in bx..(bx + bw) {
                    let pix = buffer[row + px as usize];
                    r_sum += ((pix >> 16) & 0xFF) as u64;
                    g_sum += ((pix >> 8) & 0xFF) as u64;
                    b_sum += (pix & 0xFF) as u64;
                }
            }

            let avg_color = 0xFF000000
                | (((r_sum / count) as u32) << 16)
                | (((g_sum / count) as u32) << 8)
                | ((b_sum / count) as u32);

            for py in by..(by + bh) {
                let row = (py * stride) as usize;
                for px in bx..(bx + bw) {
                    buffer[row + px as usize] = avg_color;
                }
            }

            bx += bw;
        }
        by += bh;
    }
}

fn draw_step_badge_fast(buffer: &mut [u32], stride: u32, max_h: u32, cx: i32, cy: i32, step: u32, color: u32) {
    let radius = 13;
    let r_sq = radius * radius;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= r_sq {
                let is_border = dist_sq >= (radius - 2) * (radius - 2);
                let pix = if is_border { 0xFFFFFFFF } else { color };
                put_pixel(buffer, stride, max_h, cx + dx, cy + dy, pix);
            }
        }
    }

    let num_str = step.to_string();
    let text_x = cx - ((num_str.len() as i32 * 6) / 2);
    let text_y = cy - 3;
    render_text_to_buffer(buffer, stride, max_h, &num_str, text_x, text_y, 0xFFFFFFFF);
}

fn render_text_to_buffer(buffer: &mut [u32], stride: u32, max_h: u32, text: &str, start_x: i32, start_y: i32, color: u32) {
    let mut cur_x = start_x;
    for ch in text.chars() {
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
            'S' | 's' => [0x1F, 0x10, 0x1F, 0x01, 0x1F],
            'C' | 'c' => [0x1F, 0x10, 0x10, 0x10, 0x1F],
            'R' | 'r' => [0x1E, 0x11, 0x1E, 0x14, 0x12],
            'O' | 'o' => [0x0E, 0x11, 0x11, 0x11, 0x0E],
            'L' | 'l' => [0x10, 0x10, 0x10, 0x10, 0x1F],
            'M' | 'm' => [0x11, 0x1B, 0x15, 0x11, 0x11],
            'D' | 'd' => [0x1E, 0x11, 0x11, 0x11, 0x1E],
            'E' | 'e' => [0x1F, 0x10, 0x1E, 0x10, 0x1F],
            'A' | 'a' => [0x0E, 0x11, 0x1F, 0x11, 0x11],
            'T' | 't' => [0x1F, 0x04, 0x04, 0x04, 0x04],
            'I' | 'i' => [0x0E, 0x04, 0x04, 0x04, 0x0E],
            'V' | 'v' => [0x11, 0x11, 0x11, 0x0A, 0x04],
            '+' => [0x00, 0x04, 0x0E, 0x04, 0x00],
            ':' => [0x00, 0x04, 0x00, 0x04, 0x00],
            '[' => [0x0E, 0x08, 0x08, 0x08, 0x0E],
            ']' => [0x0E, 0x02, 0x02, 0x02, 0x0E],
            '(' => [0x02, 0x04, 0x04, 0x04, 0x02],
            ')' => [0x08, 0x04, 0x04, 0x04, 0x08],
            '-' => [0x00, 0x00, 0x1F, 0x00, 0x00],
            ' ' => [0x00, 0x00, 0x00, 0x00, 0x00],
            _ => [0x1F, 0x15, 0x15, 0x15, 0x1F],
        };

        for (row, byte) in bitmap.iter().enumerate() {
            for col in 0..5 {
                if (byte & (1 << (4 - col))) != 0 {
                    put_pixel(buffer, stride, max_h, cur_x + col, start_y + row as i32, color);
                }
            }
        }
        cur_x += 7;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a session without `OverlaySession::new`, which would write a config file to disk.
    fn session_with_selection(w: u32, h: u32, sel: Rect) -> OverlaySession {
        let original_bgra: Vec<u32> = (0..w * h).map(|i| 0xFF000000 | (i.wrapping_mul(2654435761) & 0x00FF_FFFF)).collect();
        let config = AppConfig::default();
        OverlaySession {
            width: w,
            height: h,
            stroke_size: config.default_stroke_size,
            config,
            dimmed_bgra: original_bgra.clone(),
            annotated_bgra: original_bgra.clone(),
            render_buffer: original_bgra.clone(),
            original_bgra,
            undo_stack: Vec::new(),
            selection: Some(sel),
            drag_mode: DragMode::None,
            drag_start: (0, 0),
            current_mouse: (0, 0),
            active_tool: ToolType::Select,
            hover_tool: None,
            hover_action: None,
            active_color: 0xFFEB2F2F,
            step_counter: 1,
            scroll_stitcher: None,
            is_scroll_mode: false,
            scroll_status: None,
            outcome: CaptureOutcome::Cancelled,
            is_finished: false,
        }
    }

    fn drag(session: &mut OverlaySession, from: (i32, i32), to: (i32, i32)) {
        session.on_mouse_down(from.0, from.1);
        session.on_mouse_move(to.0, to.1);
        session.on_mouse_up(to.0, to.1);
    }

    #[test]
    fn annotations_are_refused_once_scroll_mode_owns_the_export() {
        let sel = Rect::new(20, 20, 100, 100);
        let mut session = session_with_selection(400, 400, sel);

        // A redaction made before Scroll is part of the exported composite.
        session.active_tool = ToolType::Pixelate;
        drag(&mut session, (30, 30), (60, 60));
        let redacted = session.extract_crop(sel);
        session.trigger_action(ActionType::Scroll);
        assert!(session.scroll_stitcher.is_some());
        assert_eq!(session.get_final_image(sel), redacted);

        // Once in Scroll mode, a stroke must not change the preview without reaching the output.
        let annotated_before = session.annotated_bgra.clone();
        let undo_depth = session.undo_stack.len();
        for tool in [ToolType::Pixelate, ToolType::Pen, ToolType::Highlighter, ToolType::StepBadge] {
            session.active_tool = tool;
            drag(&mut session, (70, 70), (110, 110));
        }
        assert!(session.annotated_bgra == annotated_before);
        assert_eq!(session.undo_stack.len(), undo_depth);
        assert_eq!(session.drag_mode, DragMode::None);
        assert_eq!(session.extract_crop(sel), session.get_final_image(sel));

        // A drag already in progress when Scroll started cannot paint either.
        session.active_tool = ToolType::Pen;
        session.drag_mode = DragMode::Annotating;
        session.drag_start = (70, 70);
        session.on_mouse_move(110, 110);
        session.on_mouse_up(110, 110);
        assert!(session.annotated_bgra == annotated_before);
    }
}
