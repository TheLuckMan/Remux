use std::{io, env, time::Duration, rc::Rc, cell::RefCell};
use remux_config::lua::load_lua;
use mlua::Lua;
use crossterm::event::{KeyEvent, KeyModifiers, KeyEventKind};
use remux_core::config::UserConfig;
use remux_core::command::CommandRegistry;
use remux_core::commands::builtins::register_builtins;
use remux_core::editor::EditorEvent;
use remux_core::buffer::Selection;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    text::{Line, Span},
    Terminal,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction, Position},
    style::{Style, Color},
};
use remux_core::editor::{
    Editor,
    KeyMap,
    PhysicalModifiers,
    Modifiers,
    InputMode,
};

pub type LuaEventQueue = Rc<RefCell<Vec<EditorEvent>>>;

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

fn split_line(s: &str, start: usize, end: usize) -> (&str, &str, &str) {
    let mut indices = s.char_indices().map(|(i, _)| i).collect::<Vec<_>>();
    indices.push(s.len());

    let s_start = indices.get(start).copied().unwrap_or(s.len());
    let s_end   = indices.get(end).copied().unwrap_or(s.len());

    (&s[..s_start], &s[s_start..s_end], &s[s_end..])
}

fn handle_input (
    lua: &Lua,
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
			   editor.borrow_mut().execute_named(&cmd, lua);
			    return Ok(());
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
                    KeyCode::Enter => editor.borrow_mut().execute_minibuffer(lua),
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


fn render_buffer(
    lines: &[String],
    selection: Option<Selection>,
) -> Vec<Line<'static>> {
    lines.iter().enumerate().map(|(y, line)| {
        if let Some(sel) = &selection {
            let sy = sel.start.y;
	    let sx = sel.start.x;
	    let ey = sel.end.y;
	    let ex = sel.end.x;
	    

            if y < sy || y > ey {
                return Line::from(line.clone());
            }

            let sel_start = if y == sy { sx } else { 0 };
            let sel_end   = if y == ey { ex } else { line.chars().count() };

            let (a, b, c) = split_line(line, sel_start, sel_end);

            Line::from(vec![
                Span::raw(a.to_string()),
                Span::styled(b.to_string(), Style::default().bg(Color::Blue)),
                Span::raw(c.to_string()),
            ])
        } else {
            Line::from(line.clone())
        }
    }).collect()
}


fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut registry = CommandRegistry::new();
    register_builtins(&mut registry);
    let lua = Lua::new();
    let editor = Rc::new(RefCell::new(Editor::new(registry)));
    let lua_events = Rc::new(RefCell::new(Vec::new()));
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
	&lua,
	editor.clone(),
	keymap.clone(),
	lua_events.clone(),
	user_config.clone(),
    ) {
	eprintln!("Lua error: {e}");
    }
    
    while !editor.borrow().should_quit {	
	if event::poll(Duration::from_millis(250))? {
            handle_input(&lua, &editor, &keymap, &user_config)?;
	}
	
	
        terminal.draw(|f| {
	    let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
		    Constraint::Min(1),
		    Constraint::Length(1),
		    Constraint::Length(1),
		])
		.split(f.size());

	    let snapshot = {
		let mut ed = editor.borrow_mut();
		let mut lua_ev = lua_events.borrow_mut();
		ed.event_queue.extend(lua_ev.drain(..));
		let height = chunks[0].height.saturating_sub(2) as usize;
		ed.viewport_height = height;
		
		let start = ed.scroll_y;
		let end = (start + height).min(ed.buffer.lines.len());
		
		let visible_lines = ed.buffer.lines[start..end].to_vec();

		(
		    visible_lines,
		    ed.buffer.selection().map(|sel| sel.translate_y(start)),
		    ed.buffer.cursor_x,
		    ed.buffer.cursor_y.saturating_sub(start),
		    status_line(&ed),
		    ed.minibuffer.get().to_string(),
		)
	    };
	    
	    
	    let (lines, selection, cursor_x, cursor_y, status, msg) = snapshot;

	    f.set_cursor(cursor_x as u16 + 1, cursor_y as u16 + 1);

	    let rendered = render_buffer(&lines, selection);
	    
	    let buffer = Paragraph::new(rendered)
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

