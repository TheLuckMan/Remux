use remux_core::editor::editor::{Editor, EditorEvent};

pub fn handle_isearch_event(editor: &mut Editor, ev: &EditorEvent) {
    match ev {
        EditorEvent::ISearchUpdate { .. } => {
            // НИЧЕГО не делаем
            // Editor сам уже обновил isearch state
        }

        EditorEvent::ISearchFinished | EditorEvent::ISearchAborted => {
            editor.isearch = None;
        }

        _ => {}
    }
}
