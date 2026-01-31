use super::{Highlight, HighlightGroup};
use ratatui::prelude::Style;


pub struct View {
    pub render: RenderState,
}

#[derive(Default)]
pub struct RenderState {
    highlights: Vec<Highlight>,
}


impl RenderState {
    pub fn clear_group(&mut self, group: HighlightGroup) {
        self.highlights.retain(|h| h.group != group);
    }

    pub fn add(&mut self, hl: Highlight) {
        self.highlights.push(hl);
    }

    pub fn extend<I: IntoIterator<Item = Highlight>>(&mut self, iter: I) {
        self.highlights.extend(iter);
    }

    pub fn clear(&mut self) {
        self.highlights.clear();
    }

		pub fn style_at(&self, x: usize, y: usize) -> Option<Style> {
        self.highlights
            .iter()
            .filter(|hl| hl.covers(x, y))
            .max_by_key(|hl| hl.priority)
            .map(|hl| hl.style)
    }
}

