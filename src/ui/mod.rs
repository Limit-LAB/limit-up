mod_use::mod_use!(install, widgets, frontend);

use cursive::{
    theme::{BaseColor::*, Color::*, PaletteColor::*},
    Cursive, CursiveExt,
};

static LOGO: &'static str = r#"|      _)            _)  |
  |       |  __ `__ \   |  __|
  |       |  |   |   |  |  |  
  _____| _| _|  _|  _| _| \__|"#;

pub struct Ui {
    ui: Cursive,
}

impl Ui {
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

        init_install_ui(&mut ui);

        Self { ui }
    }

    pub fn exec(mut self) {
        self.ui.run();
    }
}
