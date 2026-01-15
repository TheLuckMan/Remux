use ratatui::{
    Frame,
    layout::{Layout, Constraint, Direction, Rect},
    widgets::{Block, Borders, Paragraph},
    style::{Style, Color},
};
use mlua::Lua;
use remux_core::editor::editor::Editor;

use crate::render::{
    buffer::render_buffer,
    status::render_status,
    minibuffer::render_minibuffer,
};

pub fn render_editor(
    f: &mut Frame,
    editor: &mut Editor,
    lua: &Lua,
) {
    let size = f.size();

    let status_info = remux_core::status::build_status(editor);
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

    render_buffer_area(f, editor, chunks[0], lua);
    f.render_widget(status_bar, chunks[1]);
    render_minibuffer(f, editor, chunks[2]);

    render_cursor(f, editor, chunks[0]);
}

/// ─────────────────────────────────────────────────────────────
/// BUFFER
/// ─────────────────────────────────────────────────────────────

fn render_buffer_area(
    f: &mut Frame,
    editor: &mut Editor,
    area: Rect,
    lua: &Lua,
) {
    editor.hooks.run_once(lua, "after-init-once", "");

    let show_borders = editor.user_config.borrow().buffer_borders;

    let block = if show_borders {
        Block::default().borders(Borders::ALL)
    } else {
        Block::default().borders(Borders::NONE)
    };

    let inner = block.inner(area);

    editor.viewport_width  = inner.width  as usize;
    editor.viewport_height = inner.height as usize;

    f.render_widget(block, area);
    render_buffer(f, editor, inner);
}

fn render_cursor(
    f: &mut Frame,
    editor: &Editor,
    area: Rect,
) {
    let (cx, cy) = editor.cursor_visual_pos();

    let show_borders = editor.user_config.borrow().buffer_borders;

    let x = area.x + cx as u16;
    let y = area.y + cy.saturating_sub(editor.scroll_y) as u16;

    if show_borders {
    f.set_cursor(x+1, y+1);
    } else {
    f.set_cursor(x, y);
    }
    
}
