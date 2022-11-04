use crossterm::event::{Event, KeyCode};
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
    Frame,
};

use super::{UiState, Page};

pub struct Welcome;

impl Welcome {
    pub fn new() -> Self {
        Self
    }
}

impl<B: Backend> Page<B> for Welcome {
    /// Paint Welcome page
    fn paint(&self, f: &mut Frame<B>) {
        let mut title = Text::styled(super::LOGO, Style::default().fg(Color::LightCyan));
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
    }

    /// Process event
    fn process(&mut self, ui_state: &mut UiState) {
        if let Event::Key(key) = ui_state.event {
            match key.code {
                KeyCode::Char('q') => ui_state.runnable = false,
                _ => ui_state.step = 1,
            }
        }
    }
}
