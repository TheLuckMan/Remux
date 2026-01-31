use remux_core::editor::editor::Editor;

use crate::view::{
    render_state::RenderState,
    highlight::{Highlight, HighlightGroup},
    HighlightPriority,
};

pub fn apply_isearch(editor: &Editor, render: &mut RenderState) {

		 if !editor.user_config.borrow().isearch_highlight {
        render.clear_group(HighlightGroup::ISearch);
        render.clear_group(HighlightGroup::ISearchCurrent);
        return;
    }

    let Some(isearch) = &editor.isearch else {
        return;
    };

    let query = &isearch.query;
    if query.is_empty() {
        return;
    }

    render.clear_group(HighlightGroup::ISearch);
    render.clear_group(HighlightGroup::ISearchCurrent);

    for (y, line) in editor.buffer.lines.iter().enumerate() {
        let mut start = 0;
        while let Some(pos) = line.text[start..].find(query) {
            let x = start + pos;

            let group = if isearch.last_match == Some((x, y)) {
                HighlightGroup::ISearchCurrent
            } else {
                HighlightGroup::ISearch
            };

            let priority = match group {
                HighlightGroup::ISearchCurrent => HighlightPriority::High,
                _ => HighlightPriority::Normal,
            };

            render.add(Highlight {
                x,
                y,
                len: query.len(),
                group: group.clone(),
                priority,
                style: Highlight::style_for(group),
            });

            start += pos + 1;
        }
    }
}
