use std::io::Stdout;

use crossterm::{
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::Result;

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui {
    pub fn setup() -> Result<Self> {
        let mut stdout = std::io::stdout();

        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear()?;

        Ok(Self { terminal })
    }

    pub fn exec(&self) -> Result<()> {
        // TODO
        Ok(())
    }
}
