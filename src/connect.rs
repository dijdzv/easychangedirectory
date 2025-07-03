use std::{fs::File, io::Write, path::Path};

pub fn pipe_shell(path: &Path, temp_path: &str) -> anyhow::Result<()> {
  let mut f = File::create(temp_path)?;
  f.write_all(path.to_string_lossy().as_bytes())?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::path::PathBuf;

  #[test]
  fn test_pipe_shell() {
    let test_path = PathBuf::from("/tmp");
    let temp_path = "/tmp/test_pipe_shell";

    let result = pipe_shell(&test_path, temp_path);
    assert!(result.is_ok());

    let content = fs::read_to_string(temp_path).expect("Failed to read temp file in test");
    assert_eq!(content, "/tmp");

    // クリーンアップ
    let _ = fs::remove_file(temp_path);
  }
}
