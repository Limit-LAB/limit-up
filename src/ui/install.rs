use crossterm::event::{Event, KeyCode};
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders},
    Frame,
};

use super::{split_rect, Page, UiState};

pub struct Install;

impl Install {
    pub fn new() -> Self {
        Self
    }
}

impl<B: Backend> Page<B> for Install {
    fn paint(&self, f: &mut Frame<B>, r: Rect) {
        let (main, tip) = split_rect(r);
        // TODO
        f.render_widget(Block::default().borders(Borders::ALL), main);

        // Render tip
        f.render_widget(
            Block::default().title("Press ←  to previous step; Press <Q> to exit."),
            tip,
        );
    }

    fn process(&mut self, ui_state: &mut UiState) {
        if let Event::Key(key) = ui_state.event {
            // TODO
            match key.code {
                // Quit
                KeyCode::Char('q') => ui_state.runnable = false,
                KeyCode::Left => ui_state.step = 0,
                _ => {}
            }
        }
    }
}
