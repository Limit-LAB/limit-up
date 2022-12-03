use cursive::{
    theme::BaseColor,
    traits::*,
    views::{
        Button, Dialog, DummyView, HideableView, LinearLayout, NamedView, Panel, ProgressBar,
        ResizedView, ScrollView, SelectView, TextView,
    },
    Cursive,
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
                .child(DummyView {})
                .child(
                    TextView::new("Hello")
                        .scrollable()
                        .wrap_with(|detail| Panel::new(detail))
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
                .child(ProgressBar::new().with(|p| p.set_value(10)))
                .child(DummyView {}.fixed_height(2))
                .full_width(),
        )
        .child(DummyView {}.fixed_width(10))
        .full_screen()
        .wrap_with(|layout| HideableView::new(layout).hidden())
        .with_name("Install")
}

pub fn install_config_dialog() -> Dialog {
    Dialog::around(
        LinearLayout::vertical()
            .child(
                SelectView::new()
                    .item("SQLite", "sqlite")
                    .item("MySQL", "mysql")
                    .item("PostgreSQL", "postgresql")
                    .item("Remote SQL (Don't install)", "")
                    .on_submit(|ui, sql: &str| {
                        ui.user_data::<crate::ui::UserData>().unwrap().install.sql =
                            sql.to_string();

                        ui.pop_layer();
                        ui.find_name::<HideableView<ResizedView<LinearLayout>>>("Install")
                            .unwrap()
                            .unhide();
                    }),
            )
            .child(TextView::new("\nPress <Enter> to submit")),
    )
    .title("Choose the SQL you want")
}
