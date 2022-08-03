use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{env, io, path::Path};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, Row, Table, Widget},
    widgets::{ListItem, ListState},
    Frame, Terminal,
};

use crate::items;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub struct App {
    items: StatefulList<String>,
    parent_items: Vec<String>,
    grandparent_items: Vec<String>,
}

impl App {
    fn new() -> anyhow::Result<App> {
        let pwd = env::current_dir()?;
        let items = items::read_dir(&pwd)?;
        let parent_items = items::read_dir(pwd.parent().unwrap())?;
        let grandparent_items = items::read_dir(pwd.parent().unwrap().parent().unwrap())?;
        Ok(App {
            items: StatefulList::with_items(items),
            parent_items,
            grandparent_items,
        })
    }
}

pub fn app() -> anyhow::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new()?;
    self::run(&mut terminal, app)?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> crossterm::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        if let Event::Key(key) = event::read()? {
            // 終了key
            // TODO BackspaceとEscの時は開始前に戻して、ctrl+cとEnterは現在のディレクトリに移動
            // TODO 中央に現在のディレクトリのファイルのリスト
            // TODO 左に一つ上ののディレクトリのファイルのリスト
            // TODO 右に選択しているフォルダ直下のファイルのリスト
            // TODO →l ←h
            match key.code {
                // finish
                KeyCode::Backspace => return Ok(()),
                KeyCode::Esc => return Ok(()),
                // TODO: change directory
                KeyCode::Enter => return Ok(()),
                // ctrl + c
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => return Ok(()),
                // move
                KeyCode::Char('k') => app.items.previous(),
                KeyCode::Char('j') => app.items.next(),
                KeyCode::Down => app.items.next(),
                KeyCode::Up => app.items.previous(),
                _ => {}
            }
        }
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Max(100),
            Constraint::Percentage(20),
        ])
        .split(f.size());

    // grandparent
    let items = set_items(&app.grandparent_items);
    let items = List::new(items).block(Block::default().borders(Borders::all()));
    f.render_widget(items, chunks[0]);

    // parent
    let items = set_items(&app.parent_items);
    let items = List::new(items).block(Block::default().borders(Borders::all()));
    f.render_widget(items, chunks[1]);

    // current
    let items = set_items(&app.items.items);
    let items = List::new(items)
        .block(Block::default().borders(Borders::all()))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(items, chunks[2], &mut app.items.state);
}

fn set_items(items: &[String]) -> Vec<ListItem> {
    items
        .iter()
        .map(|p| {
            let lines = if Path::new(p).is_dir() {
                vec![Spans::from(Span::styled(
                    p,
                    Style::default().fg(Color::Blue),
                ))]
            } else {
                vec![Spans::from(Span::styled(p, Style::default()))]
            };
            ListItem::new(lines)
        })
        .collect()
}
