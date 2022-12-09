use cursive::{
    theme::BaseColor,
    traits::*,
    utils::markup::StyledString,
    views::{
        Button, Dialog, DummyView, HideableView, LinearLayout, NamedView, Panel, ProgressBar,
        ResizedView, ScrollView, TextView,
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
                .child(ProgressBar::new().with_name("install_progress"))
                .child(DummyView {}.fixed_height(2))
                .full_width(),
        )
        .child(DummyView {}.fixed_width(10))
        .full_screen()
        .wrap_with(|layout| HideableView::new(layout).hidden())
        .with_name("Install")
}
