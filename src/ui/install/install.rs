use std::{
    env,
    io::{BufRead, BufReader},
    iter::empty,
    path::Path,
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

use crate::{
    as_raw,
    core::installer::{find_command, Cargo, Error, ErrorKind, PackageManager, Result, Rustup},
    select,
    ui::widgets::StepTabs,
};

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
                        .child(TextView::new("Installing...").with_name("install_tip"))
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

enum CargoConfig {
    InstallForMe,
    Path(String),
}

enum InstallMethod {
    /// install limit server from binary
    Binary,
    /// install limit server from source
    Source(CargoConfig),
}

impl Default for InstallMethod {
    fn default() -> Self {
        InstallMethod::Binary
    }
}

#[derive(Default)]
struct InstallConfig {
    pkg_manager: Option<PackageManager>,
    dependencies: Vec<String>,
    method: InstallMethod,
    install_root: String,
}

pub fn prepare_install(ui: &mut Cursive) {
    let mut screens = ScreensView::new().with(|screens| {
        screens.add_screen(notes_dialog());
        screens.add_screen(config_dialog());
        screens.add_screen(cancel_dialog());
    });

    let mut config = InstallConfig::default();

    if find_command("curl", empty::<&str>()).is_empty() {
        config.dependencies.push("curl".into());
    }

    if find_command("redis-server", empty::<&str>()).is_empty() {
        config.dependencies.push("redis".into());
    }

    if config.dependencies.is_empty() {
        screens.set_active_screen(1);
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

fn notes_dialog() -> Dialog {
    Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new(
                "Do you want us to install dependencies for you?",
            ))
            .child(
                EditView::new()
                    .secret()
                    .on_submit(|ui, _| {
                        focus_on!(ui, DialogFocus::Button(0));
                    })
                    .with_name("password")
                    .wrap_with(Panel::new)
                    .title("Root Password (if any)")
                    .title_position(HAlign::Left)
                    .wrap_with(|password| PaddedView::lrtb(0, 0, 1, 0, password))
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
            .child(DummyView {})
            .child(
                TextView::new("WARN: Automatic installation may effect your local environment.")
                    .style(BaseColor::Yellow.light()),
            )
            .fixed_width(40),
    )
    .title("Notes")
    .button("Yes", |ui| {
        #[cfg(windows)]
        let mgr = PackageManager::new();

        #[cfg(unix)]
        let mgr = PackageManager::new_with_passwd(
            &*ui.find_name::<EditView>("password").unwrap().get_content(),
        );

        match mgr {
            Ok(p) => ui.user_data::<InstallConfig>().unwrap().pkg_manager = Some(p),
            Err(e) => {
                ui.add_layer(Dialog::info(format!("Error: {}", e)).title("Oops"));
                return;
            }
        };

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
    let mut method_group: RadioGroup<bool> =
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
                    .child(method_group.button(true, "From binary"))
                    .child(method_group.button(false, "From source"))
                    .child(TextView::new("Press <Enter> to select"))
                    .wrap_with(|s| {
                        Panel::new(s)
                            .title("Install limit-server")
                            .title_position(HAlign::Left)
                    }),
            )
            .child(
                TextArea::new()
                    .content(format!("{}/.cargo", env::var("HOME").unwrap_or_default()))
                    .with_name("install_root")
                    .min_size((30, 2))
                    .max_size((50, 2))
                    .wrap_with(Panel::new)
                    .title("Install root")
                    .title_position(HAlign::Left),
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
                                        .unwrap_or_default(),
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
                            .child(TextArea::new().min_size((30, 2)).max_size((50, 2)))
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
    .button("Confirm", move |ui| {
        let install_root = ui.find_name::<TextArea>("install_root").unwrap();
        // if !Path::new(install_root.get_content()).exists() {
        //     ui.add_layer(Dialog::info("Invalid install root").title("Oops"));
        //     return;
        // }

        ui.user_data::<InstallConfig>().unwrap().install_root = install_root.get_content().into();

        ui.user_data::<InstallConfig>().unwrap().method = match &*method_group.selection() {
            true => InstallMethod::Binary,
            false => {
                match ui
                    .find_name::<SelectView>("cargo_selector")
                    .unwrap()
                    .selection()
                    .unwrap()
                    .as_str()
                {
                    "0" => InstallMethod::Source(CargoConfig::InstallForMe),
                    "1" => {
                        let path_edit = ui
                            .find_name::<HideableView<LinearLayout>>("cargo_path")
                            .unwrap();

                        let specific = Path::new(
                            path_edit
                                .get_inner()
                                .get_child(1)
                                .unwrap()
                                .downcast_ref::<ResizedView<ResizedView<TextArea>>>()
                                .unwrap()
                                .get_inner()
                                .get_inner()
                                .get_content(),
                        );

                        match specific.file_name() {
                            Some(name) if name == "cargo" && specific.is_file() => {
                                InstallMethod::Source(CargoConfig::Path(
                                    specific.to_str().unwrap().into(),
                                ))
                            }
                            _ => {
                                ui.add_layer(Dialog::info("Invalid cargo path").title("Oops"));
                                return;
                            }
                        }
                    }
                    path => InstallMethod::Source(CargoConfig::Path(path.into())),
                }
            }
        };

        ui.pop_layer();

        ui.find_name::<HideableView<ResizedView<LinearLayout>>>("Install")
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
                                Dialog::text(format!("An error occurred while installing: {}", e))
                                    .title("Error")
                                    .button("Quit", |ui| ui.quit())
                                    .max_width(50),
                            );
                        }))
                        .unwrap();
                }
            });
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
                    .set_content("Installing dependencies...")
            }))
            .unwrap();

        let pkg_manager = config.pkg_manager.take().unwrap();
        let mut proc = pkg_manager.install(config.dependencies)?;

        trace_process!(proc, 39, |s| Error::new(
            ErrorKind::Other,
            format!("Package manager exit with {}", s),
        ));
    }

    counter.set(40);

    // install limit-server
    match config.method {
        InstallMethod::Binary => {
            // todo
        }
        InstallMethod::Source(cargo) => {
            let path = match cargo {
                CargoConfig::InstallForMe => {
                    cb_sink
                        .send(Box::new(|ui| {
                            ui.find_name::<TextView>("install_tip")
                                .unwrap()
                                .set_content("Setup rust...")
                        }))
                        .unwrap();

                    let mut proc = Rustup::install()?;

                    trace_process!(proc, 49, |s| Error::new(
                        ErrorKind::Other,
                        format!("Setup rust failed: {}", s),
                    ));

                    format!(
                        "{}/.cargo/bin/cargo",
                        env::var("HOME").map_err(|_| Error::new(
                            ErrorKind::NotFound,
                            "Can not locate cargo path"
                        ))?
                    )
                }
                CargoConfig::Path(p) => p,
            };

            cb_sink
                .send(Box::new(|ui| {
                    ui.find_name::<TextView>("install_tip")
                        .unwrap()
                        .set_content("Installing limit-server...")
                }))
                .unwrap();

            // cargo install
            let mut proc = Cargo::new(path).install(config.install_root)?;

            trace_process!(proc, 99, |s| Error::new(
                ErrorKind::Other,
                format!("Install limit-server failed: {}", s),
            ));
        }
    }

    // finished
    cb_sink
        .send(Box::new(|ui| {
            ui.find_name::<StepTabs>("step_tabs").unwrap().next();
        }))
        .unwrap();

    Ok(())
}
