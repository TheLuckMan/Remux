use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Paragraph, Block, Borders, Clear},
    style::{Style, Color},
    text::{Line, Span},
};

use unicode_width::{UnicodeWidthStr, UnicodeWidthChar};
use remux_core::editor::editor::{Editor, VisibleVisualLines};

pub fn render_buffer(
    f: &mut Frame,
    editor: &mut Editor,
    area: Rect,
) {
    editor.buffer.ensure_visuals(
        editor.viewport_width,
        editor.wrap_mode,
    );
    editor.ensure_cursor_visible();
    let visuals = editor.iter_visible_visual_lines();
    let lines   = render_visual_lines(editor, visuals);
    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

/// ─────────────────────────────────────────────────────────────
/// VISUAL LINES
/// ─────────────────────────────────────────────────────────────

fn render_visual_lines(
    editor: &Editor,
    visuals: VisibleVisualLines,
) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    let selection = editor.buffer.selection();
    for v in visuals {
        let line = &editor.buffer.lines[v.buffer_y];
        let line_len = line.char_len;
	let vis_start =  v.start_x;
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
                    &line.text,
                    vis_start,
                    vis_end,
                    sel_start,
                    sel_end,
                );
            }
            _ => {
		let mut s = slice_by_cell(&line.text, v.start_x, v.len);
		let pad = v.len.saturating_sub(UnicodeWidthStr::width(s.as_str()));
		s.extend(std::iter::repeat(' ').take(pad));
		out.push(Line::raw(s));
            }
        }
    }
    let height = editor.viewport_height;
    while out.len() < height {
	out.push(Line::raw(" ".repeat(editor.viewport_width)));
    }
    out.truncate(height);
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
    let sel_a = sel_start.clamp(vis_start, vis_end);
    let sel_b = sel_end.clamp(vis_start, vis_end);
    let mut spans = Vec::new();
    // A: before selection
    if vis_start < sel_a {
        spans.push(Span::raw(
            slice_by_cell(line, vis_start, sel_a - vis_start)
        ));
    }
    // B: selection
    if sel_a < sel_b {
        spans.push(Span::styled(
            slice_by_cell(line, sel_a, sel_b - sel_a),
            Style::default().bg(Color::White).fg(Color::Black),
        ));
    }
    // C: after selection
    if sel_b < vis_end {
        spans.push(Span::raw(
            slice_by_cell(line, sel_b, vis_end - sel_b)
        ));
    }
    // pad to visual width (NOT viewport)
    let total_width: usize = spans.iter()
        .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
        .sum();

    let target = vis_end - vis_start;
    if total_width < target {
        spans.push(Span::raw(" ".repeat(target - total_width)));
    }

    out.push(Line::from(spans));
}

/// ─────────────────────────────────────────────────────────────
/// UTF-8 HELPERS
/// ─────────────────────────────────────────────────────────────

fn slice_by_cell(s: &str, start: usize, width: usize) -> String {
    const TABSTOP: usize = 4;
    let mut cur = 0;
    let mut out = String::new();
    for ch in s.chars() {
        if ch == '\t' {
            let tab_w = TABSTOP - (cur % TABSTOP);
            if cur + tab_w <= start {
                cur += tab_w;
                continue;
            }
            let visible_start = start.saturating_sub(cur);
            let visible_end = (start + width).saturating_sub(cur);
            let from = visible_start.min(tab_w);
            let to   = visible_end.min(tab_w);

            if from < to {
                out.extend(std::iter::repeat(' ').take(to - from));
            }
            cur += tab_w;
            if cur >= start + width {
                break;
            }
            continue;
        }
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if cur + w <= start {
            cur += w;
            continue;
        }
        if cur >= start + width {
            break;
        }
        out.push(ch);
        cur += w;
    }
    out
}
