extern crate radiant_rs as radiant;
extern crate radiant_utils;
extern crate rodio;
extern crate shred;
#[macro_use]
extern crate shred_derive;
extern crate specs;
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

use prelude::*;
use level::Level;
use menu::Menu;

fn main() {

    let dummy = sound::Sound::load("res/sound/projectile/pew1a.ogg").unwrap();
    rodio::play_raw(&rodio::default_output_device().unwrap(), dummy.samples());


    let display = Display::builder().dimensions((1920, 1080)).vsync().build().unwrap();
    let renderer =  Renderer::new(&display).unwrap();
    let input = Input::new(&display);
    let mut level = Level::new(&input, &renderer.context());
    let mut menu = Menu::new(&input, &renderer.context());

    let mut cmd = cmd::Cmd::new();
    cmd.register("test", Box::new([cmd::Type::Str, cmd::Type::Int]), Box::new(|p| println!("{:?}", p) ));
    cmd.exec("test \"Hello World!\" 12");

    display.grab_cursor();
    display.set_fullscreen().unwrap();

    renderloop(|frame| {
        display.poll_events();

        display.clear_frame(Color::BLACK);
        level.process(&renderer, frame.delta_f32);
        menu.process(&renderer, frame.delta_f32);
        display.swap_frame();

        !display.was_closed() && !input.down(InputId::Escape)
    });
}
