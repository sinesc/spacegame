use prelude::*;
use cmd::Cmd;
use cmd::Type::*;
use menu::Menu;
use timeframe::Timeframe;

pub struct CommandContext {
    pub menu            : Rc<Menu>,
    pub timeframe       : Timeframe,
    pub exit_requested  : bool,
}

pub fn init_cmd(menu: &Rc<Menu>) -> Cmd<CommandContext> {

    let mut cmd = Cmd::new(CommandContext {
        menu            : menu.clone(),
        timeframe       : Timeframe::new(),
        exit_requested  : false,
    });

    cmd.register("pause", &[], Box::new(|cmd, p| {
        cmd.context_mut().timeframe.lerp_rate(0.0, Duration::from_millis(500));
    }));

    cmd.register("resume", &[], Box::new(|cmd, p| {
        cmd.context_mut().timeframe.lerp_rate(1.0, Duration::from_millis(500));
    }));

    cmd.register("menu_toggle", &[], Box::new(|cmd, p| {
        if cmd.context().menu.visible() {
            cmd.call("resume", &[]);
            cmd.context().menu.hide();
        } else {
            cmd.call("pause", &[]);
            cmd.context().menu.group("main");
        }
    }));

    cmd.register("menu_switch", &[Str], Box::new(|cmd, p| {
        cmd.context_mut().menu.group(&p[0].to_string());
    }));

    cmd.register("level_start", &[Int], Box::new(|cmd, p| {

    }));

    cmd.register("exit", &[], Box::new(|cmd, p| {
        cmd.context_mut().exit_requested = true
    }));

    cmd
}