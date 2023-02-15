mod_use::mod_use!(welcome, install);

use cursive::{traits::*, views::DummyView, Cursive};

use super::widgets::StepTabs;

pub fn init_setup_ui(ui: &mut Cursive) {
    let tab = StepTabs::new()
        .with_tab(welcome())
        .with_tab(install())
        // TODO
        .with_tab(DummyView {}.with_name("Config & Deploy"))
        .with_name("step_tabs");

    ui.add_fullscreen_layer(tab);
}
