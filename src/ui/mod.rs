mod_use::mod_use!(welcome, install);

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    io::Stdout,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::Rect,
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};

use crate::Result;

static LOGO: &'static str = r#"|      _)            _)  |
 |       |  __ `__ \   |  __|
 |       |  |   |   |  |  |  
 _____| _| _|  _|  _| _| \__|"#;

trait Page<B: Backend> {
    /// Draw page according to state
    fn paint(&self, f: &mut Frame<B>);
    /// Processing events and updating state
    fn process(&mut self, ui_state: &mut UiState);
}

type UiBackend = CrosstermBackend<Stdout>;

pub struct Ui {
    terminal: RefCell<Terminal<UiBackend>>,
    state: UiState,
    pages: Vec<Box<dyn Page<UiBackend>>>,
}

pub struct UiState {
    pub step: usize,
    pub event: Event,
    pub runnable: bool,
}

impl Ui {
    pub fn setup() -> Result<Self> {
        let mut stdout = std::io::stdout();

        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode()?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear()?;

        Ok(Self {
            terminal: RefCell::new(terminal),
            state: UiState {
                step: 0,
                event: Event::FocusGained,
                runnable: true,
            },
            pages: vec![Box::new(Welcome), Box::new(Install::new())],
        })
    }

    pub fn exec(mut self) -> Result<()> {
        while self.state.runnable {
            self.paint()?;
            self.state.borrow_mut().event = event::read()?;
            self.process()?;
        }

        Ok(())
    }

    fn paint(&self) -> Result<()> {
        self.terminal
            .borrow_mut()
            .draw(|f| {
                self.basic_ui(f);
                self.pages[self.state.step].paint(f);
            })?;

        Ok(())
    }

    fn process(&mut self) -> Result<()> {
        self.pages[self.state.step].process(&mut self.state);

        Ok(())
    }

    fn basic_ui<B: Backend>(&self, f: &mut Frame<B>) {
        let state = self.state.borrow();

        // Render the tabs
        let titles = ["Welcome", "Install Limit", "Config & Deploy"]
            .iter()
            .cloned()
            .map(Spans::from)
            .collect();

        let tab = Tabs::new(titles)
            .block(Block::default().title(" Limit Up ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .select(state.step)
            .highlight_style(Style::default().fg(Color::Cyan))
            .divider(">");

        f.render_widget(tab, f.size());

        // Render tip
        let tip = Spans::from(format!(
            "{}Press <Q> to exit.",
            (state.step > 0)
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
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        execute!(
            self.terminal.borrow_mut().backend_mut(),
            LeaveAlternateScreen
        )
        .unwrap();
        disable_raw_mode().unwrap();
    }
}
