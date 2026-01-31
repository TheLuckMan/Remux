use ratatui::style::{Style, Color};
use remux_core::editor::editor::Editor;

use crate::view::{
    RenderState,
    Highlight,
    HighlightGroup,
    HighlightPriority,
};

pub fn apply_selection(editor: &Editor, render: &mut RenderState) {
    render.clear_group(HighlightGroup::Selection);

    let Some(sel) = editor.buffer.selection() else {
        return;
    };

    let style = Style::default()
        .bg(Color::White)
        .fg(Color::Black);

    for y in sel.start.y..=sel.end.y {
        let line = &editor.buffer.lines[y];
        let line_len = line.char_len;

        let start_x = if y == sel.start.y { sel.start.x } else { 0 };
        let end_x   = if y == sel.end.y   { sel.end.x }   else { line_len };

        if start_x >= end_x {
            continue;
        }

        render.add(Highlight {
            x: start_x,
            y,
            len: end_x - start_x,
            group: HighlightGroup::Selection,
            priority: HighlightPriority::High,
            style,
        });
    }
}
