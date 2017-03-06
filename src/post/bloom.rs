use radiant_rs::{Postprocessor, RenderContext, Renderer, Color, Texture, TextureFilter, Program, BlendMode, Rect, Point2, Vec2, blendmodes};
use std::sync::Mutex;

pub struct Bloom {
    targets         : [[Texture; 5]; 2],
    blur_program    : Mutex<Program>,
    combine_program : Mutex<Program>,
}

pub struct BloomArgs {
    pub iterations  : u32,
    pub iter_blend  : BlendMode,
    pub final_blend : BlendMode,
    pub spread      : u8,
    pub color       : Color,
}

impl Default for BloomArgs {
    fn default() -> Self {
         BloomArgs {
            iterations  : 3,
            iter_blend  : blendmodes::COPY,
            final_blend : blendmodes::ALPHA,
            spread      : 5,
            color       : Color::white(),
        }
    }
}

impl Postprocessor for Bloom {
    type T = BloomArgs;

    /// Returns the target where the postprocessor expects the unprocessed input.
    fn target(self: &Self) -> &Texture {
        &self.targets[0][0]
    }

    /// Process received data.
    fn process(self: &Self, renderer: &Renderer, args: &Self::T) {
        use std::ops::DerefMut;
        use std::cmp::min;

        let spread = min(self.targets[0].len(), args.spread as usize);

        // Copy to progressively smaller textures
        for i in 1..spread {
            renderer.render_to(&self.targets[0][i], || {
                renderer.copy_from(&self.targets[0][i-1], TextureFilter::Linear);
            });
        }

        let mut blur = self.blur_program.lock().unwrap();
        let blur = blur.deref_mut();

        for _ in 0..args.iterations {

            // Apply horizontal blur
            blur.set_uniform("horizontal", &true);
            for i in 0..spread {
                renderer.render_to(&self.targets[1][i], || {
                    renderer.fill().blendmode(args.iter_blend).program(&blur).texture(&self.targets[0][i]).draw();
                });
            }

            // Apply vertical blur
            blur.set_uniform("horizontal", &false);
            for i in 0..spread {
                renderer.render_to(&self.targets[0][i], || {
                    renderer.fill().blendmode(args.iter_blend).program(&blur).texture(&self.targets[1][i]).draw();
                });
            }
        }
    }

    /// Draw processed input. The renderer has already set the correct target.
    fn draw(self: &Self, renderer: &Renderer, args: &Self::T) {
        use std::ops::DerefMut;
        let mut combine = self.combine_program.lock().unwrap();
        let combine = combine.deref_mut();
        combine.set_uniform("bloom_color", &args.color);
        renderer.fill().blendmode(args.final_blend).program(&combine).draw();
    }
}

impl Bloom {
    pub fn new(context: &RenderContext) -> Self {
        use std::ops::DerefMut;

        let blur_program = Program::from_string(&context, include_str!("blur.fs")).unwrap();
        let combine_program = Program::from_string(&context, include_str!("combine.fs")).unwrap();
        let display = context.display();
        let Point2(width, height) = display.dimensions();

        let result = Bloom {
            blur_program    : Mutex::new(blur_program),
            combine_program : Mutex::new(combine_program),
            targets: [ [
                Texture::new(&context, width / 2, height / 2),
                Texture::new(&context, width / 4, height / 4),
                Texture::new(&context, width / 8, height / 8),
                Texture::new(&context, width / 16, height / 16),
                Texture::new(&context, width / 32, height / 32),
            ], [
                Texture::new(&context, width / 2, height / 2),
                Texture::new(&context, width / 4, height / 4),
                Texture::new(&context, width / 8, height / 8),
                Texture::new(&context, width / 16, height / 16),
                Texture::new(&context, width / 32, height / 32),
            ] ]
        };

        {
            let mut combine = result.combine_program.lock().unwrap();
            let combine = combine.deref_mut();
            combine.set_uniform("sample0", &result.targets[0][0]);
            combine.set_uniform("sample1", &result.targets[0][1]);
            combine.set_uniform("sample2", &result.targets[0][2]);
            combine.set_uniform("sample3", &result.targets[0][3]);
            combine.set_uniform("sample4", &result.targets[0][4]);
        }

        result
    }
}
