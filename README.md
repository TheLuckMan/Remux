# Remux Editor

Remux is a re-designed, extensible UNIX text editor inspired by Emacs,
with a focus on ergonomics, extensibility, and memory safety.

The project is in early alpha stage.
The API is unstable and may change.

# Freatures 
- Mini-buffer in the style of Emacs
- Calling teams through 'M-x'
- Highly extensible via Lua configuration and plugins
- Designed from scratch with modern constraints
- TUI based on 'ratatui'/'crossterm'

## Architecture

- core - editor engine (buffer, core-commands, minibuffer)
- config - Lua, Hooks (Soon migrated into from core)
- tui  - terminal frontend
- gui  - graphical frontend (planned)

## Development

**Dependencies:**
To build Remux from source, the following dependencies are required:
- Rust-toolchain (Rustc, Cargo)
- pkg-config
- lua 5.4

**Debian/Ubuntu**
  ```sh
  sudo apt install rustc cargo lua5.4 liblua5.4-dev pkg-config
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
**OpenBSD**
  ```sh
  pkg_add rust lua-5.4.7
  ```
  
## Building

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

## Note
To learn all available commands for bind() 
Lua and default configuration for Remux, check out - `init.lua` file

**Please copy `init.lua` file to `~/.config/remux/` before running Remux!**

## Name

The name "Remux" follows traditional UNIX naming conventions and is
inspired by the historical origins of UNIX (MULTICS → UNICS → UNIX),
as well as Emacs naming patterns.


License: MIT
