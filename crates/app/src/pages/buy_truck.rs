//! Buy a truck: one of the trucks the game's dealers sell ready-made (`gamedata::dealer_trucks`),
//! added to the company when saving, in a garage with a free truck slot, for nothing. The list has
//! the trucks of the base game and of the DLCs this save was made with.

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
use crate::session::{BoughtTruck, Session};
use crate::ui::Choice;

pub struct BuyTruckForm {
    session: Entity<Session>,
    truck: Entity<SelectState<SearchableVec<Choice>>>,
    garage: Entity<SelectState<SearchableVec<Choice>>>,
    /// Whether the lists have been filled (the dealers' trucks are read on first use).
    is_filled: bool,
    _subscriptions: Vec<Subscription>,
}

impl BuyTruckForm {
    fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let select = |window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| {
                SelectState::new(SearchableVec::new(Vec::new()), None, window, cx).searchable(true)
            })
        };
        let (truck, garage) = (select(window, cx), select(window, cx));
        // Picking (the only event) changes whether Buy can be pressed.
        let _subscriptions = vec![
            cx.subscribe(
                &truck,
                |_, _, _: &SelectEvent<SearchableVec<Choice>>, cx| cx.notify(),
            ),
            cx.subscribe(
                &garage,
                |_, _, _: &SelectEvent<SearchableVec<Choice>>, cx| cx.notify(),
            ),
        ];
        Self {
            session,
            truck,
            garage,
            is_filled: false,
            _subscriptions,
        }
    }

    /// Fills both lists once the dealers' trucks are read: the trucks this save can have (the base
    /// game's and its DLCs'), and the garages with a free slot.
    fn fill(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_filled {
            return;
        }
        let Some(trucks) = game_data::dealer_trucks(cx) else {
            return;
        };
        let session = self.session.read(cx);
        let currency = session.game().currency_symbol();
        let choices: Vec<Choice> = trucks
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
                    price = format!("{currency} {}", crate::format::thousands(truck.price))
                ),
            })
            .collect();
        let garages: Vec<Choice> = session
            .buy_targets()
            .into_iter()
            .map(|garage| Choice {
                name: game_data::city_name(cx, garage.strip_prefix("garage.").unwrap_or(&garage))
                    .into(),
                value: garage.into(),
            })
            .collect();
        // The only garage with room is chosen already: nothing to pick.
        let only_garage = (garages.len() == 1).then(|| garages[0].value.clone());
        self.truck.update(cx, |select, cx| {
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
    fn purchase(&self, cx: &mut Context<Self>) -> Result<BoughtTruck, SharedString> {
        if !game_data::has_data(cx) {
            return Err(if game_data::is_loading(cx) {
                t!(cx, "buy-truck-reading")
            } else {
                t!(cx, "buy-truck-needs-game")
            });
        }
        let Some(trucks) = game_data::dealer_trucks(cx) else {
            return Err(t!(cx, "buy-truck-reading"));
        };
        if self.session.read(cx).buy_targets().is_empty() {
            return Err(t!(cx, "buy-truck-no-room"));
        }
        let (Some(path), Some(garage)) = (
            Self::selected(&self.truck, cx),
            Self::selected(&self.garage, cx),
        ) else {
            return Err(t!(cx, "buy-truck-choose"));
        };
        let truck = trucks
            .iter()
            .find(|truck| truck.path == path)
            .ok_or_else(|| t!(cx, "buy-truck-choose"))?;
        Ok(BoughtTruck {
            model: truck.model.clone(),
            accessories: truck.accessories.clone(),
            garage,
        })
    }

    fn buy(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Ok(truck) = self.purchase(cx) else {
            return;
        };
        self.session
            .update(cx, |session, cx| session.buy_truck(truck, cx));
        window.close_sheet(cx);
        window.push_notification(
            Notification::new()
                .with_type(NotificationType::Success)
                .title(t!(cx, "buy-truck-added")),
            cx,
        );
    }
}

impl Render for BuyTruckForm {
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
        v_flex()
            .pt_2()
            .gap_4()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "buy-truck-description")),
            )
            .child(field(
                t!(cx, "buy-truck-model"),
                &self.truck,
                placeholder.clone(),
            ))
            .child(field(t!(cx, "column-garage"), &self.garage, placeholder))
            .child(
                v_flex()
                    .gap_2()
                    .items_start()
                    .child(
                        Button::new("buy-truck")
                            .small()
                            .primary()
                            .label(t!(cx, "buy-truck-buy"))
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

/// Opens Buy a truck in a side panel.
pub fn open(session: Entity<Session>, window: &mut Window, cx: &mut App) {
    let view = cx.new(|cx| BuyTruckForm::new(session, window, cx));
    window.open_sheet(cx, move |sheet, _, cx| {
        sheet
            .title(t!(cx, "buy-truck-title"))
            .size(px(crate::pages::details::SHEET_WIDTH))
            .child(view.clone())
    });
}
