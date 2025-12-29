use std::io::{self, ErrorKind};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub file_path: Option<PathBuf>,
}


impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
	    file_path: None,
        }
    }

    fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
	s.char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(s.len())
    }


    pub fn insert_char(&mut self, c: char) {
	let y = self.cursor_y;
	let x = self.cursor_x;

	self.lines[y].insert(x, c);
	self.cursor_x += 1;
    }
    
    pub fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_y) {
            if self.cursor_x < line.len() {
                self.cursor_x += 1;
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.clamp_cursor_x();
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.clamp_cursor_x();
        }
    }

    pub fn move_bol(&mut self) {
        self.cursor_x = 0;
    }

    pub fn move_eol(&mut self) {
        let y = self.cursor_y;
        if let Some(line) = self.lines.get(y) {
            self.cursor_x = line.chars().count();
        }
    }

    pub fn move_beginning_of_buffer(&mut self) {
	self.cursor_y = 0;
	self.cursor_x = 0;
    }

    pub fn move_end_of_buffer(&mut self) {
	if self.lines.is_empty() {
            self.cursor_y = 0;
            self.cursor_x = 0;
            return;
	}

	self.cursor_y = self.lines.len() - 1;
	self.cursor_x = self.lines[self.cursor_y]
            .chars()
            .count();
    }


    fn clamp_cursor_x(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_y) {
            if self.cursor_x > line.len() {
                self.cursor_x = line.len();
            }
        }
    }

    pub fn delete_char(&mut self) {
	let y = self.cursor_y;
	let line = &mut self.lines[y];

	let char_count = line.chars().count();
	if self.cursor_x >= char_count {
            return;
	}

	let start = Self::char_to_byte_idx(line, self.cursor_x);
	let end   = Self::char_to_byte_idx(line, self.cursor_x + 1);

	line.replace_range(start..end, "");
    }

    pub fn backward_delete_char(&mut self) {
	if self.cursor_x == 0 {
            return;
	}

	let y = self.cursor_y;
	let line = &mut self.lines[y];

	let start = Self::char_to_byte_idx(line, self.cursor_x - 1);
	let end   = Self::char_to_byte_idx(line, self.cursor_x);

	line.replace_range(start..end, "");
	self.cursor_x -= 1;
    }
    
    pub fn insert_newline(&mut self) {
	let line = &mut self.lines[self.cursor_y];
	let rest = line.split_off(self.cursor_x);

	self.cursor_y += 1;
	self.cursor_x = 0;
	self.lines.insert(self.cursor_y, rest);
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
