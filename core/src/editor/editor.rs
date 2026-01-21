use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use bitflags::bitflags;
use mlua::Lua;
use crate::{
    command::{CommandRegistry, CommandContext, CommandArg, Interactive},
    minibuffer::{MiniBuffer, MiniBufferMode},
    editor::hooks::HookRegistry,
    editor::layout::LineWrapMode,
    buffer::Buffer,
    config::UserConfig,
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

impl InputMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            InputMode::Normal => "normal",
            InputMode::MiniBuffer => "minibuffer",
        }
    }
}


// ---- Editor Events ----
#[derive(Debug)]
pub enum EditorEvent {
    ExecuteCommand(String),
    Message(String),
    OpenFile(String),
    AddHook { name: String, func: mlua::RegistryKey },
}

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ISearchDir {
    Forward,
    Backward,
}

pub struct ISearchState {
    pub original_x: usize,
    pub original_y: usize,
    pub query: String,
    pub dir: ISearchDir,
    pub last_match: Option<(usize, usize)>,
}


pub enum ScrollIntent {
    FollowCursor,
    Manual,
}

pub struct VisibleVisualLines<'a> {
    editor: &'a Editor,
    cur_vy: usize,
    remaining: usize,
    buf_y: usize,
    buf_vy_base: usize,
}

impl<'a> VisibleVisualLines<'a> {
    pub fn empty(editor: &'a Editor) -> Self {
        Self {
            editor,
            cur_vy: 0,
            remaining: 0,
            buf_y: 0,
            buf_vy_base: 0,
        }
    }
}

impl<'a> Iterator for VisibleVisualLines<'a> {
    type Item = VisualLine;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let ed = self.editor;

        while self.buf_y < ed.buffer.lines.len() {
            let line = &ed.buffer.lines[self.buf_y];
	    let vh = line.visual_height;

            if self.cur_vy >= self.buf_vy_base + vh {
                self.buf_vy_base += vh;
                self.buf_y += 1;
                continue;
            }

            let sub = self.cur_vy - self.buf_vy_base;

            let start_x = match ed.wrap_mode {
                LineWrapMode::Truncate => ed.scroll_x,
                LineWrapMode::Wrap => sub * ed.viewport_width,
            };

            self.cur_vy += 1;
            self.remaining -= 1;

            return Some(VisualLine {
                buffer_y: self.buf_y,
                start_x,
                len: ed.viewport_width,
            });
        }

        None
    }
}

// ---- Editor ----
pub struct Editor {
    pub buffer: Buffer,
    pub keymap: Rc<RefCell<KeyMap>>,
    pub kill_buffer: Option<String>,
    pub minibuffer: MiniBuffer,
    pub user_config: Rc<RefCell<UserConfig>>,
    pub commands: CommandRegistry,
    pub hooks: HookRegistry,
    pub mode: InputMode,
    pub should_quit: bool,
    pub event_queue: Vec<EditorEvent>,
    pub wrap_mode: LineWrapMode,
    pub scroll_y: usize,
    pub scroll_x: usize,
    pub command_arg: CommandArg,
    pub viewport_height: usize,
    pub viewport_width: usize,
    pub pending_prefix: Option<usize>,
    pub prefix: PrefixState,
    pub pending_command: Option<String>,
    pub last_cursor: (usize, usize),
    pub scroll_intent: ScrollIntent,
    pub isearch: Option<ISearchState>,

}

impl Editor {
    pub fn new(commands: CommandRegistry, keymap: Rc<RefCell<KeyMap>>, user_config: Rc<RefCell<UserConfig>>) -> Self {
        Self {
            buffer: Buffer::new(),
	    keymap,
	    kill_buffer: None,
            minibuffer: MiniBuffer::default(),
	    user_config,
            commands,
            hooks: HookRegistry::new(),
            mode: InputMode::Normal,
            should_quit: false,
            event_queue: Vec::new(),
            wrap_mode: LineWrapMode::Wrap,
            scroll_y: 0,
            scroll_x: 0,
	    command_arg: CommandArg::None,
            viewport_height: 0,
            viewport_width: 0,
            pending_prefix: None,
            prefix: PrefixState::None,
            pending_command: None,
	    last_cursor: (0,0),
	    scroll_intent: ScrollIntent::FollowCursor,
	    isearch: None,
        }
    }

    // ---- Event Handling ----
    pub fn process_events(&mut self, lua: &Lua) {
        let events = std::mem::take(&mut self.event_queue);
        for ev in events {
	    self.hooks.run(lua, "on-event", &format!("{:?}", ev));
            match ev {
                EditorEvent::ExecuteCommand(name) => self.execute_named(&name, lua),
                EditorEvent::Message(msg) => self.minibuffer.activate(&msg, MiniBufferMode::Message { ttl: 2 }),
		EditorEvent::OpenFile(path) => {
		    if self.buffer.open_file(path.clone().into()).is_ok() {
			self.scroll_y = 0;
			self.scroll_x = 0;
			self.rebuild_visual_metrics();
			self.hooks.run(lua, "buffer-loaded", &path);
		    }
		}
		EditorEvent::AddHook { name, func } => self.hooks.add_key(name, func),
	    }
        }
	self.minibuffer.tick();
    }
    
    fn emit_cursor_moved(&mut self, lua: &Lua) {
	let cur = (self.buffer.cursor_x, self.buffer.cursor_y);
	if cur != self.last_cursor {
            self.last_cursor = cur;
            self.hooks.run(lua, "cursor-moved", &format!("{},{}", cur.0, cur.1));
	}
    }
    
    fn emit_buffer_changed(&mut self, lua: &Lua, reason: &str) {
        self.hooks.run(lua, "buffer-changed", reason);
    }
    
    pub fn insert_char(&mut self, lua: &Lua, ch: char) {
	self.hooks.run(lua, "before-insert-char", "");
	self.buffer.insert_char_raw(ch);
	self.scroll_intent = ScrollIntent::FollowCursor;
	self.ensure_cursor_visible();
	self.emit_cursor_moved(lua);
	self.emit_buffer_changed(lua, "insert-char");
	self.hooks.run(lua, "after-insert-char", "");
    }
    
    pub fn set_mode(&mut self, lua: &Lua, mode: InputMode) {
        if self.mode != mode {
            self.mode = mode;
            self.hooks.run(lua, "mode-changed", mode.as_str());
        }
    }
    // ---- Scroll / Viewport ----
    pub fn scroll_indicator(&self) -> String {
	let total = *self.buffer.visual.prefix_sum.last().unwrap_or(&0);
	let vh = self.viewport_height;
	if total <= vh { return "All".to_string(); }
	if self.scroll_y == 0 { return "Top".to_string(); }
	if self.scroll_y >= total - vh { return "Bot".to_string(); }
	let percent = self.scroll_y.saturating_mul(100) / (total - vh).max(1);
	format!("{}%", percent)
    }

    fn max_scroll_y(&self) -> usize {
	match self.wrap_mode {
            LineWrapMode::Truncate =>
		self.buffer.lines.len().saturating_sub(self.viewport_height),
            LineWrapMode::Wrap =>
		usize::MAX
	}
    }
    
    pub fn clamp_scroll(&mut self) { self.scroll_y = self.scroll_y.min(self.max_scroll_y()); }
    
    fn cursor_global_visual_y(&self) -> usize {
	let ps = &self.buffer.visual.prefix_sum;

	if ps.is_empty() {
            return 0;
	}

	let cy = self.buffer.cursor_y.min(ps.len() - 1);
	let mut vy = ps[cy];

	if self.wrap_mode == LineWrapMode::Wrap {
            let w = self.viewport_width.max(1);
            vy += self.buffer.cursor_x / w;
	}

	vy
    }

    pub fn goto_line(&mut self, line_1based: usize) {
	if self.buffer.lines.is_empty() {
            self.buffer.cursor_x = 0;
            self.buffer.cursor_y = 0;
            return;
	}
	let y = line_1based
            .saturating_sub(1)
            .min(self.buffer.lines.len() - 1);

	self.buffer.cursor_y = y;

	let line_len = self.buffer.lines[y].char_len;
	self.buffer.cursor_x = self.buffer.cursor_x.min(line_len);

	self.ensure_cursor_visible();
    }

    pub fn ensure_cursor_visible(&mut self) {
	if let ScrollIntent::Manual = self.scroll_intent {
            return;
	}
	self.buffer.ensure_visuals(
            self.viewport_width,
            self.wrap_mode,
	);
	let height = self.viewport_height.max(1);
	let cy = self.cursor_global_visual_y();
	if cy < self.scroll_y {
            self.scroll_y = cy;
	} else if cy >= self.scroll_y + height {
            self.scroll_y = cy + 1 - height;
	}
	if self.wrap_mode == LineWrapMode::Truncate {
            let width = self.viewport_width.max(1);
            let cx = self.buffer.cursor_x;
            if cx < self.scroll_x {
		self.scroll_x = cx;
            } else if cx >= self.scroll_x + width {
		self.scroll_x = cx + 1 - width;
            }
            let max = self.buffer.lines
		.get(self.buffer.cursor_y)
		.map(|l| l.text.chars().count().saturating_sub(width))
		.unwrap_or(0);

            self.scroll_x = self.scroll_x.min(max);
	}
	self.clamp_scroll();
    }
    
    pub fn iter_visible_visual_lines(&self) -> VisibleVisualLines<'_> {
	let ps = &self.buffer.visual.prefix_sum;
	if ps.is_empty() {
            return VisibleVisualLines::empty(self);
	}
	let buf_y = match ps.binary_search(&self.scroll_y) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
	};
	let buf_vy_base = ps[buf_y];
	VisibleVisualLines {
            editor: self,
            cur_vy: self.scroll_y,
            remaining: self.viewport_height,
            buf_y,
            buf_vy_base,
	}
    }
    
    pub fn rebuild_visual_metrics(&mut self) {
	let w = self.viewport_width;
	let wrap = self.wrap_mode;
	self.buffer.rebuild_visual_metrics(w, wrap);
    }
    pub fn cursor_visual_pos(&mut self) -> (usize, usize) {
	self.buffer.ensure_visuals(self.viewport_width, self.wrap_mode);
	let width = self.viewport_width.max(1);
	let mut visual_y = self.buffer.visual.prefix_sum[self.buffer.cursor_y];
	if self.wrap_mode == LineWrapMode::Wrap {
            visual_y += self.buffer.cursor_x / width;
	}
	let screen_y = visual_y.saturating_sub(self.scroll_y);
	let screen_x = match self.wrap_mode {
            LineWrapMode::Truncate =>
		self.buffer.cursor_x.saturating_sub(self.scroll_x),
            LineWrapMode::Wrap =>
		self.buffer.cursor_x % width,
	};
	(screen_x, screen_y)
    }

    #[deprecated(note = "O(N) — do not use in rendering")]
    pub fn build_visual_lines(&self) -> Vec<VisualLine> {
	let width = self.viewport_width.max(1);
	let mut all = Vec::new();
	for (y, line) in self.buffer.lines.iter().enumerate() {
	    let len = line.char_len;
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

      pub fn scroll_up(&mut self) {
	self.scroll_intent = ScrollIntent::Manual;
	self.scroll_y = self.scroll_y.saturating_sub(1);
	self.clamp_scroll();
    }

    pub fn scroll_down(&mut self) {
	self.scroll_intent = ScrollIntent::Manual;
	self.scroll_y += 1;
	self.clamp_scroll();
    }

    pub fn scroll_down_command(&mut self) {
	self.scroll_intent = ScrollIntent::Manual;
        let max = self.buffer.lines.len().saturating_sub(1);
        self.scroll_y = (self.scroll_y + self.viewport_height)
            .min(max);

        if self.buffer.cursor_y < self.scroll_y {
            self.buffer.cursor_y = self.scroll_y;
        }
    }


    pub fn scroll_up_command(&mut self) {
	self.scroll_intent = ScrollIntent::Manual;
        if self.scroll_y >= self.viewport_height {
            self.scroll_y -= self.viewport_height;
	    self.buffer.cursor_y -= self.viewport_height;
        } else {
            self.scroll_y = 0;
        }

        if self.buffer.cursor_y < self.scroll_y {
            self.buffer.cursor_y = self.scroll_y;
        }
    }
    
    pub fn scroll_left(&mut self) {
	if self.wrap_mode == LineWrapMode::Truncate {
            self.scroll_intent = ScrollIntent::Manual;
            self.scroll_x = self.scroll_x.saturating_sub(4);
	}
    }
    
    pub fn scroll_right(&mut self) {
	if self.wrap_mode == LineWrapMode::Truncate {
            self.scroll_intent = ScrollIntent::Manual;

	    let max = self.buffer.lines
		.get(self.buffer.cursor_y)
		.map(|l| l.text.chars().count().saturating_sub(self.viewport_width.max(1)))
		.unwrap_or(0);

            self.scroll_x = (self.scroll_x + 4).min(max);
	}
    }
    
    pub fn set_wrap_mode(&mut self, mode: LineWrapMode) {
	if self.wrap_mode != mode {
            self.wrap_mode = mode;
            self.scroll_x = 0;
	}
    }
    
    // ---- Searching ----
    pub fn isearch_start(&mut self, dir: ISearchDir) {
	if let Some(state) = &mut self.isearch {
            state.dir = dir;
            self.isearch_next();
            return;
	}

	self.isearch = Some(ISearchState {
            original_x: self.buffer.cursor_x,
            original_y: self.buffer.cursor_y,
            query: String::new(),
            dir,
            last_match: None,
	});
	self.mode = InputMode::MiniBuffer;

	let prompt = match dir {
            ISearchDir::Forward => "I-search: ",
            ISearchDir::Backward => "I-search backward: ",
	};

	self.minibuffer.activate(prompt, MiniBufferMode::ISearchForward);
    }

    pub fn isearch_update(&mut self) {
	let Some(state) = &mut self.isearch else { return };
	let query = self.minibuffer
            .get()
            .splitn(2, ": ")
            .nth(1)
            .unwrap_or("")
            .to_string();
	state.query = query.clone();
	if query.is_empty() {
            self.buffer.cursor_x = state.original_x;
            self.buffer.cursor_y = state.original_y;
            return;
	}
	let start = state
            .last_match
            .unwrap_or((self.buffer.cursor_x, self.buffer.cursor_y));
	let found = match state.dir {
            ISearchDir::Forward =>
		self.buffer.search_forward_from(&query, start),
            ISearchDir::Backward =>
		self.buffer.search_backward(&query, start),
	};
	if let Some((x, y)) = found {
            self.buffer.cursor_x = x;
            self.buffer.cursor_y = y;
            state.last_match = Some((x, y));
            self.ensure_cursor_visible();
	}
    }

    pub fn isearch_next(&mut self) {
	let Some(state) = &mut self.isearch else { return };
	if state.query.is_empty() { return; }
	let from = state.last_match.unwrap_or((
            self.buffer.cursor_x,
            self.buffer.cursor_y,
	));
	let found = match state.dir {
            ISearchDir::Forward =>
		self.buffer.search_forward_from(&state.query, from),
            ISearchDir::Backward =>
		self.buffer.search_backward(&state.query, from),
	};
	if let Some((x, y)) = found {
            self.buffer.cursor_x = x;
            self.buffer.cursor_y = y;
            state.last_match = Some((x, y));
            self.ensure_cursor_visible();
	}
    }
    
    pub fn isearch_finish(&mut self) {
	self.isearch = None;
	self.minibuffer.deactivate();
	self.mode = InputMode::Normal;
    }

    pub fn isearch_abort(&mut self) {
	if let Some(state) = self.isearch.take() {
            self.buffer.cursor_x = state.original_x;
            self.buffer.cursor_y = state.original_y;
	}
	self.minibuffer.deactivate();
	self.mode = InputMode::Normal;
    }

    pub fn insert_newline(&mut self) {
	self.buffer.insert_newline_raw();
	self.scroll_intent = ScrollIntent::FollowCursor;
	self.ensure_cursor_visible();
    }
    
    // ---- Prefix / Command Execution ----
    pub fn run_command<F>(&mut self, lua: &Lua, name: &str, f: F)
    where F: FnOnce(&mut Self)
    {
	self.hooks.run(lua, "before-command", name);
	f(self);
	self.hooks.run(lua, "after-command", name);
	self.emit_cursor_moved(lua);
	self.scroll_intent = ScrollIntent::FollowCursor;
    }

    pub fn execute_named(&mut self, name: &str, lua: &Lua) {
	if let Some(cmd) = self.commands.get(name) {
            if name == "universal-argument" {
		self.run_command(lua, name, |ed| (cmd.as_ref().run)(CommandContext { editor: ed, arg: CommandArg::None }));
		self.process_events(lua);
		return;
            }
            if let Interactive::Str { prompt } = cmd.interactive {
		self.set_mode(lua, InputMode::MiniBuffer);
		self.pending_command = Some(name.to_string());
		let mode = match name {
                    "save-buffer-as" => MiniBufferMode::SaveBuffer,
                    "find-file" => MiniBufferMode::FindFile,
		    "goto-line" => MiniBufferMode::GotoLine,
		    "isearch-forward" => MiniBufferMode::ISearchForward,
                    _ => MiniBufferMode::Command,
		};
		self.minibuffer.activate(prompt, mode);
		return;
            }
            let arg = self.prefix.consume().map_or(CommandArg::None, CommandArg::Int);
            self.run_command(lua, name, |ed| (cmd.as_ref().run)(CommandContext { editor: ed, arg }));
            self.ensure_cursor_visible();
	} else {
            self.minibuffer.message(&format!("Unknown command: {name}"));
	}
    }
    
    pub fn push_kill(&mut self, text: String) {
	self.kill_buffer = Some(text);
    }
    
    pub fn execute_minibuffer(&mut self, lua: &Lua) {
	let mode = self.minibuffer.mode();
	let input = match mode {
	    MiniBufferMode::Command => self.minibuffer.get().strip_prefix("M-x ").unwrap_or("").trim(),
	    MiniBufferMode::FindFile => self.minibuffer.get().strip_prefix("Find file: ").unwrap_or("").trim(),
	    MiniBufferMode::SaveBuffer => self.minibuffer.get().strip_prefix("Save buffer as: ").unwrap_or("").trim(),
	    MiniBufferMode::GotoLine => self.minibuffer.get().strip_prefix("Goto line: ").unwrap_or("").trim(),
	    _ => "",
	}.to_string();
	self.minibuffer.deactivate();
	self.set_mode(lua, InputMode::Normal);
	match mode { 
	    MiniBufferMode::FindFile => {
		match self.buffer.open_file(input.clone().into()) {
		    Ok(_) => { self.minibuffer.message("Opened file"); self.emit_buffer_changed(lua, "open-file"); }
		    Err(e) => self.minibuffer.message(&format!("Open failed: {e}")),
		}
	    }
	    MiniBufferMode::SaveBuffer => {
		if input.is_empty() { self.minibuffer.message("Save failed: empty file name"); return; }
		match self.buffer.save_as(input.clone().into()) {
		    Ok(_) => {
			self.hooks.run(lua, "buffer-saved", &input);
			self.minibuffer.message("Saved buffer!");
		    }
		    Err(e) => self.minibuffer.message(&format!("Save failed: {e}")),
		}
	    }
	    MiniBufferMode::GotoLine => {
		if input.is_empty() { self.minibuffer.message("Line: 1"); return; }
		let n = input.parse::<usize>().unwrap_or(1);
		self.goto_line(n);
	    }
	    MiniBufferMode::ISearchForward => {
		self.isearch_finish();
	    }
	    MiniBufferMode::Command => self.execute_named(&input, lua),
	    _ => {}
	}
    }
    
}
