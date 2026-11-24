//! World map (the game's name for its map): the roads, driven or not, the player's truck and
//! garages, the cities, viewpoints and photo trophies (see `map_view`). Discover every road or
//! forget them, move the truck, and find a city on the map. Cities and Gallery, the other pages of
//! the sidebar's Map group, show a city or a viewpoint here (`workspace::show_on_world_map`).

use gpui_kit::component::{
    Disableable as _, Sizable as _, WindowExt as _,
    button::Button,
    h_flex,
    select::{SearchableVec, Select, SelectEvent, SelectState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    Subscription, Window, div,
};

use crate::game_data;
use crate::i18n::t;
use crate::pages::map_view::MapView;
use crate::session::Session;
use crate::ui::Choice;
use crate::ui::Tip as _;

/// Where to move the map: a city (by token), or a point in the game's metres.
#[derive(Clone, Debug, PartialEq)]
pub enum MapSpot {
    City(String),
    Point(f32, f32),
}

pub struct WorldMapPage {
    session: Entity<Session>,
    /// Moving the player's truck to another save's spot.
    position: Entity<crate::pages::position::PositionPicker>,
    map_view: Entity<MapView>,
    /// Find a city on the map.
    finder: Entity<SelectState<SearchableVec<Choice>>>,
    _subscriptions: Vec<Subscription>,
}

impl WorldMapPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let position = {
            let session = session.clone();
            cx.new(|cx| crate::pages::position::PositionPicker::new(session, window, cx))
        };
        let finder = cx.new(|cx| {
            SelectState::new(SearchableVec::new(Vec::new()), None, window, cx).searchable(true)
        });
        let map_view = {
            let session = session.clone();
            cx.new(|cx| MapView::new(session, cx))
        };
        let _subscriptions = vec![
            cx.observe(&session, |_, _, cx| cx.notify()),
            cx.subscribe_in(
                &finder,
                window,
                |this, _, event: &SelectEvent<SearchableVec<Choice>>, window, cx| {
                    if let SelectEvent::Confirm(Some(token)) = event {
                        let token = token.to_string();
                        this.map_view
                            .update(cx, |map, cx| map.find_city(&token, cx));
                        // An action, not a setting: the list goes back to its prompt.
                        this.finder
                            .update(cx, |select, cx| select.set_selected_index(None, window, cx));
                    }
                },
            ),
            game_data::observe_in(window, cx, |this, window, cx| this.fill_finder(window, cx)),
        ];
        let mut page = Self {
            session,
            position,
            map_view,
            finder,
            _subscriptions,
        };
        page.fill_finder(window, cx);
        // Development builds only: `TSE_FIND_CITY=hamburg` opens the map on that city, close up
        // (for screenshots of the markers).
        if cfg!(debug_assertions)
            && let Ok(token) = std::env::var("TSE_FIND_CITY")
        {
            page.show(MapSpot::City(token), cx);
        }
        page
    }

    /// The cities to find, by name (in the game's language, so again when its data is read).
    fn fill_finder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut cities: Vec<Choice> = self
            .session
            .read(cx)
            .cities()
            .into_iter()
            .map(|city| Choice {
                name: game_data::city_name(cx, &city.token).into(),
                value: city.token.into(),
            })
            .collect();
        cities.sort_by(|a, b| a.name.cmp(&b.name));
        self.finder.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(cities), window, cx)
        });
    }

    /// Moves the map to a city or a point, close up (Show on map from another page).
    pub fn show(&mut self, spot: MapSpot, cx: &mut Context<Self>) {
        self.map_view.update(cx, |map, cx| match &spot {
            MapSpot::City(token) => map.find_city(token, cx),
            MapSpot::Point(x, z) => map.find_point(*x, *z, cx),
        });
        cx.notify();
    }
}

impl Render for WorldMapPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Where the player's truck stands: in a city, out on the road, or nowhere yet.
        let preview = self.session.read(cx).preview();
        let where_truck = match save::position(&preview) {
            Some(save::Position {
                city: Some(city), ..
            }) => t!(
                cx,
                "player-position-now",
                place = game_data::city_name(cx, &city)
            ),
            Some(_) => t!(cx, "player-position-away"),
            None if save::own_truck(&preview).is_none() => t!(cx, "player-position-no-truck"),
            None => t!(cx, "player-position-unknown"),
        };
        v_flex()
            .size_full()
            .gap_4()
            .px_8()
            .py_6()
            .child(
                h_flex()
                    .justify_between()
                    .items_start()
                    .gap_4()
                    .flex_wrap()
                    .gap_y_2()
                    // The truck, and moving it to another save's spot. A column, so a note under
                    // the buttons doesn't move them.
                    .child(
                        v_flex()
                            .gap_2()
                            .items_start()
                            .child(div().text_sm().child(where_truck))
                            .child(self.position.clone()),
                    )
                    .child(
                        h_flex()
                            .flex_shrink_0()
                            .gap_2()
                            .child(
                                Button::new("forget-roads")
                                    .small()
                                    .outline()
                                    .label(t!(cx, "map-forget-roads"))
                                    .tip(t!(cx, "map-forget-roads-tooltip"))
                                    .disabled(self.session.read(cx).discovered_roads().1 == 0)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.session
                                            .update(cx, |session, cx| session.forget_roads(cx));
                                    })),
                            )
                            .child({
                                // Needs the game's map, read in the background after startup.
                                let roads = game_data::road_ids(cx);
                                let (_, after) = self.session.read(cx).discovered_roads();
                                let is_all = roads.as_ref().is_some_and(|ids| after >= ids.len());
                                Button::new("discover-roads")
                                    .small()
                                    .outline()
                                    .label(t!(cx, "map-discover-roads"))
                                    .tip(if roads.is_some() {
                                        t!(cx, "map-discover-roads-tooltip")
                                    } else {
                                        t!(cx, "map-discover-roads-needs-game")
                                    })
                                    .disabled(roads.is_none() || is_all)
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        if let Some(ids) = roads.clone() {
                                            confirm_discover_roads(
                                                this.session.clone(),
                                                ids,
                                                window,
                                                cx,
                                            );
                                        }
                                    }))
                            }),
                    ),
            )
            .child(
                div().w(gpui_kit::rems(14.)).child(
                    Select::new(&self.finder)
                        .small()
                        .icon(gpui_kit::component::Icon::new(
                            gpui_kit::assets::IconName::Search,
                        ))
                        .placeholder(t!(cx, "map-view-find")),
                ),
            )
            .child(div().flex_1().min_h_0().child(self.map_view.clone()))
    }
}

/// Asks before marking every road as driven: it may count toward the game's road achievements, and
/// nobody knows for sure whether it does.
fn confirm_discover_roads(
    session: Entity<Session>,
    ids: std::sync::Arc<Vec<u64>>,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let session = session.clone();
        let ids = ids.clone();
        alert
            .title(t!(cx, "map-discover-roads-confirm-title"))
            .description(t!(cx, "map-discover-roads-confirm-body"))
            .confirm()
            // The kit's own Cancel is English; ours follows the app's language.
            .cancel_text(t!(cx, "cancel"))
            .ok_text(t!(cx, "map-discover-roads"))
            .on_ok(move |_, _, cx| {
                let ids = ids.clone();
                session.update(cx, |session, cx| session.discover_roads(ids, cx));
                true
            })
    });
}
