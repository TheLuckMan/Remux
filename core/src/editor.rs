use bitflags::bitflags;
use mlua::Lua;
use crate::buffer::Buffer;
use std::collections::HashMap;
use crate::minibuffer::{MiniBuffer, MiniBufferMode, };
use crate::command::{CommandRegistry, CommandContext };
use crate::hooks::HookRegistry;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    MiniBuffer,
}


pub enum EditorEvent {
    ExecuteCommand(String),
    Message(String),
    OpenFile(String),
    AddHook {
        name: String,
        func: mlua::RegistryKey,
    },
}
 

pub struct App {
    pub lua: Lua,
    pub editor: Editor,
}

pub struct Editor {
    pub buffer: Buffer,
    pub kill_buffer: Option<String>,
    pub minibuffer: MiniBuffer,
    pub commands: CommandRegistry,
    pub hooks: HookRegistry,
    pub mode: InputMode,
    pub should_quit: bool,
    pub event_queue: Vec<EditorEvent>,
    pub scroll_y: usize,
    pub viewport_height: usize,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PhysicalModifiers: u8 {
        const CTRL  = 0b0001;
        const ALT   = 0b0010;
        const SHIFT = 0b0100;
        const SUPER = 0b1000;
    }
}

pub struct KeyEvent {
    pub modifier: Modifiers,
    pub key: char,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub mod_key: bool,
}

impl Modifiers {
    pub fn none() -> Self {
        Self { mod_key: false }
    }
}

pub struct KeyMap {
    bindings: HashMap<(Modifiers, char), String>,
}

impl KeyMap {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
    pub fn bind(&mut self, mods: Modifiers, key: char, cmd: String) {
	self.bindings.insert((mods, key), cmd);
    }

    pub fn lookup(&self, mods: Modifiers, key: char) -> Option<&String> {
	self.bindings.get(&(mods, key))
    }

}

impl Editor {
    pub fn new(commands: CommandRegistry) -> Self {
        Self {
            buffer: Buffer::new(),
	    kill_buffer: None,
	    minibuffer: MiniBuffer::default(),
	    commands,
	    event_queue: Vec::new(),
	    hooks: HookRegistry::new(),
            should_quit: false,
	    mode: InputMode::Normal,
	    scroll_y: 0,
	    viewport_height: 0,
        }
    }
    fn process_events(&mut self, lua: &Lua) {
	let events = std::mem::take(&mut self.event_queue);
/*
	if let MiniBufferMode::Message { ttl } = self.minibuffer.mode() {
	    if ttl <= 1 {
		self.minibuffer.clear();
	    } else {
		self.minibuffer.tick();
	    }
    }
*/      self.minibuffer.tick();	
	for ev in events {
            match ev {
		EditorEvent::ExecuteCommand(name) => {
                    self.execute_named(&name, lua);
		}
		EditorEvent::Message(msg) => {
                    self.minibuffer.message(&msg);
		}
		EditorEvent::OpenFile(path) => {
                    let _ = self.buffer.open_file(path.into());
		}
		EditorEvent::AddHook { name, func } => {
                    self.hooks.add_key(name, func);
                }
            }
	}
    }

    
    fn minibuffer_input(&self) -> &str {
        match self.minibuffer.mode() {
            MiniBufferMode::Command => {
                self.minibuffer
                    .get()
                    .strip_prefix("M-x ")
                    .unwrap_or("")
            }
            MiniBufferMode::FindFile => {
                self.minibuffer
                    .get()
                    .strip_prefix("Find file: ")
                    .unwrap_or("")
            }
            _ => "",
        }
    }

    pub fn clamp_scroll(&mut self) {
	let max = self.buffer.lines.len().saturating_sub(self.viewport_height);
	self.scroll_y = self.scroll_y.min(max);
    }

    pub fn ensure_cursor_visible(&mut self) {
        let vh = self.viewport_height;
        if vh == 0 {
            return;
        }

        // курсор выше окна
        if self.buffer.cursor_y < self.scroll_y {
            self.scroll_y = self.buffer.cursor_y;
        }

        // курсор ниже окна
        if self.buffer.cursor_y >= self.scroll_y + vh {
            self.scroll_y = self.buffer.cursor_y + 1 - vh;
        }

        self.clamp_scroll();
    }

    pub fn scroll_up(&mut self) {
        let max = self.buffer.lines.len().saturating_sub(1);
        self.scroll_y = (self.scroll_y + self.viewport_height)
            .min(max);

        if self.buffer.cursor_y < self.scroll_y {
            self.buffer.cursor_y = self.scroll_y;
        }
    }


    pub fn scroll_down(&mut self) {
        if self.scroll_y >= self.viewport_height {
            self.scroll_y -= self.viewport_height;
        } else {
            self.scroll_y = 0;
        }

        if self.buffer.cursor_y < self.scroll_y {
            self.buffer.cursor_y = self.scroll_y;
        }
    }

    pub fn execute_named(&mut self, name: &str, lua: &Lua) {
	self.hooks.run(lua, "before-command", name);
	if let Some(cmd) = self.commands.get(name) {
            (cmd.run)(CommandContext {
		editor: self,
		arg: None,
            });
	} else {
            self.minibuffer.message("Unknown command");
	}
	self.hooks.run(lua, "after-command", name);

	self.process_events(lua);
    }
    pub fn execute_minibuffer(&mut self, lua: &Lua) {
	let input = self.minibuffer_input().trim().to_string();
	let mode = self.minibuffer.mode();

	self.minibuffer.deactivate();
	self.mode = InputMode::Normal;

	match mode {
            MiniBufferMode::FindFile => {
		match self.buffer.open_file(input.into()) {
                    Ok(_) => self.minibuffer.message("Opened file"),
                    Err(e) => self.minibuffer.message(&format!("Open failed: {e}")),
		}
            }

            MiniBufferMode::Command => {
		self.execute_named(&input, lua);
            }

            _ => {}
	}
    }

}
