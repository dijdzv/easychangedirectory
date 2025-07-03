use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  text::Span,
  widgets::{Block, Borders, List},
  widgets::{ListItem, ListState},
};

use super::{App, AppMode, Item, ItemInfo, ItemPath, item::ItemSymlink};
use crate::Config;

/* Color
- background: rgb(10, 10, 10)
- border: gray
- title: yellow
- dir: blue
- search: green
- file, content, none: gray
- symlink: cyan
- current-highlight: bold, underlined, bright
- parent-highlight: magenta
*/

struct MyStyle;

impl MyStyle {
  fn right_border<'a>() -> Block<'a> {
    Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(Color::Gray))
  }
  fn highlight_style() -> Style {
    Style::default().fg(Color::Magenta)
  }
}

pub fn ui(f: &mut Frame, app: &mut App) {
  // Overall style
  if app.config.is_set_bg() {
    f.render_widget(Block::default().style(Style::default().bg(Color::Rgb(10, 10, 10))), f.area());
  }

  // layout
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(10), Constraint::Max(100)])
    .split(f.area());

  // top----------------------------------------------------------
  let top_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(80), Constraint::Length(1)])
    .split(chunks[0]);

  // show wd
  f.render_widget(
    Block::default().title(Span::styled(app.generate_wd_str(), Style::default().fg(Color::Yellow))),
    top_chunks[0],
  );

  // search
  let item = ItemInfo { item: Item::Search(app.search.text.clone()), index: Some(0) };
  let search_items = vec![item];
  let search_items = set_items(&search_items, app.config);
  let search_text = List::new(search_items).highlight_symbol("> ");
  let mut state = ListState::default();
  if app.mode == AppMode::Normal {
    state.select(None);
  } else if app.mode == AppMode::Search {
    state.select(Some(0));
  }
  f.render_stateful_widget(search_text, top_chunks[1], &mut state);

  // bottom------------------------------------------------------
  let bottom_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage(20),
      Constraint::Percentage(20),
      Constraint::Max(100),
      Constraint::Percentage(30),
    ])
    .split(chunks[1]);

  // grandparent
  let grandparent_items = set_items(&app.grandparent_items.items, app.config);
  let grandparent_items =
    List::new(grandparent_items).block(MyStyle::right_border()).highlight_style(MyStyle::highlight_style());
  f.render_stateful_widget(grandparent_items, bottom_chunks[0], &mut app.grandparent_items.state);

  // parent
  let parent_items = set_items(&app.parent_items.items, app.config);
  let parent_items = List::new(parent_items).block(MyStyle::right_border()).highlight_style(MyStyle::highlight_style());
  f.render_stateful_widget(parent_items, bottom_chunks[1], &mut app.parent_items.state);

  // current
  let (items, state) = match app.judge_mode() {
    AppMode::Normal => (set_items(&app.items.items, app.config), &mut app.items.state),
    AppMode::Search => (set_items(&app.search.list, app.config), &mut app.search.state),
  };
  let items = List::new(items)
    .block(MyStyle::right_border())
    .highlight_style(Style::default().add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED))
    .highlight_symbol("> ");
  f.render_stateful_widget(items, bottom_chunks[2], state);

  // child
  let child_items = set_items(&app.child_items.items, app.config);
  let child_items = List::new(child_items).highlight_style(MyStyle::highlight_style());
  f.render_stateful_widget(child_items, bottom_chunks[3], &mut app.child_items.state);
}

fn set_items(items: &[ItemInfo], config: Config) -> Vec<ListItem> {
  items
    .iter()
    .filter_map(|item| {
      let style = match item.item {
        Item::Content(_) | Item::None | Item::Path(ItemPath::File(_)) => Style::default().fg(Color::Gray),
        Item::Path(ItemPath::Dir(_)) => Style::default().fg(Color::Blue),
        Item::Search(_) => Style::default().fg(Color::Green),
        Item::Path(ItemPath::Symlink(ItemSymlink::Dir(_))) => Style::default().fg(Color::Cyan),
        Item::Path(ItemPath::Symlink(ItemSymlink::File(_))) => Style::default().fg(Color::LightCyan),
        Item::Path(ItemPath::Unknown(_)) => Style::default().fg(Color::Red),
      };

      let mut text = if let Item::Search(text) = &item.item {
        text.into()
      } else if let Item::Content(text) = &item.item {
        text.into()
      } else {
        item.generate_filename()?
      };

      if config.is_show_index(items) {
        text = format!("{} {}", item.index.unwrap_or(0) + 1, text);
      }

      Some(ListItem::new(Span::styled(text, style)))
    })
    .collect()
}
