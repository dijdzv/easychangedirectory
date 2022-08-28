use std::{
  env, io,
  path::{Path, PathBuf},
  vec,
};

use anyhow::bail;
use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use super::{Config, Item, ItemType, Search, State, StatefulList};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
  Normal,
  Search,
}

#[derive(Debug)]
pub struct App {
  pub mode: Mode,
  pub child_items: StatefulList,
  pub items: StatefulList,
  pub parent_items: StatefulList,
  pub grandparent_items: StatefulList,
  pub pwd: PathBuf,
  grandparent_path: PathBuf,
  pub search: Search,
  pub config: Config,
}

const JUMP: usize = 4;
impl App {
  fn generate_index<P: AsRef<Path>>(items: &[Item], path: P) -> usize {
    let generate_item = items.iter().enumerate().find(|(_, item)| item.get_path().unwrap() == path.as_ref());
    if let Some((i, _)) = generate_item {
      i
    } else {
      0
    }
  }
  fn generate_parent_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().parent().unwrap_or_else(|| Path::new("")).to_path_buf()
  }
  pub fn generate_pwd_str(&self) -> String {
    self.pwd.to_string_lossy().to_string()
  }
  fn get_child_index(&self) -> usize {
    self.child_items.state.selected().unwrap_or(0)
  }
  pub fn get_child_items(&self) -> Vec<Item> {
    self.child_items.items.clone()
  }
  fn get_current_index(&self) -> usize {
    self.items.state.selected().unwrap_or(0)
  }
  fn get_grandparent_index(&self) -> usize {
    self.grandparent_items.state.selected().unwrap_or(0)
  }
  pub fn get_grandparent_items(&self) -> Vec<Item> {
    self.grandparent_items.items.clone()
  }
  pub fn get_items(&self) -> Vec<Item> {
    self.items.items.clone()
  }
  fn get_parent_index(&self) -> usize {
    self.parent_items.state.selected().unwrap_or(0)
  }
  pub fn get_parent_items(&self) -> Vec<Item> {
    self.parent_items.items.clone()
  }
  fn get_search_index(&self) -> usize {
    self.search.state.selected().unwrap_or(0)
  }
  fn get_search_list(&self) -> Vec<Item> {
    self.search.list.clone()
  }
  /// If the working block is "content" `true`
  fn is_contents_in_working_block(&self) -> bool {
    let i = self.parent_items.selected();
    self.get_parent_items()[i].is_file()
  }
  fn is_empty_in_working_block(&self) -> bool {
    match self.judge_mode() {
      Mode::Normal => self.items.items.is_empty(),
      Mode::Search => self.search.list.is_empty(),
    }
  }
  pub fn judge_mode(&self) -> Mode {
    if self.search.text.is_empty() {
      Mode::Normal
    } else {
      Mode::Search
    }
  }
  pub fn make_items<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Item>> {
    Ok(if path.as_ref().to_string_lossy().is_empty() { vec![Item::default()] } else { super::read_items(path)? })
  }
  pub fn move_child(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let selected_item = match self.judge_mode() {
      Mode::Normal => self.items.items[self.items.selected()].clone(),
      Mode::Search => self.search.list[self.search.state.selected().unwrap()].clone(),
    };
    let new_pwd = if selected_item.is_dir() {
      selected_item.get_path().unwrap()
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
      (self.get_child_items().get(0).unwrap_or(&Item::default()).generate_child_items()?, 0)
    };

    let new_ci = None;
    let new_pi = match self.judge_mode() {
      Mode::Normal => Some(self.get_current_index()),
      Mode::Search => self.get_search_list()[self.get_search_index()].index,
    };
    let new_gi = self.get_parent_index();
    *self = Self {
      mode: self.mode,
      child_items: StatefulList::with_items_option(new_child_items, new_ci),
      items: StatefulList::with_items_select(self.get_child_items(), new_i),
      parent_items: StatefulList::with_items_option(self.get_items(), new_pi),
      grandparent_items: StatefulList::with_items_select(self.get_parent_items(), new_gi),
      pwd: new_pwd,
      grandparent_path: Self::generate_parent_path(&self.pwd),
      search: Search::new(),
      config: self.config,
    };
    Ok(())
  }
  pub fn move_content(&mut self, selected_item: Item) -> anyhow::Result<()> {
    let new_pi = match self.judge_mode() {
      Mode::Normal => Some(self.get_current_index()),
      Mode::Search => self.get_search_list()[self.get_search_index()].index,
    };
    let new_gi = self.get_parent_index();

    *self = Self {
      mode: self.mode,
      child_items: StatefulList::with_items(vec![Item::default()]),
      items: StatefulList::with_items(self.get_child_items()),
      parent_items: StatefulList::with_items_option(self.get_items(), new_pi),
      grandparent_items: StatefulList::with_items_select(self.get_parent_items(), new_gi),
      pwd: selected_item.get_path().unwrap(),
      grandparent_path: Self::generate_parent_path(&self.pwd),
      search: Search::new(),
      config: self.config,
    };
    Ok(())
  }
  pub fn move_end(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let last_i = match self.judge_mode() {
      Mode::Normal => self.items.items.len() - 1,
      Mode::Search => self.search.list.len() - 1,
    };
    match self.judge_mode() {
      Mode::Normal => self.items.select(last_i),
      Mode::Search => self.search.select(last_i),
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
      Mode::Normal => self.items.select(top_i),
      Mode::Search => self.search.select(top_i),
    }
    self.update_child_items(top_i)?;
    Ok(())
  }
  pub fn move_next(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let new_i = match self.judge_mode() {
      Mode::Normal => self.items.next(),
      Mode::Search => self.search.next(),
    };
    self.update_child_items(new_i)?;
    Ok(())
  }
  pub fn move_page_down(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let (last_i, old_i) = match self.judge_mode() {
      Mode::Normal => (self.items.items.len() - 1, self.get_current_index()),
      Mode::Search => (self.search.list.len() - 1, self.get_search_index()),
    };
    let new_i = if old_i > last_i - JUMP { last_i } else { old_i + JUMP };
    match self.judge_mode() {
      Mode::Normal => self.items.select(new_i),
      Mode::Search => self.search.select(new_i),
    }
    self.update_child_items(new_i)?;
    Ok(())
  }
  pub fn move_page_up(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let old_i = match self.judge_mode() {
      Mode::Normal => self.get_current_index(),
      Mode::Search => self.get_search_index(),
    };
    let new_i = if old_i < JUMP { 0 } else { old_i - JUMP };
    match self.judge_mode() {
      Mode::Normal => self.items.select(new_i),
      Mode::Search => self.search.select(new_i),
    };
    self.update_child_items(new_i)?;
    Ok(())
  }
  pub fn move_parent(&mut self) -> anyhow::Result<()> {
    let new_pwd = if let Some(pwd) = self.pwd.parent() {
      pwd.to_path_buf()
    } else {
      return Ok(());
    };

    let new_grandparent_path = Self::generate_parent_path(&self.grandparent_path);
    let new_grandparent_items = Self::make_items(&new_grandparent_path)?;

    let new_ci = if self.is_contents_in_working_block() {
      None
    } else {
      match self.judge_mode() {
        Mode::Normal => Some(self.get_current_index()),
        Mode::Search => {
          if let Some(item) = self.get_search_list().get(self.get_search_index()) {
            item.index
          } else {
            Some(self.get_current_index())
          }
        }
      }
    };
    let new_i = self.get_parent_index();
    let new_pi = self.get_grandparent_index();
    let new_gi = Self::generate_index(&new_grandparent_items, &self.grandparent_path);

    *self = Self {
      mode: self.mode,
      child_items: StatefulList::with_items_option(self.get_items(), new_ci),
      items: StatefulList::with_items_select(self.get_parent_items(), new_i),
      parent_items: StatefulList::with_items_select(self.get_grandparent_items(), new_pi),
      grandparent_items: StatefulList::with_items_select(new_grandparent_items, new_gi),
      pwd: new_pwd,
      grandparent_path: new_grandparent_path,
      search: Search::new(),
      config: self.config,
    };

    Ok(())
  }
  pub fn move_previous(&mut self) -> anyhow::Result<()> {
    if self.is_empty_in_working_block() {
      return Ok(());
    }

    let new_i = match self.judge_mode() {
      Mode::Normal => self.items.previous(),
      Mode::Search => self.search.previous(),
    };
    self.update_child_items(new_i)?;
    Ok(())
  }
  fn new() -> anyhow::Result<App> {
    let pwd = env::current_dir()?;
    let items = super::read_items(&pwd)?;

    // Initial selection is 0
    let child_path = if items[0].is_dir() { items[0].get_path().unwrap() } else { PathBuf::new() };
    let parent_path = Self::generate_parent_path(&pwd);
    let grandparent_path = Self::generate_parent_path(&parent_path);
    let parent_items = Self::make_items(&parent_path)?;
    let grandparent_items = Self::make_items(&grandparent_path)?;
    let pi = Self::generate_index(&parent_items, &pwd);
    let gi = Self::generate_index(&grandparent_items, &parent_path);

    let mut app = App {
      mode: Mode::Normal,
      child_items: StatefulList::with_items_option(Self::make_items(child_path)?, None),
      items: StatefulList::with_items(items),
      parent_items: StatefulList::with_items(parent_items),
      grandparent_items: StatefulList::with_items(grandparent_items),
      pwd,
      grandparent_path,
      search: Search::new(),
      config: Config::new()?,
    };

    app.parent_items.select(pi);
    app.grandparent_items.select(gi);

    Ok(app)
  }
  pub fn search_sort_to_vec(&self) -> Vec<Item> {
    self
      .items
      .items
      .iter()
      .filter_map(|item| -> Option<Item> {
        if let ItemType::Content(s) = &item.item {
          if s.contains(&self.search.text) {
            Some(item.clone())
          } else {
            None
          }
        } else if item.get_path()?.file_name()?.to_string_lossy().to_string().contains(&self.search.text) {
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
      Mode::Normal => self.get_items(),
      Mode::Search => self.get_search_list(),
    };

    self.child_items =
      StatefulList::with_items_option(items.get(index).unwrap_or(&Item::default()).generate_child_items()?, ci);
    if items[index].is_file() {
      self.child_items.unselect();
    }

    Ok(())
  }
  pub fn update_search_effect(&mut self) -> anyhow::Result<()> {
    self.search.list = self.search_sort_to_vec();

    let now_i = match self.judge_mode() {
      Mode::Normal => self.get_current_index(),
      Mode::Search => self.get_search_index(),
    };

    self.update_child_items(now_i)?;

    Ok(())
  }
}

pub fn app() -> anyhow::Result<PathBuf> {
  // setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let app = App::new()?;
  let path = match super::run(&mut terminal, app) {
    Ok(path) => path,
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

  Ok(path)
}