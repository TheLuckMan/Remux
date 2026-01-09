use std::{
    io,
    rc::Rc,
    cell::RefCell,
    time::Duration,
};

use crossterm::{
    execute,
    terminal::{
        enable_raw_mode,
        disable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    event,
};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
};

use mlua::Lua;

use remux_core::{
    editor::{Editor, KeyMap},
    config::UserConfig,
    command::CommandRegistry,
};

use remux_config::{
    lua::load_lua,
};

use crate::{
    input::handle_input,
    render::editor::render_editor,
};

pub struct App {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    pub editor: Rc<RefCell<Editor>>,
    lua: Lua,
    pub keymap: Rc<RefCell<KeyMap>>,
    pub user_config: Rc<RefCell<UserConfig>>,
}

impl App {
    /// Инициализация TUI + Editor
    pub fn init(
	args: &[String],
	registry: CommandRegistry,
    ) -> io::Result<Self> {
	enable_raw_mode()?;

	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen)?;

	let backend = CrosstermBackend::new(stdout);
	let terminal = Terminal::new(backend)?;
	let lua_events: Rc<RefCell<Vec<remux_core::editor::EditorEvent>>> = Rc::new(RefCell::new(Vec::new()));

	// создаём один KeyMap
	let keymap = Rc::new(RefCell::new(KeyMap::new()));
	let editor = Rc::new(RefCell::new(Editor::new(registry, keymap.clone())));
	let user_config = Rc::new(RefCell::new(UserConfig::default()));

	let lua = Lua::new();

	// Lua теперь работает с тем же keymap, что и editor
	load_lua(
	    &lua,
	    editor.clone(),
	    keymap.clone(),
	    lua_events.clone(),
	    user_config.clone(),
	).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

	// открыть файл из argv
	if args.len() > 1 {
            let path = args[1].clone();
            if let Err(e) = editor.borrow_mut().buffer.open_file(path.into()) {
		eprintln!("Failed to open file: {e}");
            }
	}

	Ok(Self {
            terminal,
            editor,
            lua,
            keymap,
            user_config,
	})
    }


    /// Главный event loop
    pub fn run(&mut self) -> io::Result<()> {
        while !self.editor.borrow().should_quit {
            self.tick()?;
        }
        self.shutdown()
    }

    fn tick(&mut self) -> io::Result<()> {
        // input
        if event::poll(Duration::from_millis(250))? {
           handle_input(&self.lua, &self.editor, &self.keymap, &self.user_config)?;

        }

        // hooks / background
        self.editor
            .borrow_mut()
            .hooks
            .run(&self.lua, "after-init", "");

        // render
        self.draw()
    }

    fn draw(&mut self) -> io::Result<()> {
        let editor = self.editor.clone();

        self.terminal.draw(|f| {
            let mut ed = editor.borrow_mut();
            render_editor(f, &mut ed);
        })?;

        Ok(())
    }

    fn shutdown(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen
        )?;
        Ok(())
    }
}
