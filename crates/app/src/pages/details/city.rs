//! One city: how often the player has been there, its truck dealer and recruitment agency, and the
//! company's garage there. Each fact has its action beside it: mark the city as visited or not,
//! unlock or lock its dealer or agency, open its garage. Opened from the map (a click on the city's name) or the
//! list of cities (double-click or Enter on its row).

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable as _, WindowExt as _, button::Button, h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, ElementId, Entity, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, Subscription, Window, div, px,
};
use save::Place;

use super::{SHEET_WIDTH, fact, group};
use crate::game_data;
use crate::i18n::t;
use crate::session::Session;

/// Shows a city on the Map page's map (by token): given by whoever opens the panel there.
pub type ShowOnMap = std::rc::Rc<dyn Fn(&str, &mut Window, &mut App)>;

pub struct CityDetail {
    session: Entity<Session>,
    /// The city's token (`berlin`).
    token: String,
    show_on_map: Option<ShowOnMap>,
    _subscriptions: Vec<Subscription>,
}

impl CityDetail {
    fn new(
        session: Entity<Session>,
        token: String,
        show_on_map: Option<ShowOnMap>,
        cx: &mut Context<Self>,
    ) -> Self {
        let _subscriptions = vec![
            cx.observe(&session, |_, _, cx| cx.notify()),
            // Which cities have a dealer or agency comes from the game's data, read after startup.
            game_data::observe(cx, |_, cx| cx.notify()),
        ];
        Self {
            session,
            token,
            show_on_map,
            _subscriptions,
        }
    }
}

/// A city's dealer or agency, as far as the save and the game's data tell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlaceState {
    Unlocked,
    /// The city has one, not found yet.
    Locked,
    /// The city has none.
    Missing,
    /// Not unlocked, and without the game's data nobody knows whether the city has one.
    Unknown,
}

fn place_state(session: &Session, place: Place, token: &str, cx: &App) -> PlaceState {
    if session.unlocked(place).contains(token) {
        return PlaceState::Unlocked;
    }
    match session.place_cities(place, cx) {
        Some(cities) if cities.contains(token) => PlaceState::Locked,
        Some(_) => PlaceState::Missing,
        None => PlaceState::Unknown,
    }
}

/// A fact's value with its action beside it (none when there's nothing to do).
fn value_with_action(value: SharedString, action: Option<Button>) -> AnyElement {
    h_flex()
        .gap_2()
        .items_center()
        .child(value)
        .children(action)
        .into_any_element()
}

impl Render for CityDetail {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Everything read first: the buttons' listeners need the context mutably.
        let session = self.session.read(cx);
        let Some(city) = session
            .cities()
            .into_iter()
            .find(|city| city.token == self.token)
        else {
            return div().child(t!(cx, "detail-gone")).into_any_element();
        };
        let garage = session
            .garages()
            .into_iter()
            .find(|garage| garage.city == self.token);
        let dealer = place_state(session, Place::Dealer, &self.token, cx);
        let recruitment = place_state(session, Place::Recruitment, &self.token, cx);
        let is_unknown = dealer == PlaceState::Unknown || recruitment == PlaceState::Unknown;

        // Visited or not, the button turns it the other way.
        let is_visited = city.visits > 0;
        let token = self.token.clone();
        let visit = Button::new("visit-city")
            .xsmall()
            .outline()
            .label(if is_visited {
                t!(cx, "city-unvisit")
            } else {
                t!(cx, "city-visit")
            })
            .tooltip(if is_visited {
                t!(cx, "city-unvisit-tooltip")
            } else {
                t!(cx, "city-visit-tooltip")
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.session.update(cx, |session, cx| {
                    if is_visited {
                        session.unvisit_city(&token, cx);
                    } else {
                        session.visit_city(&token, cx);
                    }
                });
            }));
        let visits = value_with_action(t!(cx, "map-visits", count = city.visits), Some(visit));
        let place = |place: Place, state: PlaceState, cx: &mut Context<Self>| {
            let value = match state {
                PlaceState::Unlocked => t!(cx, "map-place-unlocked"),
                PlaceState::Locked => t!(cx, "map-place-locked"),
                PlaceState::Missing => t!(cx, "city-place-missing"),
                PlaceState::Unknown => t!(cx, "city-place-unknown"),
            };
            let id: ElementId = match place {
                Place::Dealer => "unlock-dealer".into(),
                Place::Recruitment => "unlock-recruitment".into(),
            };
            let token = self.token.clone();
            // Found or not, the button turns it the other way; nothing to do where the city has
            // none, or nobody knows.
            let is_unlocked = match state {
                PlaceState::Unlocked => true,
                PlaceState::Locked => false,
                PlaceState::Missing | PlaceState::Unknown => return value_with_action(value, None),
            };
            let action = Button::new(id)
                .xsmall()
                .outline()
                .label(if is_unlocked {
                    t!(cx, "city-lock")
                } else {
                    t!(cx, "city-unlock")
                })
                .tooltip(if is_unlocked {
                    t!(cx, "city-lock-tooltip")
                } else {
                    t!(cx, "city-unlock-tooltip")
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.session.update(cx, |session, cx| {
                        if is_unlocked {
                            session.lock(place, &token, cx);
                        } else {
                            session.unlock(place, &token, cx);
                        }
                    });
                }));
            let action = Some(action);
            value_with_action(value, action)
        };
        let dealer = place(Place::Dealer, dealer, cx);
        let recruitment = place(Place::Recruitment, recruitment, cx);
        let garage = garage.map(|garage| {
            let state: SharedString = if !garage.is_owned {
                t!(cx, "garage-not-owned")
            } else if garage.is_large() {
                t!(cx, "garage-large")
            } else {
                t!(cx, "garage-slots", count = garage.slots)
            };
            // The garage's own panel buys or upgrades it, and lists what's in it.
            let open = Button::new("open-garage")
                .xsmall()
                .outline()
                .label(t!(cx, "city-open-garage"))
                .on_click(cx.listener(move |this, _, window, cx| {
                    super::open_garage(this.session.clone(), &garage, window, cx);
                }));
            value_with_action(state, Some(open))
        });

        v_flex()
            .pt_2()
            .gap_6()
            .child(group(
                t!(cx, "detail-about"),
                v_flex()
                    .gap_1()
                    .child(fact(t!(cx, "column-visits"), visits, cx))
                    .child(fact(t!(cx, "column-dealer"), dealer, cx))
                    .child(fact(t!(cx, "column-recruitment"), recruitment, cx))
                    .children(garage.map(|garage| fact(t!(cx, "column-garage-size"), garage, cx))),
            ))
            // The map moves to the city, and the panel closes so the map shows.
            .children(self.show_on_map.clone().map(|show_on_map| {
                let token = self.token.clone();
                h_flex().child(
                    Button::new("show-on-map")
                        .small()
                        .outline()
                        .icon(Icon::new(IconName::Map))
                        .label(t!(cx, "city-show-on-map"))
                        .on_click(move |_, window, cx| {
                            show_on_map(&token, window, cx);
                            window.close_sheet(cx);
                        }),
                )
            }))
            // Not just "not known": say why, and what makes it known.
            .children(is_unknown.then(|| {
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "map-needs-game"))
            }))
            .into_any_element()
    }
}

/// Opens a city's panel, titled with its name; with `show_on_map`, it has Show on map too.
pub fn open(
    session: Entity<Session>,
    token: &str,
    show_on_map: Option<ShowOnMap>,
    window: &mut Window,
    cx: &mut App,
) {
    let title: SharedString = game_data::city_name(cx, token).into();
    let token = token.to_string();
    let view = cx.new(|cx| CityDetail::new(session, token, show_on_map, cx));
    window.open_sheet(cx, move |sheet, _, _| {
        sheet
            .title(title.clone())
            .size(px(SHEET_WIDTH))
            .child(view.clone())
    });
}
