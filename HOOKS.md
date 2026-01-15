# Hooks

Hooks are the main extension mechanism in **Remux**. They allow Lua code to react to editor actions and lifecycle events without modifying core logic.

A hook is identified by a **string name** and is associated with one or more Lua functions.

```lua
add_hook("hook-name", function(arg)
  -- your code here
end)
```

---

## General rules

* Hooks are **synchronous** and executed immediately when triggered
* Multiple hooks can be registered under the same name
* Hooks are executed in **registration order**
* Hook handlers must not panic; errors are ignored by the editor
* Hooks may receive **zero or one string argument**, depending on the hook

---

## Available hooks (v0.5.0)

### Lifecycle hooks

#### `after-init`

Called on every editor tick after initialization.

**Argument:** empty string

```lua
add_hook("after-init", function()
  -- background logic
end)
```

---

#### `after-init-once`

Called exactly **once**, right after the editor has finished initializing.

Useful for one-time setup.

**Argument:** empty string

```lua
add_hook("after-init-once", function()
  set_buffer_borders(false)
end)
```

---

### Command hooks

#### `before-command`

Called before a command is executed.

**Argument:** command name

```lua
add_hook("before-command", function(cmd)
  message("Running: " .. cmd)
end)
```

---

#### `after-command`

Called after a command has been executed.

**Argument:** command name

```lua
add_hook("after-command", function(cmd)
  message("Executed: " .. cmd)
end)
```

---

### Buffer hooks

#### `buffer-loaded`

Called after a buffer has been successfully loaded from disk.

**Argument:** file path

```lua
add_hook("buffer-loaded", function(path)
  message("Opened: " .. path)
end)
```

---

#### `buffer-saved`

Called after a buffer has been successfully saved.

**Argument:** file path

```lua
add_hook("buffer-saved", function(path)
  message("Saved: " .. path)
end)
```

---

#### `buffer-changed`

Called whenever the buffer contents change.

**Argument:** empty string

```lua
add_hook("buffer-changed", function()
  -- react to edits
end)
```

---

### Cursor & input hooks

#### `cursor-moved`

Called when the cursor position changes.

**Argument:** empty string

```lua
add_hook("cursor-moved", function()
  -- update UI state
end)
```

---

#### `before-insert-char`

Called before a character is inserted into the buffer.

**Argument:** empty string

---

#### `after-insert-char`

Called after a character has been inserted into the buffer.

**Argument:** empty string

```lua
add_hook("after-insert-char", function()
  -- auto-pair, formatting, etc
end)
```

---

### Mode hooks

#### `mode-changed`

Called when the editor input mode changes (e.g. Normal ↔ Minibuffer).

**Argument:** mode name (string)

```lua
add_hook("mode-changed", function(mode)
  message("Mode: " .. mode)
end)
```

---

## Notes

* Hooks are intentionally simple and string-based
* This design keeps the Lua API stable while allowing internal refactors
* More hooks may be added in future releases

---

## Recommended usage patterns

* Use `after-init-once` for configuration
* Use `before-command` / `after-command` for logging and metrics
* Use `buffer-changed` sparingly (it can be frequent)
* Avoid heavy work in `after-init`

---

*End of hooks documentation*
