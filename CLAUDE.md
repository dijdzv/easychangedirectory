## Rules

- language = Japanese
- character_code = UTF-8
- コメントやエラーメッセージは英語で書く

## Project Overview

**easychangedirectory** is a Rust-based CLI utility that provides visual, interactive directory navigation through a terminal user interface (TUI). Users type `ed` in their terminal to launch an interactive directory browser with vim-like keybindings.

## Tech Stack

- **Language**: Rust (Edition 2024)
- **CLI Framework**: clap v4 with derive features
- **TUI Library**: tui v0.19.0 with crossterm backend
- **Error Handling**: anyhow for error management
- **Configuration**: Environment variables via envy
- **Shell Integration**: handlebars templates for multi-shell support

## Essential Commands

```bash
# Development
cargo build --verbose          # Development build
cargo build --release         # Production build with LTO
cargo test --verbose          # Run all tests
cargo fmt --all --check      # Check formatting

# Debug
./debug.sh                    # Development debug script
```

## Code Architecture

### Core Structure
- **`src/main.rs`**: Entry point, orchestrates CLI and app execution
- **`src/cli.rs`**: Command-line argument parsing with clap
- **`src/shell.rs`**: Shell integration templates (Bash, Fish, Zsh, PowerShell)
- **`src/app/`**: TUI application logic with state management
- **`src/config/`**: Environment-based configuration system

### Key Patterns

1. **Shell Integration**: The binary generates shell functions that wrap the executable to enable `cd` functionality after directory selection
2. **TUI State Management**: Uses StatefulList pattern for managing directory/file listings
3. **Cross-Shell Support**: Template-based approach generates shell-specific integration code
4. **Environment Configuration**: Runtime behavior controlled via `_ED_*` environment variables

### Shell Integration Architecture

The tool works by generating shell functions that:
1. Call the Rust binary to get user's directory selection
2. Parse the output to extract the chosen directory
3. Execute `cd` to change to that directory in the current shell session

Templates in `src/shell.rs` handle the shell-specific implementations.

## Configuration

Key environment variables:
- `_ED_PWD`: Print current directory after execution
- `_ED_SET_BG`: Set black background
- `_ED_SHOW_INDEX`: Display index in directory listing
- `_ED_VIEW_FILE_CONTENTS`: Enable file content preview
- `_ED_LOG`: Enable logging to `~/.easychangedirectory/ed.log`

## Release Process

CI/CD builds for Windows, Linux, and macOS targets. Release builds use LTO optimization and single codegen unit for performance.