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
mod timeframe;

use prelude::*;
use level::Level;
use menu::Menu;
use cmd::Cmd;
use cmd::Type::*;

struct CommandContext {
    menu            : Rc<Menu>,
    exit_requested  : bool,
}

fn main() {

    let dummy = sound::Sound::load("res/sound/projectile/pew1a.ogg").unwrap();
    rodio::play_raw(&rodio::default_output_device().unwrap(), dummy.samples());

    let display = Display::builder().dimensions((1920, 1080)).vsync().build().unwrap();
    display.grab_cursor();
    display.set_fullscreen().unwrap();
    let renderer =  Renderer::new(&display).unwrap();
    let input = Input::new(&display);
    let mut level = Level::new(&input, &renderer.context());

    // create menu and command parser

    let menu = Rc::new(Menu::new(&input, &renderer.context()));
    menu.group("main");

    let mut cmd = Cmd::new(CommandContext {
        menu: menu.clone(),
        exit_requested: false,
    });

    cmd.register("menu_close", vec![], Box::new(|c, p| { c.menu.hide(); }));
    cmd.register("menu_switch", vec![Str], Box::new(|c, p| { c.menu.group(&p[0].to_string()); }));
    cmd.register("exit", vec![], Box::new(|c, p| c.exit_requested = true ));

    // game main loop

    renderloop(|frame| {

        display.poll_events();

        if input.pressed(InputId::Escape, false) {
            if !menu.visible() {
                menu.group("main");
            } else {
                menu.hide();
            }
        }

        display.clear_frame(Color::BLACK);
        level.process(&renderer, !menu.visible(), menu.visible());
        menu.process(&renderer, &cmd);
        display.swap_frame();

        !display.was_closed() && !cmd.context().exit_requested
    });
}
