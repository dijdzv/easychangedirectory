use std::process::Command;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use tui::{Terminal, backend::Backend};

use crate::{Log, action::Action};

use super::{App, AppMode};

pub fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> anyhow::Result<Action> {
  if app.config.is_log() {
    Log::init();
  }
  loop {
    terminal.draw(|f| super::ui(f, &mut app))?;
    if let Ok(Event::Key(key)) = event::read() {
      if key.kind == KeyEventKind::Release {
        continue;
      }
      if app.config.is_log() {
        Log::write(&app, &key);
      }
      match app.mode {
        AppMode::Normal => {
          match key.code {
            // finish
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => return Ok(Action::Keep),
            KeyCode::Char('q') => return Ok(Action::Keep),
            KeyCode::Esc => return Ok(Action::Keep),

            // change directory
            KeyCode::Char('c') => return Ok(Action::Change(app.wd)),
            KeyCode::Char(';') => return Ok(Action::Change(app.wd)),
            KeyCode::Enter => return Ok(Action::Change(app.wd)),

            // move
            KeyCode::Home => app.move_home()?,
            KeyCode::End => app.move_end()?,
            KeyCode::PageUp => app.move_page_up()?,
            KeyCode::PageDown => app.move_page_down()?,
            KeyCode::Char('j') => app.move_next()?,
            KeyCode::Down => app.move_next()?,
            KeyCode::Char('k') => app.move_previous()?,
            KeyCode::Up => app.move_previous()?,
            KeyCode::Char('h') => app.move_parent()?,
            KeyCode::Left => app.move_parent()?,
            KeyCode::Char('l') => app.move_child()?,
            KeyCode::Right => app.move_child()?,

            // search
            KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => app.mode = AppMode::Search,
            KeyCode::Insert => app.mode = AppMode::Search,
            KeyCode::Backspace => {
              app.search.text.pop();
              app.update_search_effect()?;
            }
            KeyCode::Delete => {
              app.search.text.clear();
              app.update_search_effect()?;
            }

            // Execute command
            KeyCode::Char('V') => {
              Command::new("code").arg(&app.wd).output()?;
            }
            // KeyCode::Char('L') => {
            //   Command::new("lapce").arg(&app.wd).output()?;
            // }

            // print selected filepath
            KeyCode::Char('p') => return Ok(Action::Print(app.get_selected_filepath())),

            _ => {}
          }
        }
        AppMode::Search => {
          match key.code {
            // finish
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => return Ok(Action::Keep),
            KeyCode::Esc => return Ok(Action::Keep),

            // change directory
            KeyCode::Enter => return Ok(Action::Change(app.wd)),

            // search
            KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => app.mode = AppMode::Normal,
            KeyCode::Insert => app.mode = AppMode::Normal,

            // input
            KeyCode::Char(c) => {
              app.search.text.push(c);
              app.update_search_effect()?;
            }
            KeyCode::Backspace => {
              app.search.text.pop();
              app.update_search_effect()?;
            }
            KeyCode::Delete => {
              app.search.text.clear();
              app.update_search_effect()?;
            }

            // move
            KeyCode::Home => app.move_home()?,
            KeyCode::End => app.move_end()?,
            KeyCode::PageUp => app.move_page_up()?,
            KeyCode::PageDown => app.move_page_down()?,
            KeyCode::Down => app.move_next()?,
            KeyCode::Up => app.move_previous()?,
            KeyCode::Left => app.move_parent()?,
            KeyCode::Right => app.move_child()?,

            _ => {}
          }
        }
      }
    }
  }
}
