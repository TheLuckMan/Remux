use ratatui::{
		style::{Style, Color},
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum HighlightPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HighlightGroup {
    ISearch,
		ISearchCurrent,
    Selection,
    // Diagnostics,
}

#[derive(Clone, Debug)]
pub struct Highlight {
    pub x: usize,
    pub y: usize,
    pub len: usize,
    pub group: HighlightGroup,
    pub priority: HighlightPriority,
    pub style: Style,
}

impl Highlight {
    pub fn style(&self) -> Style {
        match self.group {
            HighlightGroup::ISearch =>
                Style::default().bg(Color::Yellow).fg(Color::Black),

						HighlightGroup::ISearchCurrent =>
                Style::default().bg(Color::LightRed).fg(Color::Black),

            HighlightGroup::Selection =>
                Style::default().bg(Color::White).fg(Color::Black),
						
        }
    }
		
    pub fn style_for(group: HighlightGroup) -> Style {
        match group {
            HighlightGroup::ISearch =>
                Style::default().bg(Color::Yellow).fg(Color::Black),

            HighlightGroup::ISearchCurrent =>
                Style::default().bg(Color::LightRed).fg(Color::Black),

            HighlightGroup::Selection =>
                Style::default().bg(Color::White).fg(Color::Black),
        }
    }
		
		 pub fn covers(&self, x: usize, y: usize) -> bool {
        self.y == y && x >= self.x && x < self.x + self.len
    }
}
