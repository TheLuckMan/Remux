# Remux Editor

## About 
A Re-designed (Or Rust), extensible UNIX text editor inspired by Emacs.
(
Emacs have 'CS' at end, I'm inspired UNIX name origins MULTICS -> UNICS -> UNIX
So therefore my editor ending by 'X' (^_^)
)


Remux - a minimalist memory-safety terminal text editor,
Inspired by Emacs and focused on extensibility.

The project is in the early stages of development (alpha).
The API may change since the development!

# Freatures 
- Editing text files
- Moving the cursor
- Mini-buffer in the style of Emacs
- Calling teams through 'M-x'
- ‘find-file’ – Opening files
- 'save-buffer' – Preserving the current buffer
- Lua configuration (keymap, commands)
- TUI based on 'ratatui'/'crossterm'

## Architecture

- core — editor engine (buffer, core-commands, minibuffer)
- tui  — terminal frontend
- gui  — graphical frontend (planned)
- lua  — scripting / plugins (planned)

  
## Versioning

Remux uses a custom semantic versioning scheme:

- **X** — project stage
  - 0 — alpha (unstable, experimental) (Code name: "Bootstrap")
  - 1 — beta (API mostly stable)
  - 2 — stable releases
- **Y** — feature releases
- **Z** — bugfix releases

Major releases (X) may have code names.


## Development

To build for develop: 

```sh
cargo run -p remux-tui
```
To build for Release:

```sh
cargo build --release
```

## Running:

```sh
./target/release/remux
./target/release/remux test.txt
```

To learn all available commands for bind() 
Lua and default configuration for Remux, check out - `init.lua` file


License: MIT
