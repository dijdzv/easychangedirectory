use std::{
  env, io, mem,
  path::{Path, PathBuf},
  vec,
};

use anyhow::bail;
use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture},
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use tui::{Terminal, backend::CrosstermBackend};

use super::{Item, ItemInfo, Search, State, StatefulList};
use crate::error::{AppError, FileSystemError, UiError};
use crate::{Config, action::Action};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
  Normal,
  Search,
}

#[derive(Debug)]
pub struct App {
  pub mode: AppMode,
  pub child_items: StatefulList,
  pub items: StatefulList,
  pub parent_items: StatefulList,
  pub grandparent_items: StatefulList,
  pub wd: PathBuf,
  grandparent_path: PathBuf,
  pub search: Search,
  pub config: Config,
}

const JUMP: usize = 4;
impl App {
  fn generate_index<P: AsRef<Path>>(items: &[ItemInfo], path: P) -> usize {
    let generate_item =
      items.iter().enumerate().find(|(_, item)| item.get_path().map_or(false, |p| p == path.as_ref()));
    if let Some((i, _)) = generate_item { i } else { 0 }
  }
  fn generate_parent_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().parent().unwrap_or_else(|| Path::new("")).into()
  }
  pub fn generate_wd_str(&self) -> String {
    self.wd.to_string_lossy().into()
  }
  fn get_child_index(&self) -> usize {
    self.child_items.state.selected().unwrap_or(0)
  }
  pub fn get_child_items(&self) -> Vec<ItemInfo> {
    self.child_items.items.clone()
  }
  fn get_current_index(&self) -> usize {
    self.items.state.selected().unwrap_or(0)
  }
  pub fn get_items(&self) -> Vec<ItemInfo> {
    self.items.items.clone()
  }
  pub fn get_parent_items(&self) -> Vec<ItemInfo> {
    self.parent_items.items.clone()
  }
  fn get_search_index(&self) -> usize {
    self.search.state.selected().unwrap_or(0)
  }
  fn get_search_list(&self) -> Vec<ItemInfo> {
    self.search.list.clone()
  }
  fn get_selected_item(&self) -> Result<ItemInfo, AppError> {
    match self.judge_mode() {
      AppMode::Normal => {
        let index = self.items.selected();
        self.items.items.get(index).ok_or_else(|| UiError::InvalidSelection(index).into()).map(|item| item.clone())
      }
      AppMode::Search => {
        let index = self.search.state.selected().unwrap_or(0);
        self.search.list.get(index).ok_or_else(|| UiError::InvalidSelection(index).into()).map(|item| item.clone())
      }
    }
  }
  pub fn get_selected_filepath(&self) -> Result<PathBuf, AppError> {
    let item = self.get_selected_item()?;
    item.get_path().ok_or_else(|| FileSystemError::InvalidPath("Selected item has no valid path".to_string()).into())
  }
  /// If the working block is "content" `true`
  fn is_contents_in_working_block(&self) -> bool {
    let i = self.parent_items.selected();
    self.get_parent_items()[i].is_file()
  }
  fn is_child_empty(&self) -> bool {
    self.child_items.items.is_empty()
  }
  fn is_empty_in_working_block(&self) -> bool {
    match self.judge_mode() {
      AppMode::Normal => self.items.items.is_empty(),
      AppMode::Search => self.search.list.is_empty(),
    }
  }
  pub fn judge_mode(&self) -> AppMode {
    if self.search.text.is_empty() { AppMode::Normal } else { AppMode::Search }
  }
  pub fn make_items<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<ItemInfo>> {
    Ok(if path.as_ref().to_string_lossy().is_empty() { vec![ItemInfo::default()] } else { super::read_items(path)? })
  }
  pub fn move_child(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() || self.is_child_empty() {
      return Ok(());
    }

    let selected_item = self.get_selected_item()?;
    let new_wd = if selected_item.is_dir() {
      selected_item
        .get_path()
        .ok_or_else(|| FileSystemError::InvalidPath("Directory item has no valid path".to_string()))?
    } else if selected_item.is_file() && self.config.is_view_file_contents() {
      self.move_content(selected_item)?;
      return Ok(());
    } else {
      return Ok(());
    };

    let selected_ci = self.get_child_index();

    // The index of `items` is set to the index of `child_items` if it is selected. If not, it is set to `0`.
    let (new_child_items, new_i) = if let Some(items) = self.get_child_items().get(selected_ci) {
      (items.generate_child_items()?, self.get_child_index())
    } else {
      (self.get_child_items().first().unwrap_or(&ItemInfo::default()).generate_child_items()?, 0)
    };

    let new_pi = match self.judge_mode() {
      AppMode::Normal => Some(self.get_current_index()),
      AppMode::Search => self.get_search_list()[self.get_search_index()].index,
    };

    let new_grandparent_path = Self::generate_parent_path(&self.wd);

    self.wd = new_wd;
    self.grandparent_path = new_grandparent_path;
    self.search = Search::new();
    self.grandparent_items = mem::replace(
      &mut self.parent_items,
      mem::replace(
        &mut self.items,
        mem::replace(&mut self.child_items, StatefulList::with_items_option(new_child_items, None)),
      ),
    );
    self.items.state.select(Some(new_i));
    self.parent_items.state.select(new_pi);

    Ok(())
  }
  pub fn move_content(&mut self, selected_item: ItemInfo) -> anyhow::Result<()> {
    let new_pi = match self.judge_mode() {
      AppMode::Normal => Some(self.get_current_index()),
      AppMode::Search => self.get_search_list()[self.get_search_index()].index,
    };
    let new_grandparent_path = Self::generate_parent_path(&self.wd);

    self.wd = selected_item
      .get_path()
      .ok_or_else(|| FileSystemError::InvalidPath("Content item has no valid path".to_string()))?;
    self.grandparent_path = new_grandparent_path;
    self.search = Search::new();
    self.grandparent_items = mem::replace(
      &mut self.parent_items,
      mem::replace(
        &mut self.items,
        mem::replace(&mut self.child_items, StatefulList::with_items(vec![ItemInfo::default()])),
      ),
    );
    self.items.state.select(Some(0));
    self.parent_items.state.select(new_pi);

    Ok(())
  }
  pub fn move_end(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let last_i = match self.judge_mode() {
      AppMode::Normal => self.items.items.len() - 1,
      AppMode::Search => self.search.list.len() - 1,
    };
    match self.judge_mode() {
      AppMode::Normal => self.items.select(last_i),
      AppMode::Search => self.search.select(last_i),
    };
    self.update_child_items(last_i)?;
    Ok(())
  }
  pub fn move_home(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let top_i = 0;
    match self.judge_mode() {
      AppMode::Normal => self.items.select(top_i),
      AppMode::Search => self.search.select(top_i),
    }
    self.update_child_items(top_i)?;
    Ok(())
  }
  pub fn move_next(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let new_i = match self.judge_mode() {
      AppMode::Normal => self.items.next(),
      AppMode::Search => self.search.next(),
    };
    self.update_child_items(new_i)?;
    Ok(())
  }
  pub fn move_page_down(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let (last_i, old_i) = match self.judge_mode() {
      AppMode::Normal => (self.items.items.len() - 1, self.get_current_index()),
      AppMode::Search => (self.search.list.len() - 1, self.get_search_index()),
    };
    let new_i = if <isize as std::convert::TryInto<usize>>::try_into(last_i as isize - JUMP as isize).is_ok()
      && old_i.le(&(last_i - JUMP))
    {
      old_i + JUMP
    } else {
      last_i
    };
    match self.judge_mode() {
      AppMode::Normal => self.items.select(new_i),
      AppMode::Search => self.search.select(new_i),
    }
    self.update_child_items(new_i)?;
    Ok(())
  }
  pub fn move_page_up(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let old_i = match self.judge_mode() {
      AppMode::Normal => self.get_current_index(),
      AppMode::Search => self.get_search_index(),
    };
    let new_i = old_i.saturating_sub(JUMP);
    match self.judge_mode() {
      AppMode::Normal => self.items.select(new_i),
      AppMode::Search => self.search.select(new_i),
    };
    self.update_child_items(new_i)?;
    Ok(())
  }
  pub fn move_parent(&mut self) -> anyhow::Result<()> {
    let new_wd = if let Some(wd) = self.wd.parent() {
      wd.into()
    } else {
      return Ok(());
    };

    let new_grandparent_path = Self::generate_parent_path(&self.grandparent_path);
    let new_grandparent_items = Self::make_items(&new_grandparent_path)?;

    let new_ci = if self.is_contents_in_working_block() {
      None
    } else {
      match self.judge_mode() {
        AppMode::Normal => Some(self.get_current_index()),
        AppMode::Search => {
          if let Some(item) = self.get_search_list().get(self.get_search_index()) {
            item.index
          } else {
            Some(self.get_current_index())
          }
        }
      }
    };
    let new_gi = Self::generate_index(&new_grandparent_items, &self.grandparent_path);

    self.wd = new_wd;
    self.grandparent_path = new_grandparent_path;
    self.search = Search::new();
    self.child_items = mem::replace(
      &mut self.items,
      mem::replace(
        &mut self.parent_items,
        mem::replace(&mut self.grandparent_items, StatefulList::with_items_select(new_grandparent_items, new_gi)),
      ),
    );
    self.child_items.state.select(new_ci);

    Ok(())
  }
  pub fn move_previous(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let new_i = match self.judge_mode() {
      AppMode::Normal => self.items.previous(),
      AppMode::Search => self.search.previous(),
    };
    self.update_child_items(new_i)?;
    Ok(())
  }
  fn new() -> anyhow::Result<App> {
    let wd = env::current_dir()?;
    let items = super::read_items(&wd)?;

    // Initial selection is 0
    let child_path = match items.first() {
      Some(item) => {
        if item.is_dir() {
          item.get_path().unwrap_or_else(|| PathBuf::new())
        } else {
          PathBuf::new()
        }
      }
      None => PathBuf::new(),
    };
    let parent_path = Self::generate_parent_path(&wd);
    let grandparent_path = Self::generate_parent_path(&parent_path);
    let parent_items = Self::make_items(&parent_path)?;
    let grandparent_items = Self::make_items(&grandparent_path)?;
    let pi = Self::generate_index(&parent_items, &wd);
    let gi = Self::generate_index(&grandparent_items, &parent_path);

    let mut app = App {
      mode: AppMode::Normal,
      child_items: StatefulList::with_items_option(Self::make_items(child_path)?, None),
      items: StatefulList::with_items(items),
      parent_items: StatefulList::with_items(parent_items),
      grandparent_items: StatefulList::with_items(grandparent_items),
      wd,
      grandparent_path,
      search: Search::new(),
      config: Config::new()?,
    };

    app.parent_items.select(pi);
    app.grandparent_items.select(gi);

    Ok(app)
  }
  pub fn search_sort_to_vec(&self) -> Vec<ItemInfo> {
    self
      .items
      .items
      .iter()
      .filter_map(|item| -> Option<ItemInfo> {
        if let Item::Content(s) = &item.item {
          if s.contains(&self.search.text) { Some(item.clone()) } else { None }
        } else if item
          .get_path()
          .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
          .map_or(false, |name| name.contains(&self.search.text))
        {
          Some(item.clone())
        } else {
          None
        }
      })
      .collect()
  }
  fn update_child_items(&mut self, index: usize) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      self.child_items = StatefulList::with_items_option(vec![], None);
      return Ok(());
    }

    let ci = self.child_items.state.selected();

    let items = match self.judge_mode() {
      AppMode::Normal => self.get_items(),
      AppMode::Search => self.get_search_list(),
    };

    self.child_items =
      StatefulList::with_items_option(items.get(index).unwrap_or(&ItemInfo::default()).generate_child_items()?, ci);
    if items[index].is_file() {
      self.child_items.unselect();
    }

    Ok(())
  }
  pub fn update_search_effect(&mut self) -> anyhow::Result<()> {
    self.search.list = self.search_sort_to_vec();

    let now_i = match self.judge_mode() {
      AppMode::Normal => self.get_current_index(),
      AppMode::Search => self.get_search_index(),
    };

    self.update_child_items(now_i)?;

    Ok(())
  }
}

pub fn app() -> anyhow::Result<Action> {
  // setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let app = App::new()?;
  let action = match super::run(&mut terminal, app) {
    Ok(action) => action,
    Err(e) => {
      // restore terminal
      disable_raw_mode()?;
      execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
      terminal.show_cursor()?;

      bail!(e)
    }
  };

  // restore terminal
  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
  terminal.show_cursor()?;

  Ok(action)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  #[test]
  fn test_app_mode() {
    assert_eq!(AppMode::Normal, AppMode::Normal);
    assert_ne!(AppMode::Normal, AppMode::Search);
  }

  #[test]
  fn test_make_items_empty_path() {
    let result = App::make_items("");
    assert!(result.is_ok());
    let items = result.expect("Failed to create items for empty path");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].item, Item::new());
  }

  #[test]
  fn test_generate_parent_path() {
    let path = PathBuf::from("/home/user/documents");
    let parent = App::generate_parent_path(&path);
    assert_eq!(parent, PathBuf::from("/home/user"));
  }

  #[test]
  fn test_generate_parent_path_root() {
    let path = PathBuf::from("/");
    let parent = App::generate_parent_path(&path);
    assert_eq!(parent, PathBuf::from(""));
  }
}
