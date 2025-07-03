use std::fmt;

#[derive(Debug)]
pub enum AppError {
  Config(ConfigError),
  FileSystem(FileSystemError),
  Ui(UiError),
  Io(std::io::Error),
  Other(String),
}

#[derive(Debug)]
pub enum ConfigError {
  HomeDirectoryNotFound,
  LogDirectoryCreationFailed(std::io::Error),
  LogFileCreationFailed(std::io::Error),
  LogInitializationFailed(String),
}

#[derive(Debug)]
pub enum FileSystemError {
  PathNotFound(String),
  PermissionDenied(String),
  InvalidPath(String),
  DirectoryReadFailed(std::io::Error),
}

#[derive(Debug)]
pub enum UiError {
  NoItemSelected,
  InvalidSelection(usize),
  EmptyItemList,
}

impl fmt::Display for AppError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      AppError::Config(e) => write!(f, "設定エラー: {}", e),
      AppError::FileSystem(e) => write!(f, "ファイルシステムエラー: {}", e),
      AppError::Ui(e) => write!(f, "UI エラー: {}", e),
      AppError::Io(e) => write!(f, "I/O エラー: {}", e),
      AppError::Other(msg) => write!(f, "エラー: {}", msg),
    }
  }
}

impl fmt::Display for ConfigError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ConfigError::HomeDirectoryNotFound => write!(f, "ホームディレクトリが見つかりません"),
      ConfigError::LogDirectoryCreationFailed(e) => write!(f, "ログディレクトリの作成に失敗しました: {}", e),
      ConfigError::LogFileCreationFailed(e) => write!(f, "ログファイルの作成に失敗しました: {}", e),
      ConfigError::LogInitializationFailed(msg) => write!(f, "ログの初期化に失敗しました: {}", msg),
    }
  }
}

impl fmt::Display for FileSystemError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      FileSystemError::PathNotFound(path) => write!(f, "パス '{}' が見つかりません", path),
      FileSystemError::PermissionDenied(path) => write!(f, "パス '{}' へのアクセス権限がありません", path),
      FileSystemError::InvalidPath(path) => write!(f, "無効なパス: '{}'", path),
      FileSystemError::DirectoryReadFailed(e) => write!(f, "ディレクトリの読み取りに失敗しました: {}", e),
    }
  }
}

impl fmt::Display for UiError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      UiError::NoItemSelected => write!(f, "アイテムが選択されていません"),
      UiError::InvalidSelection(index) => write!(f, "無効な選択: インデックス {}", index),
      UiError::EmptyItemList => write!(f, "アイテムリストが空です"),
    }
  }
}

impl std::error::Error for AppError {}
impl std::error::Error for ConfigError {}
impl std::error::Error for FileSystemError {}
impl std::error::Error for UiError {}

impl From<std::io::Error> for AppError {
  fn from(error: std::io::Error) -> Self {
    AppError::Io(error)
  }
}

impl From<ConfigError> for AppError {
  fn from(error: ConfigError) -> Self {
    AppError::Config(error)
  }
}

impl From<FileSystemError> for AppError {
  fn from(error: FileSystemError) -> Self {
    AppError::FileSystem(error)
  }
}

impl From<UiError> for AppError {
  fn from(error: UiError) -> Self {
    AppError::Ui(error)
  }
}


pub trait PrintError {
  fn eprintln(&self);
}

impl PrintError for anyhow::Error {
  fn eprintln(&self) {
    eprintln!("\x1b[31mError:\x1b[m {self}");
  }
}

impl PrintError for AppError {
  fn eprintln(&self) {
    eprintln!("\x1b[31mError:\x1b[m {self}");
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow::anyhow;

  #[test]
  fn test_print_error() {
    let error = anyhow!("test error");
    error.eprintln();
  }

  #[test]
  fn test_app_error_display() {
    let config_error = AppError::Config(ConfigError::HomeDirectoryNotFound);
    assert_eq!(config_error.to_string(), "設定エラー: ホームディレクトリが見つかりません");

    let fs_error = AppError::FileSystem(FileSystemError::PathNotFound("/test".to_string()));
    assert_eq!(fs_error.to_string(), "ファイルシステムエラー: パス '/test' が見つかりません");

    let ui_error = AppError::Ui(UiError::NoItemSelected);
    assert_eq!(ui_error.to_string(), "UI エラー: アイテムが選択されていません");
  }

  #[test]
  fn test_error_conversions() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let app_error: AppError = io_error.into();
    assert!(matches!(app_error, AppError::Io(_)));

    let config_error = ConfigError::HomeDirectoryNotFound;
    let app_error: AppError = config_error.into();
    assert!(matches!(app_error, AppError::Config(_)));

    let fs_error = FileSystemError::PathNotFound("test".to_string());
    let app_error: AppError = fs_error.into();
    assert!(matches!(app_error, AppError::FileSystem(_)));

    let ui_error = UiError::NoItemSelected;
    let app_error: AppError = ui_error.into();
    assert!(matches!(app_error, AppError::Ui(_)));
  }

  #[test]
  fn test_app_error_to_anyhow() {
    let app_error = AppError::Other("test error".to_string());
    let anyhow_error = anyhow::Error::new(app_error);
    assert_eq!(anyhow_error.to_string(), "エラー: test error");
  }
}
