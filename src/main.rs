extern crate radiant_rs;
extern crate specs;
// #[macro_use] extern crate lazy_static;
use radiant_rs::*;

mod level;
use level::*;

fn main() {

    let display = Display::new(DisplayInfo { width: 1600, height: 900, vsync: true, ..DisplayInfo::default() });
    let renderer =  Renderer::new(&display);
    let input = Input::new(&display);
    let context = renderer.context();
    let mut level = Level::new(&input, &context);

    utils::renderloop(|frame| {

        display.poll_events();

        renderer.clear_target(Color::black());
        level.process(&renderer, frame.delta_f32);
        renderer.swap_target();

        !display.was_closed() && !input.down(InputId::Escape)
    });
}
