use std::env;

use cursive::{
    align::HAlign,
    theme::BaseColor,
    traits::*,
    view::ScrollStrategy,
    views::{
        Button, Dialog, DialogFocus, DummyView, HideableView, LinearLayout, NamedView, PaddedView,
        Panel, ProgressBar, ResizedView, ScreensView, ScrollView, TextArea, TextView,
    },
    CbSink, Cursive,
};

use r18::tr;

use crate::{
    core::{
        installer::{self, InstallConfig},
        RT,
    },
    ui::widgets::StepTabs,
    Result,
};

// convenient function to create an error dialog
fn error_dialog(message: impl ToString, default_button: bool) -> ResizedView<Dialog> {
    Dialog::text(tr!("Error: {}", message.to_string()))
        .title(tr!("Oops"))
        .with(|d| {
            default_button.then(|| d.add_button(tr!("Ok"), |ui| ui.quit()));
        })
        .max_width(50)
}

// returns install(ing) page,
// it is hidden by default and shown on on_install
pub fn install() -> NamedView<impl View> {
    LinearLayout::vertical()
        .child(
            TextView::new(crate::ui::LOGO)
                .center()
                .style(BaseColor::Cyan.light())
                .full_height(),
        )
        .child(
            TextView::empty()
                .scrollable()
                .scroll_strategy(ScrollStrategy::StickToBottom)
                .wrap_with(Panel::new)
                .full_height()
                .wrap_with(|detail| HideableView::new(detail).hidden())
                .with_name("install_detail"),
        )
        .child(DummyView {})
        .child(
            LinearLayout::horizontal()
                .child(TextView::new(tr!("Installing...")).with_name("install_tip"))
                .child(DummyView {}.full_width())
                .child(Button::new_raw(tr!("[ Detail ]"), |ui| {
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
        .wrap_with(|layout| PaddedView::lrtb(10, 10, 0, 2, layout))
        .full_screen()
        .wrap_with(|layout| HideableView::new(layout).hidden())
        .with_name(tr!("Install"))
}

// initialize configure ui
// this function will be called when the user clicks Next button on the welcome page
pub fn prepare_install(ui: &mut Cursive) {
    // PackageManager for FreeBSD requires Root permission
    #[cfg(target_os = "freebsd")]
    if !nix::unistd::Uid::effective().is_root() {
        ui.add_layer(error_dialog(
            tr!("Permission denied, please rerun as Root"),
            true,
        ));

        return;
    }

    let screens = ScreensView::new().with(|screens| {
        screens.add_screen(config_dialog());
        screens.add_screen(cancel_dialog());
    });

    ui.set_user_data(InstallConfig::default());
    ui.add_layer(screens.with_name("install_screens"));
}

// configure automatic installation
// this dialog will appear when the user wants install automatically
fn config_dialog() -> Dialog {
    Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new(tr!(
                "Do you want us to install dependencies for you?"
            )))
            .child(DummyView {})
            .child(
                TextArea::new()
                    .content(format!(
                        "{}/.limit-lab",
                        env::var("HOME").unwrap_or_default()
                    ))
                    .with_name("install_root")
                    .min_size((30, 2))
                    .max_size((50, 2))
                    .wrap_with(Panel::new)
                    .title(tr!("Install root"))
                    .title_position(HAlign::Left),
            )
            .child(DummyView {})
            .child(
                TextView::new(tr!(
                    "WARN: Automatic installation may effect your local environment."
                ))
                .style(BaseColor::Yellow.light()),
            )
            .fixed_width(40)
            .scrollable(),
    )
    .title(tr!("Notes"))
    .button(tr!("Yes"), on_install)
    .button(tr!("No, I will install them myself"), |ui| {
        ui.find_name::<ScreensView<Dialog>>("install_screens")
            .unwrap()
            .set_active_screen(1);
    })
}

// help information about manual install
// this dialog will appear when the user doesn't want install automatically
fn cancel_dialog() -> Dialog {
    // TODO
    Dialog::around(TextView::new("http://example.com"))
        .button(tr!("Previous"), |ui| {
            ui.find_name::<ScreensView<Dialog>>("install_screens")
                .unwrap()
                .set_active_screen(0);
        })
        .button(tr!("Ok, I know"), |ui| ui.quit())
        .with(|dialog| {
            dialog.set_focus(DialogFocus::Button(1));
        })
        .title(tr!("Installation Cancelled"))
}

// this function will be called when the user confirms automatic installation
fn on_install(ui: &mut Cursive) {
    ui.user_data::<InstallConfig>().unwrap().install_root = ui
        .find_name::<TextArea>("install_root")
        .unwrap()
        .get_content()
        .into();

    ui.pop_layer();

    ui.find_name::<HideableView<ResizedView<PaddedView<LinearLayout>>>>(tr!("Install"))
        .unwrap()
        .unhide();

    let config = ui.take_user_data::<InstallConfig>().unwrap();
    let cb_sink = ui.cb_sink().clone();

    RT.spawn(install_task(cb_sink, config));
}

// install limit backend
async fn install_task(cb_sink: CbSink, config: InstallConfig) {
    if let Err(e) = install_task_inner(&cb_sink, config).await {
        cb_sink
            .send(Box::new(|ui| {
                ui.add_layer(error_dialog(e, true));
            }))
            .unwrap();
    }

    // finished
    cb_sink
        .send(Box::new(|ui| {
            ui.find_name::<StepTabs>("step_tabs").unwrap().next();
        }))
        .unwrap();
}

// linux implementation
#[cfg(target_os = "linux")]
async fn install_task_inner(cb_sink: &CbSink, config: InstallConfig) -> Result<()> {
    cb_sink
        .send(Box::new(move |ui| {
            ui.find_name::<TextView>("install_tip")
                .unwrap()
                .set_content(tr!("Downloading limit-server..."));
        }))
        .unwrap();

    let cb_sink = cb_sink.clone();
    let callback = move |progress| {
        cb_sink
            .send(Box::new(move |ui| {
                ui.find_name::<ProgressBar>("install_progress")
                    .unwrap()
                    .set_value(progress);
            }))
            .unwrap();
    };

    installer::install(config, callback).await
}

// freebsd implementation
#[cfg(target_os = "freebsd")]
async fn install_task_inner(cb_sink: &CbSink, config: InstallConfig) -> Result<()> {
    use cursive::utils::markup::StyledString;

    cb_sink
        .send(Box::new(move |ui| {
            ui.find_name::<TextView>("install_tip")
                .unwrap()
                .set_content(tr!("Installing Elixir..."));
        }))
        .unwrap();

    let cb_sink = cb_sink.clone();
    installer::install(config, move |progress, out, err| {
        cb_sink
            .send(Box::new(move |ui| {
                if !out.is_empty() || !err.is_empty() {
                    let new_line = match out.is_empty() {
                        true => StyledString::from(out),
                        false => StyledString::styled(err, BaseColor::Red.light()),
                    };

                    ui.find_name::<HideableView<ResizedView<Panel<ScrollView<TextView>>>>>(
                        "install_detail",
                    )
                    .unwrap()
                    .get_inner_mut()
                    .get_inner_mut()
                    .get_inner_mut()
                    .get_inner_mut()
                    .append(new_line);
                }

                ui.find_name::<ProgressBar>("install_progress")
                    .unwrap()
                    .set_value(progress);
            }))
            .unwrap();
    })
    .await
}

// windows implementation
#[cfg(target_os = "windows")]
async fn install_task_inner(cb_sink: &CbSink, config: InstallConfig) -> Result<()> {
    installer::install(config, move |_p| {}).await
}

// dummy implementation
#[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "windows")))]
async fn install_task_inner(cb_sink: &CbSink, config: InstallConfig) -> Result<()> {
    Err(tr!("Unsupported platform").into())
}
