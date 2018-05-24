use prelude::*;
use cmd::Cmd;
use cmd::Type::*;
use cmd::Param;
use menu::Menu;
use level::Level;
use timeframe::Timeframe;

pub struct CommandContext<'a, 'b> {
    pub menu            : Rc<Menu>,
    pub level           : Rc<RefCell<Level<'a, 'b>>>,
    pub timeframe       : Timeframe,
    pub exit_requested  : bool,
}

pub fn init_cmd<'a, 'b>(menu: &Rc<Menu>, level: &Rc<RefCell<Level<'a, 'b>>>) -> Cmd<CommandContext<'a, 'b>> {

    let mut cmd = Cmd::new(CommandContext {
        menu            : menu.clone(),
        level           : level.clone(),
        timeframe       : Timeframe::new(),
        exit_requested  : false,
    });

    cmd.register("pause", &[], Box::new(|cmd, _| {
        cmd.context_mut().timeframe.lerp_rate(0.0, Duration::from_millis(500));
    }));

    cmd.register("resume", &[], Box::new(|cmd, _| {
        cmd.context_mut().timeframe.lerp_rate(1.0, Duration::from_millis(500));
    }));

    cmd.register("menu_toggle", &[], Box::new(|cmd, _| {
        if cmd.context().menu.visible() {
            cmd.call("menu_hide", &[]).unwrap();
        } else {
            cmd.call("menu_show", &[Param::Str("main".to_string())]).unwrap();
        }
    }));

    cmd.register("menu_show", &[Str], Box::new(|cmd, p| {
        cmd.call("pause", &[]).unwrap();
        cmd.context().menu.group(&p[0].to_string());
    }));

    cmd.register("menu_hide", &[], Box::new(|cmd, _| {
        cmd.call("resume", &[]).unwrap();
        cmd.context().menu.hide();
    }));

    /*cmd.register("level_start", &[Int], Box::new(|cmd, p| {

    }));*/

    cmd.register("exit", &[], Box::new(|cmd, _| {
        cmd.context_mut().exit_requested = true
    }));

    cmd
}