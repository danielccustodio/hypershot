use crate::core::filters::Rect;
use image::{Rgba, RgbaImage};

/// Available annotation tools in the vertical toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    Select,
    Pen,
    Line,
    Arrow,
    Rectangle,
    Highlighter,
    Pixelate,
    StepBadge,
    ColorPicker,
    Undo,
}

/// Quick action buttons in the horizontal toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Close,
    Save,
    Copy,
    Pin,
    Scroll,
}

pub const TOOLBAR_BTN_SIZE: u32 = 32;
pub const TOOLBAR_PADDING: u32 = 4;
pub const TOOLBAR_SPACING: u32 = 2;

/// Geometric layout for the Lightshot-style Dual Dock toolbars.
#[derive(Debug, Clone)]
pub struct ToolbarLayout {
    pub vertical_rect: Rect,
    pub horizontal_rect: Rect,
    pub tools: Vec<(ToolType, Rect)>,
    pub actions: Vec<(ActionType, Rect)>,
}

impl ToolbarLayout {
    /// Computes the exact screen placement for both toolbars docked around the selection rectangle.
    pub fn compute(selection: Rect, screen_w: u32, screen_h: u32) -> Self {
        let tools_list = [
            ToolType::Pen,
            ToolType::Line,
            ToolType::Arrow,
            ToolType::Rectangle,
            ToolType::Highlighter,
            ToolType::Pixelate,
            ToolType::StepBadge,
            ToolType::ColorPicker,
            ToolType::Undo,
        ];

        let actions_list = [
            ActionType::Close,
            ActionType::Save,
            ActionType::Copy,
            ActionType::Pin,
            ActionType::Scroll,
        ];

        // 1. Vertical Toolbar (Right side of selection)
        let v_width = TOOLBAR_BTN_SIZE + (TOOLBAR_PADDING * 2);
        let v_height = (tools_list.len() as u32 * (TOOLBAR_BTN_SIZE + TOOLBAR_SPACING))
            - TOOLBAR_SPACING
            + (TOOLBAR_PADDING * 2);

        // Place to the right of selection; if offscreen, place inside right edge
        let v_x = if selection.x + selection.width + 6 + v_width <= screen_w {
            selection.x + selection.width + 6
        } else if selection.width >= v_width + 10 {
            selection.x + selection.width.saturating_sub(v_width + 6)
        } else {
            selection.x.saturating_sub(v_width + 6)
        };

        // Align top with selection top, clamped to screen bounds
        let v_y = selection.y.clamp(6, screen_h.saturating_sub(v_height + 6));
        let vertical_rect = Rect::new(v_x, v_y, v_width, v_height);

        let mut tools_coords = Vec::new();
        for (idx, tool) in tools_list.iter().enumerate() {
            let btn_y = v_y + TOOLBAR_PADDING + (idx as u32 * (TOOLBAR_BTN_SIZE + TOOLBAR_SPACING));
            let btn_rect = Rect::new(v_x + TOOLBAR_PADDING, btn_y, TOOLBAR_BTN_SIZE, TOOLBAR_BTN_SIZE);
            tools_coords.push((*tool, btn_rect));
        }

        // 2. Horizontal Toolbar (Bottom of selection)
        let h_height = TOOLBAR_BTN_SIZE + (TOOLBAR_PADDING * 2);
        let h_width = (actions_list.len() as u32 * (TOOLBAR_BTN_SIZE + TOOLBAR_SPACING))
            - TOOLBAR_SPACING
            + (TOOLBAR_PADDING * 2);

        // Place below selection; if offscreen, place inside bottom edge
        let h_y = if selection.y + selection.height + 6 + h_height <= screen_h {
            selection.y + selection.height + 6
        } else if selection.height >= h_height + 10 {
            selection.y + selection.height.saturating_sub(h_height + 6)
        } else {
            selection.y.saturating_sub(h_height + 6)
        };

        // Align right edge of horizontal toolbar with right edge of selection
        let right_anchor = selection.x + selection.width;
        let h_x = if right_anchor >= h_width {
            (right_anchor - h_width).min(screen_w.saturating_sub(h_width + 6))
        } else {
            6
        };
        let horizontal_rect = Rect::new(h_x, h_y, h_width, h_height);

        let mut actions_coords = Vec::new();
        for (idx, action) in actions_list.iter().enumerate() {
            let btn_x = h_x + TOOLBAR_PADDING + (idx as u32 * (TOOLBAR_BTN_SIZE + TOOLBAR_SPACING));
            let btn_rect = Rect::new(btn_x, h_y + TOOLBAR_PADDING, TOOLBAR_BTN_SIZE, TOOLBAR_BTN_SIZE);
            actions_coords.push((*action, btn_rect));
        }

        Self {
            vertical_rect,
            horizontal_rect,
            tools: tools_coords,
            actions: actions_coords,
        }
    }

    /// Hit-tests a coordinate against tool buttons.
    pub fn hit_test_tool(&self, px: i32, py: i32) -> Option<ToolType> {
        for (tool, rect) in &self.tools {
            if rect.contains(px, py) {
                return Some(*tool);
            }
        }
        None
    }

    /// Hit-tests a coordinate against action buttons.
    pub fn hit_test_action(&self, px: i32, py: i32) -> Option<ActionType> {
        for (action, rect) in &self.actions {
            if rect.contains(px, py) {
                return Some(*action);
            }
        }
        None
    }

    /// Checks if a point lies anywhere inside either toolbar panel.
    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        self.vertical_rect.contains(px, py) || self.horizontal_rect.contains(px, py)
    }

    /// Renders both toolbars directly onto the target RGBA frame buffer.
    pub fn render(
        &self,
        target: &mut RgbaImage,
        active_tool: ToolType,
        hover_tool: Option<ToolType>,
        hover_action: Option<ActionType>,
        active_color: [u8; 4],
    ) {
        // Tokyo Night Dark Palette
        let panel_bg = Rgba([26, 27, 38, 245]);
        let panel_border = Rgba([65, 72, 104, 255]);
        let btn_hover_bg = Rgba([41, 46, 66, 255]);
        let btn_active_bg = Rgba([59, 130, 246, 255]); // Vibrant blue
        let icon_color = Rgba([224, 230, 245, 255]);
        let icon_muted = Rgba([154, 165, 206, 255]);

        // Draw Vertical Panel
        draw_rounded_rect(target, self.vertical_rect, panel_bg, panel_border, 6);
        for (tool, rect) in &self.tools {
            let is_active = *tool == active_tool;
            let is_hover = hover_tool == Some(*tool);

            if is_active {
                draw_rounded_rect(target, *rect, btn_active_bg, btn_active_bg, 4);
            } else if is_hover {
                draw_rounded_rect(target, *rect, btn_hover_bg, panel_border, 4);
            }

            render_tool_icon(target, *tool, *rect, if is_active { Rgba([255, 255, 255, 255]) } else if is_hover { icon_color } else { icon_muted }, active_color);
        }

        // Draw Horizontal Panel
        draw_rounded_rect(target, self.horizontal_rect, panel_bg, panel_border, 6);
        for (action, rect) in &self.actions {
            let is_hover = hover_action == Some(*action);

            if is_hover {
                let hover_color = match action {
                    ActionType::Close => Rgba([239, 68, 68, 220]), // Red
                    ActionType::Copy => Rgba([16, 185, 129, 220]),  // Emerald Green
                    _ => btn_hover_bg,
                };
                draw_rounded_rect(target, *rect, hover_color, panel_border, 4);
            }

            render_action_icon(target, *action, *rect, if is_hover { Rgba([255, 255, 255, 255]) } else { icon_color });
        }
    }
}

/// Helper to render a rounded rectangle with 1px border.
fn draw_rounded_rect(image: &mut RgbaImage, rect: Rect, fill: Rgba<u8>, border: Rgba<u8>, _radius: u32) {
    let w = image.width();
    let h = image.height();

    let clamped = match rect.clamp_to_image(w, h) {
        Some(r) => r,
        None => return,
    };

    for y in clamped.y..(clamped.y + clamped.height) {
        for x in clamped.x..(clamped.x + clamped.width) {
            let is_border = x == clamped.x
                || x == (clamped.x + clamped.width - 1)
                || y == clamped.y
                || y == (clamped.y + clamped.height - 1);

            let color = if is_border { border } else { fill };
            image.put_pixel(x, y, color);
        }
    }
}

/// Procedural pixel-art rendering of tools icons (no font/asset dependencies).
fn render_tool_icon(image: &mut RgbaImage, tool: ToolType, rect: Rect, color: Rgba<u8>, active_color: [u8; 4]) {
    let cx = rect.x as i32 + (rect.width as i32 / 2);
    let cy = rect.y as i32 + (rect.height as i32 / 2);

    match tool {
        ToolType::Pen => {
            // Draw diagonal pencil shape
            for i in -6..=6 {
                put_pixel_safe(image, cx + i, cy - i, color);
                put_pixel_safe(image, cx + i + 1, cy - i, color);
            }
            put_pixel_safe(image, cx - 6, cy + 6, color);
        }
        ToolType::Line => {
            // Horizontal line
            for dx in -7..=7 {
                put_pixel_safe(image, cx + dx, cy, color);
                put_pixel_safe(image, cx + dx, cy + 1, color);
            }
        }
        ToolType::Arrow => {
            // Arrow pointing top-right
            for i in -6..=6 {
                put_pixel_safe(image, cx + i, cy - i, color);
            }
            // Arrow head
            for i in 0..=5 {
                put_pixel_safe(image, cx + 6 - i, cy - 6, color);
                put_pixel_safe(image, cx + 6, cy - 6 + i, color);
            }
        }
        ToolType::Rectangle => {
            // Square frame
            for dx in -6..=6 {
                put_pixel_safe(image, cx + dx, cy - 6, color);
                put_pixel_safe(image, cx + dx, cy + 6, color);
            }
            for dy in -6..=6 {
                put_pixel_safe(image, cx - 6, cy + dy, color);
                put_pixel_safe(image, cx + 6, cy + dy, color);
            }
        }
        ToolType::Highlighter => {
            // Angled broad chisel marker
            for dx in -5..=5 {
                for dy in -3..=3 {
                    put_pixel_safe(image, cx + dx, cy + dy, color);
                }
            }
        }
        ToolType::Pixelate => {
            // 3x3 checkerboard mosaic pattern
            for dy in -6..=6 {
                for dx in -6..=6 {
                    if ((dx + 6) / 4 + (dy + 6) / 4) % 2 == 0 {
                        put_pixel_safe(image, cx + dx, cy + dy, color);
                    }
                }
            }
        }
        ToolType::StepBadge => {
            // Circle with step dot in the center
            for dy in -6..=6 {
                for dx in -6..=6 {
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq >= 25 && dist_sq <= 40 {
                        put_pixel_safe(image, cx + dx, cy + dy, color);
                    }
                }
            }
            // Number 1 stem
            for dy in -3..=3 {
                put_pixel_safe(image, cx, cy + dy, color);
            }
            put_pixel_safe(image, cx - 1, cy - 2, color);
        }
        ToolType::ColorPicker => {
            // Swatch showing current active color
            let swatch_color = Rgba(active_color);
            for dy in -6..=6 {
                for dx in -6..=6 {
                    put_pixel_safe(image, cx + dx, cy + dy, swatch_color);
                }
            }
            // White border around swatch
            for dx in -7..=7 {
                put_pixel_safe(image, cx + dx, cy - 7, color);
                put_pixel_safe(image, cx + dx, cy + 7, color);
            }
            for dy in -7..=7 {
                put_pixel_safe(image, cx - 7, cy + dy, color);
                put_pixel_safe(image, cx + 7, cy + dy, color);
            }
        }
        ToolType::Undo => {
            // Circular arc back-arrow
            for dx in -5..=5 {
                put_pixel_safe(image, cx + dx, cy - 4, color);
            }
            for dy in -4..=2 {
                put_pixel_safe(image, cx + 5, cy + dy, color);
            }
            // Arrow tip on left
            for i in 0..=3 {
                put_pixel_safe(image, cx - 5 + i, cy - 4 - i, color);
                put_pixel_safe(image, cx - 5 + i, cy - 4 + i, color);
            }
        }
        _ => {}
    }
}

/// Procedural pixel-art rendering of action buttons.
fn render_action_icon(image: &mut RgbaImage, action: ActionType, rect: Rect, color: Rgba<u8>) {
    let cx = rect.x as i32 + (rect.width as i32 / 2);
    let cy = rect.y as i32 + (rect.height as i32 / 2);

    match action {
        ActionType::Close => {
            // 'X' icon
            for i in -5..=5 {
                put_pixel_safe(image, cx + i, cy + i, color);
                put_pixel_safe(image, cx + i + 1, cy + i, color);
                put_pixel_safe(image, cx + i, cy - i, color);
                put_pixel_safe(image, cx + i + 1, cy - i, color);
            }
        }
        ActionType::Save => {
            // Diskette icon
            for dx in -6..=6 {
                for dy in -6..=6 {
                    let is_outer = dx == -6 || dx == 6 || dy == -6 || dy == 6;
                    let is_cutout = (dx >= -3 && dx <= 3 && dy >= -5 && dy <= -2)
                        || (dx >= -4 && dx <= 4 && dy >= 1 && dy <= 5);
                    if is_outer || is_cutout {
                        put_pixel_safe(image, cx + dx, cy + dy, color);
                    }
                }
            }
        }
        ActionType::Copy => {
            // Two overlapping clipboards
            // Back clipboard
            for dx in -3..=5 {
                put_pixel_safe(image, cx + dx, cy - 5, color);
                put_pixel_safe(image, cx + dx, cy + 3, color);
            }
            for dy in -5..=3 {
                put_pixel_safe(image, cx - 3, cy + dy, color);
                put_pixel_safe(image, cx + 5, cy + dy, color);
            }
            // Front clipboard
            for dx in -5..=3 {
                put_pixel_safe(image, cx + dx, cy - 3, color);
                put_pixel_safe(image, cx + dx, cy + 5, color);
            }
            for dy in -3..=5 {
                put_pixel_safe(image, cx - 5, cy + dy, color);
                put_pixel_safe(image, cx + 3, cy + dy, color);
            }
        }
        ActionType::Pin => {
            // Pushpin icon
            for dy in -6..=-2 {
                put_pixel_safe(image, cx, cy + dy, color);
                put_pixel_safe(image, cx - 1, cy + dy, color);
            }
            for dx in -4..=3 {
                put_pixel_safe(image, cx + dx, cy - 2, color);
            }
            for dy in -1..=5 {
                put_pixel_safe(image, cx, cy + dy, color);
            }
        }
        ActionType::Scroll => {
            // Scroll icon (vertical document with downward arrows)
            for dy in -6..=6 {
                put_pixel_safe(image, cx - 4, cy + dy, color);
                put_pixel_safe(image, cx + 4, cy + dy, color);
            }
            // Downward arrow
            for dy in -2..=3 {
                put_pixel_safe(image, cx, cy + dy, color);
            }
            put_pixel_safe(image, cx - 2, cy + 1, color);
            put_pixel_safe(image, cx - 1, cy + 2, color);
            put_pixel_safe(image, cx + 1, cy + 2, color);
            put_pixel_safe(image, cx + 2, cy + 1, color);
        }
    }
}

fn put_pixel_safe(image: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x >= 0 && y >= 0 && (x as u32) < image.width() && (y as u32) < image.height() {
        image.put_pixel(x as u32, y as u32, color);
    }
}
