use serde::Deserialize;

use crate::app::{Item, ItemInfo};

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Config {
  _ed_pwd: Option<u8>,
  _ed_set_bg: Option<u8>,
  _ed_show_index: Option<u8>,
  _ed_view_file_contents: Option<u8>,
  _ed_log: Option<u8>,
}

impl Config {
  pub fn new() -> anyhow::Result<Self> {
    Ok(envy::from_env::<Self>()?)
  }

  pub fn is_pwd(&self) -> bool {
    self._ed_pwd.eq(&Some(1))
  }
  pub fn is_show_index(&self, items: &[ItemInfo]) -> bool {
    self._ed_show_index.eq(&Some(1)) && !items.is_empty() && !matches!(items[0].item, Item::Search(_))
  }
  pub fn is_view_file_contents(&self) -> bool {
    self._ed_view_file_contents.eq(&Some(1))
  }
  pub fn is_set_bg(&self) -> bool {
    self._ed_set_bg.eq(&Some(1))
  }
  pub fn is_log(&self) -> bool {
    self._ed_log.eq(&Some(1))
  }

  pub fn show_all(&self) {
    println!("_ED_PWD = {}", self._ed_pwd.map(|u| u.to_string()).unwrap_or_default());
    println!("_ED_SET_BG = {}", self._ed_set_bg.map(|u| u.to_string()).unwrap_or_default());
    println!("_ED_SHOW_INDEX = {}", self._ed_show_index.map(|u| u.to_string()).unwrap_or_default());
    println!("_ED_VIEW_FILE_CONTENTS = {}", self._ed_view_file_contents.map(|u| u.to_string()).unwrap_or_default());
    println!("_ED_LOG = {}", self._ed_log.map(|u| u.to_string()).unwrap_or_default());
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn verify_config() {
    let config = Config {
      _ed_pwd: Some(1),
      _ed_set_bg: Some(1),
      _ed_show_index: Some(1),
      _ed_view_file_contents: Some(1),
      _ed_log: Some(1),
    };
    assert!(config.is_pwd());
    assert!(config.is_show_index(&[ItemInfo::default()]));
    assert!(config.is_view_file_contents());
    assert!(config.is_set_bg());
    assert!(config.is_log());
  }
}
