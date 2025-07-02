pub trait PrintError {
  fn eprintln(&self);
}

impl PrintError for anyhow::Error {
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
    // このテストでは実際の出力をテストするのではなく、
    // eprintln!が呼ばれることを確認する
    error.eprintln();
  }
}
