use crate::command::{Command, CommandRegistry, CommandContext};
use crate::minibuffer::MiniBufferMode;
use crate::editor::InputMode;

fn move_left(ctx: CommandContext) {
    ctx.editor.buffer.move_left();
    ctx.editor.ensure_cursor_visible();
}

fn move_right(ctx: CommandContext) {
    ctx.editor.buffer.move_right();
    ctx.editor.ensure_cursor_visible();
}

fn move_up(ctx: CommandContext) {
    ctx.editor.buffer.move_up();
    ctx.editor.ensure_cursor_visible();
}

fn move_down(ctx: CommandContext) {
    ctx.editor.buffer.move_down();
    ctx.editor.ensure_cursor_visible();
}

fn move_beginning_of_buffer(ctx: CommandContext) {
    ctx.editor.buffer.move_beginning_of_buffer();
    ctx.editor.ensure_cursor_visible();
}

fn move_end_of_buffer(ctx: CommandContext) {
    ctx.editor.buffer.move_end_of_buffer();
    ctx.editor.ensure_cursor_visible();
}

fn move_beginning_of_line(ctx: CommandContext) {
    ctx.editor.buffer.move_bol();
    ctx.editor.ensure_cursor_visible();
}

fn move_end_of_line(ctx: CommandContext) {
    ctx.editor.buffer.move_eol();
    ctx.editor.ensure_cursor_visible();
}

fn kill_remux(ctx: CommandContext) {
    ctx.editor.should_quit = true;
}


fn backward_delete_char(ctx: CommandContext) {
    ctx.editor.buffer.backward_delete_char();
    ctx.editor.ensure_cursor_visible();
}

fn delete_char(ctx: CommandContext) {
    ctx.editor.buffer.delete_char();
    ctx.editor.ensure_cursor_visible();
}

fn newline(ctx: CommandContext) {
    ctx.editor.buffer.insert_newline();
    ctx.editor.ensure_cursor_visible();
}

fn undo(ctx: CommandContext) {
    ctx.editor.buffer.undo();
}

fn toggle_mark(ctx: CommandContext) {
    ctx.editor.buffer.toggle_mark();
}

fn kill_region(ctx: CommandContext) {
    if let Some(text) = ctx.editor.buffer.kill_region() {
        ctx.editor.kill_buffer = Some(text);
        ctx.editor.minibuffer.message("Killed region");
	ctx.editor.ensure_cursor_visible();
    } else {
        ctx.editor.minibuffer.message("No active region");
    }
}

fn copy_region(ctx: CommandContext) {
    if let Some(text) = ctx.editor.buffer.copy_region() {
        ctx.editor.kill_buffer = Some(text);
        ctx.editor.minibuffer.message("Copy region");
	ctx.editor.ensure_cursor_visible();
    } else {
        ctx.editor.minibuffer.message("No active region");
    }
}

fn scroll_up(ctx: CommandContext) {
    ctx.editor.scroll_up();
}

fn scroll_down(ctx: CommandContext) {
    ctx.editor.scroll_down();
}

fn yank(ctx: CommandContext) {
    if let Some(text) = ctx.editor.kill_buffer.clone() {
        ctx.editor.buffer.yank(&text);
        ctx.editor.minibuffer.message("Yanked");
	ctx.editor.ensure_cursor_visible();
    } else {
        ctx.editor.minibuffer.message("Kill buffer empty");
    }
}

fn save_buffer(ctx: CommandContext) {
    if ctx.editor.buffer.save().is_ok() {
        ctx.editor.minibuffer.activate(
            "Buffer saved!",
            MiniBufferMode::Message { ttl: 2 },
        );
    }
}

fn execute_command(ctx: CommandContext) {
    ctx.editor.mode = InputMode::MiniBuffer;
    ctx.editor.minibuffer.activate("M-x ", MiniBufferMode::Command);
}

fn find_file(ctx: CommandContext) {
    ctx.editor.mode = InputMode::MiniBuffer;
    ctx.editor.minibuffer.activate(
        "Find file: ",
        MiniBufferMode::FindFile,
    );
}


pub fn register_builtins(reg: &mut CommandRegistry) {
    reg.register(Command { name: "move-left", run: move_left });
    reg.register(Command { name: "move-right", run: move_right });
    reg.register(Command { name: "move-up", run: move_up });
    reg.register(Command { name: "move-down", run: move_down });
    reg.register(Command { name: "move-beginning-of-buffer", run: move_beginning_of_buffer });
    reg.register(Command { name: "move-end-of-buffer", run: move_end_of_buffer });
    reg.register(Command { name: "move-beginning-of-line", run: move_beginning_of_line });
    reg.register(Command { name: "move-end-of-line", run: move_end_of_line });
    reg.register(Command { name: "undo", run: undo });
    reg.register(Command { name: "kill-remux", run: kill_remux });
    reg.register(Command { name: "delete-char", run: delete_char });
    reg.register(Command { name: "backward-delete-char", run: backward_delete_char });
    reg.register(Command { name: "set-mark-command", run: toggle_mark });
    reg.register(Command { name: "newline", run: newline });
    reg.register(Command { name: "kill-region", run: kill_region });
    reg.register(Command { name: "yank", run: yank });
    reg.register(Command { name: "scroll-up-command", run: scroll_up });
    reg.register(Command { name: "scroll-down-command", run: scroll_down });
    reg.register(Command { name: "kill-ring-save", run: copy_region });
    reg.register(Command { name: "execute-command", run: execute_command }); 
    reg.register(Command { name: "save-buffer", run: save_buffer });
    reg.register(Command { name: "find-file", run: find_file });
}
