use prelude::*;
use def;
use cmd::Cmd;

pub struct Menu<T> {
    input       : Input,
    layer       : Layer,
    font        : Font,
    def         : def::MenuDef,
    current     : u32,
    cmd         : Rc<Cmd<T>>
}

impl<T> Menu<T> {
    pub fn new(input: &Input, context: &RenderContext, cmd: Rc<Cmd<T>>) -> Menu<T> {
        let layer = Layer::new((1920., 1080.));
        layer.set_blendmode(blendmodes::ALPHA);
        Menu {
            input   : input.clone(),
            layer   : layer,
            font    : Font::builder(&context).family("Arial").size(80.0).bold().build().unwrap(),
            def     : def::parse_menu().unwrap(),
            current : 0,
            cmd     : cmd,
        }
    }
    pub fn process(self: &mut Self, renderer: &Renderer, delta: f32, menu: &str) {

        use InputId::*;

        let def = &self.def[menu];
        let mut pos_y = def.top;
        let mut pos_x = def.left;
        let mut index = 0;

        if self.input.pressed(CursorDown, true) {
            self.current = min(def.items.len() as u32 -1, self.current + 1);
        } else if self.input.pressed(CursorUp, true) {
            self.current = max(1, self.current) - 1;
        } else if self.input.pressed(Return, true) {
            self.cmd.as_ref().exec(&def.items[self.current as usize].action);
        }

        for ref item in &def.items {
            
            self.font.write(&self.layer,
                &item.label,
                (pos_x * 1920., pos_y * 1080.),
                if index == self.current { Color::alpha_pm(0.3) } else { Color::alpha_pm(0.1) }
            );

            pos_x += if let Some(stride_x) = item.stride_x { stride_x } else { def.stride_x };
            pos_y += if let Some(stride_y) = item.stride_y { stride_y } else { def.stride_y };
            index += 1;
        }

        renderer.draw_layer(&self.layer, 0);
        self.layer.clear();
    }
}