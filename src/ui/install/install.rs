use std::env;

use cursive::{
    align::HAlign,
    event::Key,
    theme::BaseColor,
    traits::*,
    views::{
        Button, Dialog, DialogFocus, DummyView, EditView, HideableView, LinearLayout, NamedView,
        OnEventView, PaddedView, Panel, ProgressBar, RadioGroup, ResizedView, ScreensView,
        ScrollView, SelectView, TextArea, TextView,
    },
    Cursive,
};
use nix::unistd::Uid;

use crate::core::installer::find_command;

pub fn install() -> NamedView<impl View> {
    LinearLayout::horizontal()
        .child(DummyView {}.fixed_width(10))
        .child(
            LinearLayout::vertical()
                .child(
                    TextView::new(crate::ui::LOGO)
                        .center()
                        .style(BaseColor::Cyan.light())
                        .full_height(),
                )
                .child(
                    TextView::new("Hello")
                        .scrollable()
                        .wrap_with(Panel::new)
                        .full_height()
                        .wrap_with(|detail| HideableView::new(detail).hidden())
                        .with_name("install_detail"),
                )
                .child(DummyView {})
                .child(
                    LinearLayout::horizontal()
                        .child(TextView::new("Installing..."))
                        .child(DummyView {}.full_width())
                        .child(Button::new_raw("[ Detail ]", |ui| {
                            let mut detail = ui
                                .find_name::<HideableView<ResizedView<Panel<ScrollView<TextView>>>>>(
                                    "install_detail",
                                )
                                .unwrap();
                            let visible = !detail.is_visible();
                            detail.set_visible(visible);
                        })),
                )
                .child(DummyView {})
                .child(ProgressBar::new().with_name("install_progress"))
                .child(DummyView {}.fixed_height(2))
                .full_width(),
        )
        .child(DummyView {}.fixed_width(10))
        .full_screen()
        .wrap_with(|layout| HideableView::new(layout).hidden())
        .with_name("Install")
}

pub fn prepare_install(ui: &mut Cursive) {
    ui.add_layer(
        ScreensView::new()
            .with(|screens| {
                screens.add_screen(notes_dialog());
                screens.add_screen(config_dialog());
                screens.add_screen(cancel_dialog());
            })
            .with_name("install_screens"),
    );
}

macro_rules! focus_on {
    ($ui:expr, $focus:expr) => {
        $ui.find_name::<ScreensView<Dialog>>("install_screens")
            .unwrap()
            .screen_mut()
            .unwrap()
            .set_focus($focus);
    };
}

fn notes_dialog() -> Dialog {
    Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new(
                "Do you want us to install dependencies for you?",
            ))
            .child(
                Panel::new(
                    EditView::new()
                        .secret()
                        .on_submit(|ui, _| {
                            focus_on!(ui, DialogFocus::Button(0));
                        })
                        .with_name("password"),
                )
                .title("Root Password (if any)")
                .title_position(HAlign::Left)
                .wrap_with(|password| PaddedView::lrtb(0, 0, 1, 0, password))
                .wrap_with(HideableView::new)
                .with(|password| {
                    Uid::effective().is_root().then(|| password.hide());
                }),
            )
            .child(DummyView {})
            .child(
                TextView::new("WARN: Automatic installation may effect your local environment.")
                    .style(BaseColor::Yellow.light()),
            )
            .fixed_width(40),
    )
    .title("Notes")
    .button("Yes", |ui| {
        ui.find_name::<ScreensView<Dialog>>("install_screens")
            .unwrap()
            .set_active_screen(1);
    })
    .button("No, I will install them myself", |ui| {
        ui.find_name::<ScreensView<Dialog>>("install_screens")
            .unwrap()
            .set_active_screen(2);
    })
}

fn config_dialog() -> Dialog {
    let mut install_group: RadioGroup<bool> =
        RadioGroup::new().on_change(|ui, is_from_binary: &bool| {
            ui.find_name::<HideableView<Panel<LinearLayout>>>("cargo")
                .unwrap()
                .set_visible(!*is_from_binary);

            if *is_from_binary {
                focus_on!(ui, DialogFocus::Button(1));
            } else {
                ui.focus_name("cargo").unwrap();
            }
        });

    Dialog::around(
        LinearLayout::vertical()
            .child(
                LinearLayout::vertical()
                    .child(install_group.button(true, "From binary"))
                    .child(install_group.button(false, "From source"))
                    .child(TextView::new("Press <Enter> to select"))
                    .wrap_with(|s| {
                        Panel::new(s)
                            .title("Install limit-server")
                            .title_position(HAlign::Left)
                    }),
            )
            .child(
                LinearLayout::vertical()
                    .child(
                        SelectView::new()
                            .with(|cargo_selector| {
                                let paths = find_command(
                                    "cargo",
                                    env::var("HOME")
                                        .map(|s| vec![format!("{}/.cargo/bin", s)])
                                        .unwrap_or(Vec::new()),
                                );

                                if paths.is_empty() {
                                    cargo_selector.add_item(
                                        "<Install for me (using rustup)>",
                                        String::from("0"),
                                    );
                                } else {
                                    for path in paths {
                                        let path = path.to_str().unwrap();
                                        cargo_selector.add_item(path, path.into())
                                    }
                                }
                            })
                            .item("<Specific path>", String::from("1"))
                            .on_select(|ui, selected| {
                                ui.find_name::<HideableView<LinearLayout>>("cargo_path")
                                    .unwrap()
                                    .set_visible(selected == "1");
                            })
                            .on_submit(|ui, _: &String| {
                                focus_on!(ui, DialogFocus::Button(1));
                            })
                            .with_name("cargo_selector"),
                    )
                    .child(
                        LinearLayout::horizontal()
                            .child(TextView::new("Path: "))
                            .child(TextArea::new().min_size((30, 2)).max_size((60, 3)))
                            .wrap_with(|edit| HideableView::new(edit).hidden())
                            .with_name("cargo_path"),
                    )
                    .wrap_with(|layout| {
                        Panel::new(layout)
                            .title("Cargo")
                            .title_position(HAlign::Left)
                    })
                    .wrap_with(|layout| HideableView::new(layout).hidden())
                    .with_name("cargo"),
            )
            .scrollable(),
    )
    .title("Installation Configuration")
    .button("Previous", |ui| {
        ui.find_name::<ScreensView<Dialog>>("install_screens")
            .unwrap()
            .set_active_screen(0);
    })
    .button("Confirm", |ui| {
        ui.pop_layer();
        start_install(ui);
    })
}

fn cancel_dialog() -> Dialog {
    Dialog::around(TextView::new("http://example.com"))
        .button("Previous", |ui| {
            ui.find_name::<ScreensView<Dialog>>("install_screens")
                .unwrap()
                .set_active_screen(0);
        })
        .button("Ok, I know", |ui| ui.quit())
        .with(|dialog| {
            dialog.set_focus(DialogFocus::Button(1));
        })
        .title("Installation Cancelled")
}

fn start_install(ui: &mut Cursive) {
    ui.find_name::<HideableView<ResizedView<LinearLayout>>>("Install")
        .unwrap()
        .unhide();
}
