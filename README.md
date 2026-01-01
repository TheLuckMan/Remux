# Remux

Remux is a re-designed, extensible UNIX text editor inspired by Emacs,
with a focus on ergonomics, extensibility, memory safety, and designed 
from scratch with modern constraints.

The project is in early alpha stage.
The API is unstable and may change.

## Features

- Extensibility-first design using Lua
- Emacs-style command execution (`M-x`)
- Minibuffer-driven workflow
- Clean-slate architecture without legacy constraints
- Keyboard-centric and ergonomic interaction model
- Terminal UI built on `ratatui` / `crossterm`

## Architecture

- `core`   — editor engine (buffers, core commands, minibuffer)
- `config` — Lua configuration and hooks
- `tui`    — terminal frontend
- `gui`    — graphical frontend (planned)
- `lua`    — scripting / plugins (planned)

## Development

Run in development mode:

```sh
cargo run -p remux-tui
```


Build release version:

```sh
cargo build --release
```

Run:

```sh
./target/release/remux
./target/release/remux test.txt
```

See init.lua for available commands, key bindings,
and default configuration.

## Name

The name "Remux" follows traditional UNIX naming conventions and is
inspired by the historical origins of UNIX (MULTICS → UNICS → UNIX),
as well as Emacs naming patterns.


License: MIT
