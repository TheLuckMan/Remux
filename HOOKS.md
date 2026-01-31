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

## Available hooks (v0.7.0)

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

#### `before-exit`

Called once right before the editor exits.

**Argument:** empty string

```lua
add_hook("before-exit", function()
  message("Exiting Remux")
end)
```

or (Work in progress)

```lua
add_hook("before-exit", function()
  local file = current_buffer_path() or "<unnamed>"

  minibuffer_prompt(
    "Save file " .. file .. "? (y, n)",
    "confirm-save-and-exit")
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

#### `before-buffer-write`

Called before a buffer is written to disk.

**Argument:** file path (string)

```lua
add_hook("before-buffer-write", function(path)
  format_buffer()
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

**Argument:** `"x,y"`

```lua
add_hook("cursor-moved", function(pos)
  local x, y = pos:match("(%d+),(%d+)")
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

### Selection hooks

#### `selection-changed`

Called when the selection changes.

**Argument:** table

* `start` — `{ x, y }`
* `end` — `{ x, y }`
* OR `{ cleared = true }`

```lua
add_hook("selection-changed", function(sel)
  if sel.cleared then
    message("Selection cleared")
  else
    message(
      "Selection: "
      .. sel.start.x .. "," .. sel.start.y
      .. " -> "
      .. sel["end"].x .. "," .. sel["end"].y
    )
  end
end)
```

---

### Incremental search hooks

#### `isearch-started`

Called when incremental search starts.

**Argument:** empty string

```lua
add_hook("isearch-started", function()
  set_isearch_highlight(true)
end)
```

---

#### `isearch-end`

Called when incremental search ends.

**Argument:** empty string

```lua
add_hook("isearch-end", function()
  set_isearch_highlight(false)
end)
```

---

#### `isearch-update`

Called whenever the isearch query changes.

**Argument:** table with fields:

* `dir` — `"forward"` | `"backward"`
* `query` — current search string
* `found` — boolean
* `cursor` — table `{ x, y }`

```lua
add_hook("isearch-update", function(ev)
  if ev.found then
    message("Found: " .. ev.query)
  end
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
