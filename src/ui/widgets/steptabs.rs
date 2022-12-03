use cursive::{
    event::{AnyCb, Event, EventResult},
    theme::BaseColor,
    traits::*,
    utils::markup::StyledString,
    view::Selector,
    views::{
        BoxedView, LinearLayout, NamedView, PaddedView, Panel, ResizedView, ScreensView, TextView,
    },
    Vec2, View,
};

pub struct StepTabs {
    titles: Vec<String>,
    layout: Panel<PaddedView<LinearLayout>>,
}

impl StepTabs {
    pub fn new() -> Self {
        Self {
            titles: Vec::new(),
            layout: LinearLayout::vertical()
                .child(TextView::new("").full_width())
                .child(ScreensView::<BoxedView>::new().full_screen())
                .wrap_with(|layout| PaddedView::lrtb(1, 1, 0, 0, layout))
                .wrap_with(|layout| Panel::new(layout)),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.layout.set_title(title);
        self
    }

    pub fn with_tab(mut self, tab: NamedView<impl View>) -> Self {
        self.titles.push(tab.name().to_string());

        self.get_inner_view_mut::<ResizedView<ScreensView>>(1)
            .get_inner_mut()
            .add_screen(BoxedView::boxed(tab));

        self.update_bar();

        self
    }

    pub fn active_tab(&self) -> usize {
        self.get_inner_view::<ResizedView<ScreensView>>(1)
            .get_inner()
            .active_screen()
    }

    pub fn next(&mut self) {
        let index = self.active_tab();

        if self.titles.is_empty() || index + 1 == self.titles.len() {
            return;
        }

        self.get_inner_view_mut::<ResizedView<ScreensView>>(1)
            .get_inner_mut()
            .set_active_screen(index + 1);

        self.update_bar();
    }

    pub fn prev(&mut self) {
        let index = self.active_tab();

        if index == 0 {
            return;
        }

        self.get_inner_view_mut::<ResizedView<ScreensView>>(1)
            .get_inner_mut()
            .set_active_screen(index - 1);

        self.update_bar();
    }

    fn get_inner_view<V: View>(&self, i: usize) -> &V {
        self.layout
            .get_inner()
            .get_inner()
            .get_child(i)
            .and_then(|tab| tab.downcast_ref::<V>())
            .unwrap()
    }

    fn get_inner_view_mut<V: View>(&mut self, i: usize) -> &mut V {
        self.layout
            .get_inner_mut()
            .get_inner_mut()
            .get_child_mut(i)
            .and_then(|tab| tab.downcast_mut())
            .unwrap()
    }

    fn update_bar(&mut self) {
        let text = self
            .titles
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let mut t = match i == self.active_tab() {
                    true => StyledString::styled(name, BaseColor::Cyan.light()),
                    false => StyledString::plain(name),
                };
                t.append_plain(if i + 1 == self.titles.len() {
                    ""
                } else {
                    " > "
                });
                t
            })
            .collect::<StyledString>();

        self.get_inner_view_mut::<ResizedView<TextView>>(0)
            .get_inner_mut()
            .set_content(text);
    }
}

impl View for StepTabs {
    fn draw(&self, printer: &cursive::Printer) {
        self.layout.draw(printer);
    }

    fn layout(&mut self, xy: cursive::Vec2) {
        self.layout.layout(xy)
    }

    fn needs_relayout(&self) -> bool {
        self.layout.needs_relayout()
    }

    fn required_size(&mut self, req: Vec2) -> Vec2 {
        self.layout.required_size(req)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        self.layout.on_event(event)
    }

    fn call_on_any<'a>(&mut self, sel: &Selector<'_>, any: AnyCb<'a>) {
        self.layout.call_on_any(sel, any);
    }

    fn focus_view(
        &mut self,
        sel: &Selector<'_>,
    ) -> Result<EventResult, cursive::view::ViewNotFound> {
        self.layout.focus_view(sel)
    }

    fn take_focus(
        &mut self,
        source: cursive::direction::Direction,
    ) -> Result<EventResult, cursive::view::CannotFocus> {
        self.layout.take_focus(source)
    }

    fn important_area(&self, view_size: Vec2) -> cursive::Rect {
        self.layout.important_area(view_size)
    }
}
