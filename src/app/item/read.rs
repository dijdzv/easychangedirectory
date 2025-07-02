use std::fs;
use std::path::Path;

use crate::app::{Item, ItemInfo};

use super::_item::{ItemPath, ItemSymlink};

pub fn read_items<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<ItemInfo>> {
  let mut items = match fs::read_dir(&path) {
    Ok(read_dir) => read_dir
      .filter_map(|entry| {
        let entry = entry.ok()?;
        let filepath = entry.path();
        let path = if filepath.is_file() && filepath.is_symlink() {
          ItemPath::Symlink(ItemSymlink::File(filepath))
        } else if filepath.is_dir() && filepath.is_symlink() {
          ItemPath::Symlink(ItemSymlink::Dir(filepath))
        } else if filepath.is_file() {
          ItemPath::File(filepath)
        } else if filepath.is_dir() {
          ItemPath::Dir(filepath)
        } else {
          ItemPath::Unknown(filepath)
        };
        Some(ItemInfo { item: Item::Path(path), index: Some(0) })
      })
      .collect::<Vec<_>>(),
    _ => {
      return Ok(vec![ItemInfo::default()]);
    }
  };

  items.sort_by_key(|item| item.get_path());
  Ok(
    items
      .iter_mut()
      .enumerate()
      .map(|(i, item)| {
        item.index = Some(i);
        item.clone()
      })
      .collect(),
  )
}
