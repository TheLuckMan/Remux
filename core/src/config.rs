use std::path::PathBuf;
use crate::editor::PhysicalModifiers;

#[derive(Clone)]
pub struct UserConfig {
    pub mod_masks: [PhysicalModifiers; 3], // 0..=2: mod1, mod2, prefix-x
    pub prefix_keys: [Option<char>; 3],    // Which key activates prefix
    pub prefix_masks: [PhysicalModifiers; 3], // Physical Modifiers with activates prefix
    pub buffer_borders: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            mod_masks: [
                PhysicalModifiers::CTRL,            // 0 — mod1
                PhysicalModifiers::ALT,             // 1 — mod2
                PhysicalModifiers::CTRL,         // 2 — Ctrl-x, now it none
            ],
	      prefix_keys: [
                None,
                None,
                Some('x'),
            ],

            prefix_masks: [
                PhysicalModifiers::empty(),
                PhysicalModifiers::empty(),
                PhysicalModifiers::CTRL,
            ],
	    
	    buffer_borders: false,
        }
    }
}

// Import ~/.config/remux/init.lua
pub fn config_path() -> PathBuf {
    let mut path = dirs::config_dir()
        .expect("No XDG config directory found");

    path.push("remux");
    path.push("init.lua");

    path
}
