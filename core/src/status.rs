use crate::editor::Editor;

#[derive(Debug, Clone)]
pub struct StatusInfo {
    pub file_name: String,
    pub modified: bool,
    pub undo_depth: usize,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_percent: String,
}

pub fn build_status(editor: &Editor) -> StatusInfo {
    let buffer = &editor.buffer;

    StatusInfo {
        file_name: buffer.file_name(),
        modified: buffer.is_modified(),
        undo_depth: buffer.undo_depth(),
        cursor_line: buffer.cursor_y + 1,
        cursor_col: buffer.cursor_x + 1,
	scroll_percent: editor.scroll_indicator(),
        
    }
}


