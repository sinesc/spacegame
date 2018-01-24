extern crate radiant_rs;
extern crate shred;
#[macro_use]
extern crate shred_derive;
extern crate specs;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod prelude;
pub mod def;
mod level;

use prelude::*;
use level::Level;

fn main() {

    let display = Display::builder().dimensions((1600, 900)).vsync().transparent().build();
    let renderer =  Renderer::new(&display).unwrap();
    let input = Input::new(&display);
    let mut level = Level::new(&input, &renderer.context());

    display.grab_cursor();

    utils::renderloop(|frame| {
        display.poll_events();

        display.clear_frame(Color::BLACK);
        level.process(&renderer, frame.delta_f32);
        display.swap_frame();

        !display.was_closed() && !input.down(InputId::Escape)
    });
}
