extern crate radiant_rs;
extern crate shred;
#[macro_use]
extern crate shred_derive;
extern crate specs;
use radiant_rs::*;

mod level;
mod post;
use level::Level;

fn main() {

    let display = Display::builder().dimensions((1600, 900)).vsync().transparent().build();
    let renderer =  Renderer::new(&display).unwrap();
    let input = Input::new(&display);
    let mut level = Level::new(&input, &renderer.context());

    utils::renderloop(|frame| {
        display.poll_events();

        display.clear_frame(Color::black());
        level.process(&renderer, frame.delta_f32);
        display.swap_frame();

        !display.was_closed() && !input.down(InputId::Escape)
    });
}
