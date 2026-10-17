use crate::core::filters::Rect;

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

        let v_x = if selection.x + selection.width + 6 + v_width <= screen_w {
            selection.x + selection.width + 6
        } else if selection.width >= v_width + 10 {
            selection.x + selection.width.saturating_sub(v_width + 6)
        } else {
            selection.x.saturating_sub(v_width + 6)
        };

        let max_v_y = screen_h.saturating_sub(v_height + 6);
        let v_y = if max_v_y >= 6 {
            selection.y.clamp(6, max_v_y)
        } else {
            max_v_y
        };
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

        let h_y = if selection.y + selection.height + 6 + h_height <= screen_h {
            selection.y + selection.height + 6
        } else if selection.height >= h_height + 10 {
            selection.y + selection.height.saturating_sub(h_height + 6)
        } else {
            selection.y.saturating_sub(h_height + 6)
        };

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

    pub fn hit_test_tool(&self, px: i32, py: i32) -> Option<ToolType> {
        for (tool, rect) in &self.tools {
            if rect.contains(px, py) {
                return Some(*tool);
            }
        }
        None
    }

    pub fn hit_test_action(&self, px: i32, py: i32) -> Option<ActionType> {
        for (action, rect) in &self.actions {
            if rect.contains(px, py) {
                return Some(*action);
            }
        }
        None
    }

    /// Renders both toolbars directly onto the raw 32-bit screen buffer (Zero allocations).
    pub fn render_to_buffer(
        &self,
        buffer: &mut [u32],
        width: u32,
        height: u32,
        active_tool: ToolType,
        hover_tool: Option<ToolType>,
        hover_action: Option<ActionType>,
        active_color: u32,
    ) {
        let panel_bg: u32 = 0xFF1A1B26;
        let panel_border: u32 = 0xFF414868;
        let btn_hover_bg: u32 = 0xFF292E42;
        let btn_active_bg: u32 = 0xFF3B82F6; // Vibrant blue
        let icon_color: u32 = 0xFFE0E6F5;
        let icon_muted: u32 = 0xFF9AA5CE;

        // 1. Draw Vertical Panel
        draw_rect_to_buffer(buffer, width, height, self.vertical_rect, panel_bg, panel_border);
        for (tool, rect) in &self.tools {
            let is_active = *tool == active_tool;
            let is_hover = hover_tool == Some(*tool);

            if is_active {
                draw_rect_to_buffer(buffer, width, height, *rect, btn_active_bg, btn_active_bg);
            } else if is_hover {
                draw_rect_to_buffer(buffer, width, height, *rect, btn_hover_bg, panel_border);
            }

            let color = if is_active { 0xFFFFFFFF } else if is_hover { icon_color } else { icon_muted };
            render_tool_icon(buffer, width, height, *tool, *rect, color, active_color);
        }

        // 2. Draw Horizontal Panel
        draw_rect_to_buffer(buffer, width, height, self.horizontal_rect, panel_bg, panel_border);
        for (action, rect) in &self.actions {
            let is_hover = hover_action == Some(*action);

            if is_hover {
                let hover_color = match action {
                    ActionType::Close => 0xFFEF4444, // Red
                    ActionType::Copy => 0xFF10B981,  // Emerald
                    _ => btn_hover_bg,
                };
                draw_rect_to_buffer(buffer, width, height, *rect, hover_color, panel_border);
            }

            render_action_icon(buffer, width, height, *action, *rect, if is_hover { 0xFFFFFFFF } else { icon_color });
        }
    }
}

fn draw_rect_to_buffer(buffer: &mut [u32], stride: u32, max_h: u32, rect: Rect, fill: u32, border: u32) {
    let x_end = (rect.x + rect.width).min(stride);
    let y_end = (rect.y + rect.height).min(max_h);

    for y in rect.y..y_end {
        let row_offset = (y * stride) as usize;
        for x in rect.x..x_end {
            let is_border = x == rect.x || x == x_end - 1 || y == rect.y || y == y_end - 1;
            buffer[row_offset + x as usize] = if is_border { border } else { fill };
        }
    }
}

fn render_tool_icon(buffer: &mut [u32], stride: u32, max_h: u32, tool: ToolType, rect: Rect, color: u32, active_color: u32) {
    let cx = rect.x as i32 + (rect.width as i32 / 2);
    let cy = rect.y as i32 + (rect.height as i32 / 2);

    match tool {
        ToolType::Pen => {
            for i in -6..=6 {
                put_pixel(buffer, stride, max_h, cx + i, cy - i, color);
                put_pixel(buffer, stride, max_h, cx + i + 1, cy - i, color);
            }
            put_pixel(buffer, stride, max_h, cx - 6, cy + 6, color);
        }
        ToolType::Line => {
            for dx in -7..=7 {
                put_pixel(buffer, stride, max_h, cx + dx, cy, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy + 1, color);
            }
        }
        ToolType::Arrow => {
            for i in -6..=6 {
                put_pixel(buffer, stride, max_h, cx + i, cy - i, color);
            }
            for i in 0..=5 {
                put_pixel(buffer, stride, max_h, cx + 6 - i, cy - 6, color);
                put_pixel(buffer, stride, max_h, cx + 6, cy - 6 + i, color);
            }
        }
        ToolType::Rectangle => {
            for dx in -6..=6 {
                put_pixel(buffer, stride, max_h, cx + dx, cy - 6, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy + 6, color);
            }
            for dy in -6..=6 {
                put_pixel(buffer, stride, max_h, cx - 6, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 6, cy + dy, color);
            }
        }
        ToolType::Highlighter => {
            for dx in -5..=5 {
                for dy in -3..=3 {
                    put_pixel(buffer, stride, max_h, cx + dx, cy + dy, color);
                }
            }
        }
        ToolType::Pixelate => {
            for dy in -6..=6 {
                for dx in -6..=6 {
                    if ((dx + 6) / 4 + (dy + 6) / 4) % 2 == 0 {
                        put_pixel(buffer, stride, max_h, cx + dx, cy + dy, color);
                    }
                }
            }
        }
        ToolType::StepBadge => {
            for dy in -6..=6 {
                for dx in -6..=6 {
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq >= 25 && dist_sq <= 40 {
                        put_pixel(buffer, stride, max_h, cx + dx, cy + dy, color);
                    }
                }
            }
            for dy in -3..=3 {
                put_pixel(buffer, stride, max_h, cx, cy + dy, color);
            }
            put_pixel(buffer, stride, max_h, cx - 1, cy - 2, color);
        }
        ToolType::ColorPicker => {
            for dy in -5..=5 {
                for dx in -5..=5 {
                    put_pixel(buffer, stride, max_h, cx + dx, cy + dy, active_color);
                }
            }
            for dx in -6..=6 {
                put_pixel(buffer, stride, max_h, cx + dx, cy - 6, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy + 6, color);
            }
            for dy in -6..=6 {
                put_pixel(buffer, stride, max_h, cx - 6, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 6, cy + dy, color);
            }
        }
        ToolType::Undo => {
            for dx in -5..=5 {
                put_pixel(buffer, stride, max_h, cx + dx, cy - 4, color);
            }
            for dy in -4..=2 {
                put_pixel(buffer, stride, max_h, cx + 5, cy + dy, color);
            }
            for i in 0..=3 {
                put_pixel(buffer, stride, max_h, cx - 5 + i, cy - 4 - i, color);
                put_pixel(buffer, stride, max_h, cx - 5 + i, cy - 4 + i, color);
            }
        }
        _ => {}
    }
}

fn render_action_icon(buffer: &mut [u32], stride: u32, max_h: u32, action: ActionType, rect: Rect, color: u32) {
    let cx = rect.x as i32 + (rect.width as i32 / 2);
    let cy = rect.y as i32 + (rect.height as i32 / 2);

    match action {
        ActionType::Close => {
            for i in -5..=5 {
                put_pixel(buffer, stride, max_h, cx + i, cy + i, color);
                put_pixel(buffer, stride, max_h, cx + i + 1, cy + i, color);
                put_pixel(buffer, stride, max_h, cx + i, cy - i, color);
                put_pixel(buffer, stride, max_h, cx + i + 1, cy - i, color);
            }
        }
        ActionType::Save => {
            for dx in -6..=6 {
                for dy in -6..=6 {
                    let is_outer = dx == -6 || dx == 6 || dy == -6 || dy == 6;
                    let is_cutout = (dx >= -3 && dx <= 3 && dy >= -5 && dy <= -2)
                        || (dx >= -4 && dx <= 4 && dy >= 1 && dy <= 5);
                    if is_outer || is_cutout {
                        put_pixel(buffer, stride, max_h, cx + dx, cy + dy, color);
                    }
                }
            }
        }
        ActionType::Copy => {
            for dx in -3..=5 {
                put_pixel(buffer, stride, max_h, cx + dx, cy - 5, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy + 3, color);
            }
            for dy in -5..=3 {
                put_pixel(buffer, stride, max_h, cx - 3, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 5, cy + dy, color);
            }
            for dx in -5..=3 {
                put_pixel(buffer, stride, max_h, cx + dx, cy - 3, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy + 5, color);
            }
            for dy in -3..=5 {
                put_pixel(buffer, stride, max_h, cx - 5, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 3, cy + dy, color);
            }
        }
        ActionType::Pin => {
            for dy in -6..=-2 {
                put_pixel(buffer, stride, max_h, cx, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx - 1, cy + dy, color);
            }
            for dx in -4..=3 {
                put_pixel(buffer, stride, max_h, cx + dx, cy - 2, color);
            }
            for dy in -1..=5 {
                put_pixel(buffer, stride, max_h, cx, cy + dy, color);
            }
        }
        ActionType::Scroll => {
            for dy in -6..=6 {
                put_pixel(buffer, stride, max_h, cx - 4, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 4, cy + dy, color);
            }
            for dy in -2..=3 {
                put_pixel(buffer, stride, max_h, cx, cy + dy, color);
            }
            put_pixel(buffer, stride, max_h, cx - 2, cy + 1, color);
            put_pixel(buffer, stride, max_h, cx - 1, cy + 2, color);
            put_pixel(buffer, stride, max_h, cx + 1, cy + 2, color);
            put_pixel(buffer, stride, max_h, cx + 2, cy + 1, color);
        }
    }
}

#[inline(always)]
fn put_pixel(buffer: &mut [u32], stride: u32, max_h: u32, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 && (x as u32) < stride && (y as u32) < max_h {
        buffer[(y as u32 * stride + x as u32) as usize] = color;
    }
}
