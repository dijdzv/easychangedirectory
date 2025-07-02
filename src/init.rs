use std::env::temp_dir;

use handlebars::Handlebars;
use serde_json::json;

use crate::shell::Shell;

pub fn init(shell: &Shell) -> anyhow::Result<()> {
  let shellscript = Handlebars::new()
    .render_template(shell.get_template(), &json!({ "temp_path": temp_dir().join("_easychangedirectory.txt") }))?;

  println!("{shellscript}");

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_init_bash() {
    let result = init(&Shell::Bash);
    assert!(result.is_ok());
  }

  #[test]
  fn test_init_fish() {
    let result = init(&Shell::Fish);
    assert!(result.is_ok());
  }

  #[test]
  fn test_init_zsh() {
    let result = init(&Shell::Zsh);
    assert!(result.is_ok());
  }

  #[test]
  fn test_init_powershell() {
    let result = init(&Shell::Powershell);
    assert!(result.is_ok());
  }
}
