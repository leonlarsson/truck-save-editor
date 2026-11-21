//! The start screen: every save on this computer, to pick one to edit. The player's own saves come
//! first; the game's autosaves follow. Finding them reads only the small `info.sii` of each slot, in
//! the background.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Selectable as _, Sizable as _, StyledExt as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    h_flex,
    list::ListItem,
    spinner::Spinner,
    v_flex,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use gpui_kit::{
    AppContext as _, Context, EventEmitter, InteractiveElement as _, IntoElement, ObjectFit,
    ParentElement as _, Render, RenderImage, SharedString, StatefulInteractiveElement as _,
    Styled as _, StyledImage as _, Task, Window, div, img, prelude::FluentBuilder as _, rems,
};
use save::{Game, Profile, Slot};

use crate::APP_NAME;
use crate::format::thousands;
use crate::i18n::t;
use crate::labels;

/// Asks the workspace to open a slot.
pub struct OpenSlot {
    pub profile: Profile,
    pub slot: Slot,
}

pub struct OpenSavePage {
    found: Option<Vec<(Profile, Vec<Slot>)>>,
    selected_profile: usize,
    /// Each slot's preview picture, by slot folder, as they load.
    previews: HashMap<PathBuf, Arc<RenderImage>>,
    _search: Option<Task<()>>,
    _previews: Option<Task<()>>,
}

impl EventEmitter<OpenSlot> for OpenSavePage {}

impl OpenSavePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut page = Self {
            found: None,
            selected_profile: 0,
            previews: HashMap::new(),
            _search: None,
            _previews: None,
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
                page.load_previews(cx);
                cx.notify();
            });
        }));
    }

    /// Reads every slot's preview picture in the background; rows show them as they arrive.
    fn load_previews(&mut self, cx: &mut Context<Self>) {
        let dirs: Vec<PathBuf> = self
            .found
            .iter()
            .flatten()
            .flat_map(|(_, slots)| slots.iter().map(|slot| slot.dir.clone()))
            .collect();
        self._previews = Some(cx.spawn(async move |this, cx| {
            let previews = cx
                .background_spawn(async move {
                    dirs.into_iter()
                        .filter_map(|dir| {
                            let bytes = std::fs::read(dir.join("preview.tga")).ok()?;
                            let picture = save::decode_tga(&bytes)?;
                            Some((dir, Arc::new(render_image(picture)?)))
                        })
                        .collect::<HashMap<_, _>>()
                })
                .await;
            let _ = this.update(cx, |page, cx| {
                page.previews = previews;
                cx.notify();
            });
        }));
    }

    fn render_profiles(
        &self,
        profiles: &[(Profile, Vec<Slot>)],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let describe = |profile: &Profile| -> SharedString {
            let place = labels::location(cx, profile.location);
            match profile.game {
                Game::Ets2 => t!(cx, "profile-place", name = &profile.name, place = place),
                Game::Ats => t!(
                    cx,
                    "profile-game-place",
                    name = &profile.name,
                    game = "ATS",
                    place = place
                ),
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

        let group = |title: SharedString, slots: Vec<&Slot>, cx: &mut Context<Self>| {
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
                            parts.push(t!(cx, "start-slot-number", number = number).to_string());
                        }
                        if let Some(modified) = slot.modified {
                            parts.push(labels::ago(cx, modified).to_string());
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
                        .accessibility_label(labels::slot_name(cx, slot))
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
                                    h_flex()
                                        .gap_3()
                                        .min_w_0()
                                        .child(thumbnail(self.previews.get(&slot.dir).cloned(), cx))
                                        .child(
                                            v_flex()
                                                .min_w_0()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_medium()
                                                        .child(labels::slot_name(cx, slot)),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(detail),
                                                ),
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
                this.child(group(t!(cx, "start-your-saves"), own, cx))
            })
            .when(!game_made.is_empty(), |this| {
                this.child(group(t!(cx, "start-game-saves"), game_made, cx))
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
                    .flex_1()
                    .child(div().text_lg().font_semibold().child(APP_NAME))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(t!(cx, "start-choose")),
                    ),
            )
            // Settings before a save is open too: the language, and the game's folder if it
            // wasn't found.
            .child(
                Button::new("start-settings")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::Settings))
                    .label(t!(cx, "settings-open"))
                    .on_click(|_, window, cx| crate::pages::settings::open_sheet(window, cx)),
            );

        let body = match &self.found {
            None => h_flex()
                .gap_2()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(Spinner::new().small())
                .child(t!(cx, "start-looking"))
                .into_any_element(),
            Some(profiles) if profiles.is_empty() => v_flex()
                .px_3()
                .gap_3()
                .items_start()
                .child(div().font_medium().child(t!(cx, "start-none-title")))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(t!(cx, "start-none-body")),
                )
                .child(
                    Button::new("search-again")
                        .outline()
                        .label(t!(cx, "start-look-again"))
                        .on_click(cx.listener(|this, _, _, cx| this.search(cx))),
                )
                .into_any_element(),
            Some(profiles) => {
                let selected = self.selected_profile.min(profiles.len() - 1);
                let (profile, slots) = &profiles[selected];
                v_flex()
                    .gap_6()
                    .child(div().px_3().child(self.render_profiles(profiles, cx)))
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
                    // Everything on the slot rows' text edge: the rows pad their content so their
                    // hover fill reaches past it; the header and the rest line up with that text.
                    .child(div().px_3().child(header))
                    .child(body),
            )
    }
}

/// A preview picture as GPUI draws it (the decoder already gives blue-green-red-alpha order).
fn render_image(picture: save::Picture) -> Option<RenderImage> {
    let buffer = image::RgbaImage::from_raw(picture.width, picture.height, picture.bgra)?;
    Some(RenderImage::new(vec![image::Frame::new(buffer)]))
}

/// A slot's preview at the size of the game's own thumbnails, or a quiet placeholder.
fn thumbnail(preview: Option<Arc<RenderImage>>, cx: &Context<OpenSavePage>) -> impl IntoElement {
    let frame = div()
        .w(rems(6.))
        .h(rems(3.))
        .flex_shrink_0()
        .rounded(cx.theme().radius)
        .overflow_hidden()
        .bg(cx.theme().muted)
        .border_1()
        .border_color(cx.theme().border);
    match preview {
        Some(preview) => frame.child(img(preview).size_full().object_fit(ObjectFit::Cover)),
        None => frame
            .flex()
            .items_center()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .child(Icon::new(IconName::Truck).small()),
    }
}
