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

struct CommandContext {
    current_menu: Option<String>,
}

fn main() {

    let dummy = sound::Sound::load("res/sound/projectile/pew1a.ogg").unwrap();
    rodio::play_raw(&rodio::default_output_device().unwrap(), dummy.samples());


    let display = Display::builder().dimensions((1920, 1080)).vsync().build().unwrap();
    let renderer =  Renderer::new(&display).unwrap();
    let input = Input::new(&display);
    let mut level = Level::new(&input, &renderer.context());

    let cmd = {
        use cmd::Type::*;

        let mut cmd = cmd::Cmd::new(CommandContext { 
            current_menu: Some("main".to_string()), 
        });

        cmd.register("test", vec![Str, Int], Box::new(|c, p| println!("2 args: {:?}", p) ));
        cmd.register("menu_close", vec![], Box::new(|c, p| { println!("menu close"); c.current_menu = None; }));

        Rc::new(cmd)
    };

    let mut menu = Menu::new(&input, &renderer.context(), cmd.clone());

    display.grab_cursor();
    display.set_fullscreen().unwrap();

    renderloop(|frame| {
        display.poll_events();

        display.clear_frame(Color::BLACK);
        level.process(&renderer, frame.delta_f32);

        let current_menu = cmd.context().current_menu.clone();

        if current_menu.is_some() {
            menu.process(&renderer, frame.delta_f32, &current_menu.unwrap());
        }

        display.swap_frame();

        !display.was_closed() && !input.down(InputId::Escape)
    });
}
