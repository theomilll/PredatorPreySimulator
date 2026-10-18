use crate::palette::*;
use crate::widgets::Ui;
use ecosim::{Animal, Cause, Genes, Sample, Sex, Species, State, Trait, World, YEAR};
use macroquad::prelude::*;

pub const PANEL_WIDTH: f32 = 368.0;
pub const SPEEDS: [f32; 6] = [0.5, 1.0, 2.0, 4.0, 8.0, 16.0];
const PAD: f32 = 18.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Inspect,
    Spawn(Species),
    Meadow,
}

impl Tool {
    pub const ALL: [Tool; 6] = [
        Tool::Inspect,
        Tool::Spawn(Species::Rabbit),
        Tool::Spawn(Species::Deer),
        Tool::Spawn(Species::Fox),
        Tool::Spawn(Species::Wolf),
        Tool::Meadow,
    ];

    fn label(self) -> &'static str {
        match self {
            Tool::Inspect => "Look",
            Tool::Spawn(s) => s.profile().name,
            Tool::Meadow => "Grass",
        }
    }
}

pub struct Controls {
    pub paused: bool,
    pub speed: usize,
    pub tool: Tool,
    pub chart: Trait,
    pub lens: bool,
    pub vision: bool,
    pub intent: bool,
    pub follow: bool,
}

pub enum Action {
    Reset,
    NewSeed,
    Step,
}

pub fn season_name(world: &World) -> &'static str {
    ["Spring", "Summer", "Autumn", "Winter"][((world.season() + 0.125) * 4.0) as usize % 4]
}

fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn section(ui: &Ui, title: &str, x: f32, y: f32) {
    ui.bold(title, x, y, 11.0, FAINT);
}

pub fn marker(species: Species, center: Vec2, r: f32, color: Color) {
    if species.is_predator() {
        let [a, b, c] = [Vec2::X * 1.6, Vec2::from_angle(2.45), Vec2::from_angle(-2.45)].map(|p| center + p * r);
        draw_triangle(a, b, c, color);
    } else {
        draw_circle(center.x, center.y, r, color);
    }
}

pub fn panel(ui: &mut Ui, world: &mut World, c: &mut Controls, sim_ms: f32) -> Option<Action> {
    let size = ui.size();
    let rect = Rect::new(size.x - PANEL_WIDTH, 0.0, PANEL_WIDTH, size.y);
    ui.claim(rect);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL);
    draw_line(rect.x, 0.0, rect.x, size.y, 1.0, BORDER);
    let (x, w) = (rect.x + PAD, rect.w - PAD * 2.0);
    let right = x + w;
    let mut action = None;

    let mut y = 34.0;
    ui.bold("PREDATOR", x, y, 22.0, TEXT);
    ui.bold("PREY", x + ui.width("PREDATOR ", 22.0) + 6.0, y, 22.0, ACCENT);
    ui.text_right(&format!("{:.0} fps", get_fps()), right, y - 6.0, 11.0, FAINT);
    ui.text_right(&format!("{sim_ms:.1} ms sim"), right, y + 7.0, 11.0, FAINT);
    y += 22.0;
    let year = world.tick() / YEAR + 1;
    let line = format!(
        "tick {} · year {year} · {} · seed {}",
        thousands(world.tick()),
        season_name(world),
        world.seed()
    );
    ui.text(&line, x, y, 12.0, MUTED);

    y += 30.0;
    section(ui, "POPULATION  (log scale)", x, y);
    let chart = Rect::new(x, y + 10.0, w, 128.0);
    population_chart(ui, chart, world.history());
    y = chart.bottom() + 22.0;
    let latest = world.history().last().cloned();
    for (k, s) in Species::ALL.iter().enumerate() {
        let cx = x + (k % 2) as f32 * (w / 2.0);
        let cy = y + (k / 2) as f32 * 20.0;
        marker(*s, vec2(cx + 5.0, cy - 4.0), 4.5, species(*s));
        ui.text(s.profile().name, cx + 16.0, cy, 13.0, TEXT);
        let count = thousands(world.population(*s) as u64);
        let generation = latest.as_ref().map_or(0, |l| l.generation[s.index()]);
        ui.text_right(&format!("{count}  g{generation}"), cx + w / 2.0 - 12.0, cy, 13.0, MUTED);
    }
    y += 40.0;
    draw_rectangle(x, y - 9.0, 10.0, 3.0, GRASS_LINE);
    ui.text(
        &format!("Grass {:.0}%", world.grass().mean() * 100.0),
        x + 16.0,
        y,
        13.0,
        TEXT,
    );
    ui.text_right(
        &format!("{} agents", thousands(world.animals().len() as u64)),
        right - 12.0,
        y,
        13.0,
        MUTED,
    );

    y += 30.0;
    section(ui, "EVOLUTION  (mean gene, 1.0 = ancestor)", x, y);
    y += 10.0;
    let tab = (w - 12.0) / 4.0;
    for (k, t) in Trait::ALL.iter().enumerate() {
        if ui.button(
            Rect::new(x + k as f32 * (tab + 4.0), y, tab, 24.0),
            t.name(),
            c.chart == *t,
        ) {
            c.chart = *t;
        }
    }
    let chart = Rect::new(x, y + 32.0, w, 108.0);
    trait_chart(ui, chart, world.history(), c.chart);
    y = chart.bottom() + 28.0;

    section(ui, "LIFE & DEATH", x, y);
    y += 20.0;
    let totals = world.totals().clone();
    let cells = [
        ("Births", totals.births, ACCENT),
        (
            "Eaten",
            totals.deaths[Cause::Predation as usize],
            death(Cause::Predation),
        ),
        (
            "Starved",
            totals.deaths[Cause::Starvation as usize],
            death(Cause::Starvation),
        ),
        ("Old age", totals.deaths[Cause::OldAge as usize], death(Cause::OldAge)),
    ];
    for (k, (label, n, color)) in cells.iter().enumerate() {
        let cx = x + (k % 2) as f32 * (w / 2.0);
        let cy = y + (k / 2) as f32 * 20.0;
        draw_rectangle(cx, cy - 9.0, 3.0, 11.0, *color);
        ui.text(label, cx + 10.0, cy, 13.0, MUTED);
        ui.text_right(&thousands(*n), cx + w / 2.0 - 12.0, cy, 13.0, TEXT);
    }

    y += 46.0;
    section(ui, "SIMULATION", x, y);
    y += 10.0;
    if ui.button(
        Rect::new(x, y, 84.0, 28.0),
        if c.paused { "Play" } else { "Pause" },
        c.paused,
    ) {
        c.paused = !c.paused;
    }
    if ui.button(Rect::new(x + 90.0, y, 28.0, 28.0), "-", false) {
        c.speed = c.speed.saturating_sub(1);
    }
    let speed = SPEEDS[c.speed];
    let label = if speed < 1.0 {
        "0.5x".to_owned()
    } else {
        format!("{speed:.0}x")
    };
    ui.text(
        &label,
        x + 124.0 + (40.0 - ui.width(&label, 14.0)) / 2.0,
        y + 19.0,
        14.0,
        TEXT,
    );
    if ui.button(Rect::new(x + 168.0, y, 28.0, 28.0), "+", false) {
        c.speed = (c.speed + 1).min(SPEEDS.len() - 1);
    }
    if ui.button(Rect::new(right - 124.0, y, 124.0, 28.0), "Step", false) {
        action = Some(Action::Step);
    }
    y += 40.0;

    let p = &mut world.params;
    let shown = [
        format!("{:.3}", p.grass_growth),
        format!("{:.0}%", p.seasons * 100.0),
        format!("σ {:.3}", p.mutation),
        format!("{:.2}x", p.hunt_success),
    ];
    let sliders: [(&'static str, &str, &mut f32, (f32, f32)); 4] = [
        ("growth", "Regrowth", &mut p.grass_growth, (0.002, 0.03)),
        ("seasons", "Seasons", &mut p.seasons, (0.0, 1.0)),
        ("mutation", "Mutation", &mut p.mutation, (0.0, 0.2)),
        ("hunt", "Hunting", &mut p.hunt_success, (0.2, 2.0)),
    ];
    for (k, ((id, label, value, range), shown)) in sliders.into_iter().zip(shown).enumerate() {
        let cx = x + (k % 2) as f32 * (w / 2.0 + 6.0);
        let cy = y + (k / 2) as f32 * 40.0;
        ui.slider(id, Rect::new(cx, cy, w / 2.0 - 6.0, 30.0), label, value, range, &shown);
    }
    y += 84.0;

    let toggle = (w - 12.0) / 4.0;
    let toggles: [(&str, &mut bool); 4] = [
        ("Migrants", &mut world.params.migration),
        ("Vision", &mut c.vision),
        ("Intent", &mut c.intent),
        ("Gene lens", &mut c.lens),
    ];
    for (k, (label, value)) in toggles.into_iter().enumerate() {
        if ui.button(Rect::new(x + k as f32 * (toggle + 4.0), y, toggle, 26.0), label, *value) {
            *value = !*value;
        }
    }

    y += 48.0;
    section(ui, "TOOLS  (click the map)", x, y);
    y += 10.0;
    let tool = (w - 20.0) / 6.0;
    for (k, t) in Tool::ALL.iter().enumerate() {
        if ui.button(
            Rect::new(x + k as f32 * (tool + 4.0), y, tool, 28.0),
            t.label(),
            c.tool == *t,
        ) {
            c.tool = *t;
        }
    }
    y += 40.0;
    let half = (w - 6.0) / 2.0;
    if ui.button(Rect::new(x, y, half, 28.0), "Reset", false) {
        action = Some(Action::Reset);
    }
    if ui.button(Rect::new(x + half + 6.0, y, half, 28.0), "New seed", false) {
        action = Some(Action::NewSeed);
    }
    action
}

fn frame(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::from_hex(0x0c1110));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BORDER);
}

fn polyline(points: impl Iterator<Item = Option<Vec2>>, thickness: f32, color: Color) {
    let mut last: Option<Vec2> = None;
    for p in points {
        if let (Some(a), Some(b)) = (last, p) {
            draw_line(a.x, a.y, b.x, b.y, thickness, color);
        }
        last = p;
    }
}

fn x_of(rect: Rect, history: &[Sample], tick: u64) -> f32 {
    let (t0, t1) = (
        history[0].tick,
        history[history.len() - 1].tick.max(history[0].tick + 1),
    );
    rect.x + (tick - t0) as f32 / (t1 - t0) as f32 * rect.w
}

fn population_chart(ui: &Ui, rect: Rect, history: &[Sample]) {
    frame(rect);
    let peak = history.iter().flat_map(|s| s.population).max().unwrap_or(0).max(10);
    let decades = (peak as f32).log10().ceil();
    let y_of = |n: f32| rect.bottom() - (1.0 + n).log10() / decades * (rect.h - 4.0);
    for d in 1..decades as i32 {
        let gy = y_of(10f32.powi(d));
        draw_line(rect.x, gy, rect.right(), gy, 1.0, BORDER);
        let label = ["1", "10", "100", "1k", "10k"][d as usize];
        ui.text(label, rect.x + 4.0, gy - 3.0, 10.0, FAINT);
    }
    if history.len() < 2 {
        return;
    }
    let grass_y = |g: f32| rect.bottom() - g * rect.h;
    for pair in history.windows(2) {
        let (x0, x1) = (x_of(rect, history, pair[0].tick), x_of(rect, history, pair[1].tick));
        let (y0, y1) = (grass_y(pair[0].grass), grass_y(pair[1].grass));
        let base = rect.bottom();
        draw_triangle(vec2(x0, base), vec2(x0, y0), vec2(x1, y1), GRASS_LINE.with_alpha(0.12));
        draw_triangle(
            vec2(x0, base),
            vec2(x1, y1),
            vec2(x1, base),
            GRASS_LINE.with_alpha(0.12),
        );
    }
    polyline(
        history
            .iter()
            .map(|s| Some(vec2(x_of(rect, history, s.tick), grass_y(s.grass)))),
        1.0,
        GRASS_LINE.with_alpha(0.5),
    );
    for s in Species::ALL {
        polyline(
            history.iter().map(|h| {
                let n = h.population[s.index()];
                (n > 0).then(|| vec2(x_of(rect, history, h.tick), y_of(n as f32)))
            }),
            1.6,
            species(s),
        );
    }
}

fn trait_chart(ui: &Ui, rect: Rect, history: &[Sample], t: Trait) {
    frame(rect);
    let values = || {
        history.iter().flat_map(move |h| {
            Species::ALL
                .iter()
                .filter(move |s| h.population[s.index()] > 0)
                .map(move |s| h.traits[s.index()][t as usize])
        })
    };
    let lo = values().fold(1.0f32, f32::min).max(Genes::MIN) - 0.04;
    let hi = values().fold(1.0f32, f32::max).min(Genes::MAX) + 0.04;
    let y_of = |v: f32| rect.bottom() - 4.0 - (v - lo) / (hi - lo) * (rect.h - 8.0);
    let one = y_of(1.0);
    let mut dash = rect.x;
    while dash < rect.right() {
        draw_line(dash, one, (dash + 4.0).min(rect.right()), one, 1.0, FAINT);
        dash += 8.0;
    }
    ui.text("1.0", rect.x + 4.0, one - 3.0, 10.0, FAINT);
    ui.text(&format!("{hi:.2}"), rect.x + 4.0, rect.y + 12.0, 10.0, FAINT);
    ui.text(&format!("{lo:.2}"), rect.x + 4.0, rect.bottom() - 4.0, 10.0, FAINT);
    if history.len() < 2 {
        return;
    }
    for s in Species::ALL {
        polyline(
            history.iter().map(|h| {
                (h.population[s.index()] > 0)
                    .then(|| vec2(x_of(rect, history, h.tick), y_of(h.traits[s.index()][t as usize])))
            }),
            1.6,
            species(s),
        );
    }
}

pub fn inspector(ui: &mut Ui, a: &Animal, following: bool) {
    let rect = Rect::new(16.0, 16.0, 272.0, 204.0);
    ui.claim(rect);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BORDER);
    let (x, right) = (rect.x + 16.0, rect.right() - 16.0);
    let mut y = rect.y + 28.0;
    marker(a.species, vec2(x + 6.0, y - 5.0), 6.0, species(a.species));
    ui.bold(a.species.profile().name, x + 20.0, y, 16.0, TEXT);
    let sex = if a.sex == Sex::Female { "female" } else { "male" };
    ui.text_right(&format!("#{} · {sex}", a.id), right, y, 12.0, MUTED);
    y += 20.0;
    let state = match a.state {
        State::Wander => "wandering",
        State::Forage => "grazing",
        State::Flee => "fleeing",
        State::Hunt => "hunting",
        State::Court => "courting",
    };
    ui.text(&format!("generation {} · {state}", a.generation), x, y, 12.0, MUTED);
    y += 22.0;
    let bar = |ui: &Ui, y: f32, label: &str, t: f32, shown: &str, color: Color| {
        ui.text(label, x, y, 12.0, MUTED);
        ui.text_right(shown, right, y, 12.0, TEXT);
        let track = Rect::new(x + 82.0, y - 7.0, right - x - 132.0, 5.0);
        draw_rectangle(track.x, track.y, track.w, track.h, SURFACE_HOVER);
        draw_rectangle(track.x, track.y, track.w * t.clamp(0.0, 1.0), track.h, color);
    };
    let energy = a.energy / a.max_energy();
    bar(
        ui,
        y,
        "Energy",
        energy,
        &format!("{:.0}%", energy * 100.0),
        mix(HUNT, ACCENT, energy),
    );
    y += 18.0;
    let life = a.age as f32 / a.lifespan as f32;
    bar(ui, y, "Age", life, &format!("{}", a.age), MUTED);
    y += 26.0;
    for t in Trait::ALL {
        let v = a.genes.get(t);
        bar(
            ui,
            y,
            t.name(),
            (v - Genes::MIN) / (Genes::MAX - Genes::MIN),
            &format!("{v:.2}x"),
            heat((v - 0.6) / 0.8),
        );
        y += 18.0;
    }
    y += 6.0;
    let hint = if following {
        "following · F to stop · Esc to close"
    } else {
        "F to follow · Esc to close"
    };
    ui.text(hint, x, y, 11.0, FAINT);
}

pub fn hints(ui: &Ui) {
    let y = ui.size().y - 16.0;
    let text = "space pause · ←/→ speed · . step · 1-6 tools · drag pan · wheel zoom · V vision · I intent · L lens · R reset · N new seed";
    draw_rectangle(0.0, y - 15.0, ui.width(text, 11.0) + 32.0, 31.0, PANEL);
    ui.text(text, 16.0, y, 11.0, MUTED);
}
