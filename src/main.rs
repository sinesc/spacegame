extern crate radiant_rs as radiant;

mod prelude;
mod sound;
#[path="game/game.rs"]
mod game;
mod bloom;
mod timeframe;
#[path="scripting/scripting.rs"]
mod scripting;

use crate::prelude::*;
use crate::game::Game;
use rodio::DeviceSinkBuilder;

fn main() {

    let display = Arc::new(Display::builder().dimensions((1280, 720)).vsync().build().unwrap());
    display.grab_cursor();
    // Starts windowed: the monitor list is only available after the first event
    // pump, so fullscreen is entered via the menu (toggle_fullscreen) instead.
    let fullscreen = false;
    eprintln!("[debug] main: display created, dimensions = {:?}, fullscreen = {}", display.dimensions(), fullscreen);
    let renderer =  Renderer::new(&display).unwrap();
    let (w, h) = display.dimensions();
    let mut debug_layer = Layer::new((w as f32, h as f32));
    let debug_font = Font::builder(&display.context()).family("Arial").size(20.0).build().unwrap().arc();
    let input = Input::new(&display);
    let mut audio_sink = DeviceSinkBuilder::open_default_sink().unwrap();
    audio_sink.log_on_drop(false);
    let audio = audio_sink.mixer().clone();
    let mut game = Game::new(&input, display.clone(), fullscreen, &audio);

    // game main loop

    let mut last_age = 0.;

    renderloop(|frame| {

        display.poll_events();

        // ingame time and delta

        let age = game.game_age();
        let rate = game.game_rate();
        let delta = age - last_age;
        last_age = age;

        display.clear_frame(Color::BLACK);

        // menu handling (open/close, input, actions) lives in the Itsy script.

        game.process(&renderer, age as f32, delta as f32);

        debug_font.write(&debug_layer, &format!("Renderer\nFPS: {}\nDelta: {:.4}", frame.fps, frame.delta_f32), (10.0, 10.0), Color::alpha_pm(0.4));
        debug_font.write(&debug_layer,
            &format!("Time\nRate: {:.3}\nElapsed: {:.2}\nDelta: {:.4}", rate, age, delta),
            (10.0, 140.0),
            Color::alpha_pm(0.4)
        );

        renderer.draw_layer(&debug_layer, 0);
        debug_layer.clear();

        display.swap_frame();

        // the Itsy script can change the resolution live (windowed mode);
        // applied here, after swap_frame, so no frame is in flight.
        if let Some((w, h)) = game.take_resolution_request() {
            game.apply_resolution(w, h);
            let (w, h) = display.dimensions();
            debug_layer = Layer::new((w as f32, h as f32));
        }

        // the Itsy script can request a level restart (menu "New Game" /
        // "Exit to Menu"); rebuild the level and keep running.
        if game.restart_requested() {
            let fullscreen_now = game.is_fullscreen();
            game = Game::new(&input, display.clone(), fullscreen_now, &audio);
            last_age = 0.;
        }

        !display.was_closed() && !game.exit_requested()
    });
}
