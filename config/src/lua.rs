use mlua::{Result, Lua};
use std::rc::Rc;
use std::cell::RefCell;
use remux_core::editor::{KeyMap, Editor, Modifiers, PhysicalModifiers, EditorEvent};
use remux_core::config::{config_path, UserConfig};


pub fn parse_modifiers(s: &str) -> Modifiers {
    let mut mods = Modifiers::empty();

    for part in s.split('+') {
        let idx = match part.trim().to_lowercase().as_str() {
            "mod0" => Some(0),
            "mod1" => Some(1),
            "mod2" => Some(2),
            _ => None,
        };

        if let Some(i) = idx {
            mods.insert(Modifiers::from_bits_truncate(1 << i));
        }
    }

    mods
}


pub fn parse_mod_mask(s: &str) -> (PhysicalModifiers, Option<char>) {
    let mut phys = PhysicalModifiers::empty();
    let mut key = None;

    for part in s.split('+') {
        match part.trim().to_lowercase().as_str() {
            "ctrl" | "control" => phys |= PhysicalModifiers::CTRL,
            "alt"              => phys |= PhysicalModifiers::ALT,
            "shift"            => phys |= PhysicalModifiers::SHIFT,
            "super" | "meta"   => phys |= PhysicalModifiers::SUPER,
            s if s.len() == 1  => {
                key = Some(s.chars().next().unwrap());
            }
            _ => {}
        }
    }

    (phys, key)
}

pub fn load_lua(
    lua: &Lua,
    editor: Rc<RefCell<Editor>>,
    keymap: Rc<RefCell<KeyMap>>,
    lua_events: Rc<RefCell<Vec<EditorEvent>>>,
    config: Rc<RefCell<UserConfig>>,
) -> Result<()> {
    let editor_hooks = editor.clone();
    let border_config = config.clone();
    let events = lua_events.clone();
    
    lua.globals().set(
	"bind",
	lua.create_function(move |_, (mod_str, key, cmd): (String, String, String)| {
	    let key = key.chars().next().unwrap();
            let mods = parse_modifiers(&mod_str);
	    keymap.borrow_mut().bind(mods, key, cmd);
            Ok(())
	})?,
    )?;
    
   lua.globals().set(
    "bind_mod",
    lua.create_function(move |_, (n, combo): (usize, String)| {
        if n >= 3 {
            return Err(mlua::Error::RuntimeError(
                "bind_mod: MOD index out of range".into(),
            ));
        }

        let (phys, key) = parse_mod_mask(&combo);
        let mut cfg = config.borrow_mut();

        match key {
            Some(c) => {
                cfg.prefix_keys[n]  = Some(c);
                cfg.prefix_masks[n] = phys;
                cfg.mod_masks[n]    = PhysicalModifiers::empty();
            }
            None => {
                cfg.prefix_keys[n]  = None;
                cfg.prefix_masks[n] = PhysicalModifiers::empty();
                cfg.mod_masks[n]    = phys;
            }
        }

        Ok(())
    })?,
)?;

    
    lua.globals().set(
	"execute",
	lua.create_function(move |_, name: String| {
	    events.borrow_mut()
		.push(EditorEvent::ExecuteCommand(name));
            Ok(())
	})?,
    )?;

    lua.globals().set(
	"add_hook",
	lua.create_function(move |lua, (name, func): (String, mlua::Function)| {
	    let key = lua.create_registry_value(func)?;
	    editor_hooks.borrow_mut()
		.event_queue
		.push(EditorEvent::AddHook { name, func: key });
            Ok(())
	})?,
    )?;

    let events = lua_events.clone();
    lua.globals().set(
	"message",
	lua.create_function(move |_, msg: String| {
	    events.borrow_mut().push(EditorEvent::Message(msg));
            Ok(())
	})?,
    )?;

    lua.globals().set(
	"set_buffer_borders",
	lua.create_function(move |_, enabled: bool| {
            border_config.borrow_mut().buffer_borders = enabled;
            Ok(())
	})?,
    )?;

    let path = config_path();
    if path.exists() {
	lua.load(std::fs::read_to_string(&path)?).exec()?;
    } else {
	println!("Configuration file not found! Ctrl-c to exit [copy 'init.lua' to '~/.config/remux/init.lua']");
    }

   Ok(())
}
