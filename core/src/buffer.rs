use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum UndoAction {
    Insert { x: usize, y: usize, text: String },
    Delete { x: usize, y: usize, text: String },
    InsertNewline { x: usize, y: usize },
    JoinLine { x: usize, y: usize },
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

pub struct Buffer {
    pub lines: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub file_path: Option<PathBuf>,
    modified: bool,
    mark: Option<Position>,
    undo_stack: Vec<UndoAction>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            file_path: None,
            modified: false,
            mark: None,
            undo_stack: Vec::new(),
        }
    }

    fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
        s.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or(s.len())
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
                    let start = Self::char_to_byte_idx(line, x);
                    let end = Self::char_to_byte_idx(line, x + text.chars().count());
                    line.replace_range(start..end, "");
                    self.cursor_x = x;
                    self.cursor_y = y;
                }
                UndoAction::Delete { x, y, text } => {
                    let line = &mut self.lines[y];
                    let byte = Self::char_to_byte_idx(line, x);
                    line.insert_str(byte, &text);
                    self.cursor_x = x + text.chars().count();
                    self.cursor_y = y;
                }
                UndoAction::InsertNewline { x, y } => {
                    let next = self.lines.remove(y + 1);
                    self.lines[y].push_str(&next);
                    self.cursor_x = x;
                    self.cursor_y = y;
                }
                UndoAction::JoinLine { x, y } => {
                    let tail = self.lines[y].split_off(x);
                    self.lines.insert(y + 1, tail);
                    self.cursor_x = 0;
                    self.cursor_y = y + 1;
                }
            }
        }
    }

    pub fn insert_text_at(&mut self, x: usize, y: usize, text: &str) {
        let line = &mut self.lines[y];
        let byte = Self::char_to_byte_idx(line, x);
        line.insert_str(byte, text);
        self.push_undo(UndoAction::Insert { x, y, text: text.to_string() });
        self.cursor_x = x + text.chars().count();
        self.cursor_y = y;
    }

    pub fn insert_char(&mut self, ch: char) {
        self.insert_text_at(self.cursor_x, self.cursor_y, &ch.to_string());
    }

    pub fn insert_newline(&mut self) {
        let line = &mut self.lines[self.cursor_y];
        let rest = line.split_off(self.cursor_x);
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.lines.insert(self.cursor_y, rest);
        self.push_undo(UndoAction::InsertNewline { x: self.cursor_x, y: self.cursor_y - 1 });
    }
    
    pub fn delete_range(&mut self, start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> String {
        if start_y == end_y {
            let line = &mut self.lines[start_y];
            let a = Self::char_to_byte_idx(line, start_x);
            let b = Self::char_to_byte_idx(line, end_x);
            let deleted = line[a..b].to_string();
            line.replace_range(a..b, "");
            self.push_undo(UndoAction::Delete { x: start_x, y: start_y, text: deleted.clone() });
            self.cursor_x = start_x;
            self.cursor_y = start_y;
            deleted
        } else {
            let mut deleted = String::new();
            let first_line = &mut self.lines[start_y];
            let a = Self::char_to_byte_idx(first_line, start_x);
            deleted.push_str(&first_line[a..]);
            first_line.truncate(a);

            for _ in start_y + 1..end_y {
                deleted.push('\n');
                deleted.push_str(&self.lines.remove(start_y + 1));
            }

            let last_line = &mut self.lines[start_y + 1];
            let b = Self::char_to_byte_idx(last_line, end_x);
            deleted.push('\n');
            deleted.push_str(&last_line[..b]);

            let rest = last_line[b..].to_string();
            self.lines[start_y].push_str(&rest);
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
        if x >= line.chars().count() {
            return None;
        }

        Some(self.delete_range(x, y, x + 1, y))
    }

    fn delete_backward_char(&mut self) -> Option<String> {
        if self.cursor_x == 0 {
            return None;
        }

        let y = self.cursor_y;
        let x = self.cursor_x;

        Some(self.delete_range(x - 1, y, x, y))
    }

    pub fn move_cursor(&mut self, motion: Motion) {
        match motion {
            Motion::Left => { if self.cursor_x > 0 { self.cursor_x -= 1 } },
            Motion::Right => { if let Some(line) = self.lines.get(self.cursor_y) {
                if self.cursor_x < line.chars().count() { self.cursor_x += 1 } } },
            Motion::Up => { if self.cursor_y > 0 { self.cursor_y -= 1; self.clamp_cursor_x() } },
            Motion::Down => { if self.cursor_y + 1 < self.lines.len() { self.cursor_y += 1; self.clamp_cursor_x() } },
            Motion::Bol => self.cursor_x = 0,
            Motion::Eol => self.cursor_x = self.lines.get(self.cursor_y).map(|l| l.chars().count()).unwrap_or(0),
            Motion::BufferStart => { self.cursor_x = 0; self.cursor_y = 0 },
            Motion::BufferEnd => { self.cursor_y = self.lines.len().saturating_sub(1);
                                   self.cursor_x = self.lines.get(self.cursor_y).map(|l| l.chars().count()).unwrap_or(0) },
            Motion::WordLeft => self.move_word_left(),
            Motion::WordRight => self.move_word_right(),
        }
    }

    fn clamp_cursor_x(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_y) {
            self.cursor_x = self.cursor_x.min(line.chars().count());
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
        let chars: Vec<char> = line.chars().collect();

        let (start, end) = calc(&chars, x)?;
        if start == end { return None; }

        let killed: String = chars[start..end].iter().collect();

        let a = Self::char_to_byte_idx(line, start);
        let b = Self::char_to_byte_idx(line, end);
        line.replace_range(a..b, "");

        self.cursor_x = start;

        self.push_undo(UndoAction::Delete {
            x: start,
            y,
            text: killed.clone(),
        });

        Some(killed)
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
            let chars: Vec<char> = line.chars().collect();
            if idx == 0 { return }
            while idx > 0 && chars[idx - 1].is_whitespace() { idx -= 1 }
            while idx > 0 && !chars[idx - 1].is_whitespace() { idx -= 1 }
            self.cursor_x = idx;
        }
    }

    fn move_word_right(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_y) {
            let mut idx = self.cursor_x;
            let chars: Vec<char> = line.chars().collect();
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
            let a = Self::char_to_byte_idx(line, start_x);
            let b = Self::char_to_byte_idx(line, end_x);
            line[a..b].to_string()
        } else {
            let mut s = String::new();
            let first = &self.lines[start_y];
            s.push_str(&first[Self::char_to_byte_idx(first, start_x)..]);
            for y in start_y + 1..end_y {
                s.push('\n');
                s.push_str(&self.lines[y]);
            }
            let last = &self.lines[end_y];
            s.push('\n');
            s.push_str(&last[..Self::char_to_byte_idx(last, end_x)]);
            s
        }
    }

    pub fn yank(&mut self, text: &str) {
        let y = self.cursor_y;
        let x = self.cursor_x;
        for (i, line) in text.split('\n').enumerate() {
            if i != 0 { self.insert_newline() }
            for ch in line.chars() { self.insert_char(ch) }
        }
        self.push_undo(UndoAction::Insert { x, y, text: text.to_string() });
    }

    pub fn open_file(&mut self, path: PathBuf) -> io::Result<()> {
        if !path.exists() { self.lines = vec![String::new()]; self.file_path = Some(path); self.cursor_x = 0; self.cursor_y = 0; return Ok(()) }
        let content = std::fs::read_to_string(&path)?;
        self.lines = content.lines().map(|s| s.to_string()).collect();
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
        std::fs::write(&path, self.lines.join("\n"))?;
        self.file_path = Some(path);
        Ok(())
    }

    pub fn is_modified(&self) -> bool { self.modified }
    pub fn file_name(&self) -> String {
        self.file_path.as_ref().and_then(|p| p.file_name().and_then(|s| s.to_str())).unwrap_or("[No Name]").to_string()
    }
    pub fn undo_depth(&self) -> usize { self.undo_stack.len() }
}
