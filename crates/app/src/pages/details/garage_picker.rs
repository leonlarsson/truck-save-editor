//! The garage a truck or driver is in, as a picker: choosing another garage moves them there, with
//! the rest of their slot (a truck's driver, a driver's truck). Only garages with a free slot are
//! offered; when it can't move, it says why.

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, rems,
};
use save::MoveProblem;

use crate::game_data;
use crate::i18n::t;
use crate::session::Session;

pub struct GaragePicker {
    session: Entity<Session>,
    /// The truck's or driver's id.
    id: String,
    /// Whether it's a truck (for the wording about who moves along).
    is_truck: bool,
    select: Entity<SelectState<SearchableVec<GarageOption>>>,
    _subscriptions: Vec<Subscription>,
}

impl GaragePicker {
    pub fn new(
        session: Entity<Session>,
        id: String,
        is_truck: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let select = cx.new(|cx| {
            SelectState::new(SearchableVec::new(Vec::new()), None, window, cx).searchable(true)
        });
        let _subscriptions = vec![
            cx.observe_in(&session, window, |this, _, window, cx| {
                this.sync(window, cx)
            }),
            cx.subscribe_in(
                &select,
                window,
                |this, _, event: &SelectEvent<SearchableVec<GarageOption>>, window, cx| {
                    let SelectEvent::Confirm(Some(garage)) = event else {
                        return;
                    };
                    let garage = garage.to_string();
                    let id = this.id.clone();
                    // Only a garage with room; anything else (the current one, a full one) just
                    // puts the picker back.
                    let has_room = this
                        .session
                        .read(cx)
                        .move_options(&id)
                        .is_ok_and(|options| options.targets.contains(&garage));
                    if has_room {
                        this.session
                            .update(cx, |session, cx| session.move_to_garage(&id, &garage, cx));
                    } else {
                        this.sync(window, cx);
                    }
                },
            ),
        ];
        let mut picker = Self {
            session,
            id,
            is_truck,
            select,
            _subscriptions,
        };
        picker.sync(window, cx);
        picker
    }

    /// The current garage first, then every garage with room, by city name.
    fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Ok(options) = self.session.read(cx).move_options(&self.id) else {
            return;
        };
        let city = |garage: &str| {
            game_data::city_name(cx, garage.strip_prefix("garage.").unwrap_or(garage))
        };
        let option = |garage: &str, name: String, is_full: bool| GarageOption {
            name: name.into(),
            id: garage.to_string().into(),
            is_full,
        };
        let current = option(&options.from, city(&options.from), false);
        // Full garages are listed too, greyed out, so a search for one finds it and says why.
        let mut others: Vec<GarageOption> = options
            .targets
            .iter()
            .map(|garage| option(garage, city(garage), false))
            .chain(options.full.iter().map(|garage| {
                let name = t!(cx, "move-garage-full", city = city(garage)).to_string();
                option(garage, name, true)
            }))
            .collect();
        others.sort_by(|a, b| a.name.cmp(&b.name));
        let mut items = vec![current.clone()];
        items.extend(others);
        self.select.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(items), window, cx);
            select.set_selected_value(&current.id, window, cx);
        });
    }
}

impl Render for GaragePicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let hint = |text: SharedString, cx: &Context<Self>| {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(text)
        };
        match self.session.read(cx).move_options(&self.id) {
            Ok(options) => {
                let has_room = !options.targets.is_empty();
                let current_city = game_data::city_name(
                    cx,
                    options
                        .from
                        .strip_prefix("garage.")
                        .unwrap_or(&options.from),
                );
                let note = if !has_room {
                    t!(cx, "move-no-room")
                } else if options.partner.is_some() {
                    if self.is_truck {
                        t!(cx, "move-with-driver")
                    } else {
                        t!(cx, "move-with-truck")
                    }
                } else {
                    t!(cx, "move-hint")
                };
                v_flex()
                    .items_end()
                    .gap_1()
                    .child(
                        div().w(rems(11.)).child(
                            Select::new(&self.select)
                                .small()
                                // Shown while a search hides the current garage from the list.
                                .placeholder(current_city)
                                .disabled(!has_room)
                                .accessibility_label(t!(cx, "column-garage")),
                        ),
                    )
                    .child(hint(note, cx))
                    .into_any_element()
            }
            Err(problem) => {
                // Not movable: the garage as plain text, and why (your own truck stays with you).
                let garage = save::garage_of(&self.session.read(cx).preview(), &self.id)
                    .map(|garage| {
                        game_data::city_name(cx, garage.strip_prefix("garage.").unwrap_or(&garage))
                    })
                    .unwrap_or_else(|| t!(cx, "no-garage").to_string());
                let why = (problem == MoveProblem::IsPlayers).then(|| t!(cx, "move-is-players"));
                v_flex()
                    .items_end()
                    .gap_1()
                    .child(div().text_sm().child(garage))
                    .children(why.map(|why| hint(why, cx)))
                    .into_any_element()
            }
        }
    }
}

/// A garage in the picker: shown by its city, chosen by its id.
#[derive(Clone)]
struct GarageOption {
    id: SharedString,
    name: SharedString,
    is_full: bool,
}

impl SelectItem for GarageOption {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.id
    }

    fn disabled(&self) -> bool {
        self.is_full
    }
}
