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

        // 3. Render Floating Tooltip Pill for Hovered Tool
        if let Some(tool) = hover_tool {
            if let Some((_, rect)) = self.tools.iter().find(|(t, _)| *t == tool) {
                let tooltip_text = tool.tooltip();
                let pill_w = ((tooltip_text.len() as u32 * 6) + 14) as i32;
                let tx = if rect.x >= pill_w as u32 + 10 {
                    rect.x as i32 - pill_w - 6
                } else {
                    (rect.x + rect.width + 6) as i32
                };
                let ty = rect.y as i32 + ((rect.height as i32 - 22) / 2);
                crate::ui::font::draw_tooltip_pill(buffer, width, height, tooltip_text, tx, ty);
            }
        }

        // 4. Render Floating Tooltip Pill for Hovered Action
        if let Some(action) = hover_action {
            if let Some((_, rect)) = self.actions.iter().find(|(a, _)| *a == action) {
                let tooltip_text = action.tooltip();
                let pill_w = ((tooltip_text.len() as u32 * 6) + 14) as i32;
                let tx = (rect.x as i32 + (rect.width as i32 / 2)) - (pill_w / 2);
                let ty = if rect.y >= 30 {
                    rect.y as i32 - 28
                } else {
                    (rect.y + rect.height + 6) as i32
                };
                crate::ui::font::draw_tooltip_pill(buffer, width, height, tooltip_text, tx, ty);
            }
        }
    }
}

impl ToolType {
    pub fn tooltip(&self) -> &'static str {
        match self {
            ToolType::Select => "Selecionar (V)",
            ToolType::Pen => "Caneta Livre (P)",
            ToolType::Line => "Linha Reta (L)",
            ToolType::Arrow => "Seta Direcional (A)",
            ToolType::Rectangle => "Retangulo (R)",
            ToolType::Highlighter => "Marca-Texto (H)",
            ToolType::Pixelate => "Desfocar / Censurar (B)",
            ToolType::StepBadge => "Passo Numerico (S)",
            ToolType::ColorPicker => "Trocar Cor (C)",
            ToolType::Undo => "Desfazer (Ctrl+Z)",
        }
    }
}

impl ActionType {
    pub fn tooltip(&self) -> &'static str {
        match self {
            ActionType::Close => "Cancelar (Esc)",
            ActionType::Save => "Salvar Imagem (Ctrl+S)",
            ActionType::Copy => "Copiar Imagem (Ctrl+C)",
            ActionType::Pin => "Fixar na Tela (Ctrl+P)",
            ActionType::Scroll => "Captura de Rolagem (Scroll)",
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
        // High-definition angled fountain pen with nib and grip
        ToolType::Pen => {
            // Pen shaft (slanted 45 deg, 2px wide)
            for i in -5..=4 {
                put_pixel(buffer, stride, max_h, cx + i, cy - i, color);
                put_pixel(buffer, stride, max_h, cx + i + 1, cy - i, color);
            }
            // Pen grip band
            put_pixel(buffer, stride, max_h, cx - 2, cy + 1, 0xFF38BDF8);
            put_pixel(buffer, stride, max_h, cx - 1, cy + 2, 0xFF38BDF8);
            // Sharp writing nib
            put_pixel(buffer, stride, max_h, cx - 6, cy + 5, color);
            put_pixel(buffer, stride, max_h, cx - 5, cy + 6, color);
            put_pixel(buffer, stride, max_h, cx - 7, cy + 7, color); // Point tip
            // Ink droplet
            put_pixel(buffer, stride, max_h, cx - 8, cy + 8, active_color);
            // Pen top cap
            put_pixel(buffer, stride, max_h, cx + 5, cy - 5, color);
            put_pixel(buffer, stride, max_h, cx + 6, cy - 6, color);
        }

        // Clean 45-degree straight line with solid caps
        ToolType::Line => {
            for i in -6..=6 {
                put_pixel(buffer, stride, max_h, cx + i, cy - i, color);
                put_pixel(buffer, stride, max_h, cx + i, cy - i + 1, color);
            }
            put_pixel(buffer, stride, max_h, cx - 7, cy + 7, color);
            put_pixel(buffer, stride, max_h, cx + 7, cy - 7, color);
        }

        // Directional arrow with shaft and solid filled triangular arrowhead
        ToolType::Arrow => {
            // Shaft
            for i in -6..=2 {
                put_pixel(buffer, stride, max_h, cx + i, cy - i, color);
                put_pixel(buffer, stride, max_h, cx + i, cy - i + 1, color);
            }
            // Filled solid arrowhead pointing top-right
            for dy in -7i32..=-1 {
                for dx in 1i32..=7 {
                    let rel_x = dx - 1;
                    let rel_y = dy + 7;
                    if rel_x + rel_y >= 5 {
                        put_pixel(buffer, stride, max_h, cx + dx, cy + dy, color);
                    }
                }
            }
        }

        // Rounded rectangle frame with clean borders
        ToolType::Rectangle => {
            for dx in -7..=7 {
                put_pixel(buffer, stride, max_h, cx + dx, cy - 6, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy - 5, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy + 5, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy + 6, color);
            }
            for dy in -5..=5 {
                put_pixel(buffer, stride, max_h, cx - 7, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx - 6, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 6, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 7, cy + dy, color);
            }
        }

        // Slanted highlighter with chisel tip and neon stroke
        ToolType::Highlighter => {
            // Marker body
            for dy in -6..=-1 {
                for dx in -2..=5 {
                    put_pixel(buffer, stride, max_h, cx + dx, cy + dy, color);
                }
            }
            // Chisel tip (angled)
            for dy in 0..=3 {
                for dx in -5 + dy..=-1 {
                    put_pixel(buffer, stride, max_h, cx + dx, cy + dy, 0xFFFBBF24); // Neon amber
                }
            }
            // Fluorescent highlight line beneath
            for dx in -7..=3 {
                put_pixel(buffer, stride, max_h, cx + dx, cy + 6, 0xAAFACC15); // Vibrant yellow
                put_pixel(buffer, stride, max_h, cx + dx, cy + 7, 0xAAFACC15);
            }
        }

        // Universal 3x3 Privacy Mosaic / Pixelate icon with alternating shaded tiles
        ToolType::Pixelate => {
            let tile_size = 4;
            let c_dark = 0xFF475569;
            let c_mid = 0xFF94A3B8;
            let c_light = 0xFFF1F5F9;

            for ty in 0..3 {
                for tx in 0..3 {
                    let tile_color = match (tx + ty) % 3 {
                        0 => c_light,
                        1 => c_mid,
                        _ => c_dark,
                    };
                    let start_x = cx - 6 + (tx * tile_size);
                    let start_y = cy - 6 + (ty * tile_size);
                    for py in 0..3 {
                        for px in 0..3 {
                            put_pixel(buffer, stride, max_h, start_x + px, start_y + py, tile_color);
                        }
                    }
                }
            }
        }

        // Circular badge with bold, centered numeral "1"
        ToolType::StepBadge => {
            // Filled circle badge
            for dy in -7..=7 {
                for dx in -7..=7 {
                    let d_sq = dx * dx + dy * dy;
                    if d_sq <= 49 {
                        put_pixel(buffer, stride, max_h, cx + dx, cy + dy, 0xFF3B82F6); // Blue badge
                    }
                    if d_sq >= 40 && d_sq <= 52 {
                        put_pixel(buffer, stride, max_h, cx + dx, cy + dy, color);
                    }
                }
            }
            // Centered sharp "1"
            for dy in -4..=4 {
                put_pixel(buffer, stride, max_h, cx, cy + dy, 0xFFFFFFFF);
                put_pixel(buffer, stride, max_h, cx + 1, cy + dy, 0xFFFFFFFF);
            }
            put_pixel(buffer, stride, max_h, cx - 1, cy - 3, 0xFFFFFFFF);
            put_pixel(buffer, stride, max_h, cx - 2, cy - 2, 0xFFFFFFFF);
            for dx in -2..=3 {
                put_pixel(buffer, stride, max_h, cx + dx, cy + 4, 0xFFFFFFFF);
            }
        }

        // Color palette with live active color preview disc
        ToolType::ColorPicker => {
            // Outer round bezel
            for dy in -7..=7 {
                for dx in -7..=7 {
                    let dist = dx * dx + dy * dy;
                    if dist <= 50 {
                        // Inner active color disc
                        let c = if dist <= 28 { active_color } else { color };
                        put_pixel(buffer, stride, max_h, cx + dx, cy + dy, c);
                    }
                }
            }
            // White center reflection
            put_pixel(buffer, stride, max_h, cx - 2, cy - 2, 0xFFFFFFFF);
            put_pixel(buffer, stride, max_h, cx - 1, cy - 3, 0xFFFFFFFF);
        }

        // Curved undo return arrow
        ToolType::Undo => {
            // Arc
            for dx in -4..=4 {
                put_pixel(buffer, stride, max_h, cx + dx, cy - 5, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy - 4, color);
            }
            for dy in -4..=2 {
                put_pixel(buffer, stride, max_h, cx + 4, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 5, cy + dy, color);
            }
            // Left arrowhead
            for i in 0..=4 {
                put_pixel(buffer, stride, max_h, cx - 4 + i, cy - 5 - i, color);
                put_pixel(buffer, stride, max_h, cx - 4 + i, cy - 5 + i, color);
            }
            for dy in -7..=-3 {
                put_pixel(buffer, stride, max_h, cx - 4, cy + dy, color);
            }
        }
        _ => {}
    }
}

fn render_action_icon(buffer: &mut [u32], stride: u32, max_h: u32, action: ActionType, rect: Rect, color: u32) {
    let cx = rect.x as i32 + (rect.width as i32 / 2);
    let cy = rect.y as i32 + (rect.height as i32 / 2);

    match action {
        // Thick clean 'X' cancel icon
        ActionType::Close => {
            for i in -6..=6 {
                put_pixel(buffer, stride, max_h, cx + i, cy + i, color);
                put_pixel(buffer, stride, max_h, cx + i + 1, cy + i, color);
                put_pixel(buffer, stride, max_h, cx + i, cy - i, color);
                put_pixel(buffer, stride, max_h, cx + i + 1, cy - i, color);
            }
        }

        // Save tray / disk icon with down arrow
        ActionType::Save => {
            // Tray base & walls
            for dx in -7..=7 {
                put_pixel(buffer, stride, max_h, cx + dx, cy + 6, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy + 7, color);
            }
            for dy in 2..=7 {
                put_pixel(buffer, stride, max_h, cx - 7, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx - 6, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 6, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 7, cy + dy, color);
            }
            // Downward arrow entering tray
            for dy in -6..=1 {
                put_pixel(buffer, stride, max_h, cx, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx - 1, cy + dy, color);
            }
            for i in 0..=4 {
                put_pixel(buffer, stride, max_h, cx - i, cy + 1 - i, color);
                put_pixel(buffer, stride, max_h, cx + i - 1, cy + 1 - i, color);
            }
        }

        // Two overlapping copy documents
        ActionType::Copy => {
            // Back sheet (top-left)
            for dx in -6..=2 {
                put_pixel(buffer, stride, max_h, cx + dx, cy - 6, color);
            }
            for dy in -6..=2 {
                put_pixel(buffer, stride, max_h, cx - 6, cy + dy, color);
            }
            // Front sheet (bottom-right)
            for dx in -2..=6 {
                put_pixel(buffer, stride, max_h, cx + dx, cy - 2, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy + 6, color);
            }
            for dy in -2..=6 {
                put_pixel(buffer, stride, max_h, cx - 2, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 6, cy + dy, color);
            }
            // Document text lines inside front sheet
            for dx in 0..=4 {
                put_pixel(buffer, stride, max_h, cx + dx, cy + 1, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy + 3, color);
            }
        }

        // Clean pushpin icon
        ActionType::Pin => {
            // Round pin head
            for dy in -7..=-4 {
                for dx in -4..=3 {
                    put_pixel(buffer, stride, max_h, cx + dx, cy + dy, color);
                }
            }
            // Neck
            for dy in -3..=-1 {
                for dx in -2..=1 {
                    put_pixel(buffer, stride, max_h, cx + dx, cy + dy, color);
                }
            }
            // Shoulder ledge
            for dx in -5..=4 {
                put_pixel(buffer, stride, max_h, cx + dx, cy, color);
            }
            // Sharp needle
            for dy in 1..=6 {
                put_pixel(buffer, stride, max_h, cx - 1, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx, cy + dy, color);
            }
            put_pixel(buffer, stride, max_h, cx, cy + 7, color);
        }

        // Document with vertical scrolling flow indicator
        ActionType::Scroll => {
            // Document outline
            for dx in -5..=5 {
                put_pixel(buffer, stride, max_h, cx + dx, cy - 7, color);
                put_pixel(buffer, stride, max_h, cx + dx, cy + 7, color);
            }
            for dy in -7..=7 {
                put_pixel(buffer, stride, max_h, cx - 5, cy + dy, color);
                put_pixel(buffer, stride, max_h, cx + 5, cy + dy, color);
            }
            // Vertical flow arrow
            for dy in -4..=2 {
                put_pixel(buffer, stride, max_h, cx, cy + dy, 0xFF10B981); // Emerald flow
            }
            for i in 1..=3 {
                put_pixel(buffer, stride, max_h, cx - i, cy + 2 - i, 0xFF10B981);
                put_pixel(buffer, stride, max_h, cx + i, cy + 2 - i, 0xFF10B981);
            }
        }
    }
}

#[inline(always)]
fn put_pixel(buffer: &mut [u32], stride: u32, max_h: u32, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 && (x as u32) < stride && (y as u32) < max_h {
        buffer[(y as u32 * stride + x as u32) as usize] = color;
    }
}
