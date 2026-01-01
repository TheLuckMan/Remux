use std::collections::HashMap;
use mlua::{Lua, RegistryKey, Result};

pub struct HookRegistry {
    hooks: HashMap<String, Vec<RegistryKey>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    pub fn add(&mut self, lua: &Lua, name: &str, func: mlua::Function) -> Result<()> {
        let key = lua.create_registry_value(func)?;
        self.hooks.entry(name.to_string()).or_default().push(key);
        Ok(())
    }

     pub fn add_key(&mut self, name: String, key: RegistryKey) {
        self.hooks.entry(name).or_default().push(key);
    }

    pub fn run(&self, lua: &Lua, name: &str, arg: &str) {
        if let Some(funcs) = self.hooks.get(name) {
            for key in funcs {
                if let Ok(func) = lua.registry_value::<mlua::Function>(key) {
                    let _ = func.call::<_, ()>(arg);
                }
            }
        }
    }
}
