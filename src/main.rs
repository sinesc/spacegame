//#![allow(dead_code)]
//#![allow(unused_variables)]

extern crate radiant_rs as radiant;
extern crate radiant_utils;
extern crate rodio;
extern crate shred;
#[macro_use]
extern crate shred_derive;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate serde;
extern crate serde_yaml;
extern crate yaml_merge_keys;
#[macro_use]
extern crate serde_derive;
extern crate unicode_segmentation;

mod prelude;
mod def;
mod sound;
mod level;
mod bloom;
mod menu;
mod cmd;
mod timeframe;
mod console;
mod repository;
mod completion;

use prelude::*;
use level::Level;
use menu::Menu;
use timeframe::Timeframe;


fn main() {

    let dummy = sound::Sound::load("res/sound/projectile/pew1a.ogg").unwrap();
    rodio::play_raw(&rodio::default_output_device().unwrap(), dummy.samples());

    let display = Display::builder().dimensions((1920, 1080)).vsync().build().unwrap();
    display.grab_cursor();
    display.set_fullscreen().unwrap();
    let renderer =  Renderer::new(&display).unwrap();
    let debug_layer = Layer::new((1920., 1080.));
    let debug_font = Font::builder(&renderer.context()).family("Arial").size(20.0).build().unwrap().arc();
    let input = Input::new(&display);

    // create menu and command parser

    let level = Rc::new(RefCell::new(Level::new(&input, &renderer.context())));
    let menu = Rc::new(Menu::new(&input, &renderer.context()));
    let cmd = console::init_cmd(&menu, &level);

    // game main loop

    let mut last_age = 0.;

    renderloop(|frame| {

        display.poll_events();

        // ingame time and delta

        let age = Timeframe::duration_to_secs(cmd.context().timeframe.elapsed());
        let rate = cmd.context().timeframe.rate();
        let delta = age - last_age;
        last_age = age;

        if input.pressed(InputId::Escape, false) {
            cmd.call("menu_toggle", &[]).unwrap();
        }

        display.clear_frame(Color::BLACK);

        level.borrow_mut().process(&renderer, age as f32, delta as f32, !menu.visible(), menu.visible());
        menu.process(&renderer, &cmd);

        debug_font.write(&debug_layer, &format!("Renderer\nFPS: {}\nDelta: {:.4}", frame.fps, frame.delta_f32), (10.0, 10.0), Color::alpha_pm(0.4));
        debug_font.write(&debug_layer,
            &format!("Time\nRate: {:.3}\nElapsed: {:.2}\nDelta: {:.4}", rate, age, delta),
            (10.0, 140.0),
            Color::alpha_pm(0.4)
        );

        renderer.draw_layer(&debug_layer, 0);
        debug_layer.clear();

        display.swap_frame();

        !display.was_closed() && !cmd.context().exit_requested
    });
}
