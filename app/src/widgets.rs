use crate::palette::*;
use crate::scene::screen_space;
use macroquad::prelude::*;

pub fn ui_scale() -> f32 {
    (screen_height() / 940.0).clamp(0.62, 1.3)
}

pub struct Ui {
    regular: Font,
    bold: Font,
    pub scale: f32,
    pub mouse: Vec2,
    pub pressed: bool,
    pub over_ui: bool,
    active: Option<&'static str>,
}

impl Ui {
    pub fn new() -> Self {
        let load = |bytes: &[u8]| load_ttf_font_from_bytes(bytes).expect("bundled font is valid");
        Self {
            regular: load(include_bytes!("../assets/JetBrainsMono-Regular.ttf")),
            bold: load(include_bytes!("../assets/JetBrainsMono-Bold.ttf")),
            scale: 1.0,
            mouse: Vec2::ZERO,
            pressed: false,
            over_ui: false,
            active: None,
        }
    }

    pub fn begin(&mut self) {
        self.scale = ui_scale();
        self.mouse = Vec2::from(mouse_position()) / self.scale;
        self.pressed = is_mouse_button_pressed(MouseButton::Left);
        if !is_mouse_button_down(MouseButton::Left) {
            self.active = None;
        }
        self.over_ui = self.active.is_some();
        set_camera(&screen_space(Rect::new(0.0, 0.0, self.size().x, self.size().y)));
    }

    pub fn size(&self) -> Vec2 {
        vec2(screen_width(), screen_height()) / self.scale
    }

    pub fn claim(&mut self, rect: Rect) {
        if rect.contains(self.mouse) {
            self.over_ui = true;
        }
    }

    fn params(&self, bold: bool, size: f32, color: Color) -> TextParams<'_> {
        let physical = (size * self.scale).round().max(1.0);
        TextParams {
            font: Some(if bold { &self.bold } else { &self.regular }),
            font_size: physical as u16,
            font_scale: 1.0 / self.scale,
            color,
            ..Default::default()
        }
    }

    pub fn text(&self, text: &str, x: f32, y: f32, size: f32, color: Color) -> f32 {
        draw_text_ex(text, x, y, self.params(false, size, color)).width
    }

    pub fn bold(&self, text: &str, x: f32, y: f32, size: f32, color: Color) -> f32 {
        draw_text_ex(text, x, y, self.params(true, size, color)).width
    }

    pub fn width(&self, text: &str, size: f32) -> f32 {
        let physical = (size * self.scale).round().max(1.0) as u16;
        measure_text(text, Some(&self.regular), physical, 1.0 / self.scale).width
    }

    pub fn text_right(&self, text: &str, right: f32, y: f32, size: f32, color: Color) {
        self.text(text, right - self.width(text, size), y, size, color);
    }

    pub fn button(&mut self, rect: Rect, label: &str, selected: bool) -> bool {
        let hover = rect.contains(self.mouse) && self.active.is_none();
        let fill = match (selected, hover) {
            (true, _) => mix(SURFACE, ACCENT, 0.22),
            (false, true) => SURFACE_HOVER,
            (false, false) => SURFACE,
        };
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            1.0,
            if selected { ACCENT } else { BORDER },
        );
        let color = if selected { TEXT } else { MUTED };
        let size = 13.0;
        self.text(
            label,
            rect.center().x - self.width(label, size) / 2.0,
            rect.center().y + 4.5,
            size,
            color,
        );
        hover && self.pressed
    }

    pub fn slider(
        &mut self,
        id: &'static str,
        rect: Rect,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
        shown: &str,
    ) {
        self.text(label, rect.x, rect.y + 11.0, 13.0, MUTED);
        self.text_right(shown, rect.right(), rect.y + 11.0, 13.0, TEXT);
        let track = Rect::new(rect.x, rect.y + 20.0, rect.w, 4.0);
        let hit = Rect::new(rect.x - 4.0, rect.y + 12.0, rect.w + 8.0, 20.0);
        if self.pressed && hit.contains(self.mouse) {
            self.active = Some(id);
            self.over_ui = true;
        }
        if self.active == Some(id) {
            let t = ((self.mouse.x - track.x) / track.w).clamp(0.0, 1.0);
            *value = range.0 + (range.1 - range.0) * t;
        }
        let t = ((*value - range.0) / (range.1 - range.0)).clamp(0.0, 1.0);
        draw_rectangle(track.x, track.y, track.w, track.h, SURFACE_HOVER);
        draw_rectangle(track.x, track.y, track.w * t, track.h, mix(FAINT, ACCENT, 0.7));
        draw_circle(
            track.x + track.w * t,
            track.y + 2.0,
            6.0,
            if self.active == Some(id) { TEXT } else { ACCENT },
        );
    }
}
