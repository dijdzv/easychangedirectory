use easychangedirectory as ed;

use ed::error::PrintError;

fn main() {
  let cli = ed::cli();

  cli.match_options();

  let action = match ed::app() {
    Ok(action) => action,
    Err(e) => {
      e.eprintln();
      return;
    }
  };

  let action_path = action.execute();

  cli.prepare_cd(&action_path);

  if let Ok(config) = ed::Config::new() {
    if config.is_pwd() {
      println!("Now: {}", action_path.display());
    }
    if config.is_log() {
      if let Ok(log_path) = ed::Log::output_path() {
        println!("Log output location: {}", log_path.display());
      }
    }
  }
}
