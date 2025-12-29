use mlua::{Lua, Result};
use std::rc::Rc;
use std::cell::RefCell;

use crate::editor::{KeyMap, Editor, Modifiers, Command, PhysicalModifiers};
use crate::config::{config_path, UserConfig};

fn parse_mod_mask(s: &str) -> PhysicalModifiers {
    let mut mods = PhysicalModifiers::empty();

    for part in s.split('+') {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => mods |= PhysicalModifiers::CTRL,
            "alt"              => mods |= PhysicalModifiers::ALT,
            "shift"            => mods |= PhysicalModifiers::SHIFT,
            "super" | "meta"   => mods |= PhysicalModifiers::SUPER,
            _ => {}
        }
    }

    mods
}

 fn parse_modifiers(s: &str) -> Modifiers {
    let mut mods = Modifiers::none();

    for part in s.split('+') {
        match part.trim().to_lowercase().as_str() {
            "mod" => mods.mod_key = true,
            _ => {
	    }
        }
    }

    mods
}

pub fn load_lua(
    editor: Rc<RefCell<Editor>>,
    keymap: Rc<RefCell<KeyMap>>,
    config: Rc<RefCell<UserConfig>>,
) -> Result<()> {

    let lua = Lua::new();  
    
    let bind = lua.create_function(move |_, (mod_str, key, cmd): (String, String, String)| {

	
	let key = key.chars().next().unwrap();
        let mods = parse_modifiers(&mod_str);
        let command = match cmd.as_str() {
            "move-left" => Command::MoveLeft,
            "move-right" => Command::MoveRight,
            "move-up" => Command::MoveUp,
            "move-down" => Command::MoveDown,
	    "move-beginning-of-line" => Command::MoveBeginningOfLine,
	    "move-end-of-line" => Command::MoveEndOfLine,
	    "move-beginning-of-buffer" => Command::MoveBeginningOfBuffer,
	    "move-end-of-buffer" => Command::MoveEndOfBuffer,
            "kill-remux" => Command::KillRemux,
	    "execute-command" => Command::ExecuteCommand,
	    "delete-char" => Command::DeleteChar,
	    "backward-delete-char" => Command::BackwardDeleteChar,
	    "newline" => Command::NewLine,
//	    "find-file" => Command::FindFile,
	    "save-buffer" => Command::SaveBuffer,
	    
            _ => return Err(mlua::Error::RuntimeError("unknown command".into())),
        };
	keymap.borrow_mut().bind(mods, key, command);
//	keymap.borrow_mut().bind(mods, keycode, cmd);
        Ok(())
    })?;

    
    lua.globals().set("bind", bind)?;

    lua.globals().set(
    "bind_mod",
    lua.create_function(move |_, mods: String| {
        let mask = parse_mod_mask(&mods);
        config.borrow_mut().mod_mask = mask;
		Ok(())
    })?,
)?;

    let path = config_path();
    if path.exists() {
        lua.load(std::fs::read_to_string(path)?).exec()?;
    }

    let editor_open = editor.clone();
lua.globals().set(
    "find_file",
    lua.create_function(move |_, path: String| {
        editor_open.borrow_mut().buffer.open_file(path.into()).unwrap();
        Ok(())
    })?,
)?;

let editor_save = editor.clone();
lua.globals().set(
    "save_buffer",
    lua.create_function(move |_, ()| {
        editor_save.borrow_mut().buffer.save().unwrap();
        Ok(())
    })?,
)?;


    	Ok(())
}
