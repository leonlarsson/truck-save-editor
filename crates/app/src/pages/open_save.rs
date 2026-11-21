//! The start screen: every save on this computer, to pick one to edit. The player's own saves come
//! first; the game's autosaves follow. Finding them reads only the small `info.sii` of each slot, in
//! the background.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Selectable as _, Sizable as _, StyledExt as _,
    button::{Button, ButtonGroup},
    h_flex,
    list::ListItem,
    spinner::Spinner,
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, EventEmitter, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Task,
    Window, div, prelude::FluentBuilder as _, rems,
};
use save::{Game, Location, Profile, Slot};

use crate::APP_NAME;
use crate::format::{ago, thousands};

/// Asks the workspace to open a slot.
pub struct OpenSlot {
    pub profile: Profile,
    pub slot: Slot,
}

pub struct OpenSavePage {
    found: Option<Vec<(Profile, Vec<Slot>)>>,
    selected_profile: usize,
    _search: Option<Task<()>>,
}

impl EventEmitter<OpenSlot> for OpenSavePage {}

impl OpenSavePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut page = Self {
            found: None,
            selected_profile: 0,
            _search: None,
        };
        page.search(cx);
        page
    }

    /// Looks for profiles and their slots again, in the background.
    pub fn search(&mut self, cx: &mut Context<Self>) {
        self.found = None;
        cx.notify();
        self._search = Some(cx.spawn(async move |this, cx| {
            let found = cx
                .background_spawn(async move {
                    Game::ALL
                        .into_iter()
                        .flat_map(save::find_profiles)
                        .map(|profile| {
                            let slots = save::profile_slots(&profile.dir);
                            (profile, slots)
                        })
                        .filter(|(_, slots)| !slots.is_empty())
                        .collect::<Vec<_>>()
                })
                .await;
            let _ = this.update(cx, |page, cx| {
                page.found = Some(found);
                page.selected_profile = 0;
                cx.notify();
            });
        }));
    }

    fn render_profiles(
        &self,
        profiles: &[(Profile, Vec<Slot>)],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let describe = |profile: &Profile| -> String {
            let place = match profile.location {
                Location::SteamCloud => "Steam Cloud",
                Location::Local => "This PC",
            };
            match profile.game {
                Game::Ets2 => format!("{} · {place}", profile.name),
                Game::Ats => format!("{} · ATS · {place}", profile.name),
            }
        };

        if profiles.len() == 1 {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(describe(&profiles[0].0))
                .into_any_element();
        }
        ButtonGroup::new("profiles")
            .outline()
            .small()
            .children(profiles.iter().enumerate().map(|(index, (profile, _))| {
                Button::new(("profile", index))
                    .label(describe(profile))
                    .selected(index == self.selected_profile)
            }))
            .on_click(cx.listener(|this, clicked: &Vec<usize>, _, cx| {
                if let Some(&index) = clicked.first() {
                    this.selected_profile = index;
                    cx.notify();
                }
            }))
            .into_any_element()
    }

    fn render_slots(
        &self,
        profile: &Profile,
        slots: &[Slot],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (own, game_made): (Vec<&Slot>, Vec<&Slot>) =
            slots.iter().partition(|slot| slot.kind.is_manual());
        let currency = profile.game.currency_symbol();

        let group = |title: &'static str, slots: Vec<&Slot>, cx: &mut Context<Self>| {
            v_flex()
                .gap_1()
                .child(
                    div()
                        .px_3()
                        .pb_1()
                        .text_xs()
                        .font_medium()
                        .text_color(cx.theme().muted_foreground)
                        .child(title),
                )
                .children(slots.into_iter().map(|slot| {
                    let detail = {
                        let mut parts = Vec::new();
                        if let save::SlotKind::Manual(number) = slot.kind {
                            parts.push(format!("Slot {number}"));
                        }
                        if let Some(modified) = slot.modified {
                            parts.push(ago(modified));
                        }
                        parts.join(" · ")
                    };
                    let money = slot
                        .info
                        .money
                        .map(|money| format!("{currency} {}", thousands(money)))
                        .unwrap_or_default();
                    let profile = profile.clone();
                    let opened = slot.clone();
                    ListItem::new(SharedString::from(format!("slot-{}", slot.folder)))
                        .py_2()
                        .px_3()
                        .rounded(cx.theme().radius)
                        .accessibility_label(slot.display_name())
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(OpenSlot {
                                profile: profile.clone(),
                                slot: opened.clone(),
                            });
                        }))
                        .child(
                            h_flex()
                                .w_full()
                                .justify_between()
                                .gap_4()
                                .child(
                                    v_flex()
                                        .min_w_0()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_medium()
                                                .child(slot.display_name()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(detail),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(money),
                                ),
                        )
                }))
        };

        v_flex()
            .gap_6()
            .when(!own.is_empty(), |this| {
                this.child(group("Your saves", own, cx))
            })
            .when(!game_made.is_empty(), |this| {
                this.child(group("Saved by the game", game_made, cx))
            })
    }
}

impl Render for OpenSavePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = h_flex()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_10()
                    .flex_shrink_0()
                    .rounded(cx.theme().radius_lg)
                    .bg(cx.theme().primary)
                    .text_color(cx.theme().primary_foreground)
                    .child(Icon::new(IconName::Truck)),
            )
            .child(
                v_flex()
                    .child(div().text_lg().font_semibold().child(APP_NAME))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Choose a save to edit"),
                    ),
            );

        let body = match &self.found {
            None => h_flex()
                .gap_2()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(Spinner::new().small())
                .child("Looking for your saves…")
                .into_any_element(),
            Some(profiles) if profiles.is_empty() => v_flex()
                .gap_3()
                .items_start()
                .child(div().font_medium().child("No saves found"))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(
                            "Saves are looked for in Steam Cloud and in Documents\\Euro Truck \
                             Simulator 2\\profiles. Save once in the game, then look again.",
                        ),
                )
                .child(
                    Button::new("search-again")
                        .outline()
                        .label("Look again")
                        .on_click(cx.listener(|this, _, _, cx| this.search(cx))),
                )
                .into_any_element(),
            Some(profiles) => {
                let selected = self.selected_profile.min(profiles.len() - 1);
                let (profile, slots) = &profiles[selected];
                v_flex()
                    .gap_6()
                    .child(self.render_profiles(profiles, cx))
                    .child(self.render_slots(profile, slots, cx))
                    .into_any_element()
            }
        };

        div()
            .id("open-save-scroll")
            .size_full()
            .overflow_y_scroll()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                v_flex()
                    .w_full()
                    .max_w(rems(40.))
                    .mx_auto()
                    .px_8()
                    .py_12()
                    .gap_8()
                    .child(header)
                    .child(body),
            )
    }
}
