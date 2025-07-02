use std::{
  fs,
  path::{Path, PathBuf},
};

use anyhow::Context;

use super::App;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
  Path(ItemPath),
  Content(String),
  Search(String),
  None,
}

impl Item {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self::Path(ItemPath::Dir(PathBuf::new()))
  }

  pub fn create_dir<P: AsRef<Path>>(path: P) -> Self {
    Self::Path(ItemPath::Dir(path.as_ref().into()))
  }

  pub fn is_dir(&self) -> bool {
    if let Item::Path(path) = self { path.is_dir() } else { false }
  }

  pub fn is_file(&self) -> bool {
    if let Item::Path(path) = self { path.is_file() } else { false }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemPath {
  Dir(PathBuf),
  File(PathBuf),
  Symlink(ItemSymlink),
  Unknown(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemSymlink {
  Dir(PathBuf),
  File(PathBuf),
}

impl ItemSymlink {
  fn get_path(&self) -> &PathBuf {
    match self {
      ItemSymlink::Dir(path) => path,
      ItemSymlink::File(path) => path,
    }
  }
}

impl ItemPath {
  fn get_path(&self) -> PathBuf {
    match self {
      ItemPath::Dir(path) => path,
      ItemPath::File(path) => path,
      ItemPath::Symlink(symlink) => symlink.get_path(),
      ItemPath::Unknown(path) => path,
    }
    .into()
  }

  fn read_link(&self) -> anyhow::Result<PathBuf> {
    Ok(
      match self {
        ItemPath::Dir(path) => path,
        ItemPath::File(path) => path,
        ItemPath::Symlink(symlink) => symlink.get_path(),
        ItemPath::Unknown(path) => path,
      }
      .read_link()?,
    )
  }

  fn is_dir(&self) -> bool {
    matches!(self, ItemPath::Dir(_) | ItemPath::Symlink(ItemSymlink::Dir(_)))
  }

  fn is_file(&self) -> bool {
    matches!(self, ItemPath::File(_) | ItemPath::Symlink(ItemSymlink::File(_)))
  }
}

#[derive(Debug, Clone)]
pub struct ItemInfo {
  pub item: Item,
  pub index: Option<usize>,
}

impl ItemInfo {
  pub fn default() -> Self {
    Self { item: Item::new(), index: None }
  }
  pub fn generate_child_items(&self) -> anyhow::Result<Vec<Self>> {
    if self.is_symlink() {
      if let Item::Path(path) = &self.item {
        return App::make_items(path.read_link()?);
      }
    }
    Ok(if self.is_dir() {
      App::make_items(self.get_path().unwrap())?
    } else if self.is_file() && self.can_read() {
      if let Ok(s) = fs::read_to_string(self.get_path().context("Non-string files are being read.")?) {
        s.lines().enumerate().map(|(i, s)| Self { item: Item::Content(s.to_string()), index: Some(i) }).collect()
      } else {
        vec![Self::default()]
      }
    } else {
      vec![Self::default()]
    })
  }
  pub fn generate_filename(&self) -> Option<String> {
    Some(self.get_path()?.file_name()?.to_string_lossy().into())
  }
  pub fn can_read(&self) -> bool {
    if let Item::Path(path) = &self.item { path.is_file() } else { false }
  }
  pub fn is_dir(&self) -> bool {
    self.item.is_dir()
  }
  pub fn is_file(&self) -> bool {
    self.item.is_file()
  }
  fn is_symlink(&self) -> bool {
    if let Some(p) = self.get_path() { p.is_symlink() } else { false }
  }
  pub fn get_path(&self) -> Option<PathBuf> {
    if let Item::Path(path) = &self.item { Some(path.get_path()) } else { None }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_item() {
    let item = ItemInfo { item: Item::create_dir("test"), index: None };
    assert!(item.is_dir());
    assert!(!item.is_file());
    assert!(!item.is_symlink());
    assert!(!item.can_read());
    assert_eq!(item.get_path(), Some(PathBuf::from("test")));
    assert_eq!(item.generate_filename(), Some("test".into()));
    let item = ItemInfo { item: Item::Content("test".into()), index: None };
    assert!(!item.can_read());
    assert!(!item.is_symlink());
    assert_eq!(item.get_path(), None);
  }
}
