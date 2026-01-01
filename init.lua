-- Default Remux Configuration (Inspired Emacs)
-- Copy this to ~/.config/remux/init.lua

-- Binding Mod-key
-- Syntax:
-- bind_mod("<Mod-key>")
-- Options for Mod: alt

bind_mod("alt")

-- Keybinds
-- Syntax:
-- bind("mod", "<Char>", "<Remux Function>")
-- register is important! [ Character 'A' is not 'a'! ]
-- [ A = Shift+a ]

--- 1. Moving
bind("mod", "b", "move-left")
bind("mod", "f", "move-right")
bind("mod", "p", "move-up")
bind("mod", "n", "move-down")
bind("mod", "a", "move-beginning-of-line")
bind("mod", "e", "move-end-of-line")
bind("mod", "<", "move-beginning-of-buffer")
bind("mod", ">", "move-end-of-buffer")

--- 2. Scrolling
bind("mod", "V", "scroll-down-command")
bind("mod", "v", "scroll-up-command")


--- 3. Text edit 
bind("mod", "d", "backward-delete-char")
bind("mod", "D", "delete-char")
bind("mod", "l", "newline")
bind("mod", "u", "undo")


--- 4. Selecting text, Cut, Copy, Paste
bind("mod", "m", "set-mark-command")
bind("mod", "y", "yank")
bind("mod", "w", "kill-ring-save")
bind("mod", "W", "kill-region")

--- 5. Execute Remux Command and Kill Remux.
bind("mod", "Q", "kill-remux")
bind("mod", "x", "execute-command")

--[[
 Available Remux Commands for "execute-command" (mod+x):
 find-file | Open file
 save-buffer | Save file (buffer)
 kill-remux | Quit (Kill Remux)
 move-left
 move-right
 move-up
 move-down
 move-beginning-of-line
 move-end-of-line
 move-beginning-of-buffer
 move-end-of-buffer
 scroll-down-command
 scroll-up-command
 backward-delete-char
 delete-char
 newline
 undo
 set-mark-command
 yank
 kill-ring-save
 kill-region

 Available Hooks (Only for the testing!):

 Example:
 add_hook("after-command", function(cmd)
  message("Executed:", cmd)
end)

 Supported:
 "after-command", "before-command"
 Supported functions:
 message("<String>", 
--]]
