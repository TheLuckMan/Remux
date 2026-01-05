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
- ‘find-file’ - Opening files
- 'save-buffer' - Preserving the current buffer
- Lua configuration (keymap, commands)
- TUI based on 'ratatui'/'crossterm'

## Architecture

- core - editor engine (buffer, core-commands, minibuffer)
- config - Lua, Hooks (Soon migrated into from core)
- tui  - terminal frontend
- gui  - graphical frontend (planned)
- lua  - scripting / plugins (planned)

## Development

**Dependencies:**
To build Remux from source, the following dependencies are required:
- Rust-toolchain (Rustc, Cargo)
- pkg-config
- lua 5.4

**Debian/Ubuntu**
  ```sh
  sudo apt install rustc cargo lua5.4 liblua5.4 pkg-config
  ```
**Arch**
  ```sh
  sudo pacman -S rust lua pkgconf
  ```
**Fedora**
  ```sh
  sudo dnf install rust cargo lua pkg-config
  ```
**Gentoo**
  ```sh
  emerge dev-lang/rust dev-lang/lua pkgconf
  ```
**Nix**
  ```sh
  nix-shell -p rustc cargo lua pkg-config
  ```
**FreeBSD**
  ```sh
  pkg install rust lua5.4 pkgconf
  ```

To build for develop: 

```sh
cargo run -p remux-tui
```
To build for Release:

```sh
cargo build --release
```

Running:

```sh
./target/release/remux
./target/release/remux test.txt
```

To learn all available commands for bind() 
Lua and default configuration for Remux, check out - `init.lua` file




License: MIT
