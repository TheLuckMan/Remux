use std::path::PathBuf;
use crate::editor::PhysicalModifiers;

#[derive(Clone)]
pub struct UserConfig {
    pub mod_mask: PhysicalModifiers,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            mod_mask: PhysicalModifiers::ALT,
        }
    }
}

// Это импорт ~/.config/remux/init.lua
pub fn config_path() -> PathBuf {
    let mut path = dirs::config_dir()
        .expect("No XDG config directory found");

    path.push("remux");
    path.push("init.lua");

    path
}
