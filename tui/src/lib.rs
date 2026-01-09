pub mod app;
pub mod input;
pub mod render;

pub type LuaEventQueue = std::rc::Rc<std::cell::RefCell<Vec<remux_core::editor::EditorEvent>>>;

