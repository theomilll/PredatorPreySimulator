use crate::palette::*;
use ecosim::{Animal, Cause, Event, State, Trait, WORLD_SIZE, World};
use macroquad::prelude::*;
use std::collections::VecDeque;
use std::f32::consts::TAU;

pub struct View {
    pub center: Vec2,
    pub zoom: f32,
}

impl View {
    pub fn fit(panel: f32) -> Self {
        let screen = vec2(screen_width(), screen_height());
        let room = vec2((screen.x - panel).max(200.0), screen.y);
        let zoom = (room.x / WORLD_SIZE.x).min(room.y / WORLD_SIZE.y) * 0.95;
        Self {
            center: WORLD_SIZE / 2.0 + vec2(panel / 2.0 / zoom, 0.0),
            zoom,
        }
    }

    fn half_extent(&self) -> Vec2 {
        vec2(screen_width(), screen_height()) / self.zoom / 2.0
    }

    pub fn camera(&self) -> Camera2D {
        let half = self.half_extent();
        screen_space(Rect::new(
            self.center.x - half.x,
            self.center.y - half.y,
            half.x * 2.0,
            half.y * 2.0,
        ))
    }

    pub fn to_world(&self, screen: Vec2) -> Vec2 {
        self.center + (screen - vec2(screen_width(), screen_height()) / 2.0) / self.zoom
    }

    pub fn zoom_at(&mut self, screen: Vec2, factor: f32) {
        let anchor = self.to_world(screen);
        self.zoom = (self.zoom * factor).clamp(0.25, 14.0);
        self.center += anchor - self.to_world(screen);
    }
}

pub fn screen_space(rect: Rect) -> Camera2D {
    let mut camera = Camera2D::from_display_rect(rect);
    camera.zoom.y = -camera.zoom.y;
    camera
}

pub struct Terrain {
    image: Image,
    texture: Texture2D,
}

impl Terrain {
    pub fn new(world: &World) -> Self {
        let grass = world.grass();
        let image = Image::gen_image_color(grass.cols() as u16, grass.rows() as u16, BACKGROUND);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Linear);
        Self { image, texture }
    }

    pub fn draw(&mut self, world: &World) {
        let grass = world.grass();
        let winter = 0.5 - 0.5 * (TAU * world.season()).sin();
        let meadow = mix(MEADOW_SUMMER, MEADOW_WINTER, winter * world.params.seasons.min(1.0));
        for ((px, b), k) in self
            .image
            .bytes
            .chunks_exact_mut(4)
            .zip(grass.biomass())
            .zip(grass.capacity())
        {
            let c = mix(mix(SOIL, BARREN, *k), meadow, b.powf(0.8));
            px.copy_from_slice(&[(c.r * 255.0) as u8, (c.g * 255.0) as u8, (c.b * 255.0) as u8, 255]);
        }
        self.texture.update(&self.image);
        draw_texture_ex(
            &self.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(WORLD_SIZE),
                ..Default::default()
            },
        );
        draw_rectangle_lines(0.0, 0.0, WORLD_SIZE.x, WORLD_SIZE.y, 2.0, BORDER);
    }
}

pub struct Overlay {
    pub lens: Option<Trait>,
    pub vision: bool,
    pub intent: bool,
    pub selected: Option<u32>,
}

pub fn draw_animals(world: &World, zoom: f32, overlay: &Overlay) {
    let px = 1.0 / zoom;
    for a in world.animals() {
        let mine = overlay.selected == Some(a.id);
        if overlay.vision || mine {
            draw_circle_lines(
                a.pos.x,
                a.pos.y,
                a.vision(),
                px,
                species(a.species).with_alpha(if mine { 0.5 } else { 0.1 }),
            );
        }
        if let Some(target) = a.focus.filter(|_| overlay.intent || mine) {
            let color = match a.state {
                State::Hunt => HUNT,
                State::Court => COURT,
                _ => FLEE,
            };
            draw_line(
                a.pos.x,
                a.pos.y,
                target.x,
                target.y,
                px * 1.2,
                color.with_alpha(if mine { 0.9 } else { 0.35 }),
            );
        }
    }
    for a in world.animals() {
        draw_animal(a, px, overlay.lens);
    }
    if let Some(a) = overlay.selected.and_then(|id| world.animal(id)) {
        let pulse = 1.0 + 0.15 * (get_time() as f32 * 6.0).sin();
        draw_circle_lines(a.pos.x, a.pos.y, (a.radius() + 5.0 * px) * pulse + 3.0, px * 2.0, TEXT);
    }
}

fn draw_animal(a: &Animal, px: f32, lens: Option<Trait>) {
    let r = (a.radius() * if a.species.is_predator() { 1.6 } else { 1.2 }).max(2.0 * px);
    let vigor = (a.energy / a.max_energy()).clamp(0.0, 1.0);
    let base = lens.map_or(species(a.species), |t| heat((a.genes.get(t) - 0.6) / 0.8));
    let body = mix(mix(BACKGROUND, base, 0.35), base, vigor);
    let edge = match a.state {
        State::Hunt => HUNT,
        State::Flee => FLEE,
        _ => BACKGROUND.with_alpha(0.75),
    };
    let outline = 1.4 * px;
    if a.species.is_predator() {
        let dir = a.vel.try_normalize().unwrap_or(Vec2::X);
        let points = [
            dir * 1.7,
            Vec2::from_angle(2.45).rotate(dir),
            Vec2::from_angle(-2.45).rotate(dir),
        ];
        let [tip, left, right] = points.map(|p| a.pos + p * (r + outline));
        draw_triangle(tip, left, right, edge);
        let [tip, left, right] = points.map(|p| a.pos + p * r);
        draw_triangle(tip, left, right, body);
    } else {
        draw_poly(a.pos.x, a.pos.y, 12, r + outline, 0.0, edge);
        draw_poly(a.pos.x, a.pos.y, 12, r, 0.0, body);
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Birth,
    Kill,
    Fade,
}

struct Effect {
    pos: Vec2,
    color: Color,
    kind: Kind,
    age: f32,
    seed: f32,
}

#[derive(Default)]
pub struct Effects(VecDeque<Effect>);

impl Effects {
    const CAP: usize = 3_000;

    pub fn push(&mut self, event: &Event) {
        let (pos, color, kind) = match *event {
            Event::Birth { pos, species: s } => (pos, species(s), Kind::Birth),
            Event::Death {
                pos,
                cause: Cause::Predation,
                ..
            } => (pos, HUNT, Kind::Kill),
            Event::Death { pos, cause, .. } => (pos, death(cause), Kind::Fade),
        };
        if self.0.len() >= Self::CAP {
            self.0.pop_front();
        }
        let seed = (pos.x * 12.9898 + pos.y * 78.233).sin().fract().abs() * TAU;
        self.0.push_back(Effect {
            pos,
            color,
            kind,
            age: 0.0,
            seed,
        });
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn update(&mut self, dt: f32) {
        self.0.iter_mut().for_each(|e| e.age += dt);
        self.0.retain(|e| e.age < 0.9);
    }

    pub fn draw(&self, zoom: f32) {
        let px = 1.0 / zoom;
        for e in &self.0 {
            let t = e.age / 0.9;
            let fade = (1.0 - t).powi(2);
            match e.kind {
                Kind::Birth => draw_circle_lines(
                    e.pos.x,
                    e.pos.y,
                    3.0 + 10.0 * t,
                    px * 1.5,
                    e.color.with_alpha(0.8 * fade),
                ),
                Kind::Kill => {
                    draw_circle_lines(
                        e.pos.x,
                        e.pos.y,
                        4.0 + 16.0 * t,
                        px * 2.0,
                        e.color.with_alpha(0.9 * fade),
                    );
                    for k in 0..6 {
                        let dir = Vec2::from_angle(e.seed + k as f32 * TAU / 6.0);
                        let p = e.pos + dir * (3.0 + 14.0 * t.sqrt());
                        draw_circle(p.x, p.y, (1.6 * (1.0 - t)).max(px), e.color.with_alpha(fade));
                    }
                }
                Kind::Fade => draw_circle(e.pos.x, e.pos.y, 3.0, e.color.with_alpha(0.6 * fade)),
            }
        }
    }
}
