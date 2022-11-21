mod_use::mod_use!(welcome);

use cursive::{traits::*, views::DummyView, Cursive};

use crate::StepTabs;

pub fn init_install_ui(ui: &mut Cursive) {
    ui.add_fullscreen_layer(
        StepTabs::new()
            .title("Limit up")
            .with_tab(welcome())
            // TODO
            .with_tab(DummyView {}.with_name("Install"))
            .with_tab(DummyView {}.with_name("Deploy & Config"))
            .with_name("tab"),
    );
}
