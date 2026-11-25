//! Buy a truck, or a trailer: one of those the game's dealers sell ready-made
//! (`gamedata::dealer_trucks`, `gamedata::dealer_trailers`), added to the company when saving, in a
//! garage with room, for nothing. The lists have the base game's and those of the DLCs this save
//! was made with.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    notification::{Notification, NotificationType},
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, prelude::FluentBuilder as _, px,
};

use crate::game_data;
use crate::i18n::t;
use crate::session::{BoughtTrailer, BoughtTruck, Session};
use crate::ui::{Choice, DetailChoice};

/// What's being bought.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buying {
    Truck,
    Trailer,
    /// A coach (Coaches DLC): it joins the player's coaches, so there's no garage to pick.
    Coach,
}

/// A purchase, ready for the session.
enum Purchase {
    Truck(BoughtTruck),
    Trailer(BoughtTrailer),
    Coach(BoughtTruck),
}

pub struct BuyForm {
    session: Entity<Session>,
    buying: Buying,
    vehicle: Entity<SelectState<SearchableVec<DetailChoice>>>,
    garage: Entity<SelectState<SearchableVec<Choice>>>,
    /// Whether the lists have been filled (the dealers' stock is read on first use).
    is_filled: bool,
    _subscriptions: Vec<Subscription>,
}

impl BuyForm {
    fn new(
        session: Entity<Session>,
        buying: Buying,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let select = |window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| {
                SelectState::new(SearchableVec::new(Vec::new()), None, window, cx).searchable(true)
            })
        };
        // The vehicles' list has two lines an item; the garages' one.
        let vehicle = cx.new(|cx| {
            SelectState::new(SearchableVec::new(Vec::new()), None, window, cx).searchable(true)
        });
        let garage = select(window, cx);
        // Picking (the only event) changes whether Buy can be pressed.
        let _subscriptions = vec![
            cx.subscribe(
                &vehicle,
                |_, _, _: &SelectEvent<SearchableVec<DetailChoice>>, cx| cx.notify(),
            ),
            cx.subscribe(
                &garage,
                |_, _, _: &SelectEvent<SearchableVec<Choice>>, cx| cx.notify(),
            ),
        ];
        Self {
            session,
            buying,
            vehicle,
            garage,
            is_filled: false,
            _subscriptions,
        }
    }

    /// The garages with room for what's being bought, as the save will be after the changes.
    fn targets(&self, cx: &App) -> Vec<String> {
        let session = self.session.read(cx);
        match self.buying {
            Buying::Truck => session.buy_targets(),
            Buying::Trailer => session.trailer_targets(),
            Buying::Coach => Vec::new(),
        }
    }

    /// The dealer's stock as choices (this save's DLCs only), once it's read.
    fn choices(&self, cx: &mut Context<Self>) -> Option<Vec<DetailChoice>> {
        let currency = self.session.read(cx).game().currency_symbol();
        let price = |price: i64| format!("{currency} {}", crate::format::thousands(price));
        match self.buying {
            Buying::Truck | Buying::Coach => {
                let trucks = game_data::dealer_trucks(cx)?;
                let session = self.session.read(cx);
                let wants_coach = self.buying == Buying::Coach;
                Some(
                    trucks
                        .iter()
                        .filter(|truck| truck.is_coach == wants_coach)
                        .filter(|truck| session.has_dlc(truck.dlc.as_deref()))
                        .map(|truck| DetailChoice {
                            value: truck.path.clone().into(),
                            name: game_data::model_name_by_id(cx, &truck.model).into(),
                            detail: t!(
                                cx,
                                "buy-truck-option",
                                chassis = game_data::resolve(cx, &truck.chassis),
                                engine = game_data::resolve(cx, &truck.engine),
                                price = price(truck.price)
                            ),
                        })
                        .collect(),
                )
            }
            Buying::Trailer => {
                let trailers = game_data::dealer_trailers(cx)?;
                let session = self.session.read(cx);
                Some(
                    trailers
                        .iter()
                        .filter(|trailer| session.has_dlc(trailer.dlc.as_deref()))
                        .map(|trailer| DetailChoice {
                            value: trailer.path.clone().into(),
                            name: game_data::trailer_name(cx, &trailer.model, &trailer.body_type)
                                .into(),
                            detail: if trailer.trailers > 1 {
                                t!(
                                    cx,
                                    "buy-trailer-option-train",
                                    chassis = game_data::resolve(cx, &trailer.chassis),
                                    count = trailer.trailers,
                                    price = price(trailer.price)
                                )
                            } else {
                                t!(
                                    cx,
                                    "buy-trailer-option",
                                    chassis = game_data::resolve(cx, &trailer.chassis),
                                    price = price(trailer.price)
                                )
                            },
                        })
                        .collect(),
                )
            }
        }
    }

    /// Fills both lists once the dealer's stock is read.
    fn fill(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_filled {
            return;
        }
        let Some(choices) = self.choices(cx) else {
            return;
        };
        let garages: Vec<Choice> = self
            .targets(cx)
            .into_iter()
            .map(|garage| Choice {
                name: game_data::city_name(cx, garage.strip_prefix("garage.").unwrap_or(&garage))
                    .into(),
                value: garage.into(),
            })
            .collect();
        // The only garage with room is chosen already: nothing to pick.
        let only_garage = (garages.len() == 1).then(|| garages[0].value.clone());
        self.vehicle.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(choices), window, cx)
        });
        self.garage.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(garages), window, cx);
            if let Some(garage) = &only_garage {
                select.set_selected_value(garage, window, cx);
            }
        });
        self.is_filled = true;
    }

    /// What would be bought, or why Buy can't be pressed yet.
    fn purchase(&self, cx: &mut Context<Self>) -> Result<Purchase, SharedString> {
        if !game_data::has_data(cx) {
            return Err(if game_data::is_loading(cx) {
                t!(cx, "buy-reading")
            } else {
                t!(cx, "buy-needs-game")
            });
        }
        if !self.is_filled {
            return Err(t!(cx, "buy-reading"));
        }
        // Made up front: a closure making it would hold on to `cx`, which the lookups below need.
        let choose_text: SharedString = match self.buying {
            Buying::Truck => t!(cx, "buy-truck-choose"),
            Buying::Trailer => t!(cx, "buy-trailer-choose"),
            Buying::Coach => t!(cx, "buy-coach-choose"),
        };
        let choose = || choose_text.clone();
        // A coach needs no garage: it joins the player's coaches, its plate made for the
        // headquarters' country (as the game gave one it sold, in ETS2).
        if self.buying == Buying::Coach {
            let path = selected(&self.vehicle, cx).ok_or_else(choose)?;
            let trucks = game_data::dealer_trucks(cx).ok_or_else(choose)?;
            let coach = trucks
                .iter()
                .find(|truck| truck.path == path)
                .ok_or_else(choose)?;
            let hq = format!("garage.{}", self.session.read(cx).career().hq_city);
            return Ok(Purchase::Coach(BoughtTruck {
                model: coach.model.clone(),
                accessories: coach.accessories.clone(),
                garage: String::new(),
                plates: game_data::new_plates(cx, &hq, "bus", 1),
            }));
        }
        if self.targets(cx).is_empty() {
            return Err(match self.buying {
                Buying::Trailer => t!(cx, "buy-trailer-no-room"),
                _ => t!(cx, "buy-truck-no-room"),
            });
        }
        let (Some(path), Some(garage)) = (selected(&self.vehicle, cx), selected(&self.garage, cx))
        else {
            return Err(choose());
        };
        match self.buying {
            Buying::Truck => {
                let trucks = game_data::dealer_trucks(cx).ok_or_else(choose)?;
                let truck = trucks
                    .iter()
                    .find(|truck| truck.path == path)
                    .ok_or_else(choose)?;
                let plates = game_data::new_plates(cx, &garage, "truck", 1);
                Ok(Purchase::Truck(BoughtTruck {
                    model: truck.model.clone(),
                    accessories: truck.accessories.clone(),
                    garage,
                    plates,
                }))
            }
            Buying::Trailer => {
                let trailers = game_data::dealer_trailers(cx).ok_or_else(choose)?;
                let trailer = trailers
                    .iter()
                    .find(|trailer| trailer.path == path)
                    .ok_or_else(choose)?;
                // A plate for each trailer of the train (a dolly has one too).
                let count = trailer
                    .units
                    .iter()
                    .filter(|unit| unit.kind == "trailer")
                    .count();
                let plates = game_data::new_plates(cx, &garage, "trailer", count);
                Ok(Purchase::Trailer(BoughtTrailer {
                    model: trailer.model.clone(),
                    body_type: trailer.body_type.clone(),
                    units: trailer.units.clone(),
                    definitions: trailer.definitions.clone(),
                    garage,
                    plates,
                }))
            }
            // Handled above, before any garage is asked for.
            Buying::Coach => Err(choose()),
        }
    }

    fn buy(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Ok(purchase) = self.purchase(cx) else {
            return;
        };
        let title = match &purchase {
            Purchase::Truck(_) => t!(cx, "buy-truck-added"),
            Purchase::Trailer(_) => t!(cx, "buy-trailer-added"),
            Purchase::Coach(_) => t!(cx, "buy-coach-added"),
        };
        self.session.update(cx, |session, cx| match purchase {
            Purchase::Truck(truck) => session.buy_truck(truck, cx),
            Purchase::Trailer(trailer) => session.buy_trailer(trailer, cx),
            Purchase::Coach(coach) => session.buy_coach(coach, cx),
        });
        window.close_sheet(cx);
        window.push_notification(
            Notification::new()
                .with_type(NotificationType::Success)
                .title(title),
            cx,
        );
    }
}

impl Render for BuyForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.fill(window, cx);
        let purchase = self.purchase(cx);
        let why_not = purchase.as_ref().err().cloned();
        let placeholder = if self.is_filled {
            t!(cx, "new-job-pick")
        } else {
            t!(cx, "truck-part-loading")
        };
        let (description, label) = match self.buying {
            Buying::Truck => (t!(cx, "buy-truck-description"), t!(cx, "buy-truck-model")),
            Buying::Trailer => (
                t!(cx, "buy-trailer-description"),
                t!(cx, "buy-trailer-model"),
            ),
            Buying::Coach => (t!(cx, "buy-coach-description"), t!(cx, "buy-coach-model")),
        };
        v_flex()
            .pt_2()
            .gap_4()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(description),
            )
            .child(field(label, &self.vehicle, placeholder.clone()))
            // Coaches aren't kept in garages.
            .when(self.buying != Buying::Coach, |this| {
                this.child(field(t!(cx, "column-garage"), &self.garage, placeholder))
            })
            .child(
                v_flex()
                    .gap_2()
                    .items_start()
                    .child(
                        Button::new("buy")
                            .small()
                            .primary()
                            .label(t!(cx, "buy-button"))
                            .disabled(purchase.is_err())
                            .on_click(cx.listener(|this, _, window, cx| this.buy(window, cx))),
                    )
                    .children(why_not.map(|why| {
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(why)
                    })),
            )
    }
}

/// The value picked in a list (an id), if any. Generic over the list's items: the vehicles' list
/// has two lines an item (`DetailChoice`), the garages' one (`Choice`).
fn selected<T>(select: &Entity<SelectState<SearchableVec<T>>>, cx: &App) -> Option<String>
where
    T: SelectItem<Value = SharedString> + 'static,
{
    select
        .read(cx)
        .selected_value()
        .map(|value| value.to_string())
}

/// A labelled list to pick from.
fn field<T>(
    label: SharedString,
    select: &Entity<SelectState<SearchableVec<T>>>,
    placeholder: SharedString,
) -> impl IntoElement
where
    T: SelectItem<Value = SharedString> + 'static,
{
    v_flex()
        .gap_1p5()
        .child(div().text_sm().child(label))
        .child(Select::new(select).small().placeholder(placeholder))
}

/// Opens Buy a truck (or a trailer) in a side panel.
pub fn open(session: Entity<Session>, buying: Buying, window: &mut Window, cx: &mut App) {
    let view = cx.new(|cx| BuyForm::new(session, buying, window, cx));
    window.open_sheet(cx, move |sheet, _, cx| {
        sheet
            .title(match buying {
                Buying::Truck => t!(cx, "buy-truck-title"),
                Buying::Trailer => t!(cx, "buy-trailer-title"),
                Buying::Coach => t!(cx, "buy-coach-title"),
            })
            .size(px(crate::pages::details::SHEET_WIDTH))
            .child(view.clone())
    });
}
