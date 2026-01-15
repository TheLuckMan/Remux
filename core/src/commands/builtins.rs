use std::sync::Arc;
use crate::command::{Command, CommandRegistry, CommandContext, CommandArg, Interactive};
use crate::minibuffer::MiniBufferMode;
use crate::editor::editor::{InputMode, LineWrapMode, PrefixState};
use crate::buffer::Motion;

fn digit_argument(ctx: CommandContext, digit: i32) {
    let ed = ctx.editor;

    ed.prefix = match ed.prefix {
        PrefixState::None => PrefixState::Digits(digit),           
        PrefixState::Digits(v) => PrefixState::Digits(v * 10 + digit), 
        PrefixState::Universal(_) => PrefixState::Digits(digit),  
    };

    let shown = match ed.prefix {
        PrefixState::Digits(v) | PrefixState::Universal(v) => v,
        PrefixState::None => 0,
    };

    ed.minibuffer.activate(
        &format!("C-u {}", shown),
        MiniBufferMode::Message { ttl: 2 },
    );
}

fn universal_argument(ctx: CommandContext) {
    let ed = ctx.editor;

    ed.prefix = match ed.prefix {
        PrefixState::None => PrefixState::Universal(4),  // first C-u → 4
        PrefixState::Universal(v) => PrefixState::Universal(v * 4),
        PrefixState::Digits(v) => PrefixState::Digits(v * 4), 
    };


    let shown = match ed.prefix {
        PrefixState::Digits(v) | PrefixState::Universal(v) => v,
        PrefixState::None => 0,
    };

    ed.minibuffer.activate(
        &format!("C-u {}", shown),
        MiniBufferMode::Message { ttl: 2 },
    );
}

fn run_kill<F>(ctx: CommandContext, f: F)
where
    F: FnOnce(&mut crate::buffer::Buffer) -> Option<String>
{
    let ed = ctx.editor;

    match f(&mut ed.buffer) {
        Some(text) => {
            ed.push_kill(text);
            ed.ensure_cursor_visible();
        }
        None => ed.minibuffer.message("Nothing to kill"),
    }
}


fn move_cursor_command(ctx: CommandContext, motion: Motion) {
    let n = match ctx.arg {
        CommandArg::Int(v) => v.max(1) as usize,
        _ => 1,
    };

    for _ in 0..n {
        ctx.editor.buffer.move_cursor(motion);
    }
}

fn keyboard_quit(ctx: CommandContext) {
    if ctx.editor.minibuffer.is_active() {
        ctx.editor.minibuffer.deactivate(); 
        ctx.editor.mode = InputMode::Normal;
        ctx.editor.minibuffer.message("Quit");
    } else {
	ctx.editor.buffer.clear_mark();
	// Cancel search
    }
}

pub fn register_builtins(reg: &mut CommandRegistry) {
    // ===============================
    // Cursor movement commands
    // ===============================
    reg.register(Arc::new(Command { name: "move-left", interactive: Interactive::None, run: |ctx| move_cursor_command(ctx, Motion::Left) }));
    reg.register(Arc::new(Command { name: "move-right", interactive: Interactive::None, run: |ctx| move_cursor_command(ctx, Motion::Right) }));
    reg.register(Arc::new(Command { name: "move-up", interactive: Interactive::None, run: |ctx| move_cursor_command(ctx, Motion::Up) }));
    reg.register(Arc::new(Command { name: "move-down", interactive: Interactive::None, run: |ctx| move_cursor_command(ctx, Motion::Down) }));
    reg.register(Arc::new(Command { name: "move-beginning-of-line", interactive: Interactive::None, run: |ctx| move_cursor_command(ctx, Motion::Bol) }));
    reg.register(Arc::new(Command { name: "move-end-of-line", interactive: Interactive::None, run: |ctx| move_cursor_command(ctx, Motion::Eol) }));
    reg.register(Arc::new(Command { name: "move-beginning-of-buffer", interactive: Interactive::None, run: |ctx| move_cursor_command(ctx, Motion::BufferStart) }));
    reg.register(Arc::new(Command { name: "move-end-of-buffer", interactive: Interactive::None, run: |ctx| move_cursor_command(ctx, Motion::BufferEnd) }));
    reg.register(Arc::new(Command { name: "move-word-left", interactive: Interactive::None, run: |ctx| move_cursor_command(ctx, Motion::WordLeft) }));
    reg.register(Arc::new(Command { name: "move-word-right", interactive: Interactive::None, run: |ctx| move_cursor_command(ctx, Motion::WordRight) }));

    // ===============================
    // Undo, quitting, basic editing
    // ===============================
    reg.register(Arc::new(Command { name: "undo", interactive: Interactive::None, run: |ctx| { ctx.editor.buffer.undo(); } }));
    reg.register(Arc::new(Command { name: "keyboard-quit", interactive: Interactive::None, run: keyboard_quit }));
    reg.register(Arc::new(Command { name: "kill-remux", interactive: Interactive::None, run: |ctx| { ctx.editor.should_quit = true; } }));
    reg.register(Arc::new(Command { name: "delete-char", interactive: Interactive::None, run: |ctx| { ctx.editor.buffer.delete(Motion::Right); ctx.editor.ensure_cursor_visible(); } }));
    reg.register(Arc::new(Command { name: "backward-delete-char", interactive: Interactive::None, run: |ctx| { ctx.editor.buffer.delete(Motion::Left); ctx.editor.ensure_cursor_visible(); } }));
    reg.register(Arc::new(Command { name: "set-mark-command", interactive: Interactive::None, run: |ctx| { ctx.editor.buffer.toggle_mark(); } }));
    reg.register(Arc::new(Command { name: "newline", interactive: Interactive::None, run: |ctx| { ctx.editor.buffer.insert_newline(); ctx.editor.ensure_cursor_visible(); } }));

    // ===============================
    // Killing/copying/yanking
    // ===============================
    reg.register(Arc::new(Command {
	name: "kill-word",
	interactive: Interactive::None,
	run: |ctx| run_kill(ctx, |b| b.kill_word()),
    }));

     reg.register(Arc::new(Command {
	name: "kill-backward-word",
	interactive: Interactive::None,
	run: |ctx| run_kill(ctx, |b| b.kill_backward_word()),
    }));

     reg.register(Arc::new(Command {
	name: "kill-sentence",
	interactive: Interactive::None,
	run: |ctx| run_kill(ctx, |b| b.kill_sentence()),
    }));

     reg.register(Arc::new(Command {
	name: "kill-region",
	interactive: Interactive::None,
	run: |ctx| run_kill(ctx, |b| b.kill_region()),
    }));


    reg.register(Arc::new(Command {
	name: "kill-ring-save",
	interactive: Interactive::None,
	run: |ctx| {
            if let Some(text) = ctx.editor.buffer.copy_region() {
		let len = text.chars().count();
		ctx.editor.kill_buffer = Some(text);
		ctx.editor.minibuffer.message(&format!("Copied {} chars", len));
		ctx.editor.buffer.clear_mark();
            } else {
		ctx.editor.minibuffer.message("No active region");
            }
	}
    }));



    reg.register(Arc::new(Command { name: "yank", interactive: Interactive::None, run: |ctx| {
	if let Some(text) = ctx.editor.kill_buffer.clone() {
	    ctx.editor.buffer.yank(&text);
	    ctx.editor.minibuffer.message("Yanked");
	    ctx.editor.ensure_cursor_visible();
	} else {
	    ctx.editor.minibuffer.message("Kill buffer empty");
	}
    }}));

    // ===============================
    // File-related commands
    // ===============================
    reg.register(Arc::new(Command {
	name: "save-buffer",
	interactive: Interactive::None,
	run: |ctx| {
	    if ctx.editor.buffer.file_path.is_some() {
		if ctx.editor.buffer.save().is_ok() {
		    ctx.editor.minibuffer.message("Buffer saved!");
		} else {
		    ctx.editor.minibuffer.message("Save failed");
		}
	    } else {
		ctx.editor.mode = InputMode::MiniBuffer;
		ctx.editor.pending_command = Some("save-buffer".to_string());
		ctx.editor.minibuffer.activate("Save buffer as: ", MiniBufferMode::SaveBuffer);
	    }
	},
    }));

    reg.register(Arc::new(Command {
	name: "save-buffer-as",
	interactive: Interactive::Str { prompt: "Save buffer as: " },
	run: |ctx| {
	    ctx.editor.mode = InputMode::MiniBuffer;
	    ctx.editor.minibuffer.activate("Save buffer as: ", MiniBufferMode::SaveBuffer);
	},
    }));


    reg.register(Arc::new(Command {
	name: "execute-command",
	interactive: Interactive::Str { prompt: "M-x " },
	run: |ctx| {
	    // just activate minibuffer; real command name comes from user input
	    ctx.editor.mode = InputMode::MiniBuffer;
	    ctx.editor.minibuffer.activate("M-x ", MiniBufferMode::Command);
	    // pending_string_arg_command stays None: we will use minibuffer input as command name
	},
    }));

    reg.register(Arc::new(Command {
	name: "find-file",
	interactive: Interactive::Str { prompt: "Find file: " },
	run: |ctx| {
	    if let CommandArg::Str(path) = ctx.arg {
		let _ = ctx.editor.buffer.open_file(path.into());
	    }
	}
    }));


    // ===============================
    // Toggle features
    // ===============================
    reg.register(Arc::new(Command { name: "toggle-line-wrap", interactive: Interactive::None, run: |ctx| {
	ctx.editor.wrap_mode = match ctx.editor.wrap_mode {
	    LineWrapMode::Wrap => LineWrapMode::Truncate,
	    LineWrapMode::Truncate => LineWrapMode::Wrap,
	};
	ctx.editor.minibuffer.message("Toggled LineWrap Mode");
	ctx.editor.scroll_x = 0;
	ctx.editor.ensure_cursor_visible();
    }}));
    
    // ===============================
    // Digital arguments
    // ===============================
    reg.register(Arc::new(Command {
	name: "digit-argument-1",
	interactive: Interactive::None,
	run: |ctx| digit_argument(ctx, 1),
    }));
    reg.register(Arc::new(Command {
	name: "digit-argument-2",
	interactive: Interactive::None,
	run: |ctx| digit_argument(ctx, 2),
    }));
    reg.register(Arc::new(Command {
	name: "digit-argument-3",
	interactive: Interactive::None,
	run: |ctx| digit_argument(ctx, 3),
    }));
    reg.register(Arc::new(Command {
	name: "digit-argument-4",
	interactive: Interactive::None,
	run: |ctx| digit_argument(ctx, 4),
    }));
    reg.register(Arc::new(Command {
	name: "digit-argument-5",
	interactive: Interactive::None,
	run: |ctx| digit_argument(ctx, 5),
    }));
    reg.register(Arc::new(Command {
	name: "digit-argument-6",
	interactive: Interactive::None,
	run: |ctx| digit_argument(ctx, 6),
    }));
    reg.register(Arc::new(Command {
	name: "digit-argument-7",
	interactive: Interactive::None,
	run: |ctx| digit_argument(ctx, 7),
    }));
    reg.register(Arc::new(Command {
	name: "digit-argument-8",
	interactive: Interactive::None,
	run: |ctx| digit_argument(ctx, 8),
    }));
    reg.register(Arc::new(Command {
	name: "digit-argument-9",
	interactive: Interactive::None,
	run: |ctx| digit_argument(ctx, 9),
    }));
    reg.register(Arc::new(Command {
	name: "digit-argument-0",
	interactive: Interactive::None,
	run: |ctx| digit_argument(ctx, 0),
    }));
    reg.register(Arc::new(Command {
	name: "universal-argument",
	interactive: Interactive::None,
	run: |ctx| universal_argument(ctx),
    }));

    // ===============================
    // Scrolling commands
    // ===============================
    reg.register(Arc::new(Command { name: "scroll-up-command", interactive: Interactive::None, run: |ctx| ctx.editor.scroll_up() }));
    reg.register(Arc::new(Command { name: "scroll-down-command", interactive: Interactive::None, run: |ctx| ctx.editor.scroll_down() }));
    reg.register(Arc::new(Command { name: "scroll-left-command", interactive: Interactive::None, run: |ctx| ctx.editor.scroll_left() }));
    reg.register(Arc::new(Command { name: "scroll-right-command", interactive: Interactive::None, run: |ctx| ctx.editor.scroll_right() }));

}
