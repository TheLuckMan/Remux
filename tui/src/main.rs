use std::env;
use remux_tui::app::App;
use remux_core::command::CommandRegistry;
use remux_core::commands::builtins::register_builtins;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut registry = CommandRegistry::new();
    register_builtins(&mut registry);
    let mut app = App::init(&args, registry)?;
    app.run()
}
