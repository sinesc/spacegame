use prelude::*;
use def;
use cmd::Cmd;

struct MenuState {
    index       : u32,
    group       : Option<String>,
}

pub struct Menu {
    input       : Input,
    layer       : Layer,
    font        : Font,
    def         : def::MenuDef,
    state       : RefCell<MenuState>,
}

impl Menu {

    pub fn new(input: &Input, context: &RenderContext) -> Menu {
        let layer = Layer::new((1920., 1080.));
        layer.set_blendmode(blendmodes::ALPHA);
        Menu {
            input   : input.clone(),
            layer   : layer,
            font    : Font::builder(&context).family("Arial").size(80.0).bold().build().unwrap(),
            def     : def::parse_menu().unwrap(),
            state   : RefCell::new(MenuState {
                index   : 0,
                group   : None,
            }),
        }
    }

    pub fn visible(self: &Self) -> bool {
        self.state.borrow_mut().group.is_some()
    }

    pub fn hide(self: &Self) {
        self.state.borrow_mut().group = None;
    }

    pub fn group(self: &Self, group: &str) -> bool {
        if self.def.contains_key(group) {
            let mut state = self.state.borrow_mut();
            state.group = Some(group.to_string());
            state.index = 0;
            true
        } else {
            false
        }
    }

    pub fn process<T>(self: &Self, renderer: &Renderer, delta: f32, cmd: &Cmd<T>) {

        use InputId::*;

        let mut action = None;

        {            
            let mut state = self.state.borrow_mut();

            if state.group != None {

                let def = &self.def[state.group.as_ref().unwrap()];
                let mut pos_y = def.top;
                let mut pos_x = def.left;
                let mut index = 0;

                if self.input.pressed(CursorDown, true) {
                    state.index = min(def.items.len() as u32 -1, state.index + 1);
                } else if self.input.pressed(CursorUp, true) {
                    state.index = max(1, state.index) - 1;
                } else if self.input.pressed(Return, true) {
                    action = Some(&def.items[state.index as usize].action);
                }

                for ref item in &def.items {
                    
                    self.font.write(&self.layer,
                        &item.label,
                        (pos_x * 1920., pos_y * 1080.),
                        if index == state.index { Color::alpha_pm(0.3) } else { Color::alpha_pm(0.1) }
                    );

                    pos_x += if let Some(stride_x) = item.stride_x { stride_x } else { def.stride_x };
                    pos_y += if let Some(stride_y) = item.stride_y { stride_y } else { def.stride_y };
                    index += 1;
                }

                renderer.draw_layer(&self.layer, 0);
            }

            self.layer.clear();
        }

        // perform action, if any (can't do above since the action might have to borrow state as well)

        if let Some(action) = action {        
            cmd.exec(action);
        }
    }
}