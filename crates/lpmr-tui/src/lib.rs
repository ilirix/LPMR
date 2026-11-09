mod app;
mod theme;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{stdout, IsTerminal, Stdout};
use std::time::Duration;

pub fn run_tui() -> Result<()> {
    if !stdout().is_terminal() {
        anyhow::bail!("TUI requires an interactive terminal (run `lpmr tui` in SSH or a local console)");
    }
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = app::App::new();
    let result = loop_ui(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn loop_ui(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut app::App) -> Result<()> {
    loop {
        terminal.draw(|f| app::render(f, app))?;
        if event::poll(Duration::from_millis(120))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                {
                    break;
                }
                app.handle_key(key.code);
            }
        }
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
