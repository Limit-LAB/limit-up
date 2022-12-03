mod_use::mod_use!(welcome, install);

use cursive::{traits::*, views::DummyView, Cursive};

use crate::StepTabs;

#[derive(Debug, Default)]
pub struct InstallConfig {
    sql: String,
}

pub fn init_install_ui(ui: &mut Cursive) {
    let tab = StepTabs::new()
        .with_tab(welcome())
        .with_tab(install())
        // TODO
        .with_tab(DummyView {}.with_name("Config & Deploy"))
        .with_name("steptabs");

    ui.add_fullscreen_layer(tab);
}
