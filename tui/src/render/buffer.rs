use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Paragraph, Block, Borders},
    style::{Style, Color},
    text::{Line, Span},
};

use remux_core::editor::{Editor, VisualLine};

/// Главная функция рендера buffer
pub fn render_buffer(
    f: &mut Frame,
    editor: &mut Editor,
    area: Rect,
) {
    editor.viewport_height = area.height as usize;
    editor.viewport_width  = area.width as usize;

    let visuals = editor.build_visual_lines();
    let lines   = render_visual_lines(editor, &visuals);

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(paragraph, area);

    render_cursor(f, editor, area);
}

/// ─────────────────────────────────────────────────────────────
/// CURSOR
/// ─────────────────────────────────────────────────────────────

fn render_cursor(
    f: &mut Frame,
    editor: &Editor,
    area: Rect,
) {
    let (cx, cy) = editor.cursor_visual_pos();

    let x = area.x + cx as u16;
    let y = area.y + cy.saturating_sub(editor.scroll_y) as u16;

    f.set_cursor(x, y);
}

/// ─────────────────────────────────────────────────────────────
/// VISUAL LINES
/// ─────────────────────────────────────────────────────────────

fn render_visual_lines(
    editor: &Editor,
    visuals: &[VisualLine],
) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    let selection = editor.buffer.selection();

    for v in visuals {
        let line = &editor.buffer.lines[v.buffer_y];
        let line_len = line.chars().count();

        let vis_start = v.start_x;
        let vis_end   = (v.start_x + v.len).min(line_len);

        let sel = selection
            .filter(|s| v.buffer_y >= s.start.y && v.buffer_y <= s.end.y)
            .map(|s| {
                let start = if v.buffer_y == s.start.y { s.start.x } else { 0 };
                let end   = if v.buffer_y == s.end.y   { s.end.x } else { line_len };
                (start, end)
            });

        match sel {
            Some((sel_start, sel_end))
                if sel_start < vis_end && sel_end > vis_start =>
            {
                render_line_with_selection(
                    &mut out,
                    line,
                    vis_start,
                    vis_end,
                    sel_start,
                    sel_end,
                );
            }

            _ => {
                let (bs, be) = char_range_to_byte_range(line, vis_start, vis_end);
                out.push(Line::raw(line[bs..be].to_string()));
            }
        }
    }

    out
}

/// ─────────────────────────────────────────────────────────────
/// LINE HELPERS
/// ─────────────────────────────────────────────────────────────

fn render_line_with_selection(
    out: &mut Vec<Line<'static>>,
    line: &str,
    vis_start: usize,
    vis_end: usize,
    sel_start: usize,
    sel_end: usize,
) {
    let a_start = vis_start;
    let a_end   = sel_start.clamp(vis_start, vis_end);
    let b_start = a_end;
    let b_end   = sel_end.clamp(vis_start, vis_end);
    let c_start = b_end;
    let c_end   = vis_end;

    let mut spans = Vec::new();

    if a_start < a_end {
        let (bs, be) = char_range_to_byte_range(line, a_start, a_end);
        spans.push(Span::raw(line[bs..be].to_string()));
    }

    if b_start < b_end {
        let (bs, be) = char_range_to_byte_range(line, b_start, b_end);
        spans.push(
            Span::styled(
                line[bs..be].to_string(),
                Style::default().bg(Color::Blue),
            )
        );
    }

    if c_start < c_end {
        let (bs, be) = char_range_to_byte_range(line, c_start, c_end);
        spans.push(Span::raw(line[bs..be].to_string()));
    }

    out.push(Line::from(spans));
}

/// ─────────────────────────────────────────────────────────────
/// UTF-8 HELPERS
/// ─────────────────────────────────────────────────────────────

fn char_range_to_byte_range(
    s: &str,
    start: usize,
    end: usize,
) -> (usize, usize) {
    let mut indices: Vec<usize> =
        s.char_indices().map(|(i, _)| i).collect();

    indices.push(s.len());

    let bs = *indices.get(start).unwrap_or(&s.len());
    let be = *indices.get(end).unwrap_or(&s.len());

    (bs, be)
}
