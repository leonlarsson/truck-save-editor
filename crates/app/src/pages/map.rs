//! Map: every city, and how often the player has been there. Visit them all in one go.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    table::{DataTable, TableState},
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    Subscription, Window, div,
};

use crate::game_data;
use crate::i18n::t;
use crate::session::Session;
use crate::ui::{RowLook, TextColumn, TextRow, TextTable};

pub struct MapPage {
    session: Entity<Session>,
    table: Entity<TableState<TextTable>>,
    visited: usize,
    count: usize,
    _subscriptions: Vec<Subscription>,
}

impl MapPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let columns = vec![
            TextColumn {
                key: "city",
                title: t!(cx, "column-city"),
                width: 240.,
            },
            TextColumn {
                key: "visits",
                title: t!(cx, "column-visits"),
                width: 140.,
            },
        ];
        let table = cx.new(|cx| TableState::new(TextTable::new(columns), window, cx));
        let _subscriptions = vec![cx.observe(&session, |this, _, cx| this.reload(cx))];
        let mut page = Self {
            session,
            table,
            visited: 0,
            count: 0,
            _subscriptions,
        };
        page.reload(cx);
        page
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let mut cities: Vec<(String, u32)> = self
            .session
            .read(cx)
            .cities()
            .into_iter()
            .map(|city| (game_data::city_name(cx, &city.token), city.visits))
            .collect();
        cities.sort_by(|a, b| a.0.cmp(&b.0));
        self.count = cities.len();
        self.visited = cities.iter().filter(|(_, visits)| *visits > 0).count();
        let rows = cities
            .into_iter()
            .map(|(name, visits)| TextRow {
                cells: vec![name.into(), t!(cx, "map-visits", count = visits)],
                look: if visits == 0 {
                    RowLook::Muted
                } else {
                    RowLook::Normal
                },
            })
            .collect();
        self.table.update(cx, |table, cx| {
            table.delegate_mut().rows = rows;
            table.refresh(cx);
            cx.notify();
        });
        cx.notify();
    }
}

impl Render for MapPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_4()
            .px_8()
            .py_6()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(t!(
                                cx,
                                "map-summary",
                                visited = self.visited,
                                count = self.count
                            )),
                    )
                    .child(
                        Button::new("visit-all")
                            .small()
                            .primary()
                            .label(t!(cx, "map-visit-all"))
                            .tooltip(t!(cx, "map-visit-all-tooltip"))
                            .disabled(self.visited == self.count)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.session
                                    .update(cx, |session, cx| session.visit_all_cities(cx));
                            })),
                    ),
            )
            .child(
                div().flex_1().min_h_0().child(
                    DataTable::new(&self.table)
                        .stripe(true)
                        .bordered(true)
                        .accessibility_label(t!(cx, "map-table")),
                ),
            )
    }
}
