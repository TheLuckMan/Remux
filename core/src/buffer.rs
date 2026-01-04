use std::io::{self};
use std::path::PathBuf;

// Я запрещаю подключать editor.rs!
// Это сломает архектитуру Remux:
// Minibuffer ↘
// Buffer -> Editor -> UI

// А лучше позже переделать что-типо в роде
// Minibuffer ↘
// Buffer -> Editor -> Main -> UI

// Буфер (buffer.rs) - это хранение текста, координаты курсора, выделение.
// Он не должен знать про размеры экрана, скроллинг, и чё там у мини-буфера

#[derive(Debug, Clone, Copy)]
pub enum UndoAction {
    InsertChar {
        x: usize,
        y: usize,
        ch: char,
    },
    DeleteChar {
        x: usize,
        y: usize,
        ch: char,
    },
     BackwardDeleteChar {
        x: usize,
        y: usize,
        ch: char,
    },
    InsertNewline {
        x: usize,
        y: usize,
    },
    JoinLine {
        x: usize,
        y: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    Bol,          // Beginning of line
    Eol,          // End of line
    BufferStart,
    BufferEnd,
    WordLeft,
    WordRight,
}


#[derive(Clone, Copy)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
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
            start: Position {
                x: self.start.x,
                y: self.start.y.saturating_sub(offset),
            },
            end: Position {
                x: self.end.x,
                y: self.end.y.saturating_sub(offset),
            },
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
	s.char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(s.len())
    }

    pub fn undo(&mut self) {
	if let Some(action) = self.undo_stack.pop() {
            match action {
		UndoAction::InsertChar { x, y, .. } => {
                    let line = &mut self.lines[y];
		    let start = Self::char_to_byte_idx(line, x);
		    let end   = Self::char_to_byte_idx(line, x + 1);
		    line.replace_range(start..end, "");
                    self.cursor_x = x;
                    self.cursor_y = y;
		}
		UndoAction::BackwardDeleteChar { x, y, ch } => {
                    let line = &mut self.lines[y];
		    let byte = Self::char_to_byte_idx(line, x);
		    line.insert_str(byte, ch.encode_utf8(&mut [0; 4]));
                    self.cursor_x = x + 1;
                    self.cursor_y = y;
		}
		UndoAction::DeleteChar { x, y, ch } => {
                    let line = &mut self.lines[y];
		    let byte = Self::char_to_byte_idx(line, x);
		    line.insert_str(byte, ch.encode_utf8(&mut [0; 4]));
                    self.cursor_x = x;
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


    pub fn insert_char(&mut self, ch: char) {
	let y = self.cursor_y;
	let x = self.cursor_x;

	let line = &mut self.lines[y];
	let byte = Self::char_to_byte_idx(line, x);
	line.insert_str(byte, ch.encode_utf8(&mut [0; 4]));
	self.cursor_x += 1;

	self.undo_stack.push(UndoAction::InsertChar { x, y, ch });
    }
    
    pub fn move_cursor(&mut self, motion: Motion) {
        match motion {
            Motion::Left => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            Motion::Right => {
                if let Some(line) = self.lines.get(self.cursor_y) {
                    if self.cursor_x < line.chars().count() {
                        self.cursor_x += 1;
                    }
                }
            }
            Motion::Up => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                    self.clamp_cursor_x();
                }
            }
            Motion::Down => {
                if self.cursor_y + 1 < self.lines.len() {
                    self.cursor_y += 1;
                    self.clamp_cursor_x();
                }
            }
            Motion::Bol => self.cursor_x = 0,
            Motion::Eol => {
                if let Some(line) = self.lines.get(self.cursor_y) {
                    self.cursor_x = line.chars().count();
                }
            }
            Motion::BufferStart => {
                self.cursor_y = 0;
                self.cursor_x = 0;
            }
            Motion::BufferEnd => {
                if !self.lines.is_empty() {
                    self.cursor_y = self.lines.len() - 1;
                    self.cursor_x = self.lines[self.cursor_y].chars().count();
                } else {
                    self.cursor_y = 0;
                    self.cursor_x = 0;
                }
            }
	    Motion::WordLeft => {
                if let Some(line) = self.lines.get(self.cursor_y) {
                    let mut idx = self.cursor_x;
                    let chars: Vec<char> = line.chars().collect();
                    if idx == 0 { return; }

                    // Сначала пропускаем пробельные символы слева
                    while idx > 0 && chars[idx - 1].is_whitespace() {
                        idx -= 1;
                    }
                    // Потом идём до начала слова
                    while idx > 0 && !chars[idx - 1].is_whitespace() {
                        idx -= 1;
                    }
                    self.cursor_x = idx;
                }
            }
            Motion::WordRight => {
                if let Some(line) = self.lines.get(self.cursor_y) {
                    let mut idx = self.cursor_x;
                    let chars: Vec<char> = line.chars().collect();
                    let len = chars.len();
                    if idx >= len { return; }

                    // Сначала пропускаем пробельные символы справа
                    while idx < len && chars[idx].is_whitespace() {
                        idx += 1;
                    }
                    // Потом идём до конца слова
                    while idx < len && !chars[idx].is_whitespace() {
                        idx += 1;
                    }
                    self.cursor_x = idx;
                }
            }
        }
    }
    
    fn clamp_cursor_x(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_y) {
            let max = line.chars().count();
	    if self.cursor_x > max {
		self.cursor_x = max;
	    }
        }
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn file_name(&self) -> String {
        match &self.file_path {
            Some(path) => {
                path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("[invalid utf-8]")
                    .to_string()
            }
            None => "[No Name]".to_string(),
        }
    }

    pub fn undo_depth(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn delete_char(&mut self) {
	let x = self.cursor_x;
	let y = self.cursor_y;
	let line = &mut self.lines[y];

	let char_count = line.chars().count();
	if self.cursor_x >= char_count { return; }
	let start = Self::char_to_byte_idx(line, x);
	let end   = Self::char_to_byte_idx(line, x + 1);
	let ch = line[start..end].chars().next().unwrap();
	line.replace_range(start..end, "");

	self.undo_stack.push(UndoAction::DeleteChar { x, y, ch });
    }
    
    pub fn backward_delete_char(&mut self) {
	if self.cursor_x == 0 { return; }

	let y = self.cursor_y;
	let x = self.cursor_x - 1;
	let line = &mut self.lines[y];

	let start = Self::char_to_byte_idx(line, x);
	let end   = Self::char_to_byte_idx(line, x + 1);

	let ch = line[start..end].chars().next().unwrap();
	line.replace_range(start..end, "");

	self.cursor_x -= 1;

	self.undo_stack.push(UndoAction::BackwardDeleteChar { x, y, ch });
    }

    
    pub fn insert_newline(&mut self) {
	let line = &mut self.lines[self.cursor_y];
	let rest = line.split_off(self.cursor_x);

	self.cursor_y += 1;
	self.cursor_x = 0;
	self.lines.insert(self.cursor_y, rest);
    }

    
    pub fn selection(&self) -> Option<Selection> {
	let mark = self.mark?;
	let cursor = Position {
            x: self.cursor_x,
            y: self.cursor_y,
	};

	let (start, end) = if (cursor.y, cursor.x) < (mark.y, mark.x) {
            (cursor, mark)
	} else {
            (mark, cursor)
	};

	Some(Selection { start, end })
    }
    
    pub fn set_mark(&mut self) {
        self.mark = Some(Position {
            x: self.cursor_x,
            y: self.cursor_y,
        });
    }

    pub fn clear_mark(&mut self) {
        self.mark = None;
    }

    pub fn toggle_mark(&mut self) {
        if self.mark.is_some() {
            self.mark = None;
        } else {
            self.set_mark();
        }
    }

     pub fn kill_word(&mut self) -> Option<String> {
        if let Some(line) = self.lines.get_mut(self.cursor_y) {
            let mut chars: Vec<char> = line.chars().collect();
            let start = self.cursor_x;
            if start >= chars.len() {
                return None;
            }

            let mut end = start;

            // сначала пропускаем пробелы
            while end < chars.len() && chars[end].is_whitespace() {
                end += 1;
            }
            // потом идём до конца слова
            while end < chars.len() && !chars[end].is_whitespace() {
                end += 1;
            }

            let killed: String = chars[start..end].iter().collect();

            chars.drain(start..end);
            *line = chars.into_iter().collect();

            Some(killed)
        } else {
            None
        }
    }

    pub fn kill_backward_word(&mut self) -> Option<String> {
        if let Some(line) = self.lines.get_mut(self.cursor_y) {
            let mut chars: Vec<char> = line.chars().collect();
            let mut start = self.cursor_x;
            if start == 0 {
                return None;
            }

            // ищем начало слова назад
            while start > 0 && chars[start - 1].is_whitespace() {
                start -= 1;
            }
            while start > 0 && !chars[start - 1].is_whitespace() {
                start -= 1;
            }

            let killed: String = chars[start..self.cursor_x].iter().collect();

            chars.drain(start..self.cursor_x);
            *line = chars.into_iter().collect();
            self.cursor_x = start;

            Some(killed)
        } else {
            None
        }
    }

    pub fn kill_sentence(&mut self) -> Option<String> {
        if let Some(line) = self.lines.get_mut(self.cursor_y) {
            let mut chars: Vec<char> = line.chars().collect();
            let start = self.cursor_x;
            if start >= chars.len() {
                return None;
            }

            let mut end = start;
            while end < chars.len() {
                if ".!?".contains(chars[end]) && (end + 1 == chars.len() || chars[end + 1].is_whitespace()) {
                    end += 1; // включаем знак препинания
                    if end < chars.len() && chars[end].is_whitespace() {
                        end += 1; // включаем пробел после
                    }
                    break;
                }
                end += 1;
            }

            let killed: String = chars[start..end].iter().collect();

            chars.drain(start..end);
            *line = chars.into_iter().collect();

            Some(killed)
        } else {
            None
        }
    }

    pub fn kill_region(&mut self) -> Option<String> {
        let sel = self.selection()?;
        let mut killed = String::new();

        if sel.start.y == sel.end.y {
            // одна строка
            let line = &mut self.lines[sel.start.y];

            let a = Self::char_to_byte_idx(line, sel.start.x);
            let b = Self::char_to_byte_idx(line, sel.end.x);

            killed.push_str(&line[a..b]);
            line.replace_range(a..b, "");
        } else {
            // первая строка
            let first = &mut self.lines[sel.start.y];
            let a = Self::char_to_byte_idx(first, sel.start.x);
            killed.push_str(&first[a..]);
            first.truncate(a);

            // средние строки
            for _ in sel.start.y + 1 .. sel.end.y {
                killed.push('\n');
                killed.push_str(&self.lines.remove(sel.start.y + 1));
            }

            // последняя строка
            let last = &mut self.lines[sel.start.y + 1];
            let b = Self::char_to_byte_idx(last, sel.end.x);
            killed.push('\n');
            killed.push_str(&last[..b]);

            let rest = last[b..].to_string();
            self.lines[sel.start.y].push_str(&rest);
            self.lines.remove(sel.start.y + 1);
        }

        self.cursor_x = sel.start.x;
        self.cursor_y = sel.start.y;
        self.clear_mark();

        Some(killed)
    }

    pub fn copy_region(&self) -> Option<String> {
        let sel = self.selection()?;
        let mut copied = String::new();

        if sel.start.y == sel.end.y {
            let line = &self.lines[sel.start.y];
            let a = Self::char_to_byte_idx(line, sel.start.x);
            let b = Self::char_to_byte_idx(line, sel.end.x);
            copied.push_str(&line[a..b]);
        } else {
            // первая строка
            let first = &self.lines[sel.start.y];
            let a = Self::char_to_byte_idx(first, sel.start.x);
            copied.push_str(&first[a..]);

            // средние строки
            for y in sel.start.y + 1 .. sel.end.y {
                copied.push('\n');
                copied.push_str(&self.lines[y]);
            }

            // последняя строка
            let last = &self.lines[sel.end.y];
            let b = Self::char_to_byte_idx(last, sel.end.x);
            copied.push('\n');
            copied.push_str(&last[..b]);
        }

        Some(copied)
    }

     pub fn yank(&mut self, text: &str) {
        for (i, line) in text.split('\n').enumerate() {
            if i == 0 {
                for ch in line.chars() {
                    self.insert_char(ch);
                }
            } else {
                self.insert_newline();
                for ch in line.chars() {
                    self.insert_char(ch);
                }
            }
        }
    }

    pub fn open_file(&mut self, path: PathBuf) -> io::Result<()> {
	if !path.exists() {
            self.lines = vec![String::new()];
            self.file_path = Some(path);
            self.cursor_x = 0;
            self.cursor_y = 0;
            return Ok(());
	}

	let content = std::fs::read_to_string(&path)?;
	self.lines = content.lines().map(|s| s.to_string()).collect();
	self.file_path = Some(path);
	self.cursor_x = 0;
	self.cursor_y = 0;
	Ok(())
    }

    pub fn save(&self) -> io::Result<()> {
	let path = self.file_path.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "No file name")
	})?;

	let content = self.lines.join("\n");
	std::fs::write(path, content)?;
	Ok(())
    }    
    
}
