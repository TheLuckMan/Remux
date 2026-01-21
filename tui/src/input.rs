use std::io;
use std::rc::Rc;
use std::cell::RefCell;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers};
use remux_core::{
    editor::editor::{Editor, InputMode, PhysicalModifiers, Modifiers, KeyMap},
    buffer::Motion,
    config::UserConfig,
    minibuffer::MiniBufferMode,
};

use mlua::Lua;

/// Convetring
pub fn physical_from_key_event(key: &KeyEvent) -> PhysicalModifiers {
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

pub fn logical_modifiers(
    physical: PhysicalModifiers,
    key: KeyCode,
    config: &UserConfig,
    editor: &mut Editor,
) -> Modifiers {
    let mut mods = Modifiers::empty();

    for i in 0..3 {
        // prefix
        if let Some(pk) = config.prefix_keys[i] {
            if let KeyCode::Char(c) = key {
                if c == pk && physical.intersects(config.prefix_masks[i]) {
                    editor.pending_prefix = Some(i);
                    return Modifiers::empty();
                }
            }
        }

        if physical.intersects(config.mod_masks[i]) {
            mods.insert(Modifiers::from_bits_truncate(1 << i));
        }
    }

    // consume prefix
    if let Some(p) = editor.pending_prefix.take() {
        mods.insert(Modifiers::from_bits_truncate(1 << p));
    }

    mods
}
pub fn handle_input(
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
            InputMode::Normal => handle_normal_input(lua, editor, keymap, user_config, key)?,
            InputMode::MiniBuffer => handle_minibuffer_input(lua, editor, keymap, user_config, key)?,
        }
    }

    Ok(())
}
// Normal mode
fn handle_normal_input(
    lua: &Lua,
    editor: &Rc<RefCell<Editor>>,
    keymap: &Rc<RefCell<KeyMap>>,
    user_config: &Rc<RefCell<UserConfig>>,
    key: KeyEvent,
) -> io::Result<()> {
    let physical = physical_from_key_event(&key);
    let mut ed = editor.borrow_mut();
    let mods = logical_modifiers(physical, key.code, &user_config.borrow(), &mut ed);

    // Ctrl-c for exit
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        ed.should_quit = true;
        return Ok(());
    }

    match key.code {
        KeyCode::Char(c) => {
            if let Some(cmd) = keymap.borrow().lookup(mods, c) {
                ed.execute_named(&cmd, lua);
            } else if mods.is_empty() {
                ed.insert_char(lua, c);
            }
        }
        KeyCode::Left => ed.buffer.move_cursor(Motion::Left),
        KeyCode::Right => ed.buffer.move_cursor(Motion::Right),
        KeyCode::Up => ed.buffer.move_cursor(Motion::Up),
        KeyCode::Down => ed.buffer.move_cursor(Motion::Down),
        KeyCode::Backspace => { _ =  ed.buffer.delete(Motion::Left); }
        KeyCode::Enter => ed.insert_newline(),
        KeyCode::Delete => { _ = ed.buffer.delete(Motion::Right); }
        _ => {}
    }

    ed.ensure_cursor_visible();
    ed.clamp_scroll();
    Ok(())
}

/// MiniBuffer Mode
fn handle_minibuffer_input(
    lua: &Lua,
    editor: &Rc<RefCell<Editor>>,
    keymap: &Rc<RefCell<KeyMap>>,
    user_config: &Rc<RefCell<UserConfig>>,
    key: KeyEvent,
) -> io::Result<()> {
    let physical = physical_from_key_event(&key);
    let mods = logical_modifiers(physical, key.code, &user_config.borrow(), &mut editor.borrow_mut());

    match key.code {
        KeyCode::Char(c) => {
            if let Some(cmd) = keymap.borrow().lookup(mods, c) {
                editor.borrow_mut().execute_named(&cmd, lua);
	    } else if mods.is_empty() {
		let mut ed = editor.borrow_mut();
		ed.minibuffer.push(c);

		if ed.minibuffer.mode() == MiniBufferMode::ISearchForward {
		    ed.isearch_update();
		}
	    }
        }
	KeyCode::Backspace => {
	    let mut ed = editor.borrow_mut();
	    ed.minibuffer.pop();

	    if ed.minibuffer.mode() == MiniBufferMode::ISearchForward {
		ed.isearch_update();
	    }
	}
        KeyCode::Enter => editor.borrow_mut().execute_minibuffer(lua),
        KeyCode::Esc => {
            editor.borrow_mut().minibuffer.deactivate();
            editor.borrow_mut().mode = InputMode::Normal;
        }
        _ => {}
    }

    Ok(())
}
