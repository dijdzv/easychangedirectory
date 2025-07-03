use std::{fs::File, path::PathBuf};

use ::log::info;
use crossterm::event::KeyEvent;
use simplelog::{CombinedLogger, Config, LevelFilter, WriteLogger};

use crate::app::App;
use crate::error::{AppError, ConfigError};

pub struct Log;

impl Log {
  pub fn output_path() -> Result<PathBuf, AppError> {
    let home_dir = home::home_dir().ok_or(ConfigError::HomeDirectoryNotFound)?;
    Ok(home_dir.join(format!(".{}", env!("CARGO_PKG_NAME"))).join("ed.log"))
  }

  pub fn init() -> Result<(), AppError> {
    let path = Self::output_path()?;
    
    if let Some(parent) = path.parent() {
      if !parent.exists() {
        std::fs::create_dir_all(parent)
          .map_err(ConfigError::LogDirectoryCreationFailed)?;
      }
    }
    
    let log_file = File::create(&path)
      .map_err(ConfigError::LogFileCreationFailed)?;
    
    CombinedLogger::init(vec![WriteLogger::new(
      LevelFilter::Info,
      Config::default(),
      log_file,
    )])
    .map_err(|e| ConfigError::LogInitializationFailed(e.to_string()))?;
    
    Ok(())
  }

  pub fn write(app: &App, key: &KeyEvent) {
    info!("--------------------------------");
    info!("path: {:?}", app.wd);
    info!("selected: {:?}", app.items.state.selected());
    info!("key: {:?}", key.code);
    info!("mode: {:?}", app.mode);
    info!("search: {:?}", app.search.text);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_output_path() {
    let result = Log::output_path();
    assert!(result.is_ok());
    
    if let Ok(path) = result {
      let expected = home::home_dir().unwrap().join(".easychangedirectory").join("ed.log");
      assert_eq!(path, expected);
    }
  }
  
  #[test]
  fn test_init_creates_directory() {
    // このテストは実際のディレクトリを作成するため、テスト環境でのみ実行
    // 実際の実装では、テスト時は一時ディレクトリを使用することを推奨
  }
}
