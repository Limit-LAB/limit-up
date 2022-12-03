mod_use::mod_use!(install, widgets, frontend);

use cursive::{
    theme::{BaseColor::*, Color::*, PaletteColor::*},
    Cursive, CursiveExt,
};

static LOGO: &'static str = r#"|      _)            _)  |
  |       |  __ `__ \   |  __|
  |       |  |   |   |  |  |  
  _____| _| _|  _|  _| _| \__|"#;

#[derive(Debug, Default)]
struct UserData {
    install: InstallConfig,
}

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

        ui.set_user_data(UserData::default());

        init_install_ui(&mut ui);

        ui.add_screen();

        init_frontend_ui(&mut ui);

        ui.set_screen(0);

        Self { ui }
    }

    pub fn exec(mut self) {
        self.ui.run();
    }
}
