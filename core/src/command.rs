use std::collections::HashMap;
use crate::editor::Editor;

pub struct CommandContext<'a> {
    pub editor: &'a mut Editor,
    pub arg: Option<&'a str>,
}

pub enum EditorEvent<'a> {
    BeforeCommand(&'a str),
    AfterCommand(&'a str),
}



pub struct Command {
    pub name: &'static str,
    pub run: fn(CommandContext),
}

pub struct CommandRegistry {
    commands: HashMap<String, Command>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, cmd: Command) {
        self.commands.insert(cmd.name.to_string(), cmd);
    }

    pub fn get(&self, name: &str) -> Option<&Command> {
        self.commands.get(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.commands.keys()
    }
}
