use clap::{Parser, arg, command};

use crate::error::PrintError;
use crate::shell::Shell;

#[derive(Parser, Debug)]
#[command(author, version, about = "Tools for easy cd\nThe `cd` functionality can also be used as-is", long_about = None)]
#[command(override_usage = "ed\n
 -------------------------------------------------------------
|   Key         |    Description                              |
|---------------|---------------------------------------------|
| ↑ k           | Move up                                     |
| ↓ j           | Move down                                   |
| ← h           | Move parent directory                       |
| → l           | Move Child directory                        |
| Home          | Move to top                                 |
| End           | Move to bottom                              |
| PageUp        | Skip a little and move up                   |
| PageDown      | Skip a little and move down                 |
| Enter c ;     | Change directory to current directory       |
| Esc Ctrl+c q  | Exit and return to original directory       |
| Insert Ctrl+s | Search mode switch (Char key will not work) |
| Backspace     | Delete one character from the search string |
| Delete        | Delete all search strings                   |
| V             | Open vscode                                 |
| p             | Print the selected filepath                 |
 -------------------------------------------------------------")]
pub struct Cli {
  #[arg(short, hide(true))]
  temp_path: Option<String>,
  #[arg(long, value_enum, value_name = "SHELL", help = "Configure shell")]
  init: Option<Shell>,
  #[arg(long, help = "Show all environment variables")]
  env: bool,
}

impl Cli {
  pub fn match_options(&self) {
    self.match_init();
    self.match_env();
  }

  fn match_init(&self) {
    if let Some(shell) = &self.init {
      if let Err(e) = crate::init(shell) {
        e.eprintln();
      }
      std::process::exit(exitcode::OK);
    }
  }

  fn match_env(&self) {
    if self.env {
      match crate::Config::new() {
        Ok(c) => c.show_all(),
        Err(e) => e.eprintln(),
      };
      std::process::exit(exitcode::OK);
    }
  }

  pub fn prepare_cd(&self, cd_path: &std::path::Path) {
    if let Some(temp_path) = self.temp_path.as_ref() {
      if let Err(e) = crate::pipe_shell(cd_path, temp_path) {
        e.eprintln();
        std::process::exit(exitcode::OK);
      }
    }
  }
}

pub fn cli() -> Cli {
  Cli::parse()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
  }
}
