use std::{
    env,
    fmt::Display,
    io::{BufRead, BufReader},
    iter::empty,
    path::Path,
    process::ExitStatus,
};

use cursive::{
    align::HAlign,
    theme::BaseColor,
    traits::*,
    utils::{markup::StyledString, Counter},
    view::ScrollStrategy,
    views::{
        Button, Dialog, DialogFocus, DummyView, EditView, HideableView, LinearLayout, NamedView,
        PaddedView, Panel, ProgressBar, RadioGroup, ResizedView, ScreensView, ScrollView,
        SelectView, TextArea, TextView,
    },
    CbSink, Cursive,
};
use r18::tr;

use crate::{
    as_raw,
    core::{
        helper::Help,
        installer::{find_command, Error, ErrorKind, PackageManager, Result},
    },
    select,
    ui::widgets::StepTabs,
};

fn error_dialog(message: impl Display, default_button: bool) -> ResizedView<Dialog> {
    Dialog::text(tr!("Error: {}", message.to_string()))
        .title(tr!("Oops"))
        .with(|d| {
            default_button.then(|| d.add_button(tr!("Ok"), |ui| ui.quit()));
        })
        .max_width(50)
}

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

#[derive(Default)]
struct InstallConfig {
    pkg_manager: Option<PackageManager>,
    dependencies: Vec<String>,
    install_root: String,
}

pub fn prepare_install(ui: &mut Cursive) {
    let screens = ScreensView::new().with(|screens| {
        screens.add_screen(config_dialog());
        screens.add_screen(cancel_dialog());
    });

    let mut config = InstallConfig::default();

    if find_command("curl", empty::<&str>()).is_empty() {
        config.dependencies.push("curl".into());
    }

    ui.set_user_data(config);
    ui.add_layer(screens.with_name("install_screens"));
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

fn config_dialog() -> Dialog {
    Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new(tr!(
                "Do you want us to install dependencies for you?"
            )))
            .child(DummyView {})
            .child(
                EditView::new()
                    .secret()
                    .on_submit(|ui, _| {
                        focus_on!(ui, DialogFocus::Button(0));
                    })
                    .with_name("password")
                    .wrap_with(Panel::new)
                    .title(tr!("Root Password (if any)"))
                    .title_position(HAlign::Left)
                    .wrap_with(|password| PaddedView::lrtb(0, 0, 0, 1, password))
                    .wrap_with(HideableView::new)
                    .with(|password| {
                        #[cfg(unix)]
                        nix::unistd::Uid::effective()
                            .is_root()
                            .then(|| password.hide());

                        #[cfg(windows)]
                        password.hide();
                    }),
            )
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
    .button(tr!("Yes"), |ui| {
        #[cfg(windows)]
        let mgr = PackageManager::new();

        #[cfg(unix)]
        let mgr = PackageManager::new_with_passwd(
            &*ui.find_name::<EditView>("password").unwrap().get_content(),
        );

        match mgr {
            Ok(p) => ui.user_data::<InstallConfig>().unwrap().pkg_manager = Some(p),
            Err(e) => {
                ui.add_layer(error_dialog(e.to_string(), true));
                return;
            }
        };

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

        ui.find_name::<ProgressBar>("install_progress")
            .unwrap()
            .start(move |counter| {
                if let Err(e) = install_task(cb_sink.clone(), counter, config) {
                    cb_sink
                        .send(Box::new(move |ui| {
                            ui.add_layer(
                                error_dialog(e, false)
                                    .with(|d| d.get_inner_mut().add_button("Quit", |ui| ui.quit())),
                            );
                        }))
                        .unwrap();
                }
            });
    })
    .button(tr!("No, I will install them myself"), |ui| {
        ui.find_name::<ScreensView<Dialog>>("install_screens")
            .unwrap()
            .set_active_screen(1);
    })
}

fn cancel_dialog() -> Dialog {
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

fn install_task(cb_sink: CbSink, counter: Counter, mut config: InstallConfig) -> Result<()> {
    macro_rules! trace_process {
        ($proc:expr, $progress_limit:expr, $on_failed:expr) => {
            let mut out = BufReader::new($proc.stdout.take().unwrap());
            let mut err = BufReader::new($proc.stderr.take().unwrap());

            loop {
                #[cfg(unix)]
                let fdset = select!(out.get_ref(), err.get_ref(); None)?;

                #[cfg(windows)]
                let fdset = {
                    let fdset = select!(out.get_ref(), err.get_ref(); 1000);

                    if fdset.is_empty() {
                        continue;
                    }

                    fdset
                };

                let (mut out_buf, mut err_buf) = (String::new(), String::new());

                if fdset.contains(as_raw!(out.get_ref())) {
                    out.read_line(&mut out_buf)?;
                }

                if fdset.contains(as_raw!(err.get_ref())) {
                    err.read_line(&mut err_buf)?;
                }


                #[cfg(unix)]
                if counter.get() < $progress_limit && fdset.highest().is_some() {
                    counter.tick(1);
                }

                #[cfg(windows)]
                if counter.get() < $progress_limit && !fdset.is_empty() {
                    counter.tick(1);
                }

                cb_sink
                    .send(Box::new(|ui| {
                        let mut detail = ui
                            .find_name::<HideableView<ResizedView<Panel<ScrollView<TextView>>>>>(
                                "install_detail",
                            )
                            .unwrap();

                        let detail = detail
                            .get_inner_mut()
                            .get_inner_mut()
                            .get_inner_mut()
                            .get_inner_mut();

                        if !out_buf.is_empty() {
                            detail.append(out_buf);
                        }

                        if !err_buf.is_empty() {
                            detail.append(StyledString::styled(err_buf, BaseColor::Red.light()));
                        }
                    }))
                    .unwrap();

                if let Some(s) = $proc.try_wait()? {
                    if s.success() {
                        break;
                    }

                    return Err($on_failed(s));
                }
            }
        };
    }

    // Install dependencies
    if !config.dependencies.is_empty() {
        cb_sink
            .send(Box::new(|ui| {
                ui.find_name::<TextView>("install_tip")
                    .unwrap()
                    .set_content(tr!("Installing dependencies..."))
            }))
            .unwrap();

        let pkg_manager = config.pkg_manager.take().unwrap();
        let name = pkg_manager.name();
        let mut proc = pkg_manager.install(config.dependencies)?;

        trace_process!(proc, 39, |s: ExitStatus| Error::new(
            ErrorKind::Other,
            tr!(
                "Package manager exit with {}\n\n{}",
                s.to_string(),
                Help::PackageManager(name).to_string()
            ),
        ));
    }

    counter.set(40);

    // install limit-server
    // TODO: download appimage

    // finished
    cb_sink
        .send(Box::new(|ui| {
            ui.find_name::<StepTabs>("step_tabs").unwrap().next();
        }))
        .unwrap();

    Ok(())
}
