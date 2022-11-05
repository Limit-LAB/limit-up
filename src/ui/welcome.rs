use std::cmp::min;

use crossterm::event::{Event, KeyCode};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Paragraph},
    Frame,
};

use super::{Page, UiState};

pub struct Welcome;

impl Welcome {
    pub fn new() -> Self {
        Self
    }
}

impl<B: Backend> Page<B> for Welcome {
    fn paint(&self, f: &mut Frame<B>, r: Rect) {
        let mut title = Text::styled(super::LOGO, Style::default().fg(Color::LightCyan));
        title.extend(
            Text::from("\nWelcome to Limit up\nPress any key to continue except <Q> :)")
                .into_iter(),
        );

        let layout = Layout::default()
            .constraints([
                Constraint::Min(title.height() as u16),
                Constraint::Length(1),
            ])
            .split(r);

        let height = title.height() as u16;

        let y = (layout[0].height / 2).saturating_sub(height / 2) + layout[0].y;

        f.render_widget(
            Paragraph::new(title).alignment(Alignment::Center),
            // Render limit logo in the center of the screen
            // It hides when it doesn't have enough space to render
            Rect {
                y,
                height: min(height, layout[0].height),
                ..layout[0]
            },
        );

        // Render tip
        f.render_widget(Block::default().title("Press <Q> to exit."), layout[1]);
    }

    fn process(&mut self, ui_state: &mut UiState) {
        if let Event::Key(key) = ui_state.event {
            match key.code {
                // Quit
                KeyCode::Char('q') => ui_state.runnable = false,
                // Turn to Install Limit
                _ => ui_state.step = 1,
            }
        }
    }
}
