// core/src/editor/events.rs
#[derive(Debug)]
pub enum EditorEvent {
    BeforeCommand { name: String },
    AfterCommand { name: String },
    BufferChanged,
    MinibufferOpened,
    MinibufferClosed,
    Message(String),
    Custom(String),
}
pub trait EditorHook {
    fn on_event(&mut self, event: &EditorEvent);
}
pub struct FnHook<F>
where
    F: FnMut(&EditorEvent),
{
    pub func: F,
}

impl<F> FnHook<F>
where
    F: FnMut(&EditorEvent),
{
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> EditorHook for FnHook<F>
where
    F: FnMut(&EditorEvent),
{
    fn on_event(&mut self, event: &EditorEvent) {
        (self.func)(event)
    }
}
