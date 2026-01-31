use std::io;
use std::path::{Path, PathBuf};
use mlua::Lua;
use crate::editor::hooks::HookRegistry;
use crate::editor::layout::LineWrapMode;

#[derive(Debug, Clone)]
pub enum UndoAction {
    Insert { x: usize, y: usize, text: String },
    Delete { x: usize, y: usize, text: String },
    InsertNewline { x: usize, y: usize },
    JoinLine { x: usize, y: usize },
}

#[derive(Clone)]
pub struct VisualMetrics {
    pub prefix_sum: Vec<usize>,

    pub dirty: bool,
    pub last_width: usize,
    pub last_wrap: LineWrapMode,
}

#[derive(Debug, Clone, Copy)]
pub enum Motion {
    Left, Right, Up, Down, Bol, Eol, BufferStart, BufferEnd, WordLeft, WordRight,
}

#[derive(Clone, Copy)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self { Self { x, y } }
}

#[derive(Clone)]
pub struct Line {
    pub text: String,
    pub char_len: usize,
    pub visual_height: usize,
    pub dirty: bool,
    pub last_width: usize,
    pub last_wrap: LineWrapMode,
}

impl Line {
    pub fn new(text: String) -> Self {
        let char_len = text.chars().count();
        Self {
						text,
						char_len,
						visual_height: 1,
						dirty: true,
						last_width: 0,
						last_wrap: LineWrapMode::Wrap
				}
    }

    pub fn empty() -> Self {
				Self {
            text: String::new(),
            char_len: 0,
            visual_height: 1,
            dirty: true,
            last_width: 0,
            last_wrap: LineWrapMode::Wrap,
				}
    }

    pub fn split_off(&mut self, x: usize) -> Line {
				let x = x.min(self.char_len);

				let byte = Buffer::char_to_byte_idx(&self.text, x);
				let rest = self.text.split_off(byte);

				let rest_len = self.char_len - x;
				self.char_len = x;

				Line {
            text: rest,
            char_len: rest_len,
						visual_height: 1,
						dirty: true,
						last_width: 0,
						last_wrap: LineWrapMode::Wrap,
				}
    }
}

#[derive(Clone, Copy)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
}

impl Selection {
    pub fn translate_y(self, offset: usize) -> Self {
        Self {
            start: Position { x: self.start.x, y: self.start.y.saturating_sub(offset) },
            end: Position { x: self.end.x, y: self.end.y.saturating_sub(offset) },
        }
    }
}

impl VisualMetrics {
    pub fn new() -> Self {
        Self {
            prefix_sum: Vec::new(),
            dirty: true,
            last_width: 0,
            last_wrap: LineWrapMode::Wrap,
        }
    }
}

#[derive(Clone)]
pub struct Buffer {
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub file_path: Option<PathBuf>,
    modified: bool,
    mark: Option<Position>,
    undo_stack: Vec<UndoAction>,
    pub visual: VisualMetrics,
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            file_path: None,
            modified: false,
            mark: None,
            undo_stack: Vec::new(),
						visual: VisualMetrics::new(),
						lines: vec![Line::empty()],
        }
    }

    fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
				if char_idx == 0 {
            return 0;
				}
				s.char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or_else(|| s.len())
    }

    fn push_undo(&mut self, action: UndoAction) {
        self.undo_stack.push(action);
        self.modified = true;
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                UndoAction::Insert { x, y, text } => {
                    let line = &mut self.lines[y];
                    let start = Self::char_to_byte_idx(&line.text, x);
                    let end = Self::char_to_byte_idx(&line.text, x + text.chars().count());
                    line.text.replace_range(start..end, "");
										let removed = text.chars().count();
										line.char_len -= removed;
                    self.cursor_x = x;
                    self.cursor_y = y;
										self.visual.dirty = true;
                }
                UndoAction::Delete { x, y, text } => {
                    let line = &mut self.lines[y];
                    let byte = Self::char_to_byte_idx(&line.text, x);
                    line.text.insert_str(byte, &text);
										let removed = text.chars().count();
										line.char_len -= removed;
                    self.cursor_x = x + text.chars().count();
                    self.cursor_y = y;
										self.visual.dirty = true;
                }
                UndoAction::InsertNewline { x, y } => {
                    let next = self.lines.remove(y + 1);
                    self.lines[y].text.push_str(&next.text);
                    self.cursor_x = x;
                    self.cursor_y = y;
										self.visual.dirty = true;
                }
                UndoAction::JoinLine { x, y } => {
                    let tail = self.lines[y].text.split_off(x);
                    self.lines.insert(y + 1, Line::new(tail));
                    self.cursor_x = 0;
                    self.cursor_y = y + 1;
										self.visual.dirty = true;
                }
            }
        }
    }

    pub fn ensure_visuals( &mut self, width: usize, wrap: LineWrapMode) {
				if self.visual.dirty
            || self.visual.last_width != width
            || self.visual.last_wrap != wrap
				{
            self.rebuild_visual_metrics(width, wrap);
            self.visual.dirty = false;
            self.visual.last_width = width;
            self.visual.last_wrap = wrap;
				}
    } 

    pub fn insert_text_at(&mut self, x: usize, y: usize, text: &str) {
        let line = &mut self.lines[y];
        let byte = Self::char_to_byte_idx(&line.text, x);
				let added = text.chars().count();
				line.text.insert_str(byte, text);
				line.char_len += added;
        self.push_undo(UndoAction::Insert { x, y, text: text.to_string() });
				self.cursor_x = x + text.chars().count();
        self.cursor_y = y;
    }
    
    pub fn insert_char_raw(&mut self, ch: char) {
				let line = &mut self.lines[self.cursor_y];
				let byte_idx = Self::char_to_byte_idx(&line.text, self.cursor_x);
				self.cursor_x += 1;
				line.text.insert(byte_idx, ch);
				line.char_len += 1;
				self.visual.dirty = true;
    }

    pub fn insert_char(
        &mut self,
        ch: char,
        lua: Option<&Lua>,
        hooks: Option<&HookRegistry>,
    ) {
        if let (Some(lua), Some(hooks)) = (lua, hooks) {
            hooks.run(lua, "before-insert", &ch.to_string());
        }

        self.insert_char_raw(ch);

        if let (Some(lua), Some(hooks)) = (lua, hooks) {
            hooks.run(lua, "after-insert", &ch.to_string());
        }
    }

    pub fn insert_newline_raw(&mut self) {
				let x = self.cursor_x;
				let y = self.cursor_y;
				let rest = self.lines[y].split_off(x);
				self.lines.insert(y + 1, rest);
				self.cursor_y += 1;
				self.cursor_x = 0;
				self.visual.dirty = true;
				self.push_undo(UndoAction::InsertNewline { x, y });
    }

    
    pub fn delete_range(&mut self, start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> String {
        if start_y == end_y {
            let line = &mut self.lines[start_y];
            let a = Self::char_to_byte_idx(&line.text, start_x);
            let b = Self::char_to_byte_idx(&line.text, end_x);
            let deleted = line.text[a..b].to_string();
            line.text.replace_range(a..b, "");
						line.char_len -= end_x - start_x;
            self.push_undo(UndoAction::Delete { x: start_x, y: start_y, text: deleted.clone() });
            self.cursor_x = start_x;
            self.cursor_y = start_y;
            deleted
        } else {
            let mut deleted = String::new();
            let first_line = &mut self.lines[start_y];
            let a = Self::char_to_byte_idx(&first_line.text, start_x);
            deleted.push_str(&first_line.text[a..]);
						first_line.char_len = start_x;
            for _ in start_y + 1..end_y {
                deleted.push('\n');
								let removed = self.lines.remove(start_y + 1);
								deleted.push_str(&removed.text);
            }
            let last_line = &mut self.lines[start_y + 1];
            let b = Self::char_to_byte_idx(&last_line.text, end_x);
            deleted.push('\n');
            deleted.push_str(&last_line.text[..b]);

            let rest = last_line.text[b..].to_string();
						self.lines[start_y].char_len += rest.chars().count();
            self.lines.remove(start_y + 1);

            self.push_undo(UndoAction::Delete { x: start_x, y: start_y, text: deleted.clone() });
            self.cursor_x = start_x;
            self.cursor_y = start_y;
            deleted
        }
    }

    pub fn delete(&mut self, motion: Motion) -> Option<String> {
        match motion {
            Motion::Left => self.delete_backward_char(),
            Motion::Right => self.delete_forward_char(),
            _ => None,
        }
    }

    fn delete_forward_char(&mut self) -> Option<String> {
        let y = self.cursor_y;
        let x = self.cursor_x;

        let line = self.lines.get(y)?;
				if x >= line.char_len {
						return None;
				}
				
				self.visual.dirty = true;
        Some(self.delete_range(x, y, x + 1, y))
    }

    fn delete_backward_char(&mut self) -> Option<String> {
				let y = self.cursor_y;
				let x = self.cursor_x;
				if x == 0 && y == 0 {
            return None;
				}
				if x > 0 {
            return Some(self.delete_range(x - 1, y, x, y));
				}
				let prev_y = y - 1;
				let prev_len = self.lines[prev_y].char_len;
				self.visual.dirty = true;
				
				Some(self.delete_range(prev_len, prev_y, 0, y))
    }
    
    pub fn rebuild_visual_metrics(&mut self, width: usize, wrap: LineWrapMode) {
				let w = width.max(1);

				self.visual.prefix_sum.clear();
				self.visual.prefix_sum.reserve(self.lines.len());

				let mut acc = 0;
				for line in &mut self.lines {
            let vh = match wrap {
								LineWrapMode::Truncate => 1,
								LineWrapMode::Wrap => {
                    let len = line.char_len.max(1);
                    (len + w - 1) / w
								}
            };

            line.visual_height = vh;

            self.visual.prefix_sum.push(acc);
            acc += vh;
				}
    }
    
    pub fn move_cursor(&mut self, motion: Motion) {
        match motion {
            Motion::Left => { if self.cursor_x > 0 { self.cursor_x -= 1 } },
            Motion::Right => {
								if let Some(line) = self.lines.get(self.cursor_y) {
                    if self.cursor_x < line.char_len { self.cursor_x += 1 }
								}
						},
            Motion::Up => {
								if self.cursor_y > 0 {
										self.cursor_y -= 1;
										self.clamp_cursor_x()
								}
						},
            Motion::Down => {
								if self.cursor_y + 1 < self.lines.len() {
										self.cursor_y += 1;
										self.clamp_cursor_x()
								}
						},
            Motion::Bol => self.cursor_x = 0,
            Motion::Eol => self.cursor_x = self.lines.get(self.cursor_y).map(|l| l.char_len).unwrap_or(0),
            Motion::BufferStart => {
								self.cursor_x = 0;
								self.cursor_y = 0
						},
            Motion::BufferEnd => {
								self.cursor_y = self.lines.len().saturating_sub(1);
                self.cursor_x = self.lines.get(self.cursor_y).map(|l| l.char_len).unwrap_or(0)
						},
            Motion::WordLeft => self.move_word_left(),
            Motion::WordRight => self.move_word_right(),
        }
    }

    fn clamp_cursor_x(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_y) {
            self.cursor_x = self.cursor_x.min(line.char_len);
        } else {
            self.cursor_x = 0;
        }
    }

    fn kill_in_line<F>(&mut self, calc: F) -> Option<String>
    where
        F: Fn(&[char], usize) -> Option<(usize, usize)>
    {
        let y = self.cursor_y;
        let x = self.cursor_x;
        let line = self.lines.get_mut(y)?;
        let chars: Vec<char> = line.text.chars().collect();
        let (start, end) = calc(&chars, x)?;
        if start == end { return None; }

        let killed: String = chars[start..end].iter().collect();

        let a = Self::char_to_byte_idx(&line.text, start);
        let b = Self::char_to_byte_idx(&line.text, end);
        line.text.replace_range(a..b, "");
				line.char_len -= end - start;
        self.cursor_x = start;

        self.push_undo(UndoAction::Delete {
            x: start,
            y,
            text: killed.clone(),
        });

        Some(killed)
    }

		pub fn search_forward(
				&self,
				needle: &str,
				from: (usize, usize),
		) -> Option<(usize, usize)> {
				let (start_x, start_y) = from;

				for y in start_y..self.lines.len() {
						let line = &self.lines[y];
						let start = if y == start_y { start_x } else { 0 };

						let hay = &line.text[Self::char_to_byte_idx(&line.text, start)..];
						if let Some(pos) = hay.find(needle) {
								let x = start + hay[..pos].chars().count();
								return Some((x, y));
						}
				}
				None
		}
		
    pub fn search_backward(&self, query: &str, from: (usize, usize)) -> Option<(usize, usize)> {
				if query.is_empty() {
            return None;
				}
				let (mut x, mut y) = from;
				while y > 0 || x > 0 {
            if x == 0 {
								y -= 1;
								x = self.lines[y].char_len;
            }
            x -= 1;
            let line = &self.lines[y].text;
            if x + query.len() <= line.len() && &line[x..x + query.len()] == query {
								return Some((x, y));
            }
				}
				None
    }

		fn clone_for_search(&self, from: (usize, usize)) -> Buffer {
        let mut b = self.clone();
        b.cursor_x = from.0;
        b.cursor_y = from.1;
        b
    }
		
		 pub fn search_forward_from(
        &self,
        needle: &str,
        from: (usize, usize),
    ) -> Option<(usize, usize)> {
        // let (old_x, old_y) = (self.cursor_x, self.cursor_y); 

        // временно считаем, что поиск стартует отсюда
        let tmp = self.clone_for_search(from);
        let res = tmp.search_forward(needle, from);

        res
    }

    pub fn search_backward_from(
        &self,
        needle: &str,
        from: (usize, usize),
    ) -> Option<(usize, usize)> {
        let tmp = self.clone_for_search(from);
        tmp.search_backward(needle, from)
    }

    pub fn kill_word(&mut self) -> Option<String> {
				self.kill_in_line(|chars, x| {
            if x >= chars.len() { return None; }
            let mut end = x;
            while end < chars.len() && chars[end].is_whitespace() { end += 1; }
            while end < chars.len() && !chars[end].is_whitespace() { end += 1; }
            Some((x, end))
				})
    }


    pub fn kill_backward_word(&mut self) -> Option<String> {
				self.kill_in_line(|chars, x| {
            if x == 0 { return None; }
            let mut start = x;
            while start > 0 && chars[start - 1].is_whitespace() { start -= 1; }
            while start > 0 && !chars[start - 1].is_whitespace() { start -= 1; }
            Some((start, x))
				})
    }

    pub fn kill_sentence(&mut self) -> Option<String> {
				self.kill_in_line(|chars, x| {
            if x >= chars.len() { return None; }
            let mut end = x;
            while end < chars.len() {
								if ".!?".contains(chars[end])
                    && (end + 1 == chars.len() || chars[end + 1].is_whitespace())
								{
                    end += 1;
                    if end < chars.len() && chars[end].is_whitespace() {
												end += 1;
                    }
                    break;
								}
								end += 1;
            }
            Some((x, end))
				})
    }

    pub fn kill_line(&mut self) -> Option<String> {
				self.kill_in_line(|chars, x| {
            Some((x, chars.len()))
				})
    }

    
    fn move_word_left(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_y) {
            let mut idx = self.cursor_x;
            let chars: Vec<char> = line.text.chars().collect();
            if idx == 0 { return }
            while idx > 0 && chars[idx - 1].is_whitespace() { idx -= 1 }
            while idx > 0 && !chars[idx - 1].is_whitespace() { idx -= 1 }
            self.cursor_x = idx;
        }
    }

    fn move_word_right(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_y) {
            let mut idx = self.cursor_x;
            let chars: Vec<char> = line.text.chars().collect();
            let len = chars.len();
            if idx >= len { return }
            while idx < len && chars[idx].is_whitespace() { idx += 1 }
            while idx < len && !chars[idx].is_whitespace() { idx += 1 }
            self.cursor_x = idx;
        }
    }

    pub fn set_mark(&mut self) { self.mark = Some(Position { x: self.cursor_x, y: self.cursor_y }) }
    pub fn clear_mark(&mut self) { self.mark = None }
    pub fn toggle_mark(&mut self) { if self.mark.is_some() { self.clear_mark() } else { self.set_mark() } }

    pub fn selection(&self) -> Option<Selection> {
        let mark = self.mark?;
        let cursor = Position { x: self.cursor_x, y: self.cursor_y };
        let (start, end) = if (cursor.y, cursor.x) < (mark.y, mark.x) { (cursor, mark) } else { (mark, cursor) };
        Some(Selection { start, end })
    }

    pub fn kill_region(&mut self) -> Option<String> {
        let sel = self.selection()?;
        Some(self.delete_range(sel.start.x, sel.start.y, sel.end.x, sel.end.y))
    }

    pub fn copy_region(&self) -> Option<String> {
        let sel = self.selection()?;
        Some(self.get_range(sel.start.x, sel.start.y, sel.end.x, sel.end.y))
    }

    fn get_range(&self, start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> String {
        if start_y == end_y {
            let line = &self.lines[start_y];
            let a = Self::char_to_byte_idx(&line.text, start_x);
            let b = Self::char_to_byte_idx(&line.text, end_x);
            line.text[a..b].to_string()
        } else {
            let mut s = String::new();
            let first = &self.lines[start_y].text;
            s.push_str(&first[Self::char_to_byte_idx(first, start_x)..]);
            for y in start_y + 1..end_y {
                s.push('\n');
                s.push_str(&self.lines[y].text);
            }
            let last = &self.lines[end_y];
            s.push('\n');
            s.push_str(&last.text[..Self::char_to_byte_idx(&last.text, end_x)]);
            s
        }
    }

    pub fn yank(&mut self, text: &str) {
        let y = self.cursor_y;
        let x = self.cursor_x;
        for (i, line) in text.split('\n').enumerate() {
            if i != 0 { self.insert_newline_raw() }
            for ch in line.chars() { self.insert_char_raw(ch) }
        }
        self.push_undo(UndoAction::Insert { x, y, text: text.to_string() });
				self.visual.dirty = true;
    }


    pub fn expand_tilde<P: AsRef<Path>>(&mut self, path: P) -> PathBuf {
				let path = path.as_ref();

				let s = match path.to_str() {
            Some(s) => s,
            None => return path.to_path_buf(),
				};

				if s == "~" || s.starts_with("~/") {
            if let Some(home) = std::env::var_os("HOME") {
								if s.len() == 1 {
                    return PathBuf::from(home);
								} else {
                    return PathBuf::from(home).join(&s[2..]);
								}
            }
				}

				path.to_path_buf()
    }
    pub fn open_file(&mut self, path: PathBuf) -> io::Result<()> {
				let path = self.expand_tilde(path);
        if !path.exists() {
						self.lines = vec![Line::new(String::new())];
						self.file_path = Some(path);
						self.cursor_x = 0;
						self.cursor_y = 0;
						return Ok(())
				}
        let content = std::fs::read_to_string(&path)?;
        self.lines = content.lines().map(|s| Line::new(s.to_string())).collect();
        self.file_path = Some(path);
        self.cursor_x = 0; self.cursor_y = 0;
        Ok(())
    }

    pub fn save(&mut self) -> io::Result<()> {
        let path = self.file_path.clone().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "empty file name"))?;
        self.save_as(path)
    }

    pub fn save_as(&mut self, mut path: PathBuf) -> io::Result<()> {
        if path.is_relative() { path = std::env::current_dir()?.join(path) }
        if let Some(parent) = path.parent() { if !parent.as_os_str().is_empty() { std::fs::create_dir_all(parent)? } }
				let mut out = String::new();

				for (i, line) in self.lines.iter().enumerate() {
						if i > 0 {
								out.push('\n');
						}
						out.push_str(&line.text);
				}
				std::fs::write(&path, out)?;
        self.file_path = Some(path);
        Ok(())
    }

    pub fn is_modified(&self) -> bool { self.modified }
    pub fn file_name(&self) -> String {
        self.file_path.as_ref().and_then(|p| p.file_name().and_then(|s| s.to_str())).unwrap_or("[No Name]").to_string()
    }
    
    pub fn undo_depth(&self) -> usize { self.undo_stack.len() }
}
