#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MiniBufferMode {
    Command,    // M-x
    FindFile,   // ожидание пути
    Message,    // просто сообщение
}

pub struct MiniBuffer {
    text: String,
    active: bool,
    mode: MiniBufferMode,
}

impl Default for MiniBuffer {
    fn default() -> Self {
        Self {
            text: String::new(),
            active: false,
            mode: MiniBufferMode::Message,
        }
    }
}

impl MiniBuffer {
    
    pub fn activate(&mut self, prompt: &str, mode: MiniBufferMode) {
        self.text.clear();
        self.text.push_str(prompt);
        self.active = true;
        self.mode = mode;
    }

    pub fn deactivate(&mut self) {
        self.text.clear();
        self.active = false;
        self.mode = MiniBufferMode::Message;
    }
    
   pub fn set_text<S: Into<String>>(&mut self, text: S) {
        self.text = text.into();
    }
    
    pub fn push(&mut self, c: char) {
        self.text.push(c);
    }

    pub fn pop(&mut self) {
        self.text.pop();
    }

    pub fn get(&self) -> &str {
        &self.text
    }

    pub fn mode(&self) -> MiniBufferMode {
        self.mode
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}
