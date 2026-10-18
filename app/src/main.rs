mod app;
mod hud;
mod palette;
mod scene;
mod widgets;

use macroquad::prelude::*;

fn conf() -> Conf {
    Conf {
        window_title: "Predator Prey".to_owned(),
        window_width: 1600,
        window_height: 940,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut app = app::App::new(42);
    loop {
        app.frame();
        next_frame().await;
    }
}
