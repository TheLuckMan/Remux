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
    editor::editor::{Editor, KeyMap},
    config::UserConfig,
    command::CommandRegistry,
};

use remux_config::{
    lua::load_lua,
};

use crate::{
    input::handle_input,
    render::editor::render_editor,
		view::render_state::RenderState,
};
use crate::hooks::isearch_highlight;
use crate::view::selection::apply_selection;
use crate::view::isearch::apply_isearch;

pub struct App {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    pub editor: Rc<RefCell<Editor>>,
    lua: Lua,
    pub lua_events: Rc<RefCell<Vec<remux_core::editor::editor::EditorEvent>>>,
    pub keymap: Rc<RefCell<KeyMap>>,
    pub user_config: Rc<RefCell<UserConfig>>,
		pub view: View,
}

pub struct View {
    pub render: RenderState,
}

impl App {
    /// Init TUI + Editor
    pub fn init(
				args: &[String],
				registry: CommandRegistry,
    ) -> io::Result<Self> {
				enable_raw_mode()?;

				let mut stdout = io::stdout();
				execute!(stdout, EnterAlternateScreen)?;

				let backend = CrosstermBackend::new(stdout);
				let terminal = Terminal::new(backend)?;
				let lua = Lua::new();
				let lua_events: Rc<RefCell<Vec<remux_core::editor::editor::EditorEvent>>> = Rc::new(RefCell::new(Vec::new()));
				let keymap = Rc::new(RefCell::new(KeyMap::new()));
				let user_config = Rc::new(RefCell::new(UserConfig::default()));
				let editor = Rc::new(RefCell::new(Editor::new(registry, keymap.clone(), user_config.clone())));



				load_lua(
						&lua,
						editor.clone(),
						keymap.clone(),
						lua_events.clone(),
						user_config.clone(),
				).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

				editor.borrow_mut().process_events(&lua);
				
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
						lua_events,
            keymap,
						user_config,
						view: View {
								render: RenderState::default(),
						},
				})
    }

   pub fn run(&mut self) -> io::Result<()> {
    while !self.editor.borrow().should_quit {
        self.tick()?;
    }
			 if self.editor.borrow().buffer.is_modified() {
					 self.editor.borrow().hooks.run(&self.lua, "before-exit", "");
			 }

    self.shutdown()
}


fn tick(&mut self) -> io::Result<()> {
    if event::poll(Duration::from_millis(250))? {
        handle_input(&self.lua, &self.editor, &self.keymap, &self.user_config)?;
    }

    {
        let mut ed = self.editor.borrow_mut();
        let mut lua_events = self.lua_events.borrow_mut();

        ed.event_queue.extend(lua_events.drain(..));
        ed.process_events(&self.lua);

        let events = std::mem::take(&mut ed.event_queue);

				for ev in &events {
						isearch_highlight::handle_isearch_event(&mut ed, ev);
				}

				apply_isearch(&ed, &mut self.view.render);
				apply_selection(&ed, &mut self.view.render);
    }

    self.draw()
}

    fn draw(&mut self) -> io::Result<()> {
        let editor = self.editor.clone();

				self.terminal.draw(|f| {
						let mut ed = editor.borrow_mut();
						render_editor(f, &mut ed, &mut self.view.render, &self.lua);
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
