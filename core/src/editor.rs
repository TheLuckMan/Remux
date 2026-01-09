use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use bitflags::bitflags;
use mlua::Lua;
use crate::{
    command::{CommandRegistry, CommandContext, CommandArg, Interactive},
    minibuffer::{MiniBuffer, MiniBufferMode},
    hooks::HookRegistry,
    buffer::Buffer,
};

/// ---- Prefix / Argument Handling ----
#[derive(Debug, Clone, Copy)]
pub enum PrefixState {
    None,
    Universal(i32),
    Digits(i32),
}

impl Default for PrefixState {
    fn default() -> Self { PrefixState::None }
}

impl PrefixState {
    pub fn consume(&mut self) -> Option<i64> {
        match *self {
            PrefixState::Digits(v) | PrefixState::Universal(v) => {
                *self = PrefixState::None;
                Some(v as i64)
            }
            PrefixState::None => None,
        }
    }
}

// ---- Editor Mode ----
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode { Normal, MiniBuffer }

// ---- Editor Events ----
pub enum EditorEvent {
    ExecuteCommand(String),
    Message(String),
    OpenFile(String),
    AddHook { name: String, func: mlua::RegistryKey },
}

// ---- Line Wrapping ----
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineWrapMode { Truncate, Wrap }

pub struct VisualLine {
    pub buffer_y: usize,
    pub start_x: usize,
    pub len: usize,
}

// ---- Modifiers / KeyMap ----
bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PhysicalModifiers: u8 {
        const CTRL = 0b0001;
        const ALT = 0b0010;
        const SHIFT = 0b0100;
        const SUPER = 0b1000;
    }
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Modifiers: u8 {
        const MOD0 = 0b0001;
        const MOD1 = 0b0010;
        const MOD2 = 0b0100;
    }
}

impl Modifiers { pub fn none() -> Self { Self::empty() } }

pub struct KeyMap {
    bindings: HashMap<(Modifiers, char), String>,
}

impl KeyMap {
    pub fn new() -> Self { Self { bindings: HashMap::new() } }
    pub fn bind(&mut self, mods: Modifiers, key: char, cmd: String) { self.bindings.insert((mods,key), cmd); }
    pub fn lookup(&self, mods: Modifiers, key: char) -> Option<&String> {
        self.bindings.iter()
            .find(|((bm, bk), _)| *bk == key && mods.contains(*bm))
            .map(|(_, cmd)| cmd)
    }
}

// ---- Editor ----
pub struct Editor {
    pub buffer: Buffer,
    pub keymap: Rc<RefCell<KeyMap>>,
    pub kill_buffer: Option<String>,
    pub minibuffer: MiniBuffer,
    pub commands: CommandRegistry,
    pub hooks: HookRegistry,
    pub mode: InputMode,
    pub should_quit: bool,
    pub event_queue: Vec<EditorEvent>,
    pub wrap_mode: LineWrapMode,
    pub scroll_y: usize,
    pub scroll_x: usize,
    pub viewport_height: usize,
    pub viewport_width: usize,
    pub pending_prefix: Option<usize>,
    pub prefix: PrefixState,
    pub pending_command: Option<String>,
}

impl Editor {
    pub fn new(commands: CommandRegistry, keymap: Rc<RefCell<KeyMap>>) -> Self {
        Self {
            buffer: Buffer::new(),
	    keymap,
	    kill_buffer: None,
            minibuffer: MiniBuffer::default(),
            commands,
            hooks: HookRegistry::new(),
            mode: InputMode::Normal,
            should_quit: false,
            event_queue: Vec::new(),
            wrap_mode: LineWrapMode::Wrap,
            scroll_y: 0,
            scroll_x: 0,
            viewport_height: 0,
            viewport_width: 0,
            pending_prefix: None,
            prefix: PrefixState::None,
            pending_command: None,
        }
    }

    // ---- Event Handling ----
    pub fn process_events(&mut self, lua: &Lua) {
        self.minibuffer.tick();
        let events = std::mem::take(&mut self.event_queue);
        for ev in events {
            match ev {
                EditorEvent::ExecuteCommand(name) => self.execute_named(&name, lua),
                EditorEvent::Message(msg) => self.minibuffer.message(&msg),
                EditorEvent::OpenFile(path) => { let _ = self.buffer.open_file(path.into()); }
                EditorEvent::AddHook { name, func } => self.hooks.add_key(name, func),
            }
        }
    }

    // ---- Scroll / Viewport ----
    pub fn scroll_indicator(&self) -> String {
	let total = match self.wrap_mode {
            LineWrapMode::Truncate => self.buffer.lines.len(),
            LineWrapMode::Wrap => self.buffer.lines.iter().map(|l| {
		let w = self.viewport_width.max(1);
		(l.chars().count().max(1) + w - 1) / w
            }).sum(),
	};

	let vh = self.viewport_height;

	if total <= vh { return "All".to_string(); }
	if self.scroll_y == 0 { return "Top".to_string(); }
	if self.scroll_y >= total - vh { return "Bot".to_string(); }

	let percent = self.scroll_y.saturating_mul(100) / (total - vh).max(1);
	format!("{}%", percent)
    }
    
    fn max_scroll_y(&self) -> usize {
        let total = match self.wrap_mode {
            LineWrapMode::Truncate => self.buffer.lines.len(),
            LineWrapMode::Wrap => self.buffer.lines.iter().map(|l| {
                let w = self.viewport_width.max(1);
                (l.chars().count().max(1) + w - 1) / w
            }).sum(),
        };
        total.saturating_sub(self.viewport_height)
    }

    pub fn clamp_scroll(&mut self) { self.scroll_y = self.scroll_y.min(self.max_scroll_y()); }

    pub fn ensure_cursor_visible(&mut self) {
        let width = self.viewport_width.max(1);
        let height = self.viewport_height.max(1);

        let (_cx, vy) = self.cursor_visual_pos();

        // Vertical
        if vy < self.scroll_y { self.scroll_y = vy; }
        else if vy >= self.scroll_y + height { self.scroll_y = vy + 1 - height; }

        // Horizontal (truncate only)
        if self.wrap_mode == LineWrapMode::Truncate {
            let cx = self.buffer.cursor_x;
            if cx < self.scroll_x { self.scroll_x = cx; }
            else if cx >= self.scroll_x + width { self.scroll_x = cx + 1 - width; }
        } else {
            self.scroll_x = 0;
        }

        self.clamp_scroll();
    }

    pub fn cursor_visual_pos(&self) -> (usize, usize) {
        let width = self.viewport_width.max(1);
        let mut vy = 0;

        for y in 0..self.buffer.cursor_y {
            let len = self.buffer.lines[y].chars().count().max(1);
            vy += match self.wrap_mode {
                LineWrapMode::Truncate => 1,
                LineWrapMode::Wrap => (len + width - 1) / width,
            };
        }

        let vx = match self.wrap_mode {
            LineWrapMode::Truncate => self.buffer.cursor_x.saturating_sub(self.scroll_x),
            LineWrapMode::Wrap => self.buffer.cursor_x % width,
        };

        let vy = match self.wrap_mode {
            LineWrapMode::Truncate => vy,
            LineWrapMode::Wrap => vy + self.buffer.cursor_x / width,
        };

        (vx, vy)
    }

    pub fn build_visual_lines(&self) -> Vec<VisualLine> {
        let width = self.viewport_width.max(1);
        let mut all = Vec::new();

        for (y, line) in self.buffer.lines.iter().enumerate() {
            let len = line.chars().count().max(1);
            match self.wrap_mode {
                LineWrapMode::Truncate => all.push(VisualLine { buffer_y: y, start_x: self.scroll_x, len: width }),
                LineWrapMode::Wrap => {
                    let mut x = 0;
                    while x < len {
                        all.push(VisualLine { buffer_y: y, start_x: x, len: width });
                        x += width;
                    }
                }
            }
        }

        all.into_iter().skip(self.scroll_y).take(self.viewport_height).collect()
    }

    pub fn scroll_up(&mut self) { self.scroll_y += self.viewport_height; self.clamp_scroll(); }
    pub fn scroll_down(&mut self) { self.scroll_y = self.scroll_y.saturating_sub(self.viewport_height); self.clamp_scroll(); }

    pub fn scroll_left(&mut self) { if self.wrap_mode == LineWrapMode::Truncate { self.scroll_x = self.scroll_x.saturating_sub(4); } }
    pub fn scroll_right(&mut self) {
        if self.wrap_mode == LineWrapMode::Truncate {
            let max = self.buffer.lines.iter().map(|l| l.chars().count().saturating_sub(self.viewport_width.max(1))).max().unwrap_or(0);
            self.scroll_x = (self.scroll_x + 4).min(max);
        }
    }

    pub fn set_wrap_mode(&mut self, mode: LineWrapMode) {
        self.wrap_mode = mode;
        if mode == LineWrapMode::Wrap { self.scroll_x = 0; }
    }

    // ---- Prefix / Command Execution ----
    pub fn run_command<F>(&mut self, lua: &Lua, name: &str, f: F)
    where F: FnOnce(&mut Self)
    {
        self.hooks.run(lua, "before-command", name);
        f(self);
        self.hooks.run(lua, "after-command", name);
    }
    
    pub fn execute_named(&mut self, name: &str, lua: &Lua) {
        if let Some(cmd) = self.commands.get(name) {
            if name == "universal-argument" {
                self.run_command(lua, name, |ed| (cmd.as_ref().run)(CommandContext { editor: ed, arg: CommandArg::None }));
                self.process_events(lua);
                return;
            }

            if let Interactive::Str { prompt } = cmd.interactive {
                self.mode = InputMode::MiniBuffer;
                self.pending_command = Some(name.to_string());
                let mode = match name {
                    "save-buffer-as" => MiniBufferMode::SaveBuffer,
                    "find-file" => MiniBufferMode::FindFile,
                    _ => MiniBufferMode::Command,
                };
                self.minibuffer.activate(prompt, mode);
                return;
            }

            let arg = self.prefix.consume().map_or(CommandArg::None, CommandArg::Int);
            self.run_command(lua, name, |ed| (cmd.as_ref().run)(CommandContext { editor: ed, arg }));
            self.ensure_cursor_visible();
            self.process_events(lua);
        } else {
            self.minibuffer.message(&format!("Unknown command: {name}"));
        }
    }

    pub fn push_kill(&mut self, text: String) {
        self.kill_buffer = Some(text);
    }

    pub fn execute_minibuffer(&mut self, lua: &Lua) {
	let mode = self.minibuffer.mode(); // сохраняем сразу
	let input = match mode {
            MiniBufferMode::Command => self.minibuffer.get().strip_prefix("M-x ").unwrap_or("").trim(),
            MiniBufferMode::FindFile => self.minibuffer.get().strip_prefix("Find file: ").unwrap_or("").trim(),
            MiniBufferMode::SaveBuffer => self.minibuffer.get().strip_prefix("Save buffer as: ").unwrap_or("").trim(),
            _ => "",
	}.to_string();

	self.minibuffer.deactivate();
	self.mode = InputMode::Normal;

	match mode { // используем сохранённый
            MiniBufferMode::FindFile => {
		match self.buffer.open_file(input.into()) {
                    Ok(_) => self.minibuffer.message("Opened file"),
                    Err(e) => self.minibuffer.message(&format!("Open failed: {e}")),
		}
            }
            MiniBufferMode::SaveBuffer => {
		if input.is_empty() { self.minibuffer.message("Save failed: empty file name"); return; }
		match self.buffer.save_as(input.into()) {
                    Ok(_) => self.minibuffer.message("Saved buffer!"),
                    Err(e) => self.minibuffer.message(&format!("Save failed: {e}")),
		}
            }
            MiniBufferMode::Command => self.execute_named(&input, lua),
            _ => {}
	}
    }

}
