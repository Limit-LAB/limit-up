use crossterm::event::{Event, KeyCode};
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Paragraph},
    Frame,
};

use super::{split_rect, Page, UiState};

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

        let width = title.width() as u16;
        let height = title.height() as u16;

        let x = (f.size().width / 2).saturating_sub(width / 2);
        let y = (f.size().height / 2).saturating_sub(height / 2);

        f.render_widget(
            Paragraph::new(title).alignment(Alignment::Center),
            // Render limit logo in the center of the screen
            // It hides when it doesn't have enough space to render
            Rect {
                x,
                y,
                width: if width + x > f.size().width { 0 } else { width },
                height: if height + y > f.size().height {
                    0
                } else {
                    height
                },
            },
        );

        // Render tip
        f.render_widget(
            Block::default().title("Press <Q> to exit."),
            split_rect(r).1,
        );
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
