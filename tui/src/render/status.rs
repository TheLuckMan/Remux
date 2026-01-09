use remux_core::status::StatusInfo;
pub fn render_status(info: &StatusInfo) -> String {
    let undo = if info.undo_depth > 0 {
        format!("U:{}", "*".repeat(info.undo_depth.min(3)))
    } else {
        "U:—".to_string()
    };

    let modified = if info.modified { "*" } else { "" };

    format!(
        " {} {}{} {:>3} ({}, {}) ",
        undo,
        info.file_name,
        modified,
        info.scroll_percent,
        info.cursor_line,
        info.cursor_col
    )
}
