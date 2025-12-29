use std::{io, env, time::Duration, rc::Rc, cell::RefCell};
use remux_core::lua::load_lua;
use crossterm::event::{KeyEvent, KeyModifiers, KeyEventKind};
use remux_core::config::UserConfig;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color},
};

use remux_core::editor::{
    Editor,
    KeyMap,
    PhysicalModifiers,
    Modifiers,
    InputMode,
};

fn physical_from_key_event(key: &KeyEvent) -> PhysicalModifiers {
    let mut mods = PhysicalModifiers::empty();
   
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        mods |= PhysicalModifiers::CTRL;
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        mods |= PhysicalModifiers::ALT;
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        mods |= PhysicalModifiers::SHIFT;
    }
    if key.modifiers.contains(KeyModifiers::SUPER) {
        mods |= PhysicalModifiers::SUPER;
    }

    mods
}

fn status_line(editor: &Editor) -> String {
    let file = editor
        .buffer
        .file_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or("[No File]".into());

    format!(
        "{}  |  Ln {}, Col {}",
        file,
        editor.buffer.cursor_y + 1,
        editor.buffer.cursor_x + 1
    )
}

fn handle_input(
    editor: &Rc<RefCell<Editor>>,
    keymap: &Rc<RefCell<KeyMap>>,
    user_config: &Rc<RefCell<UserConfig>>,
) -> io::Result<()> {
    let mode = editor.borrow().mode;
    if let Event::Key(key) = event::read()? {
	if key.kind != KeyEventKind::Press {
            return Ok(());
	}
	
        match mode {
           InputMode::Normal => {
            let physical = physical_from_key_event(&key);
            let mods = Modifiers {
                mod_key: physical.intersects(user_config.borrow().mod_mask),
            };
	       
	       match (mods.mod_key, key.code) {
		   (true, KeyCode::Char(c)) => {
		       if let Some(cmd) = keymap.borrow().lookup(mods, c) {
			   editor.borrow_mut().execute(cmd.clone());
		       }
		   }

		   (_, KeyCode::Delete) => editor.borrow_mut().buffer.delete_char(),
		   (_, KeyCode::Backspace) => editor.borrow_mut().buffer.backward_delete_char(),
		   (_, KeyCode::Enter) => editor.borrow_mut().buffer.insert_newline(),
		   
		   (false, KeyCode::Char(c)) => editor.borrow_mut().buffer.insert_char(c),
		   
		   _ => {}
	       }
            }

            InputMode::MiniBuffer => {
                match key.code {
                    KeyCode::Char(c) => editor.borrow_mut().minibuffer.push(c),
                    KeyCode::Backspace => editor.borrow_mut().minibuffer.pop(),
                    KeyCode::Enter => editor.borrow_mut().execute_minibuffer(),
                    KeyCode::Esc => {
			let mut ed = editor.borrow_mut();
                        ed.minibuffer.deactivate();
                        ed.mode = InputMode::Normal;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}


fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let editor = Rc::new(RefCell::new(Editor::new()));

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
	let path = args[1].clone();
	if let Err(e) = editor.borrow_mut().buffer.open_file(path.clone().into()) {
            eprintln!("Failed to open {}: {}", path, e);
	}
    }
   
    let keymap = Rc::new(RefCell::new(KeyMap::new()));
    let user_config = Rc::new(RefCell::new(UserConfig::default()));

    if let Err(e) = load_lua(
    editor.clone(),
    keymap.clone(),
    user_config.clone(),
) {
    eprintln!("Lua error: {e}");
}
    
    while !editor.borrow().should_quit {	
	if event::poll(Duration::from_millis(250))? {
            handle_input(&editor, &keymap, &user_config)?;
	}
	 let snapshot = {
	let ed = editor.borrow();
	(
            ed.buffer.lines.join("\n"),
            ed.buffer.cursor_x,
            ed.buffer.cursor_y,
            status_line(&ed),
            ed.minibuffer.get().to_string(),
	)
	 };

	let (text, cursor_x, cursor_y, status, msg) = snapshot;
	
        terminal.draw(|f| {
            let block = Block::default()
                .title("Remux (TUI)")
                .borders(Borders::ALL);
	    
	    let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
		    Constraint::Min(1),
		    Constraint::Length(1),
		    Constraint::Length(1),
		])
		.split(f.size());

	    f.set_cursor(cursor_x as u16 + 1, cursor_y as u16 + 1);

	    
	     let buffer = Paragraph::new(text)
		.block(Block::default().borders(Borders::ALL));
	    
	    let status_bar = Paragraph::new(status)
		.style(Style::default().bg(Color::DarkGray))
		.block(Block::default());
	    
            let mini = Paragraph::new(msg)
	    .style(Style::default().fg(Color::Yellow))
		.block(Block::default());
	    
	    f.render_widget(buffer, chunks[0]);
	    f.render_widget(status_bar, chunks[1]);
	    f.render_widget(mini, chunks[2]);
	})?;
    }
   disable_raw_mode()?;
   execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
   Ok(())
}

