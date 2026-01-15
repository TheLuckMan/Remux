-- Default Remux Configuration (Inspired Emacs)
-- Copy this to ~/.config/remux/init.lua

-- Binding Mod-key
-- Syntax:
-- bind_mod(<number>,"<Mod-key>")
-- Options for Mod: alt | ctrl | super | shift (not recomended)

bind_mod(0, "ctrl")
bind_mod(1, "alt")
bind_mod(2, "ctrl+x")

-- Keybinds
-- Syntax:
-- bind("mod", "<Char>", "<Remux Function>")
-- register is important! [ Character 'A' is not 'a'! ]
-- [ A = Shift+a ]

--- 1. Moving
bind("mod0", "b", "move-left")
bind("mod0", "f", "move-right")
bind("mod0", "p", "move-up")
bind("mod0", "n", "move-down")
bind("mod0", "a", "move-beginning-of-line")
bind("mod0", "e", "move-end-of-line")
bind("mod2", "[", "move-beginning-of-buffer")
bind("mod2", "]", "move-end-of-buffer")
bind("mod1", "b", "move-word-left")
bind("mod1", "f", "move-word-right")

--- 2. Scrolling
bind("mod1", "V", "scroll-down-command")
bind("mod1", "v", "scroll-up-command")
-- bind("mod1", "B", "scroll-left-command")
-- bind("mod1", "b", "scroll-right-command")

--- 3. Text edit 
bind("mod0", "d", "backward-delete-char")
bind("mod1", "d", "delete-char")
bind("mod0", "l", "newline")
bind("mod0", "z", "undo")


--- 4. Selecting text, Cut, Copy, Paste, Killing
bind("mod0", " ", "set-mark-command")
bind("mod0", "y", "yank")
bind("mod1", "w", "kill-ring-save")
bind("mod0", "w", "kill-region")
bind("mod0", "k", "kill-word")
bind("mod0", "K", "kill-backward-word")
bind("mod1", "k", "kill-sentence")

--- 5. Digital arguments.
bind("mod1", "1", "digit-argument-1")
bind("mod1", "2", "digit-argument-2")
bind("mod1", "3", "digit-argument-3")
bind("mod1", "4", "digit-argument-4")
bind("mod1", "5", "digit-argument-5")
bind("mod1", "6", "digit-argument-6")
bind("mod1", "7", "digit-argument-7")
bind("mod1", "8", "digit-argument-8")
bind("mod1", "9", "digit-argument-9")


--- 6. Execute Remux Command and Kill Remux.
bind("mod0", "u", "universal-argument")
bind("mod2", "c", "kill-remux")
bind("mod0", "g", "keyboard-quit")

bind("mod1", "x", "execute-command")
bind("mod2", "f", "find-file")
bind("mod1", "T", "toggle-line-wrap")

-- Customization
--- Border (true | false)

add_hook("after-init", function(cmd)
  set_buffer_borders(false)
end)

--[[
 Also Available Remux Commands for "execute-command" (mod+x):
 find-file | Open file
 save-buffer | Save file (buffer)
 save-buffer-as | Save file as <Enter> Name
 kill-remux | Quit (Kill Remux)

 Also there is "universal-command" -
 C-u C-f - moves cursor 4 characters forward
 C-u C-u C-f - Moves cursor 4*4 characters forward
 C-u C-u C-u C-u C-u C-f - Moves cursor 4*4*4*4*4 characters forward
 M-5 C-u C-f - Moves Cursor 5*4 Characters forward
 M-3 C-u C-u C-f - Moves Cursor 3*4*4 Characters forward
 I will exetend this command in future updates
--]]
