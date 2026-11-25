//! Buy a truck, or a trailer: one of those the game's dealers sell ready-made
//! (`gamedata::dealer_trucks`, `gamedata::dealer_trailers`), added to the company when saving, in a
//! garage with room, for nothing. The lists have the base game's and those of the DLCs this save
//! was made with.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    notification::{Notification, NotificationType},
    select::{SearchableVec, Select, SelectEvent, SelectState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, px,
};

use crate::game_data;
use crate::i18n::t;
use crate::session::{BoughtTrailer, BoughtTruck, Session};
use crate::ui::Choice;

/// What's being bought.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buying {
    Truck,
    Trailer,
}

/// A purchase, ready for the session.
enum Purchase {
    Truck(BoughtTruck),
    Trailer(BoughtTrailer),
}

pub struct BuyForm {
    session: Entity<Session>,
    buying: Buying,
    vehicle: Entity<SelectState<SearchableVec<Choice>>>,
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
        let (vehicle, garage) = (select(window, cx), select(window, cx));
        // Picking (the only event) changes whether Buy can be pressed.
        let _subscriptions = vec![
            cx.subscribe(
                &vehicle,
                |_, _, _: &SelectEvent<SearchableVec<Choice>>, cx| cx.notify(),
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
        }
    }

    /// The dealer's stock as choices (this save's DLCs only), once it's read.
    fn choices(&self, cx: &mut Context<Self>) -> Option<Vec<Choice>> {
        let currency = self.session.read(cx).game().currency_symbol();
        let price = |price: i64| format!("{currency} {}", crate::format::thousands(price));
        match self.buying {
            Buying::Truck => {
                let trucks = game_data::dealer_trucks(cx)?;
                let session = self.session.read(cx);
                Some(
                    trucks
                        .iter()
                        .filter(|truck| session.has_dlc(truck.dlc.as_deref()))
                        .map(|truck| Choice {
                            value: truck.path.clone().into(),
                            name: t!(
                                cx,
                                "buy-truck-option",
                                model = game_data::model_name_by_id(cx, &truck.model),
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
                        .map(|trailer| Choice {
                            value: trailer.path.clone().into(),
                            name: t!(
                                cx,
                                "buy-trailer-option",
                                model =
                                    game_data::trailer_name(cx, &trailer.model, &trailer.body_type),
                                chassis = game_data::resolve(cx, &trailer.chassis),
                                price = price(trailer.price)
                            ),
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

    fn selected(select: &Entity<SelectState<SearchableVec<Choice>>>, cx: &App) -> Option<String> {
        select
            .read(cx)
            .selected_value()
            .map(|value| value.to_string())
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
        if self.targets(cx).is_empty() {
            return Err(match self.buying {
                Buying::Truck => t!(cx, "buy-truck-no-room"),
                Buying::Trailer => t!(cx, "buy-trailer-no-room"),
            });
        }
        // Made up front: a closure making it would hold on to `cx`, which the lookups below need.
        let choose_text: SharedString = match self.buying {
            Buying::Truck => t!(cx, "buy-truck-choose"),
            Buying::Trailer => t!(cx, "buy-trailer-choose"),
        };
        let choose = || choose_text.clone();
        let (Some(path), Some(garage)) = (
            Self::selected(&self.vehicle, cx),
            Self::selected(&self.garage, cx),
        ) else {
            return Err(choose());
        };
        match self.buying {
            Buying::Truck => {
                let trucks = game_data::dealer_trucks(cx).ok_or_else(choose)?;
                let truck = trucks
                    .iter()
                    .find(|truck| truck.path == path)
                    .ok_or_else(choose)?;
                Ok(Purchase::Truck(BoughtTruck {
                    model: truck.model.clone(),
                    accessories: truck.accessories.clone(),
                    garage,
                }))
            }
            Buying::Trailer => {
                let trailers = game_data::dealer_trailers(cx).ok_or_else(choose)?;
                let trailer = trailers
                    .iter()
                    .find(|trailer| trailer.path == path)
                    .ok_or_else(choose)?;
                Ok(Purchase::Trailer(BoughtTrailer {
                    model: trailer.model.clone(),
                    body_type: trailer.body_type.clone(),
                    units: trailer.units.clone(),
                    definitions: trailer.definitions.clone(),
                    garage,
                }))
            }
        }
    }

    fn buy(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Ok(purchase) = self.purchase(cx) else {
            return;
        };
        let title = match &purchase {
            Purchase::Truck(_) => t!(cx, "buy-truck-added"),
            Purchase::Trailer(_) => t!(cx, "buy-trailer-added"),
        };
        self.session.update(cx, |session, cx| match purchase {
            Purchase::Truck(truck) => session.buy_truck(truck, cx),
            Purchase::Trailer(trailer) => session.buy_trailer(trailer, cx),
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
        let field = |label: SharedString,
                     select: &Entity<SelectState<SearchableVec<Choice>>>,
                     placeholder: SharedString| {
            v_flex()
                .gap_1p5()
                .child(div().text_sm().child(label))
                .child(Select::new(select).small().placeholder(placeholder))
        };
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
            .child(field(t!(cx, "column-garage"), &self.garage, placeholder))
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

/// Opens Buy a truck (or a trailer) in a side panel.
pub fn open(session: Entity<Session>, buying: Buying, window: &mut Window, cx: &mut App) {
    let view = cx.new(|cx| BuyForm::new(session, buying, window, cx));
    window.open_sheet(cx, move |sheet, _, cx| {
        sheet
            .title(match buying {
                Buying::Truck => t!(cx, "buy-truck-title"),
                Buying::Trailer => t!(cx, "buy-trailer-title"),
            })
            .size(px(crate::pages::details::SHEET_WIDTH))
            .child(view.clone())
    });
}
