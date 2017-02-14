extern crate radiant_rs;
extern crate specs;
use radiant_rs::*;

mod level;
use level::Level;

fn main() {

    let display = Display::new(DisplayInfo { width: 1600, height: 900, vsync: true, ..DisplayInfo::default() });
    let renderer =  Renderer::new(&display).unwrap();
    let input = Input::new(&display);
    let context = renderer.context();
    let mut level = Level::new(&input, &context);

    utils::renderloop(|frame| {
        display.poll_events();

        display.clear_frame(Color::black());
        level.process(&renderer, frame.delta_f32);
        display.swap_frame();

        !display.was_closed() && !input.down(InputId::Escape)
    });
}
