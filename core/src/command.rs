use std::sync::Arc;
use std::collections::HashMap;
use crate::editor::editor::Editor;

pub enum CommandArg {
    None,
    Int(i64),
    Str(String),
}

pub enum Interactive {
    None,
    Int,
    Str { prompt: &'static str },
}

pub struct CommandContext<'a> {
    pub editor: &'a mut Editor,
    pub arg: CommandArg,
    
}

pub enum EditorEvent<'a> {
    BeforeCommand(&'a str),
    AfterCommand(&'a str),
}



pub struct Command {
    pub name: &'static str,
    pub interactive: Interactive,
    pub run: fn(CommandContext),
}

pub struct CommandRegistry {
    commands: HashMap<String, Arc<Command>>,
}

impl Command {
    pub fn interactive_string_prompt(&self) -> Option<&'static str> {
        match self.interactive {
            Interactive::Str { prompt } => Some(prompt),
            _ => None,
        }
    }
		pub fn modifies_prefix(&self) -> bool {
				self.name == "universal-argument"
						|| self.name == "negative-argument"
						|| self.name.starts_with("digit-argument-")
		}
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, cmd: Arc<Command>) {
        self.commands.insert(cmd.name.to_string(), cmd);
    }

    pub fn get(&self, name: &str) -> Option<Arc<Command>> {
        self.commands.get(name).cloned()
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.commands.keys()
    }
}
