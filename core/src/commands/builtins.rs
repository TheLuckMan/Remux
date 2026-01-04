use crate::command::{Command, CommandRegistry, CommandContext};
use crate::minibuffer::MiniBufferMode;
use crate::editor::{InputMode, LineWrapMode};
use crate::buffer::Motion;

fn move_cursor_command(ctx: CommandContext, motion: Motion) {
    ctx.editor.buffer.move_cursor(motion);
    ctx.editor.ensure_cursor_visible();
}

fn keyboard_quit(ctx: CommandContext) {
    if ctx.editor.minibuffer.is_active() {
        ctx.editor.minibuffer.deactivate(); // сброс текста и режима
        ctx.editor.mode = InputMode::Normal; // возвращаем нормальный режим
        ctx.editor.minibuffer.message("Quit"); // можно кратко показать сообщение
    } else {
        // Если минибуфер не активен — можно ещё добавить поведение, например
        // сброс выделения, отмену поиска и т.д.
    }
}

pub fn register_builtins(reg: &mut CommandRegistry) {
    // Движения
    reg.register(Command { name: "move-left", run: |ctx| move_cursor_command(ctx, Motion::Left) });
    reg.register(Command { name: "move-right", run: |ctx| move_cursor_command(ctx, Motion::Right) });
    reg.register(Command { name: "move-up", run: |ctx| move_cursor_command(ctx, Motion::Up) });
    reg.register(Command { name: "move-down", run: |ctx| move_cursor_command(ctx, Motion::Down) });
    reg.register(Command { name: "move-beginning-of-line", run: |ctx| move_cursor_command(ctx, Motion::Bol) });
    reg.register(Command { name: "move-end-of-line", run: |ctx| move_cursor_command(ctx, Motion::Eol) });
    reg.register(Command { name: "move-beginning-of-buffer", run: |ctx| move_cursor_command(ctx, Motion::BufferStart) });
    reg.register(Command { name: "move-end-of-buffer", run: |ctx| move_cursor_command(ctx, Motion::BufferEnd) });
    reg.register(Command { name: "move-word-left", run: |ctx| move_cursor_command(ctx, Motion::WordLeft) });
    reg.register(Command { name: "move-word-right", run: |ctx| move_cursor_command(ctx, Motion::WordRight) });

    // Остальные команды
    reg.register(Command { name: "undo", run: |ctx| { ctx.editor.buffer.undo(); } });
    reg.register(Command { name: "keyboard-quit", run: keyboard_quit });
    reg.register(Command { name: "kill-remux", run: |ctx| { ctx.editor.should_quit = true; } });
    reg.register(Command { name: "delete-char", run: |ctx| { ctx.editor.buffer.delete_char(); ctx.editor.ensure_cursor_visible(); } });
    reg.register(Command { name: "backward-delete-char", run: |ctx| { ctx.editor.buffer.backward_delete_char(); ctx.editor.ensure_cursor_visible(); } });
    reg.register(Command { name: "set-mark-command", run: |ctx| { ctx.editor.buffer.toggle_mark(); } });
    reg.register(Command { name: "newline", run: |ctx| { ctx.editor.buffer.insert_newline(); ctx.editor.ensure_cursor_visible(); } });

    reg.register(Command { name: "kill-word", run: |ctx| {
    if let Some(killed) = ctx.editor.buffer.kill_word() {
	ctx.editor.kill_buffer = Some(killed);
	ctx.editor.ensure_cursor_visible();
    } else {
	ctx.editor.minibuffer.message("Nothing to kill");
    }
    }});

     reg.register(Command { name: "kill-backward-word", run: |ctx| {
    if let Some(killed) = ctx.editor.buffer.kill_backward_word() {
	ctx.editor.kill_buffer = Some(killed);
	ctx.editor.ensure_cursor_visible();
    } else {
	ctx.editor.minibuffer.message("Nothing to kill");
    }
     }});

     reg.register(Command { name: "kill-sentence", run: |ctx| {
    if let Some(killed) = ctx.editor.buffer.kill_sentence() {
	ctx.editor.kill_buffer = Some(killed);
	ctx.editor.ensure_cursor_visible();
    } else {
	ctx.editor.minibuffer.message("Nothing to kill");
    }
    }});
    
    reg.register(Command { name: "kill-region", run: |ctx| {
        if let Some(text) = ctx.editor.buffer.kill_region() {
            ctx.editor.kill_buffer = Some(text);
            ctx.editor.minibuffer.message("Killed region");
            ctx.editor.ensure_cursor_visible();
        } else {
            ctx.editor.minibuffer.message("No active region");
        }
    }});
    reg.register(Command { name: "kill-ring-save", run: |ctx| {
        if let Some(text) = ctx.editor.buffer.copy_region() {
            ctx.editor.kill_buffer = Some(text);
            ctx.editor.minibuffer.message("Copy region");
            ctx.editor.ensure_cursor_visible();
        } else {
            ctx.editor.minibuffer.message("No active region");
        }
    }});
    reg.register(Command { name: "yank", run: |ctx| {
        if let Some(text) = ctx.editor.kill_buffer.clone() {
            ctx.editor.buffer.yank(&text);
            ctx.editor.minibuffer.message("Yanked");
            ctx.editor.ensure_cursor_visible();
        } else {
            ctx.editor.minibuffer.message("Kill buffer empty");
        }
    }});
    reg.register(Command { name: "save-buffer", run: |ctx| {
        if ctx.editor.buffer.save().is_ok() {
            ctx.editor.minibuffer.activate("Buffer saved!", MiniBufferMode::Message { ttl: 2 });
        }
    }});
    reg.register(Command { name: "execute-command", run: |ctx| {
        ctx.editor.mode = InputMode::MiniBuffer;
        ctx.editor.minibuffer.activate("M-x ", MiniBufferMode::Command);
    }});
    reg.register(Command { name: "find-file", run: |ctx| {
        ctx.editor.mode = InputMode::MiniBuffer;
        ctx.editor.minibuffer.activate("Find file: ", MiniBufferMode::FindFile);
    }});
    reg.register(Command { name: "toggle-line-wrap", run: |ctx| {
        ctx.editor.wrap_mode = match ctx.editor.wrap_mode {
            LineWrapMode::Wrap => LineWrapMode::Truncate,
            LineWrapMode::Truncate => LineWrapMode::Wrap,
        };
        ctx.editor.minibuffer.message("Toggle LineWrap Mode");
        ctx.editor.scroll_x = 0;
        ctx.editor.ensure_cursor_visible();
    }});
    reg.register(Command { name: "scroll-up-command", run: |ctx| ctx.editor.scroll_up() });
    reg.register(Command { name: "scroll-down-command", run: |ctx| ctx.editor.scroll_down() });
}
