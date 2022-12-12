use cursive::{
    theme::BaseColor,
    traits::*,
    utils::markup::StyledString,
    view::Nameable,
    views::{Button, DummyView, LinearLayout, NamedView, TextView},
};

use super::StepTabs;

pub fn welcome() -> NamedView<impl View> {
    let mut logo = StyledString::styled(crate::ui::LOGO, BaseColor::Cyan.light());
    logo.append_plain(
        "\n\nWelcome to Limit up
A CLI tool that helps you to setup limit-server :)",
    );

    LinearLayout::vertical()
        .child(TextView::new(logo).center().full_screen())
        .child(
            LinearLayout::horizontal()
                .child(Button::new_raw("[ Quit ]", |ui| ui.quit()))
                .child(DummyView {}.full_width())
                .child(Button::new_raw("[ Next ]", |ui| {
                    ui.find_name::<StepTabs>("steptabs").unwrap().next();
                    super::prepare_install(ui);
                }))
                .with(|layout| {
                    layout.set_focus_index(2).unwrap();
                }),
        )
        .with_name("Welcome")
}
