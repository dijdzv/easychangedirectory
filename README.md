# easychangedirectory (Installation is not yet available.)

> **Tools for easy cd**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE-MIT)
[![Rust](https://github.com/shsyss/easychangedirectory/actions/workflows/rust.yml/badge.svg)](https://github.com/shsyss/easychangedirectory/actions/workflows/rust.yml)

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).

[Features](#features) / [Usage](#usage) / [Support](#support) / [Installation](#installation) / [After this](#after-this)

## Features

- Can change paths visually
- The `cd` functionality can also be used as-is.

![demo](./assets/demo.gif)

## Usage

Command `ed`

| Key                          | Description                           |
| ---------------------------- | ------------------------------------- |
| `k` `↑`                      | Move up                               |
| `j` `↓`                      | Move down                             |
| `h` `←`                      | Move parent directory                 |
| `l` `→`                      | Move Child directory                  |
| `Home`                       | Move to top                           |
| `End`                        | Move to bottom                        |
| `PageUp`                     | Skip a little and move up             |
| `PageDown`                   | Skip a little and move down           |
| `Enter`                      | Change directory to current directory |
| `Backspace` `Esc` `Ctrl + c` | Exit and return to original directory |
| `Ctrl + s`                   | Search mode switch                    |

Please let us know if you have any key map requests. If it is traditional, we will add it immediately.

## Support

| Shell          |    Windows    | Linux (Ubuntu) |    Mac    |
| -------------- | :-----------: | :------------: | :-------: |
| **Bash**       | **&#128504;** | **&#128504;**  | **&#63;** |
| **Fish**       |   **&#63;**   | **&#128504;**  | **&#63;** |
| **Powershell** | **&#128504;** | **&#128504;**  | **&#63;** |
| **Zsh**        |   **&#63;**   |       -        | **&#63;** |

## Installation

### Install

not yet

### Register **_easychangedirectory_** in shell

<details>
<summary>Bash</summary>

Add to `~/.bashrc` (Change as necessary)

```
eval "$(easychangedirectory --init bash)"
```

Run `. ~/.bashrc` as needed

</details>

<details>
<summary>Fish</summary>

Add to `~/.config/fish/config.fish` (Change as necessary)

```
easychangedirectory --init fish | source
```

Run `. ~/.config/fish/config.fish` as needed

</details>

<details>
<summary>Powershell</summary>

Add to the file found by `echo $profile`

```
Invoke-Expression (& { (easychangedirectory --init powershell | Out-String) } )
```

Run `. /path/to/profile.ps1` as needed

</details>

<!-- <details>
<summary>Zsh</summary>

Add to `~/.zshrc` (Change as necessary)
```
eval "$(easychangedirectory --init zsh)"
```
Run `. ~/.zshrc` as needed
</details> -->

## After this

- Image Preview
- Execute command
- Zsh: Error `__vsc_command_output_start:3` is displayed at the second and subsequent `ed` executions
- Bug: Highlight shifted when moving left or right during search
- Bug: Search suggestions are displayed from the index prior to the search
- Bug: Skip move does not work properly
- Bug: If the file content has a path, only the file name is displayed
