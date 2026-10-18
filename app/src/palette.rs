use ecosim::{Cause, Species};
use macroquad::prelude::*;

pub const BACKGROUND: Color = Color::from_hex(0x0a0d0c);
pub const PANEL: Color = Color::new(0.055, 0.07, 0.068, 0.94);
pub const SURFACE: Color = Color::from_hex(0x161d1b);
pub const SURFACE_HOVER: Color = Color::from_hex(0x1f2926);
pub const BORDER: Color = Color::from_hex(0x26312e);
pub const TEXT: Color = Color::from_hex(0xe8eeeb);
pub const MUTED: Color = Color::from_hex(0x8b9a95);
pub const FAINT: Color = Color::from_hex(0x4a5753);
pub const ACCENT: Color = Color::from_hex(0x9be37a);
pub const GRASS_LINE: Color = Color::from_hex(0x5fbf5a);
pub const SOIL: Color = Color::from_hex(0x100f0c);
pub const BARREN: Color = Color::from_hex(0x1d1912);
pub const MEADOW_SUMMER: Color = Color::from_hex(0x2a6b34);
pub const MEADOW_WINTER: Color = Color::from_hex(0x3c5a55);
pub const HUNT: Color = Color::from_hex(0xff4d4d);
pub const COURT: Color = Color::from_hex(0xff8fd0);
pub const FLEE: Color = Color::from_hex(0xffffff);

pub fn species(s: Species) -> Color {
    match s {
        Species::Rabbit => Color::from_hex(0xf4ead2),
        Species::Deer => Color::from_hex(0xe6a95a),
        Species::Fox => Color::from_hex(0xff6a2b),
        Species::Wolf => Color::from_hex(0x7fb8ff),
    }
}

pub fn death(cause: Cause) -> Color {
    match cause {
        Cause::Predation => HUNT,
        Cause::Starvation => Color::from_hex(0x9aa3a0),
        Cause::OldAge => Color::from_hex(0x6d7f99),
    }
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

pub fn heat(t: f32) -> Color {
    const STOPS: [u32; 5] = [0x3b2a8f, 0x2c72c9, 0x2fb3a3, 0x9ad94f, 0xffe14d];
    let x = t.clamp(0.0, 1.0) * (STOPS.len() - 1) as f32;
    let i = (x as usize).min(STOPS.len() - 2);
    mix(Color::from_hex(STOPS[i]), Color::from_hex(STOPS[i + 1]), x - i as f32)
}
