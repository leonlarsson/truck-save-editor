//! The window: the start screen until a save is open, then the sidebar on the leading edge, the open
//! page beside it, and the page's header with what's changed and the Save button.

use std::path::PathBuf;

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle},
    h_flex,
    notification::{Notification, NotificationType},
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem},
    spinner::Spinner,
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    Subscription, Window, div, prelude::FluentBuilder as _, px, rems,
};
use save::{Profile, Slot};

use crate::APP_NAME;
use crate::i18n::t;
use crate::labels;
use crate::pages::{
    Page,
    drivers::DriversPage,
    fleet::FleetPage,
    garages::GaragesPage,
    open_save::{OpenSavePage, OpenSlot},
    overview::OverviewPage,
};
use crate::session::Session;
use crate::theme;
use crate::ui::PageHeader;

pub struct Workspace {
    page: Page,
    open_save: Entity<OpenSavePage>,
    opened: Option<Opened>,
    /// The save being opened, while it loads.
    opening: Option<SharedString>,
    _subscriptions: Vec<Subscription>,
}

/// An open save and the pages showing it.
struct Opened {
    session: Entity<Session>,
    overview: Entity<OverviewPage>,
    fleet: Entity<FleetPage>,
    garages: Entity<GaragesPage>,
    drivers: Entity<DriversPage>,
    _observe: Subscription,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let open_save = cx.new(OpenSavePage::new);
        let subscription = cx.subscribe_in(
            &open_save,
            window,
            |this, _, event: &OpenSlot, window, cx| {
                this.open(event.profile.clone(), event.slot.clone(), window, cx);
            },
        );
        let mut workspace = Self {
            page: dev_start_page().unwrap_or(Page::Overview),
            open_save,
            opened: None,
            opening: None,
            _subscriptions: vec![subscription],
        };
        if let Some((profile, slot)) = dev_start_slot() {
            workspace.open(profile, slot, window, cx);
        }
        workspace
    }

    /// Loads a slot in the background, then shows it.
    fn open(&mut self, profile: Profile, slot: Slot, window: &mut Window, cx: &mut Context<Self>) {
        if self.opening.is_some() {
            return;
        }
        let name = labels::slot_name(cx, &slot);
        self.opening = Some(name.clone());
        cx.notify();

        let dir = slot.dir.clone();
        cx.spawn_in(window, async move |this, cx| {
            let loaded = cx.background_spawn(async move { save::load(&dir) }).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.opening = None;
                let session = loaded.and_then(|loaded| Session::open(profile, slot, loaded));
                match session {
                    Ok(session) => this.show(session, window, cx),
                    Err(error) => window.push_notification(
                        Notification::new()
                            .with_type(NotificationType::Error)
                            .title(t!(cx, "open-failed", name = &name))
                            .message(error.to_string()),
                        cx,
                    ),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn show(&mut self, session: Session, window: &mut Window, cx: &mut Context<Self>) {
        let session = cx.new(|_| session);
        let overview = cx.new(|cx| OverviewPage::new(session.clone(), window, cx));
        let fleet = cx.new(|cx| FleetPage::new(session.clone(), window, cx));
        let garages = cx.new(|cx| GaragesPage::new(session.clone(), window, cx));
        let drivers = cx.new(|cx| DriversPage::new(session.clone(), window, cx));
        let _observe = cx.observe(&session, |_, _, cx| cx.notify());
        self.opened = Some(Opened {
            session,
            overview,
            fleet,
            garages,
            drivers,
            _observe,
        });
    }

    /// Back to the start screen. Unsaved changes are asked about first.
    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(opened) = &self.opened else {
            return;
        };
        let changes = opened.session.read(cx).change_count();
        if changes == 0 {
            self.opened = None;
            self.open_save.update(cx, |page, cx| page.search(cx));
            cx.notify();
            return;
        }
        let this = cx.entity().downgrade();
        let name = labels::slot_name(cx, opened.session.read(cx).slot());
        window.open_alert_dialog(cx, move |alert, _, cx| {
            let this = this.clone();
            alert
                .title(t!(cx, "close-unsaved-title", name = &name))
                .description(t!(cx, "close-unsaved-body", count = changes))
                .confirm()
                .ok_text(t!(cx, "close-unsaved-confirm"))
                .ok_variant(gpui_kit::component::button::ButtonVariant::Danger)
                .on_ok(move |_, _, cx| {
                    let _ = this.update(cx, |this, cx| {
                        this.opened = None;
                        this.open_save.update(cx, |page, cx| page.search(cx));
                        cx.notify();
                    });
                    true
                })
        });
    }

    fn confirm_save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(opened) = &self.opened else {
            return;
        };
        let session = opened.session.clone();
        let (name, changes) = {
            let session = session.read(cx);
            (labels::slot_name(cx, session.slot()), session.changes(cx))
        };

        window.open_alert_dialog(cx, move |alert, _, cx| {
            let session = session.clone();
            let list = v_flex().gap_1().children(changes.iter().map(|change| {
                h_flex()
                    .gap_2()
                    .text_sm()
                    .child(
                        div()
                            .w(rems(9.))
                            .flex_shrink_0()
                            .font_medium()
                            .child(change.what.clone()),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(change.before.clone()),
                    )
                    .child(div().text_color(cx.theme().muted_foreground).child("→"))
                    .child(div().child(change.after.clone()))
            }));
            alert
                .title(t!(cx, "save-confirm-title", name = &name))
                .description(
                    v_flex().gap_3().child(list).child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(t!(cx, "save-confirm-note")),
                    ),
                )
                .width(px(560.))
                .footer({
                    let as_new = session.clone();
                    let over = session.clone();
                    h_flex()
                        .w_full()
                        .justify_end()
                        .gap_2()
                        .child(
                            Button::new("cancel-save")
                                .outline()
                                .label(t!(cx, "cancel"))
                                .on_click(|_, window, cx| window.close_dialog(cx)),
                        )
                        .child(
                            Button::new("save-as-new")
                                .outline()
                                .label(t!(cx, "save-as-new-slot"))
                                .tooltip(t!(cx, "save-as-new-slot-tooltip"))
                                .on_click(move |_, window, cx| {
                                    window.close_dialog(cx);
                                    start_save_as_new_slot(as_new.clone(), window, cx);
                                }),
                        )
                        .child(
                            Button::new("save-over")
                                .primary()
                                .label(t!(cx, "save"))
                                .on_click(move |_, window, cx| {
                                    window.close_dialog(cx);
                                    start_save(over.clone(), window, cx);
                                }),
                        )
                })
        });
    }

    fn render_start(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .relative()
            .child(self.open_save.clone())
            .when_some(self.opening.clone(), |this, name| {
                this.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(cx.theme().background.opacity(0.85))
                        .child(
                            h_flex()
                                .gap_2()
                                .text_sm()
                                .child(Spinner::new().small())
                                .child(t!(cx, "opening", name = &name)),
                        ),
                )
            })
    }

    fn render_sidebar(
        &self,
        session: &Entity<Session>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_dark = cx.theme().is_dark();
        let (slot_name, profile_name) = {
            let session = session.read(cx);
            (
                labels::slot_name(cx, session.slot()),
                session.profile().name.clone(),
            )
        };
        Sidebar::new("sidebar")
            .w(px(232.))
            .header(
                // Plain content, not `SidebarHeader`: that one is a menu trigger and lights up on hover.
                h_flex()
                    .p_2()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size_8()
                            .flex_shrink_0()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().sidebar_primary)
                            .text_color(cx.theme().sidebar_primary_foreground)
                            .child(Icon::new(IconName::Truck)),
                    )
                    .child(
                        v_flex()
                            .min_w_0()
                            .child(div().text_sm().font_semibold().truncate().child(slot_name))
                            .child(
                                div()
                                    .text_xs()
                                    .truncate()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(profile_name),
                            ),
                    ),
            )
            .child(SidebarGroup::new(t!(cx, "sidebar-career")).child(self.menu(&Page::CAREER, cx)))
            .child(
                SidebarGroup::new(t!(cx, "sidebar-company")).child(self.menu(&Page::COMPANY, cx)),
            )
            .child(
                SidebarGroup::new(t!(cx, "sidebar-advanced")).child(self.menu(&Page::ADVANCED, cx)),
            )
            .footer(
                // Plain content, not `SidebarFooter` (a menu trigger that lights up on hover).
                h_flex()
                    .p_2()
                    .w_full()
                    .justify_between()
                    .child(
                        Button::new("switch-save")
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::FolderOpen))
                            .label(t!(cx, "switch-save"))
                            .on_click(cx.listener(|this, _, window, cx| this.close(window, cx))),
                    )
                    .child(
                        Button::new("toggle-theme")
                            .ghost()
                            .small()
                            .icon(Icon::new(if is_dark {
                                IconName::Sun
                            } else {
                                IconName::Moon
                            }))
                            .tooltip(if is_dark {
                                t!(cx, "theme-light")
                            } else {
                                t!(cx, "theme-dark")
                            })
                            .on_click(|_, window, cx| theme::toggle_mode(window, cx)),
                    ),
            )
    }

    fn menu(&self, pages: &[Page], cx: &mut Context<Self>) -> SidebarMenu {
        SidebarMenu::new().children(pages.iter().map(|&page| {
            SidebarMenuItem::new(page.title(cx))
                .icon(Icon::new(page.icon()))
                .active(page == self.page)
                .disable(!page.is_built())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.page = page;
                    cx.notify();
                }))
        }))
    }

    fn render_header(&self, session: &Entity<Session>, cx: &mut Context<Self>) -> PageHeader {
        let (changes, is_saving, subtitle) = {
            let session = session.read(cx);
            let place = labels::location(cx, session.profile().location);
            (
                session.change_count(),
                session.is_saving(),
                t!(
                    cx,
                    "header-subtitle",
                    game = session.game().name(),
                    place = place
                ),
            )
        };
        let summary = t!(cx, "changes-count", count = changes);

        PageHeader::new(self.page.title(cx))
            .subtitle(subtitle)
            .action(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(summary),
            )
            .action(
                Button::new("discard")
                    .ghost()
                    .icon(Icon::new(IconName::Undo2))
                    .label(t!(cx, "discard"))
                    .disabled(changes == 0 || is_saving)
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(opened) = &this.opened {
                            opened.session.update(cx, |session, cx| session.discard(cx));
                        }
                    })),
            )
            .action(
                Button::new("save")
                    .primary()
                    .icon(Icon::new(IconName::Save))
                    .label(if is_saving {
                        t!(cx, "saving")
                    } else {
                        t!(cx, "save")
                    })
                    .loading(is_saving)
                    .disabled(changes == 0 || is_saving)
                    .on_click(cx.listener(|this, _, window, cx| this.confirm_save(window, cx))),
            )
    }

    fn render_page(&self, opened: &Opened, cx: &App) -> impl IntoElement {
        match self.page {
            Page::Overview => div()
                .id("overview-scroll")
                .size_full()
                .overflow_y_scroll()
                .child(opened.overview.clone())
                .into_any_element(),
            Page::Fleet => div()
                .size_full()
                .child(opened.fleet.clone())
                .into_any_element(),
            Page::Garages => div()
                .size_full()
                .child(opened.garages.clone())
                .into_any_element(),
            Page::Drivers => div()
                .size_full()
                .child(opened.drivers.clone())
                .into_any_element(),
            other => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Empty::new().header(
                        EmptyHeader::new()
                            .media(EmptyMedia::new().child(Icon::new(other.icon())))
                            .title(EmptyTitle::new().child(t!(
                                cx,
                                "page-not-built",
                                page = other.title(cx)
                            )))
                            .description(
                                EmptyDescription::new().child(t!(cx, "page-not-built-body")),
                            ),
                    ),
                )
                .into_any_element(),
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(opened) = &self.opened else {
            return self.render_start(cx).into_any_element();
        };
        let session = opened.session.clone();
        h_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_sidebar(&session, cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.render_header(&session, cx))
                    .child(div().flex_1().min_h_0().child(self.render_page(opened, cx))),
            )
            .into_any_element()
    }
}

/// Writes the session's changes in the background: back up the slot, then write it.
fn start_save(session: Entity<Session>, window: &mut Window, cx: &mut App) {
    let prepared = session.read(cx).prepared();
    let slot = match prepared {
        Ok(slot) => slot,
        Err(error) => {
            window.push_notification(
                Notification::new()
                    .with_type(NotificationType::Error)
                    .title(t!(cx, "save-prepare-failed"))
                    .message(error.to_string()),
                cx,
            );
            return;
        }
    };
    let Some(backups_root) = backups_root() else {
        window.push_notification(
            Notification::new()
                .with_type(NotificationType::Error)
                .title(t!(cx, "save-no-backup-folder-title"))
                .message(t!(cx, "save-no-backup-folder")),
            cx,
        );
        return;
    };
    let (game, profile_folder, name) = {
        let session = session.read(cx);
        (
            session.game(),
            session.profile_folder(),
            labels::slot_name(cx, session.slot()),
        )
    };
    session.update(cx, |session, cx| session.set_saving(true, cx));

    window
        .spawn(cx, async move |cx| {
            let result = cx
                .background_spawn(async move {
                    save::save(game, &slot, &backups_root, &profile_folder)
                        .map(|backup| (slot, backup))
                })
                .await;
            let _ = cx.update(|window, cx| match result {
                Ok((written, backup)) => {
                    session.update(cx, |session, cx| session.mark_saved(written, cx));
                    window.push_notification(
                        Notification::new()
                            .with_type(NotificationType::Success)
                            .title(t!(cx, "saved-title", name = &name))
                            .message(t!(
                                cx,
                                "saved-backup",
                                path = backup.dir.display().to_string()
                            )),
                        cx,
                    );
                }
                Err(error) => {
                    session.update(cx, |session, cx| session.set_saving(false, cx));
                    window.push_notification(
                        Notification::new()
                            .with_type(NotificationType::Error)
                            .title(t!(cx, "save-failed", name = &name))
                            .message(error.to_string()),
                        cx,
                    );
                }
            });
        })
        .detach();
}

/// Writes the session's changes into a new slot next to the original, which stays as it was; the
/// editor then carries on with the new slot ("Save as").
fn start_save_as_new_slot(session: Entity<Session>, window: &mut Window, cx: &mut App) {
    let slot = match session.read(cx).prepared() {
        Ok(slot) => slot,
        Err(error) => {
            window.push_notification(
                Notification::new()
                    .with_type(NotificationType::Error)
                    .title(t!(cx, "save-prepare-failed"))
                    .message(error.to_string()),
                cx,
            );
            return;
        }
    };
    let (game, original_name) = {
        let session = session.read(cx);
        (session.game(), labels::slot_name(cx, session.slot()))
    };
    let new_name = t!(cx, "new-slot-name", name = &original_name).to_string();
    session.update(cx, |session, cx| session.set_saving(true, cx));

    window
        .spawn(cx, async move |cx| {
            let name = new_name.clone();
            let result = cx
                .background_spawn(async move {
                    let dir = save::save_as_new_slot(game, &slot, &name)?;
                    let written = save::load(&dir)?;
                    Ok::<_, save::Error>((dir, written))
                })
                .await;
            let _ = cx.update(|window, cx| match result {
                Ok((dir, written)) => {
                    session.update(cx, |session, cx| session.moved_to(dir, written, cx));
                    window.push_notification(
                        Notification::new()
                            .with_type(NotificationType::Success)
                            .title(t!(cx, "saved-as-new-title", name = &new_name))
                            .message(t!(cx, "saved-as-new-body", original = &original_name)),
                        cx,
                    );
                }
                Err(error) => {
                    session.update(cx, |session, cx| session.set_saving(false, cx));
                    window.push_notification(
                        Notification::new()
                            .with_type(NotificationType::Error)
                            .title(t!(cx, "save-failed", name = &new_name))
                            .message(error.to_string()),
                        cx,
                    );
                }
            });
        })
        .detach();
}

/// Where backups go: the app's own data folder (outside the game's folders, which Steam Cloud
/// uploads).
fn backups_root() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join(APP_NAME).join("backups"))
}

/// Development builds only: `TSE_PAGE=fleet` opens that page on launch, so a screenshot can show
/// any page without clicking. Release builds always open on Overview.
fn dev_start_page() -> Option<Page> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let wanted = std::env::var("TSE_PAGE").ok()?.to_lowercase();
    Page::ALL.into_iter().find(|page| page.slug() == wanted)
}

/// Development builds only: `TSE_SLOT=<slot folder>` opens that slot on launch. Point it at a copy
/// (like `fixtures/ets2/autosave`), never at the live save folder.
fn dev_start_slot() -> Option<(Profile, Slot)> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let dir = PathBuf::from(std::env::var_os("TSE_SLOT")?);
    let folder = dir.file_name()?.to_string_lossy().into_owned();
    let info = std::fs::read(dir.join("info.sii"))
        .ok()
        .and_then(|bytes| sii_info(&bytes))
        .unwrap_or_default();
    let profile = Profile {
        game: save::Game::Ets2,
        name: "Development copy".to_string(),
        dir: dir
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(&dir)
            .to_path_buf(),
        location: save::Location::Local,
        last_saved: None,
    };
    let slot = Slot {
        kind: save::SlotKind::Other,
        modified: None,
        folder,
        info,
        dir,
    };
    Some((profile, slot))
}

fn sii_info(bytes: &[u8]) -> Option<save::SlotInfo> {
    save::read_info(bytes)
}
