mod frontend;
mod setup;

#[allow(dead_code)]
mod widgets;

use frontend::init_frontend_ui;
use setup::init_setup_ui;

use cursive::{
    theme::{BaseColor::*, Color::*, PaletteColor::*},
    Cursive, CursiveExt,
};

static LOGO: &str = r#" |     _ _|   \  | _ _| ___ |
 |       |   |\/ |   |      |
 |       |   |   |   |      |
_____| ___| _|  _| ___|    _|"#;

pub struct Ui {
    ui: Cursive,
}

impl Ui {
    /// setup ui
    pub fn setup() -> Self {
        let mut ui = Cursive::new();

        ui.with_theme(|t| {
            t.shadow = false;

            t.palette[Background] = TerminalDefault;
            t.palette[View] = TerminalDefault;
            t.palette[Primary] = White.light();
            t.palette[TitlePrimary] = Blue.light();
            t.palette[Secondary] = Blue.light();
            t.palette[Highlight] = Cyan.light();
        });

        init_setup_ui(&mut ui);

        ui.add_screen();
        init_frontend_ui(&mut ui);

        ui.set_screen(0);

        Self { ui }
    }

    /// enter cursive's event loop
    pub fn exec(mut self) {
        self.ui.run();
    }
}
