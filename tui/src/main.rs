use crossterm::event::{KeyEvent, KeyModifiers, KeyEventKind};
use std::{io, env, time::Duration, rc::Rc, cell::RefCell};
use remux_config::lua::load_lua;
use remux_core::{
    commands::builtins::register_builtins,
    status::{StatusInfo, build_status},
    command::CommandRegistry,
    config::UserConfig,
    editor::{
	PhysicalModifiers,
	EditorEvent,
	VisualLine,
	Modifiers,
	InputMode,
	Editor,
	KeyMap,
    },
};
use ratatui::{
    layout::{Layout, Constraint, Direction},
    widgets::{Block, Borders, Paragraph},
    backend::CrosstermBackend,
    style::{Style, Color},
    text::Line,
    Terminal,
};
use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    terminal::{disable_raw_mode, enable_raw_mode},
    event::{self, Event, KeyCode},
    execute,
};
use mlua::Lua;

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

fn logical_modifiers(
    physical: PhysicalModifiers,
    config: &UserConfig,
) -> Modifiers {
    let mut mods = Modifiers::empty();

    if physical.intersects(config.mod_mask) {
        mods.insert(Modifiers::MOD);
    }

    mods
}

fn char_range_to_byte_range(s: &str, start: usize, end: usize) -> (usize, usize) {
    let mut indices: Vec<usize> = s.char_indices().map(|(i, _)| i).collect();
    indices.push(s.len());
    let bs = *indices.get(start).unwrap_or(&s.len());
    let be = *indices.get(end).unwrap_or(&s.len());
    (bs, be)
}

pub fn render_status(info: &StatusInfo) -> String {
    let undo = if info.undo_depth > 0 {
        format!("U:{}", "*".repeat(info.undo_depth.min(3)))
    } else {
        "U:—".to_string()
    };

    let modified = if info.modified { "*" } else { "" };

    format!(
        " {} {}{} {:>3} ({}, {}) ",
        undo,
        info.file_name,
        modified,
        info.scroll_percent,
        info.cursor_line,
        info.cursor_col
    )
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
		let mut ed = editor.borrow_mut();
		let mods = logical_modifiers(physical, &user_config.borrow());

		if key.code == KeyCode::Char('c')
		    && key.modifiers.contains(KeyModifiers::CONTROL)
		{
		    ed.should_quit = true;
		    return Ok(());
		}
		
		match (mods.contains(Modifiers::MOD), key.code) {
		    (true, KeyCode::Char(c)) => {
			if let Some(cmd) = keymap.borrow().lookup(mods, c) {
			    ed.execute_named(&cmd, lua);
			    return Ok(());
			}
		    }
		    (_, KeyCode::Backspace) => ed.buffer.backward_delete_char(),
		    (_, KeyCode::Enter) => ed.buffer.insert_newline(),
		    (_, KeyCode::Delete) => ed.buffer.delete_char(),
		    (false, KeyCode::Char(c)) => ed.buffer.insert_char(c),
		    _ => {}
		}
            }
            InputMode::MiniBuffer => {
		let physical = physical_from_key_event(&key);
		let mods = logical_modifiers(physical, &user_config.borrow());
		match (mods.contains(Modifiers::MOD), key.code) {
		    (true, KeyCode::Char(c)) => {
			if let Some(cmd) = keymap.borrow().lookup(mods, c) {
			    editor.borrow_mut().execute_named(&cmd, lua);
			    return Ok(());
			}
		    }
                    (_, KeyCode::Backspace) => editor.borrow_mut().minibuffer.pop(),
                    (_, KeyCode::Enter) => editor.borrow_mut().execute_minibuffer(lua),
                    (_, KeyCode::Esc) => {
                        editor.borrow_mut().minibuffer.deactivate();
                        editor.borrow_mut().mode = InputMode::Normal;
                    }
		     (false, KeyCode::Char(c)) => editor.borrow_mut().minibuffer.push(c),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn render_visuals(
    editor: &Editor,
    visuals: &[VisualLine],
) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    let selection = editor.buffer.selection();

    for v in visuals {
        let line = &editor.buffer.lines[v.buffer_y];
        let line_len = line.chars().count();

        let vis_start = v.start_x;
        let vis_end   = (v.start_x + v.len).min(line_len);

        let sel = selection
            .filter(|s| v.buffer_y >= s.start.y && v.buffer_y <= s.end.y)
            .map(|s| {
                let start = if v.buffer_y == s.start.y { s.start.x } else { 0 };
                let end   = if v.buffer_y == s.end.y   { s.end.x } else { line_len };
                (start, end)
            });

        match sel {
            Some((sel_start, sel_end))
                if sel_start < vis_end && sel_end > vis_start =>
            {
                let a_start = vis_start;
                let a_end   = sel_start.clamp(vis_start, vis_end);
                let b_start = a_end;
                let b_end   = sel_end.clamp(vis_start, vis_end);
                let c_start = b_end;
                let c_end   = vis_end;

                let mut spans = Vec::new();

                if a_start < a_end {
                    let (bs, be) = char_range_to_byte_range(line, a_start, a_end);
                    spans.push(ratatui::text::Span::raw(line[bs..be].to_string()));
                }

                if b_start < b_end {
                    let (bs, be) = char_range_to_byte_range(line, b_start, b_end);
                    spans.push(
                        ratatui::text::Span::styled(
                            line[bs..be].to_string(),
                            Style::default().bg(Color::Blue),
                        )
                    );
                }

                if c_start < c_end {
                    let (bs, be) = char_range_to_byte_range(line, c_start, c_end);
                    spans.push(ratatui::text::Span::raw(line[bs..be].to_string()));
                }

                out.push(Line::from(spans));
            }

            _ => {
                let (bs, be) = char_range_to_byte_range(line, vis_start, vis_end);
                out.push(Line::raw(line[bs..be].to_string()));
            }
        }
    }

    out
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let user_config = Rc::new(RefCell::new(UserConfig::default()));
    let lua_events = Rc::new(RefCell::new(Vec::new()));
    let keymap = Rc::new(RefCell::new(KeyMap::new()));
    let args: Vec<String> = env::args().collect();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut registry = CommandRegistry::new();
    register_builtins(&mut registry);
    let lua = Lua::new();

    let editor = Rc::new(RefCell::new(Editor::new(registry)));
    
    if args.len() > 1 {
	let path = args[1].clone();
	if let Err(e) = editor.borrow_mut().buffer.open_file(path.clone().into()) {
            eprintln!("Failed to open {}: {}", path, e);
	}
    }
    
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
	editor.borrow_mut().hooks.run(&lua, "after-init", "");
        terminal.draw(|f| {
	    let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
		    Constraint::Min(1),
		    Constraint::Length(1),
		    Constraint::Length(1),
		])
		.split(f.size());
	    let mut ed = editor.borrow_mut();
	    let height = chunks[0].height.saturating_sub(2) as usize;
	    let width  = chunks[0].width.saturating_sub(2) as usize;
	    let snapshot = {
		let mut lua_ev = lua_events.borrow_mut();
		let status_info = build_status(&ed);
		ed.event_queue.extend(lua_ev.drain(..));
		(
		    render_status(&status_info),
		    ed.minibuffer.get().to_string(),
		)
	    };
	    let ( status, msg, ) = snapshot;
	    ed.viewport_height = height;
	    ed.viewport_width  = width;

	    let visuals = ed.build_visual_lines();
	    let rendered = render_visuals(&ed, &visuals);

	    let visible = rendered
		.into_iter()
		.skip(ed.scroll_y)
		.take(ed.viewport_height)
		.collect::<Vec<_>>();
	    let (cx, cy) = ed.cursor_visual_pos();
	    let buffer = if user_config.borrow().buffer_borders {
		 f.set_cursor(cx as u16 + 1, (cy.saturating_sub(ed.scroll_y)) as u16 + 1,);
		Paragraph::new(visible)
		    .block(Block::default().borders(Borders::ALL))
	    } else {
		f.set_cursor(cx as u16, (cy.saturating_sub(ed.scroll_y)) as u16,);
		Paragraph::new(visible)
		    .block(Block::default())
	    };
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


