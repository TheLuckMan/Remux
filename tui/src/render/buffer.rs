use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Paragraph, Clear},
    text::{Line, Span},
};

use unicode_width::{UnicodeWidthChar};
use remux_core::editor::editor::{Editor};
use crate::view::RenderState;

pub fn render_buffer(
    f: &mut Frame,
    editor: &Editor,
    area: Rect,
    render: &RenderState,
) {
    let mut lines = Vec::new();

    for vis in editor.iter_visible_visual_lines() {
        let buf_y = vis.buffer_y;
        let line = &editor.buffer.lines[buf_y];

        let slice = slice_by_cell(&line.text, vis.start_x, vis.len);

        let mut spans = Vec::new();
        for (i, ch) in slice.chars().enumerate() {
            let style = render.style_at(vis.start_x + i, buf_y).unwrap_or_default();
            spans.push(Span::styled(ch.to_string(), style));
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

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
