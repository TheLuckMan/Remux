use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Paragraph, Block, Borders},
    style::{Style, Color},
};
use remux_core::editor::Editor;

pub fn render_minibuffer(f: &mut Frame, editor: &Editor, area: Rect) {
    let content = editor.minibuffer.get();

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(paragraph, area);
}
