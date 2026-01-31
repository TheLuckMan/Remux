#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MiniBufferMode {
    Inactive,
    Command,    // M-x
    FindFile,   // waiting path
    SaveBuffer,
    GotoLine,
    ISearchForward,
    ISearchBackward,
    Message { ttl: u8 },    // just a message
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
            mode: MiniBufferMode::Message { ttl: 0 },
        }
    }
}

impl MiniBuffer {
    
    pub fn activate(&mut self, prompt: &str, mode: MiniBufferMode) {
        self.text.clear();
        self.text.push_str(prompt);
	self.text = self.text.to_string();
        self.active = true;
        self.mode = mode;
    }

    pub fn deactivate(&mut self) {
        self.text.clear();
        self.active = false;
        self.mode = MiniBufferMode::Message { ttl: 0 };
    }
    
   pub fn set_text<S: Into<String>>(&mut self, text: S) {
        self.text = text.into();
   }

     pub fn message(&mut self, text: &str) {
        self.activate(text, MiniBufferMode::Message { ttl: 2 });
     }

    
    pub fn clear(&mut self) {
        self.text.clear();
        self.mode = MiniBufferMode::Inactive;
    }

    pub fn tick(&mut self) {
        if let MiniBufferMode::Message { ttl } = self.mode {
            if ttl <= 1 {
                self.clear();
            } else {
                self.mode = MiniBufferMode::Message { ttl: ttl - 1 };
            }
        }
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
