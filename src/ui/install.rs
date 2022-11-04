use crossterm::event::{Event, KeyCode};
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph},
    Frame,
};

use super::{Page, UiState};

pub struct Install;

impl Install {
    pub fn new() -> Self {
        Self
    }
}

impl<B: Backend> Page<B> for Install {
    fn paint(&self, f: &mut Frame<B>) {
        // TODO
        f.render_widget(Block::default(), f.size())
    }

    fn process(&mut self, ui_state: &mut UiState) {
        if let Event::Key(key) = ui_state.event {
            // TODO
            match key.code {
                KeyCode::Char('q') => ui_state.runnable = false,
                KeyCode::Left => ui_state.step = 0,
                _ => {}
            }
        }
    }
}
