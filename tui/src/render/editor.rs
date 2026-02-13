use ratatui::{
    Frame,
    layout::{Layout, Constraint, Direction, Rect},
    widgets::{Block, Borders, Paragraph},
    style::{Style, Color},
		prelude::{Line, Span},
};
use mlua::Lua;
use remux_core::editor::editor::Editor;
use crate::view::RenderState;
use crate::render::{
    buffer::render_buffer,
    status::render_status,
    minibuffer::render_minibuffer,
};

pub fn render_editor(
    f: &mut Frame,
    editor: &mut Editor,
		render: &mut RenderState,
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
    // │  Nums  |  BUFFER              │
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

		render_buffer_area(
				f,
				editor,
				chunks[0],
				render,
				lua,
		);
    f.render_widget(status_bar, chunks[1]);
    render_minibuffer(f, editor, chunks[2]);

    render_cursor(f, editor, lua, chunks[0]);
}

/// ─────────────────────────────────────────────────────────────
/// BUFFER
/// ─────────────────────────────────────────────────────────────
fn render_buffer_area(
    f: &mut Frame,
    editor: &mut Editor,
    area: Rect,
    render: &mut RenderState,
    lua: &Lua,
) -> Rect {

    editor.hooks.run_once(lua, "after-init-once", "");

    let show_borders = editor.user_config.borrow().buffer_borders;

    let block = if show_borders {
        Block::default().borders(Borders::ALL)
    } else {
        Block::default()
    };

    f.render_widget(&block, area);

    let inner = block.inner(area);

    let buffer_rect = if line_number_mode(editor, lua) {

        let gutter_w = line_number_width(editor);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(gutter_w),
                Constraint::Min(1),
            ])
            .split(inner);

        render_line_numbers(f, editor, lua, cols[0]);

        cols[1]

    } else {
        inner
    };

    editor.viewport_width  = buffer_rect.width as usize;
    editor.viewport_height = buffer_rect.height as usize;

    editor.buffer.rebuild_visual_metrics(
        editor.viewport_width,
        editor.wrap_mode,
    );

    render_buffer(f, editor, buffer_rect, render);

    buffer_rect
}



fn render_cursor(
    f: &mut Frame,
    editor: &mut Editor,
		lua: &Lua,
    area: Rect,
) {
    let (cx, cy) = editor.cursor_visual_pos();
    let show_borders = editor.user_config.borrow().buffer_borders;
		let mut x = area.x + cx as u16;

		if line_number_mode(editor, lua) {
				x += line_number_width(editor);
		}
		
    let y = area.y + cy as u16;
    if show_borders {
	f.set_cursor(x+1, y+1);
    } else {
	f.set_cursor(x, y);
    }
}

fn line_number_mode(editor: &Editor, lua: &Lua) -> bool {
    match editor.hooks.run_collect_bool(lua, "line-number-mode", "") {
        Some(results) => results.into_iter().all(|v| v),
        None => false, // default OFF
    }
}

fn line_number_style(editor: &Editor, lua: &Lua) -> Option<Style> {
    let values = editor.hooks.run_collect(lua, "line-number-style", "")?;

    values.last().and_then(|s| parse_style(s))
}

fn parse_style(s: &str) -> Option<Style> {
    let mut style = Style::default();

    for part in s.split(',') {
        let mut kv = part.split('=');
        let key = kv.next()?.trim();
        let val = kv.next()?.trim();

        match key {
            "bg" => style = style.bg(parse_color(val)?),
            "fg" => style = style.fg(parse_color(val)?),
            _ => {}
        }
    }

    Some(style)
}

fn parse_color(name: &str) -> Option<Color> {
    Some(match name {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" => Color::Gray,
        "darkgray" => Color::DarkGray,
        _ => return None,
    })
}


fn line_number_width(editor: &Editor) -> u16 {
    let total = editor.buffer.lines.len().max(1);
    total.to_string().len() as u16 + 2
}

/*
fn render_line_numbers(
    f: &mut Frame,
    editor: &Editor,
    area: Rect,
) {
    let mut lines = Vec::new();

    for vis in editor.iter_visible_visual_lines() {
        let n = vis.buffer_y + 1;

        lines.push(Line::from(format!("{:>width$} ", n,
            width = area.width as usize - 1
        )));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
} */

fn render_line_numbers(
    f: &mut Frame,
    editor: &Editor,
    lua: &Lua,
    area: Rect,
) {
    let mut lines = Vec::new();

    let style = line_number_style(editor, lua).unwrap_or_default();

    let width = area.width.saturating_sub(1) as usize;

    for vis in editor.iter_visible_visual_lines() {
        let n = vis.buffer_y + 1;

        lines.push(
            Line::from(
                Span::styled(format!("{:>width$} ", n), style)
            )
        );
    }

    f.render_widget(Paragraph::new(lines), area);
}

