use bitflags::bitflags;
use crate::buffer::Buffer;
use std::collections::HashMap;
use crate::minibuffer::{MiniBuffer, MiniBufferMode, };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    MiniBuffer,
}

pub struct Editor {
    pub buffer: Buffer,
    pub minibuffer: MiniBuffer,
    pub mode: InputMode,
    pub should_quit: bool,
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

#[derive(Debug, Clone)]
pub enum Command {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    KillRemux,
    Insert(char),
    DeleteChar,
    BackwardDeleteChar,
    NewLine,
    FindFile(String),
    SaveBuffer,
    MoveBeginningOfLine,
    MoveEndOfLine,
    MoveBeginningOfBuffer,
    MoveEndOfBuffer,
    ExecuteCommand,
}

pub struct KeyMap {
    bindings: HashMap<(Modifiers, char), Command>,
}


impl KeyMap {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn bind(&mut self, mods: Modifiers, key: char, cmd: Command) {
        self.bindings.insert((mods, key), cmd);
    }

    pub fn lookup(&self, mods: Modifiers, key: char) -> Option<&Command> {
        self.bindings.get(&(mods, key))
    }
/*
    pub fn register_command(&mut self, name: &str, cmd: Command) {
        self.commands.insert(name.to_string(), cmd);
    }

    pub fn command_by_name(&self, name: &str) -> Option<&Command> {
        self.commands.get(name)
}
    */
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
	    minibuffer: MiniBuffer::default(),
            should_quit: false,
	    mode: InputMode::Normal
        }
    }
    
    pub fn execute_minibuffer(&mut self) {
	let input = self.minibuffer.get().trim().to_string();
	
	match self.minibuffer.mode() {
	    MiniBufferMode::Command => {
		let cmd = input.strip_prefix("M-x ").unwrap_or("").trim();
		match cmd {
		    "find-file" => {
			self.minibuffer.activate(
			    "Find file: ",
			    MiniBufferMode::FindFile,
			);
			self.mode = InputMode::MiniBuffer;
			return;
		    }
		    "save-buffer" => {
			self.buffer.save();
			self.minibuffer.deactivate();
                    }
                    "kill-remux" => self.should_quit = true,
                    _ => {
			self.minibuffer.activate(
                            "Unknown command",
                            MiniBufferMode::Message,
			);
                    }
		}
            }

	    MiniBufferMode::FindFile => {
		let path = input.strip_prefix("Find file: ").unwrap_or("").trim();
		match self.buffer.open_file(path.into()) {
		    Ok(_) => {
			self.minibuffer.activate(
			    "Opened file",
			    MiniBufferMode::Message,
			);
		    }
		    Err(e) => {
			self.minibuffer.activate(
			    &format!("Open failed: {}", e),
			    MiniBufferMode::Message,
			);
		    }
		}
	    }

	    MiniBufferMode::Message => {
		self.minibuffer.deactivate();
	    }
	}
	
	self.mode = InputMode::Normal;
    }

    pub fn execute_named_command(&mut self, name: &str) {
	match name {
            "find-file" => {
		self.mode = InputMode::MiniBuffer;
		self.minibuffer.activate("Find file: ", MiniBufferMode::FindFile);
            }
            "save-buffer" => {
		match self.buffer.save() {
                    Ok(_) => {
			self.minibuffer.activate("Buffer saved!", MiniBufferMode::Message);
                    }
                    Err(e) => {
			self.minibuffer.activate(
                            &format!("Save failed: {}", e),
                            MiniBufferMode::Message,
			);
                    }
		}
            }
            "kill-remux" => self.should_quit = true,
            _ => {
		self.minibuffer.activate("Unknown command", MiniBufferMode::Message);
            }
	}
    }
    
    
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    
    pub fn execute(&mut self, cmd: Command) {
        match cmd {
            Command::MoveLeft => self.buffer.move_left(),
            Command::MoveRight => self.buffer.move_right(),
            Command::MoveUp => self.buffer.move_up(),
            Command::MoveDown => self.buffer.move_down(),
	    Command::MoveBeginningOfLine => self.buffer.move_bol(),
	    Command::MoveEndOfLine => self.buffer.move_eol(),
	    Command::MoveBeginningOfBuffer => self.buffer.move_beginning_of_buffer(),
            Command::MoveEndOfBuffer => self.buffer.move_end_of_buffer(),
            Command::KillRemux => self.quit(),
            Command::Insert(c) => self.buffer.insert_char(c),
	    Command::DeleteChar => self.buffer.delete_char(),
	    Command::BackwardDeleteChar => self.buffer.backward_delete_char(),
	    Command::NewLine => self.buffer.insert_newline(),
	    Command::FindFile(path) => {
		match self.buffer.open_file(path.clone().into()) {
		    Ok(_) => self.minibuffer.set_text(format!("Opened {}", path)),
		    Err(e) => self.minibuffer.set_text(format!("Open failed: {}", e)),
		}
	    }
	    Command::SaveBuffer => {
		match self.buffer.save() {
		    Ok(_) => self.minibuffer.set_text("Buffer was saved"),
		    Err(e) => self.minibuffer.set_text(format!("Save failed: {}", e)),
		}
	    }
	    Command::ExecuteCommand => {
		self.mode = InputMode::MiniBuffer;
		self.minibuffer.activate("M-x ", MiniBufferMode::Command);
	    }
        }
    }
}
