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
		pub fn run_table(&self, lua: &Lua, name: &str, table: mlua::Table) {
				if let Some(funcs) = self.hooks.get(name) {
						for f in funcs {
								let func: mlua::Function = lua.registry_value(f).unwrap();
								let _ = func.call::<_, ()>(table.clone());
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
			pub fn run_collect(
				&self,
				lua: &Lua,
				name: &str,
				arg: &str
		) -> Option<Vec<String>> {

				let mut out = Vec::new();

				if let Some(funcs) = self.hooks.get(name) {
						for key in funcs {
								if let Ok(func) = lua.registry_value::<Function>(key) {
										if let Ok(Some(v)) = func.call::<_, Option<String>>(arg) {
												out.push(v);
										}
								}
						}
				}

				if out.is_empty() {
						None
				} else {
						Some(out)
				}
			}

		pub fn run_collect_bool(
				&self,
				lua: &Lua,
				name: &str,
				arg: &str
		) -> Option<Vec<bool>> {

				let mut out = Vec::new();

				if let Some(funcs) = self.hooks.get(name) {
						for key in funcs {
								if let Ok(func) = lua.registry_value::<Function>(key) {
										if let Ok(Some(v)) = func.call::<_, Option<bool>>(arg) {
												out.push(v);
										}
								}
						}
				}

				if out.is_empty() { None } else { Some(out) }
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
