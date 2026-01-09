use ratatui::{
    Frame,
    layout::{Layout, Constraint, Direction, Rect},
    widgets::{Block, Borders, Paragraph},
    style::{Style, Color},
};

use remux_core::editor::Editor;

use crate::render::{
    buffer::render_buffer,
    status::render_status,
    minibuffer::render_minibuffer,
};

pub fn render_editor(
    f: &mut Frame,
    editor: &mut Editor,
) {
    let size = f.size();

    let status_info = remux_core::status::build_status(&editor);
    let status_line = render_status(&status_info);
    let status_bar = Paragraph::new(status_line)
	.style(Style::default().bg(Color::DarkGray))
	.block(Block::default());

    

    // ────────────────────────────────────────────────────────────
    // Layout
    //
    // ┌───────────────────────────────┐
    // │           BUFFER              │
    // ├───────────────────────────────┤
    // │           STATUS              │
    // ├───────────────────────────────┤
    // │         MINIBUFFER            │
    // └───────────────────────────────┘
    // ────────────────────────────────────────────────────────────

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1), // buffer
            Constraint::Length(1), // status
            Constraint::Length(1), // minibuffer
        ])
        .split(size);

    render_buffer_area(f, editor, chunks[0]);
    f.render_widget(status_bar, chunks[1]);
    render_minibuffer(f, editor, chunks[2]);
}

/// ─────────────────────────────────────────────────────────────
/// BUFFER
/// ─────────────────────────────────────────────────────────────

fn render_buffer_area(
    f: &mut Frame,
    editor: &mut Editor,
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::NONE);

    let inner = block.inner(area);
    f.render_widget(block, area);

    render_buffer(f, editor, inner);
}
