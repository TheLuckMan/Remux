# Default Remux Configuration
# Copy this to ~/.config/remux/init.lua

bind_mod("alt")

bind("mod", "b", "move-left")
bind("mod", "f", "move-right")
bind("mod", "p", "move-up")
bind("mod", "n", "move-down")
bind("mod", "a", "move-beginning-of-line")
bind("mod", "e", "move-end-of-line")
bind("mod", "<", "move-beginning-of-buffer")
bind("mod", ">", "move-end-of-buffer")

bind("mod", "d", "backward-delete-char")
bind("mod", "D", "delete-char")
bind("mod", "l", "newline")

bind("mod", "Q", "kill-remux")
bind("mod", "x", "execute-command")

# Available Commands for "execute-command":
# M-x find-file | Open file
# M-x save-buffer | Save file (buffer)
# M-x kill-remux | Quit (Kill Remux)
