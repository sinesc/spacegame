extern crate radiant_rs as radiant;

mod prelude;
mod sound;
mod level;
mod bloom;
mod timeframe;
mod scripting;

use crate::prelude::*;
use crate::level::Level;


fn main() {

    let monitor = Display::monitors().into_iter().next();
    let display = Arc::new(Display::builder().dimensions((1920, 1080)).vsync().build().unwrap());
    display.grab_cursor();
    let fullscreen = match &monitor {
        Some(m) => { display.set_fullscreen(Some(m.clone())).unwrap(); true }
        None    => false,
    };
    let renderer =  Renderer::new(&display).unwrap();
    let debug_layer = Layer::new((1920., 1080.));
    let debug_font = Font::builder(&display.context()).family("Arial").size(20.0).build().unwrap().arc();
    let input = Input::new(&display);

    let level = Rc::new(RefCell::new(Level::new(&input, display.clone(), monitor.clone(), fullscreen)));

    // game main loop

    let mut last_age = 0.;

    renderloop(|frame| {

        display.poll_events();

        // ingame time and delta

        let age = level.borrow().game_age();
        let rate = level.borrow().game_rate();
        let delta = age - last_age;
        last_age = age;

        display.clear_frame(Color::BLACK);

        // menu handling (open/close, input, actions) lives in the Itsy script.

        level.borrow_mut().process(&renderer, age as f32, delta as f32);

        debug_font.write(&debug_layer, &format!("Renderer\nFPS: {}\nDelta: {:.4}", frame.fps, frame.delta_f32), (10.0, 10.0), Color::alpha_pm(0.4));
        debug_font.write(&debug_layer,
            &format!("Time\nRate: {:.3}\nElapsed: {:.2}\nDelta: {:.4}", rate, age, delta),
            (10.0, 140.0),
            Color::alpha_pm(0.4)
        );

        renderer.draw_layer(&debug_layer, 0);
        debug_layer.clear();

        display.swap_frame();

        // the Itsy script can request a level restart (menu "New Game" /
        // "Exit to Menu"); rebuild the level and keep running.
        if level.borrow().restart_requested() {
            let fullscreen_now = level.borrow().is_fullscreen();
            *level.borrow_mut() = Level::new(&input, display.clone(), monitor.clone(), fullscreen_now);
            last_age = 0.;
        }

        !display.was_closed() && !level.borrow().exit_requested()
    });
}
