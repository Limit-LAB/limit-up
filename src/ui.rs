use std::io::Stdout;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Spans, Text},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};

use crate::Result;

static LOGO: &'static str = r#"|      _)            _)  |
 |       |  __ `__ \   |  __|
 |       |  |   |   |  |  |  
 _____| _| _|  _|  _| _| \__|"#;

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    step: usize,
}

impl Ui {
    pub fn setup() -> Result<Self> {
        let mut stdout = std::io::stdout();

        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear()?;

        Ok(Self { terminal, step: 0 })
    }

    pub fn exec(&mut self) -> Result<()> {
        loop {
            self.paint()?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Quit limit up
                    KeyCode::Char('q') => break,
                    // Previous step
                    KeyCode::Left if self.step > 0 => self.step -= 1,
                    // Next step for test
                    KeyCode::Right => self.step += 1,
                    // Press any key on the welcome page to continue
                    _ if self.step == 0 => self.step = 1,
                    // Other undefined actions
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn paint(&mut self) -> Result<()> {
        let welcome = |f: &mut Frame<_>| {
            let mut title = Text::styled(LOGO, Style::default().fg(Color::LightCyan));
            title.extend(
                Text::from("\nWelcome to Limit up\nPress any key to continue except <Q> :)")
                    .into_iter(),
            );

            let width = title.width();
            let height = title.height();

            f.render_widget(
                Paragraph::new(title).alignment(Alignment::Center),
                Rect {
                    x: f.size().width / 2 - (width / 2) as u16,
                    y: f.size().height / 2 - (height / 2) as u16,
                    width: width as u16,
                    height: height as u16,
                },
            );
        };

        self.terminal.draw(|f| {
            let titles = ["Welcome", "Install Requirements", "Tab3"]
                .iter()
                .cloned()
                .map(Spans::from)
                .collect();

            let tab = Tabs::new(titles)
                .block(Block::default().title(" Limit Up ").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .select(self.step)
                .highlight_style(Style::default().fg(Color::Cyan))
                .divider(">");

            f.render_widget(tab, f.size());

            // Render tip
            let tip = Spans::from(format!(
                "{}Press <Q> to exit.",
                (self.step > 0)
                    .then_some("Press ←  to previous step; ")
                    .unwrap_or_default()
            ));
            let width = tip.width() as u16;
            f.render_widget(
                Block::default().title(tip),
                Rect {
                    x: 2,
                    y: f.size().height - 2,
                    width,
                    height: 1,
                },
            );

            match self.step {
                0 => welcome(f),
                _ => {}
            }
        })?;

        Ok(())
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }
}
