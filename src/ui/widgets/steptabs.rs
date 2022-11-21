use cursive::{
    event::{AnyCb, Event, EventResult},
    theme::BaseColor,
    traits::*,
    utils::markup::StyledString,
    view::Selector,
    views::{LinearLayout, NamedView, PaddedView, Panel, ResizedView, TextView},
    Vec2, View,
};
use cursive_tabs::TabView;

pub struct StepTabs {
    index: usize,
    count: usize,
    layout: Panel<PaddedView<LinearLayout>>,
}

impl StepTabs {
    pub fn new() -> Self {
        Self {
            index: 0,
            count: 0,
            layout: LinearLayout::vertical()
                .child(TextView::new("").full_width())
                .child(TabView::new().full_screen())
                .wrap_with(|layout| PaddedView::lrtb(1, 1, 0, 0, layout))
                .wrap_with(|layout| Panel::new(layout)),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.layout.set_title(title);
        self
    }

    pub fn with_tab(mut self, tab: NamedView<impl View>) -> Self {
        let index = self.index;
        let tab_view = self.get_view_mut::<ResizedView<TabView>>(1).get_inner_mut();

        tab_view.add_tab(tab);

        tab_view
            .tab_order()
            .get(index)
            .map(|id| tab_view.set_active_tab(id));

        self.update_bar();
        self.count += 1;

        self
    }

    pub fn active_tab(&self) -> usize {
        self.index
    }

    pub fn next(&mut self) {
        if self.index + 1 == self.count {
            return;
        }

        self.index += 1;
        self.update_bar();

        self.get_view_mut::<ResizedView<TabView>>(1)
            .get_inner_mut()
            .next();
    }

    pub fn prev(&mut self) {
        if self.index == 0 {
            return;
        }

        self.index -= 1;
        self.update_bar();

        self.get_view_mut::<ResizedView<TabView>>(1)
            .get_inner_mut()
            .prev();
    }

    fn get_view<V: View>(&self, i: usize) -> &V {
        self.layout
            .get_inner()
            .get_inner()
            .get_child(i)
            .and_then(|tab| tab.downcast_ref::<V>())
            .unwrap()
    }

    fn get_view_mut<V: View>(&mut self, i: usize) -> &mut V {
        self.layout
            .get_inner_mut()
            .get_inner_mut()
            .get_child_mut(i)
            .and_then(|tab| tab.downcast_mut())
            .unwrap()
    }

    fn update_bar(&mut self) {
        let tab_view = self.get_view::<ResizedView<TabView>>(1).get_inner();
        let tab_names = tab_view.tab_order();

        let text = tab_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let mut t = match i == self.index {
                    true => StyledString::styled(name, BaseColor::Cyan.light()),
                    false => StyledString::plain(name),
                };
                t.append_plain(if i + 1 == tab_names.len() { "" } else { " > " });
                t
            })
            .collect::<StyledString>();

        self.get_view_mut::<ResizedView<TextView>>(0)
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

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
