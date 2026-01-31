pub mod app;
pub mod input;
pub mod render;
pub mod hooks;
pub mod view;

pub type LuaEventQueue = std::rc::Rc<std::cell::RefCell<Vec<remux_core::editor::editor::EditorEvent>>>;

