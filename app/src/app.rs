use crate::hud::{self, Action, Controls, PANEL_WIDTH, SPEEDS, Tool};
use crate::palette::BACKGROUND;
use crate::scene::{self, Effects, Overlay, Terrain, View};
use crate::widgets::{Ui, ui_scale};
use ecosim::{Params, Rng, Trait, World};
use macroquad::prelude::*;

const TICKS_PER_SECOND: f32 = 60.0;
const MAX_TICKS_PER_FRAME: f32 = 48.0;
const SPAWN_GROUP: usize = 8;
const FRAME_BUDGET: f64 = 0.012;

pub struct App {
    world: World,
    controls: Controls,
    view: View,
    terrain: Terrain,
    effects: Effects,
    ui: Ui,
    selected: Option<u32>,
    press: Option<Vec2>,
    last_mouse: Vec2,
    dragged: bool,
    budget: f32,
    sim_ms: f32,
}

impl App {
    pub fn new(seed: u64) -> Self {
        let world = World::new(seed, Params::default());
        let ui = Ui::new();
        Self {
            terrain: Terrain::new(&world),
            view: View::fit(PANEL_WIDTH * ui_scale()),
            world,
            controls: Controls {
                paused: false,
                speed: 2,
                tool: Tool::Inspect,
                chart: Trait::Speed,
                lens: false,
                vision: false,
                intent: false,
                follow: false,
            },
            effects: Effects::default(),
            ui,
            selected: None,
            press: None,
            last_mouse: Vec2::ZERO,
            dragged: false,
            budget: 0.0,
            sim_ms: 0.0,
        }
    }

    pub fn frame(&mut self) {
        self.keyboard();
        if !self.controls.paused {
            self.budget += get_frame_time().min(0.1) * TICKS_PER_SECOND * SPEEDS[self.controls.speed];
            let ticks = self.budget.floor().min(MAX_TICKS_PER_FRAME);
            self.budget = (self.budget - ticks).min(1.0);
            self.advance(ticks as usize);
        }
        self.effects.update(get_frame_time());
        if self.controls.follow
            && let Some(a) = self.selected.and_then(|id| self.world.animal(id))
        {
            self.view.center = self.view.center.lerp(a.pos, 0.12);
        }

        clear_background(BACKGROUND);
        set_camera(&self.view.camera());
        self.terrain.draw(&self.world);
        let overlay = Overlay {
            lens: self.controls.lens.then_some(self.controls.chart),
            vision: self.controls.vision,
            intent: self.controls.intent,
            selected: self.selected,
        };
        scene::draw_animals(&self.world, self.view.zoom, &overlay);
        self.effects.draw(self.view.zoom);

        self.ui.begin();
        let action = hud::panel(&mut self.ui, &mut self.world, &mut self.controls, self.sim_ms);
        match self.selected.and_then(|id| self.world.animal(id)) {
            Some(a) => hud::inspector(&mut self.ui, a, self.controls.follow),
            None => {
                self.selected = None;
                self.controls.follow = false;
            }
        }
        hud::hints(&self.ui);
        match action {
            Some(Action::Reset) => self.restart(self.world.seed()),
            Some(Action::NewSeed) => self.restart(fresh_seed()),
            Some(Action::Step) => self.advance(1),
            None => {}
        }
        self.mouse();
    }

    fn advance(&mut self, ticks: usize) {
        if ticks == 0 {
            return;
        }
        let start = get_time();
        let mut done = 0;
        while done < ticks && (done == 0 || get_time() - start < FRAME_BUDGET) {
            self.world.step();
            self.world.events().iter().for_each(|e| self.effects.push(e));
            done += 1;
        }
        let ms = ((get_time() - start) * 1000.0) as f32 / done as f32;
        self.sim_ms += (ms - self.sim_ms) * 0.1;
    }

    fn restart(&mut self, seed: u64) {
        self.world = World::new(seed, self.world.params);
        self.terrain = Terrain::new(&self.world);
        self.effects.clear();
        self.selected = None;
        self.controls.follow = false;
    }

    fn keyboard(&mut self) {
        for key in get_keys_pressed() {
            let c = &mut self.controls;
            match key {
                KeyCode::Space => c.paused = !c.paused,
                KeyCode::Right | KeyCode::Equal => c.speed = (c.speed + 1).min(SPEEDS.len() - 1),
                KeyCode::Left | KeyCode::Minus => c.speed = c.speed.saturating_sub(1),
                KeyCode::V => c.vision = !c.vision,
                KeyCode::I => c.intent = !c.intent,
                KeyCode::L => c.lens = !c.lens,
                KeyCode::F => c.follow = !c.follow && self.selected.is_some(),
                KeyCode::Key1 | KeyCode::Key2 | KeyCode::Key3 | KeyCode::Key4 | KeyCode::Key5 | KeyCode::Key6 => {
                    c.tool = Tool::ALL[key as usize - KeyCode::Key1 as usize];
                }
                KeyCode::Escape => {
                    c.tool = Tool::Inspect;
                    self.selected = None;
                }
                KeyCode::Period => self.advance(1),
                KeyCode::R => self.restart(self.world.seed()),
                KeyCode::N => self.restart(fresh_seed()),
                _ => {}
            }
        }
    }

    fn mouse(&mut self) {
        let screen = Vec2::from(mouse_position());
        let delta = screen - std::mem::replace(&mut self.last_mouse, screen);
        let wheel = mouse_wheel().1;
        if wheel != 0.0 && !self.ui.over_ui {
            self.view.zoom_at(screen, 1.12f32.powf(wheel.signum()));
        }
        if is_mouse_button_down(MouseButton::Right) || is_mouse_button_down(MouseButton::Middle) {
            self.pan(delta);
        }
        if is_mouse_button_pressed(MouseButton::Left) && !self.ui.over_ui {
            self.press = Some(screen);
            self.dragged = false;
            if let Tool::Spawn(species) = self.controls.tool {
                self.world.spawn_group(species, self.view.to_world(screen), SPAWN_GROUP);
            }
        }
        let Some(origin) = self.press else { return };
        if !is_mouse_button_down(MouseButton::Left) {
            if self.controls.tool == Tool::Inspect && !self.dragged {
                self.select(self.view.to_world(screen));
            }
            self.press = None;
        } else if self.controls.tool == Tool::Meadow {
            self.world.fertilize(self.view.to_world(screen), 36.0);
        } else if self.controls.tool == Tool::Inspect && (self.dragged || origin.distance(screen) > 4.0) {
            self.dragged = true;
            self.pan(delta);
        }
    }

    fn pan(&mut self, delta: Vec2) {
        self.view.center -= delta / self.view.zoom;
        self.controls.follow = false;
    }

    fn select(&mut self, at: Vec2) {
        let reach = 14.0 / self.view.zoom;
        self.selected = self
            .world
            .animals()
            .iter()
            .map(|a| (a.id, a.pos.distance(at)))
            .filter(|(_, d)| *d < reach)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id);
    }
}

fn fresh_seed() -> u64 {
    let now = (macroquad::miniquad::date::now() * 1000.0) as u64;
    Rng::new(now).next_u64() % 100_000
}
