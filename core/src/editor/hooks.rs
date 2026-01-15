use std::collections::HashMap;
use mlua::{Lua, RegistryKey, Function, Result};
use crate::editor::events::EditorEvent;

pub struct HookRegistry {
    hooks: HashMap<String, Vec<RegistryKey>>,
}

impl HookRegistry {

    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }


    pub fn add(&mut self, lua: &Lua, name: &str, func: Function) -> Result<()> {
        let key = lua.create_registry_value(func)?;
        self.hooks.entry(name.to_string()).or_default().push(key);
        Ok(())
    }


    pub fn add_key(&mut self, name: String, key: RegistryKey) {
        self.hooks.entry(name).or_default().push(key);
    }

     pub fn run_once(&mut self, lua: &Lua, name: &str, arg: &str) {
        if let Some(funcs) = self.hooks.remove(name) {
            for key in funcs {
                if let Ok(func) = lua.registry_value::<mlua::Function>(&key) {
                    let _ = func.call::<_, ()>(arg);
                }
            }
        }
    }

    pub fn run(&self, lua: &Lua, name: &str, arg: &str) {
        if let Some(funcs) = self.hooks.get(name) {
            for key in funcs {
                if let Ok(func) = lua.registry_value::<Function>(key) {
                    let _ = func.call::<_, ()>(arg);
                }
            }
        }
    }
}

#[derive(Default)]
pub struct RustHookRegistry {
    pub hooks: HashMap<String, Vec<Box<dyn Fn(&EditorEvent) + Send + Sync>>>,
}

impl RustHookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }


    pub fn add<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&EditorEvent) + Send + Sync + 'static,
    {
        self.hooks.entry(name.to_string()).or_default().push(Box::new(f));
    }


    pub fn run(&self, name: &str, event: &EditorEvent) {
        if let Some(funcs) = self.hooks.get(name) {
            for f in funcs {
                f(event);
            }
        }
    }
}
