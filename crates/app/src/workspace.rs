//! The window: the start screen until a save is open, then the sidebar on the leading edge, the open
//! page beside it, and the page's header with what's changed and the Save button.

use std::path::PathBuf;

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, StyledExt as _, WindowExt as _,
    alert::Alert,
    button::{Button, ButtonVariants as _},
    command::CommandState,
    h_flex,
    notification::{Notification, NotificationType},
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem},
    spinner::Spinner,
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, FocusHandle, Global, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, SharedString, StatefulInteractiveElement as _,
    Styled as _, Subscription, WeakEntity, Window, div, prelude::FluentBuilder as _, px, rems,
};
use save::{Profile, Slot};

use crate::APP_NAME;
use crate::game_data;
use crate::i18n::t;
use crate::labels;
use crate::pages::{
    Page,
    all_data::AllDataPage,
    drivers::DriversPage,
    fleet::FleetPage,
    garages::GaragesPage,
    jobs::JobsPage,
    map::MapPage,
    open_save::{OpenSavePage, OpenSlot},
    overview::OverviewPage,
    settings::SettingsPage,
};
use crate::palette::{self, OpenPalette, PaletteCommand};
use crate::quick_start;
use crate::session::Session;
use crate::theme;
use crate::ui::PageHeader;
use save::VehicleKind;

gpui_kit::actions!(
    workspace,
    [
        Undo,
        Redo,
        /// Ctrl+S: review the changes and save, as the Save… button does.
        SaveChanges,
        /// Enter on a table row: open its detail panel.
        OpenDetails,
        GoToPage1,
        GoToPage2,
        GoToPage3,
        GoToPage4,
        GoToPage5,
        GoToPage6,
        GoToPage7,
        GoToPage8,
        GoToPage9,
    ]
);

/// The key context of the open save's window content.
pub const KEY_CONTEXT: &str = "Workspace";

/// The workspace, for app-wide shortcuts. Undo and Redo work wherever focus is, a detail panel or
/// a dialog included: those are drawn above the workspace, not inside it, so a handler on the
/// workspace would never hear them. (A focused text field's own Undo still wins: its binding is
/// more specific.)
/// Its window too, for shortcuts that open a dialog.
struct ActiveWorkspace(WeakEntity<Workspace>, gpui_kit::AnyWindowHandle);

impl Global for ActiveWorkspace {}

/// Registers the app-wide handlers for Undo and Redo.
pub fn init(cx: &mut App) {
    cx.on_action(|_: &Undo, cx| step_history(cx, true));
    cx.on_action(|_: &Redo, cx| step_history(cx, false));
    // App-wide, like Undo, so it works with a detail panel open too (the panel isn't inside
    // the workspace's element, so an action handler there wouldn't hear it).
    cx.on_action(|_: &SaveChanges, cx| {
        let Some((workspace, window)) = cx
            .try_global::<ActiveWorkspace>()
            .and_then(|active| Some((active.0.upgrade()?, active.1)))
        else {
            return;
        };
        // Just after: the key is being handled inside the window, which is busy until then.
        cx.defer(move |cx| {
            let _ = window.update(cx, |_, window, cx| {
                workspace.update(cx, |this, cx| this.save_from_shortcut(window, cx));
            });
        });
    });
}

fn step_history(cx: &mut App, is_undo: bool) {
    let session = cx
        .try_global::<ActiveWorkspace>()
        .and_then(|active| active.0.upgrade())
        .and_then(|workspace| {
            workspace
                .read(cx)
                .opened
                .as_ref()
                .map(|opened| opened.session.clone())
        });
    let Some(session) = session else {
        return;
    };
    // Not while saving: the save is being written from the changes as they were.
    if session.read(cx).is_saving() {
        return;
    }
    session.update(cx, |session, cx| {
        if is_undo {
            session.undo(cx)
        } else {
            session.redo(cx)
        }
    });
}

pub struct Workspace {
    page: Page,
    /// Looks for saves again when the window comes back to the front, so a save just made in
    /// the game shows on the start screen.
    _activation: Subscription,
    /// The window title last set, so it's only set again when it changes.
    window_title: String,
    focus_handle: FocusHandle,
    palette: Entity<CommandState>,
    open_save: Entity<OpenSavePage>,
    opened: Option<Opened>,
    /// The save being opened, while it loads.
    opening: Option<SharedString>,
    /// A save that's loaded, waiting for the game's data (names, levels) so its pages don't show
    /// fallbacks first and then change.
    waiting: Option<Session>,
    _subscriptions: Vec<Subscription>,
}

/// An open save and the pages showing it.
struct Opened {
    session: Entity<Session>,
    overview: Entity<OverviewPage>,
    statistics: Entity<crate::pages::statistics::StatisticsPage>,
    fleet: Entity<FleetPage>,
    garages: Entity<GaragesPage>,
    drivers: Entity<DriversPage>,
    all_data: Entity<AllDataPage>,
    map: Entity<MapPage>,
    jobs: Entity<JobsPage>,
    settings: Entity<SettingsPage>,
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
        // A separate line: `set_global` borrows `cx` mutably, `entity` immutably.
        let this = cx.entity().downgrade();
        cx.set_global(ActiveWorkspace(this, window.window_handle()));
        let focus_handle = cx.focus_handle();
        // Keyboard shortcuts (Ctrl+K) reach the workspace when nothing inside it has focus yet.
        focus_handle.focus(window, cx);
        let language_changed =
            cx.observe_global_in::<crate::i18n::Strings>(window, |this, window, cx| {
                if let Some(session) = this.opened.as_ref().map(|opened| opened.session.clone()) {
                    this.build_pages(session, window, cx);
                }
            });
        let game_data_loaded = game_data::observe_in(window, cx, |this, window, cx| {
            if !game_data::is_loading(cx)
                && let Some(session) = this.waiting.take()
            {
                this.opening = None;
                this.show(session, window, cx);
            }
        });
        // Closing the window with unsaved changes asks first, like closing the save does.
        let guard = cx.entity().downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            guard
                .update(cx, |this, cx| this.may_close_window(window, cx))
                .unwrap_or(true)
        });
        let _activation = cx.observe_window_activation(window, |this, window, cx| {
            if window.is_window_active() && this.opened.is_none() && this.opening.is_none() {
                this.open_save.update(cx, |page, cx| page.refresh(cx));
            }
        });
        let mut workspace = Self {
            _activation,
            window_title: String::new(),
            page: dev_start_page().unwrap_or(Page::Overview),
            focus_handle,
            palette: cx.new(|cx| CommandState::new(window, cx)),
            open_save,
            opened: None,
            opening: None,
            waiting: None,
            _subscriptions: vec![subscription, language_changed, game_data_loaded],
        };
        if let Some((profile, slot)) = dev_start_slot() {
            workspace.open(profile, slot, window, cx);
        }
        workspace
    }

    /// Loads a slot in the background, then shows it.
    pub(crate) fn open(
        &mut self,
        profile: Profile,
        slot: Slot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
                    // Still reading the game's data: keep showing "Opening…" until it's there.
                    Ok(session) if game_data::is_loading(cx) => {
                        this.opening = Some(name.clone());
                        this.waiting = Some(session);
                    }
                    Ok(session) => this.show(session, window, cx),
                    Err(error) => window.push_notification(
                        crate::report::error_notification(
                            t!(cx, "open-failed", name = &name),
                            &error,
                            "Couldn't open a save",
                        ),
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
        self.build_pages(session, window, cx);
    }

    /// Makes the pages for an open save. Also after a language change: pages make some of their
    /// text when they're built (table headings, field hints), so they're built again. The session,
    /// with every pending change and the Undo history, stays.
    fn build_pages(
        &mut self,
        session: Entity<Session>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let overview = cx.new(|cx| OverviewPage::new(session.clone(), window, cx));
        let statistics =
            cx.new(|cx| crate::pages::statistics::StatisticsPage::new(session.clone(), window, cx));
        let fleet = cx.new(|cx| FleetPage::new(session.clone(), window, cx));
        let garages = cx.new(|cx| GaragesPage::new(session.clone(), window, cx));
        let drivers = cx.new(|cx| DriversPage::new(session.clone(), window, cx));
        let all_data = cx.new(|cx| AllDataPage::new(session.clone(), window, cx));
        let map = cx.new(|cx| MapPage::new(session.clone(), window, cx));
        let jobs = cx.new(|cx| JobsPage::new(session.clone(), window, cx));
        let settings = cx.new(|cx| SettingsPage::new(window, cx));
        let _observe = cx.observe(&session, |_, _, cx| cx.notify());
        self.opened = Some(Opened {
            session,
            overview,
            statistics,
            fleet,
            garages,
            drivers,
            all_data,
            map,
            jobs,
            settings,
            _observe,
        });
        cx.notify();
    }

    /// The open save's session, if a save is open.
    #[cfg(test)]
    pub(crate) fn session(&self) -> Option<Entity<Session>> {
        self.opened.as_ref().map(|opened| opened.session.clone())
    }

    /// Ctrl+1 … Ctrl+9: the pages in the sidebar's order.
    fn go_to_page(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(page) = Page::ALL.get(index) {
            self.page = *page;
            cx.notify();
        }
    }

    /// Opens the command palette (only while a save is open: every command acts on it).
    fn open_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(opened) = &self.opened else {
            return;
        };
        let has_coaches = opened.session.read(cx).career().coach_experience.is_some();
        palette::open(
            cx.entity().downgrade(),
            self.palette.clone(),
            has_coaches,
            window,
            cx,
        );
    }

    /// Carries out a palette command.
    pub fn run(&mut self, command: PaletteCommand, window: &mut Window, cx: &mut Context<Self>) {
        let Some(session) = self.opened.as_ref().map(|opened| opened.session.clone()) else {
            return;
        };
        let edit = |cx: &mut Context<Self>, edit: Box<dyn FnOnce(&mut save::Career)>| {
            session.update(cx, |session, cx| session.edit_career(cx, edit));
        };
        match command {
            PaletteCommand::Go(page) => {
                self.page = page;
                cx.notify();
            }
            PaletteCommand::QuickStart => quick_start::open(session.clone(), window, cx),
            PaletteCommand::MaxTruckSkills => edit(
                cx,
                Box::new(|career| {
                    career.skills = save::Skills {
                        adr: career.skills.adr,
                        ..save::Skills::maxed()
                    };
                }),
            ),
            PaletteCommand::MaxCoachSkills => edit(
                cx,
                Box::new(|career| {
                    if career.coach_skills.is_some() {
                        career.coach_skills = Some(save::CoachSkills::maxed());
                    }
                }),
            ),
            PaletteCommand::AllAdr => {
                edit(cx, Box::new(|career| career.skills.adr = save::ADR_ALL))
            }
            PaletteCommand::KeepOffers => session.update(cx, |session, cx| session.keep_offers(cx)),
            PaletteCommand::RefuelAll => session.update(cx, |session, cx| {
                session.refuel_all(VehicleKind::Truck, cx);
                session.refuel_all(VehicleKind::Coach, cx);
            }),
            PaletteCommand::Repair(kind) => {
                session.update(cx, |session, cx| session.repair_all(kind, cx));
            }
            PaletteCommand::BuyAllGarages => {
                session.update(cx, |session, cx| session.upgrade_garages(true, cx));
            }
            PaletteCommand::UpgradeOwnedGarages => {
                session.update(cx, |session, cx| session.upgrade_garages(false, cx));
            }
            PaletteCommand::MaxAllDrivers => {
                session.update(cx, |session, cx| session.max_all_drivers(cx));
            }
            PaletteCommand::UnlockDealers | PaletteCommand::UnlockRecruitments => {
                let place = if command == PaletteCommand::UnlockDealers {
                    save::Place::Dealer
                } else {
                    save::Place::Recruitment
                };
                // Without the game nobody knows where they are; the Map page says so.
                match game_data::place_cities(cx, place) {
                    Some(cities) => {
                        session.update(cx, |session, cx| session.unlock_all(place, &cities, cx))
                    }
                    None => {
                        self.page = Page::Map;
                        cx.notify();
                    }
                }
            }
            PaletteCommand::VisitEveryCity => {
                session.update(cx, |session, cx| session.visit_all_cities(cx));
            }
            PaletteCommand::Save => {
                if session.read(cx).change_count() > 0 {
                    self.confirm_save(window, cx);
                }
            }
            PaletteCommand::Discard => session.update(cx, |session, cx| session.discard(cx)),
            // Not while saving: the save is being written from the changes as they were.
            PaletteCommand::Undo => {
                if !session.read(cx).is_saving() {
                    session.update(cx, |session, cx| session.undo(cx));
                }
            }
            PaletteCommand::Redo => {
                if !session.read(cx).is_saving() {
                    session.update(cx, |session, cx| session.redo(cx));
                }
            }
            PaletteCommand::Backups => self.open_backups(window, cx),
            PaletteCommand::SwitchSave => self.close(window, cx),
            PaletteCommand::ToggleTheme => theme::toggle_mode(window, cx),
            PaletteCommand::ReportProblem => crate::report::open(cx, None),
        }
    }

    /// Shows the open slot's backups, newest first, each with Restore.
    fn open_backups(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(opened) = &self.opened else {
            return;
        };
        let session = opened.session.clone();
        let (profile_folder, slot_folder) = {
            let session = session.read(cx);
            (session.profile_folder(), session.slot().folder.clone())
        };
        let backups = backups_root()
            .map(|root| save::backups(&root, &profile_folder, &slot_folder))
            .unwrap_or_default();
        let this = cx.entity().downgrade();

        window.open_sheet(cx, move |sheet, _, cx| {
            let rows = backups.iter().enumerate().map(|(index, backup)| {
                let made = std::fs::metadata(&backup.dir)
                    .and_then(|m| m.modified())
                    .map(|time| labels::ago(cx, time))
                    .unwrap_or_default();
                let restore = backup.clone();
                let reveal = backup.dir.clone();
                let this = this.clone();
                let session = session.clone();
                h_flex()
                    .justify_between()
                    .gap_3()
                    .py_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        v_flex()
                            .min_w_0()
                            .child(div().text_sm().font_medium().child(made))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(t!(cx, "backup-made-at", time = &backup.made_at)),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new(("reveal-backup", index))
                                    .ghost()
                                    .small()
                                    .icon(Icon::new(IconName::FolderOpen))
                                    .tooltip(t!(cx, "backup-show"))
                                    .accessibility_label(t!(cx, "backup-show"))
                                    .on_click(move |_, _, cx| cx.reveal_path(&reveal)),
                            )
                            .child(
                                Button::new(("restore-backup", index))
                                    .outline()
                                    .small()
                                    .label(t!(cx, "backup-restore"))
                                    .on_click(move |_, window, cx| {
                                        confirm_restore(
                                            this.clone(),
                                            session.clone(),
                                            restore.clone(),
                                            window,
                                            cx,
                                        );
                                    }),
                            ),
                    )
            });
            let body = if backups.is_empty() {
                v_flex()
                    .gap_2()
                    .child(div().text_sm().font_medium().child(t!(cx, "backups-none")))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(t!(cx, "backups-none-body")),
                    )
                    .into_any_element()
            } else {
                v_flex().children(rows).into_any_element()
            };
            sheet.title(t!(cx, "backups")).size(px(420.)).child(
                v_flex()
                    .gap_4()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(t!(cx, "backups-intro")),
                    )
                    .child(body),
            )
        });
    }

    /// Opens a slot again from disk (after restoring a backup into it).
    fn reopen(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(opened) = self.opened.take() else {
            return;
        };
        let (profile, slot) = {
            let session = opened.session.read(cx);
            (session.profile().clone(), session.slot().clone())
        };
        self.open(profile, slot, window, cx);
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
                // The kit's own Cancel is English; ours follows the app's language.
                .cancel_text(t!(cx, "cancel"))
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

    /// Whether the window may close now. With unsaved changes it asks instead (and closes itself
    /// if the player chooses to quit without saving).
    fn may_close_window(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(opened) = &self.opened else {
            return true;
        };
        let session = opened.session.read(cx);
        let changes = session.change_count();
        if changes == 0 || session.is_saving() {
            // While saving, closing waits: the write is in flight.
            return !session.is_saving();
        }
        let name = labels::slot_name(cx, session.slot());
        window.open_alert_dialog(cx, move |alert, _, cx| {
            alert
                .title(t!(cx, "quit-unsaved-title", name = &name))
                .description(t!(cx, "close-unsaved-body", count = changes))
                .confirm()
                // The kit's own Cancel is English; ours follows the app's language.
                .cancel_text(t!(cx, "cancel"))
                .ok_text(t!(cx, "quit-unsaved-confirm"))
                .ok_variant(gpui_kit::component::button::ButtonVariant::Danger)
                .on_ok(|_, window, _| {
                    window.remove_window();
                    true
                })
        });
        false
    }

    /// The window's title: the open save, with • while it has unsaved changes.
    fn update_window_title(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (title, is_edited) = match &self.opened {
            Some(opened) => {
                let session = opened.session.read(cx);
                let name = labels::slot_name(cx, session.slot());
                let is_edited = session.change_count() > 0;
                let mark = if is_edited { " •" } else { "" };
                (format!("{name}{mark} — {APP_NAME}"), is_edited)
            }
            None => (APP_NAME.to_string(), false),
        };
        if title != self.window_title {
            window.set_window_title(&title);
            window.set_window_edited(is_edited);
            self.window_title = title;
        }
    }

    /// Ctrl+S: the save dialog, when there's something to save and no save under way.
    fn save_from_shortcut(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // An amount still being typed (not yet confirmed with Enter) is part of the save: leaving
        // its field confirms it, which shows once the window has drawn the focus change, so the
        // dialog opens after that frame.
        window.blur(cx);
        let workspace = cx.entity().downgrade();
        window.on_next_frame(move |window, cx| {
            let _ = workspace.update(cx, |this, cx| {
                let Some(opened) = &this.opened else {
                    return;
                };
                let session = opened.session.read(cx);
                if session.change_count() > 0 && !session.is_saving() {
                    this.confirm_save(window, cx);
                }
            });
        });
    }

    fn confirm_save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(opened) = &self.opened else {
            return;
        };
        let session = opened.session.clone();
        let (name, is_cloud, is_loose) = {
            let session = session.read(cx);
            (
                labels::slot_name(cx, session.slot()),
                session.profile().location == save::Location::SteamCloud,
                session.profile().location == save::Location::Loose,
            )
        };
        let has_other_slots = !other_slots(&session, cx).is_empty();

        window.open_alert_dialog(cx, move |alert, _, cx| {
            let session = session.clone();
            // Read on every draw: a field left by opening the dialog confirms its amount after it
            // opened, and that change belongs in the list.
            let changes = session.read(cx).changes(cx);
            // A long list (a big Quick start) scrolls, so the buttons stay in view.
            let list = div()
                .id("save-changes")
                .max_h(rems(18.))
                .overflow_y_scroll()
                .child(crate::ui::ChangeList::new(changes));
            alert
                .title(t!(cx, "save-confirm-title", name = &name))
                .description(
                    v_flex()
                        .gap_3()
                        .child(list)
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(t!(cx, "save-confirm-note")),
                        )
                        // Steam Cloud keeps its own copy: say which one to keep when it asks.
                        .when(is_cloud, |this| {
                            this.child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(t!(cx, "save-confirm-cloud")),
                            )
                        }),
                )
                .width(px(560.))
                // Enter saves, like clicking Save (the dialog's own Confirm action); Esc cancels.
                .on_ok({
                    let session = session.clone();
                    move |_, window, cx| {
                        start_save(session.clone(), window, cx);
                        true
                    }
                })
                .footer({
                    let as_new = session.clone();
                    let over = session.clone();
                    let over_other = session.clone();
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
                            // Off for Steam Cloud: the game lists only saves Steam knows, and
                            // Steam only knows the ones the game made (the dialog says so). Off
                            // for a save on its own: there's no profile to add a slot to.
                            Button::new("save-as-new")
                                .outline()
                                .label(t!(cx, "save-as-new-slot"))
                                .disabled(is_cloud || is_loose)
                                .tooltip(if is_loose {
                                    t!(cx, "save-as-new-slot-loose")
                                } else {
                                    t!(cx, "save-as-new-slot-tooltip")
                                })
                                .on_click(move |_, window, cx| {
                                    window.close_dialog(cx);
                                    start_save_as_new_slot(as_new.clone(), window, cx);
                                }),
                        )
                        .child(
                            // Works for Steam Cloud too: the game already knows the other slot.
                            Button::new("save-over-other")
                                .outline()
                                .label(t!(cx, "save-over-other"))
                                .disabled(!has_other_slots)
                                .tooltip(t!(cx, "save-over-other-tooltip"))
                                .on_click(move |_, window, cx| {
                                    window.close_dialog(cx);
                                    choose_slot_to_save_over(over_other.clone(), window, cx);
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
            // The workspace's focus here too: with nothing focused, keys (Tab) go nowhere.
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
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
            // Under the save's name, its profile's; a save on its own has none, so it says that
            // (its folder's name would read like a profile's).
            let profile = session.profile();
            let profile_name: SharedString = if profile.location == save::Location::Loose {
                labels::location(cx, profile.location)
            } else {
                profile.name.clone().into()
            };
            (labels::slot_name(cx, session.slot()), profile_name)
        };
        Sidebar::new("sidebar")
            .w(px(232.))
            .header(
                // Plain content, not `SidebarHeader`: that one is a menu trigger and lights up on hover.
                v_flex()
                    .w_full()
                    .gap_2()
                    .child(
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
                                    .child(
                                        div().text_sm().font_semibold().truncate().child(slot_name),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .truncate()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(profile_name),
                                    ),
                            ),
                    )
                    .child(
                        Button::new("open-palette")
                            .outline()
                            .small()
                            .w_full()
                            .icon(Icon::new(IconName::Search))
                            .label(t!(cx, "palette-open"))
                            .tooltip(t!(cx, "palette-shortcut"))
                            .on_click(
                                cx.listener(|this, _, window, cx| this.open_palette(window, cx)),
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
                // A newer version, when there is one, sits above the footer's buttons.
                v_flex().w_full().children(update_notice(cx)).child(
                    h_flex()
                        .p_2()
                        .w_full()
                        .justify_between()
                        .child(
                            h_flex()
                                .gap_1()
                                .child(
                                    Button::new("switch-save")
                                        .ghost()
                                        .small()
                                        .icon(Icon::new(IconName::FolderOpen))
                                        .label(t!(cx, "switch-save"))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.close(window, cx)
                                        })),
                                )
                                .child(
                                    Button::new("backups")
                                        .ghost()
                                        .small()
                                        .icon(Icon::new(IconName::ArchiveRestore))
                                        .tooltip(t!(cx, "backups"))
                                        .accessibility_label(t!(cx, "backups"))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.open_backups(window, cx)
                                        })),
                                ),
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
                                .accessibility_label(if is_dark {
                                    t!(cx, "theme-light")
                                } else {
                                    t!(cx, "theme-dark")
                                })
                                .on_click(|_, window, cx| theme::toggle_mode(window, cx)),
                        ),
                ),
            )
    }

    fn menu(&self, pages: &[Page], cx: &mut Context<Self>) -> SidebarMenu {
        SidebarMenu::new().children(pages.iter().map(|&page| {
            SidebarMenuItem::new(page.title(cx))
                .icon(Icon::new(page.icon()))
                .active(page == self.page)
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

        let (can_undo, can_redo) = {
            let session = session.read(cx);
            (session.can_undo(), session.can_redo())
        };
        PageHeader::new(self.page.title(cx))
            .subtitle(subtitle)
            .action(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(summary),
            )
            .action(
                // A plain row, not a `ButtonGroup`: a group sets every button's disabled state to its
                // own, and Undo and Redo are each available on their own.
                h_flex()
                    .child(
                        Button::new("undo")
                            .ghost()
                            .icon(Icon::new(IconName::Undo2))
                            .tooltip(t!(cx, "undo-tooltip"))
                            .accessibility_label(t!(cx, "undo"))
                            .disabled(!can_undo || is_saving)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.run(PaletteCommand::Undo, window, cx)
                            })),
                    )
                    .child(
                        Button::new("redo")
                            .ghost()
                            .icon(Icon::new(IconName::Redo2))
                            .tooltip(t!(cx, "redo-tooltip"))
                            .accessibility_label(t!(cx, "redo"))
                            .disabled(!can_redo || is_saving)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.run(PaletteCommand::Redo, window, cx)
                            })),
                    ),
            )
            .action(
                Button::new("discard")
                    .ghost()
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
                        t!(cx, "save-open")
                    })
                    .loading(is_saving)
                    .disabled(changes == 0 || is_saving)
                    .tooltip(t!(cx, "save-tooltip"))
                    .on_click(cx.listener(|this, _, window, cx| this.confirm_save(window, cx))),
            )
    }

    fn render_page(&self, opened: &Opened) -> impl IntoElement {
        match self.page {
            Page::Overview => div()
                .id("overview-scroll")
                .size_full()
                .overflow_y_scroll()
                .child(opened.overview.clone())
                .into_any_element(),
            Page::Statistics => div()
                .size_full()
                .child(opened.statistics.clone())
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
            Page::Map => div()
                .size_full()
                .child(opened.map.clone())
                .into_any_element(),
            Page::Jobs => div()
                .size_full()
                .child(opened.jobs.clone())
                .into_any_element(),
            Page::Settings => div()
                .id("settings-scroll")
                .size_full()
                .overflow_y_scroll()
                .child(opened.settings.clone())
                .into_any_element(),
            Page::AllData => div()
                .size_full()
                .child(opened.all_data.clone())
                .into_any_element(),
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.update_window_title(window, cx);
        // When what had focus is gone (a closed save, a closed panel), the workspace takes it,
        // so the keyboard keeps working without a click first.
        if window.focused(cx).is_none() {
            self.focus_handle.focus(window, cx);
        }
        let Some(opened) = &self.opened else {
            return self.render_start(cx).into_any_element();
        };
        let session = opened.session.clone();
        h_flex()
            .size_full()
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(
                cx.listener(|this, _: &OpenPalette, window, cx| this.open_palette(window, cx)),
            )
            .on_action(cx.listener(|this, _: &GoToPage1, _, cx| this.go_to_page(0, cx)))
            .on_action(cx.listener(|this, _: &GoToPage2, _, cx| this.go_to_page(1, cx)))
            .on_action(cx.listener(|this, _: &GoToPage3, _, cx| this.go_to_page(2, cx)))
            .on_action(cx.listener(|this, _: &GoToPage4, _, cx| this.go_to_page(3, cx)))
            .on_action(cx.listener(|this, _: &GoToPage5, _, cx| this.go_to_page(4, cx)))
            .on_action(cx.listener(|this, _: &GoToPage6, _, cx| this.go_to_page(5, cx)))
            .on_action(cx.listener(|this, _: &GoToPage7, _, cx| this.go_to_page(6, cx)))
            .on_action(cx.listener(|this, _: &GoToPage8, _, cx| this.go_to_page(7, cx)))
            .on_action(cx.listener(|this, _: &GoToPage9, _, cx| this.go_to_page(8, cx)))
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_sidebar(&session, cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.render_header(&session, cx))
                    .children(newer_game_warning(&session, cx))
                    .children(unsupported_game_warning(&session, cx))
                    .child(div().flex_1().min_h_0().child(self.render_page(opened))),
            )
            .into_any_element()
    }
}

/// Says so when the save comes from a newer game than the editor was tested with: it still opens
/// and saves (what it doesn't understand is kept as it is), but the player should know.
fn newer_game_warning(session: &Entity<Session>, cx: &App) -> Option<Alert> {
    let version = session.read(cx).slot().info.version?;
    (version > save::NEWEST_TESTED_SAVE_VERSION).then(|| {
        // The words in the normal text colour (amber text on pale amber is hard to read); the
        // icon and background say it's a warning.
        Alert::warning("newer-game", t!(cx, "newer-game-warning"))
            .banner()
            .text_color(cx.theme().foreground)
            .icon(Icon::new(IconName::TriangleAlert).text_color(cx.theme().warning))
    })
}

/// Says so when the save is from a game the editor doesn't fully support yet (ATS: its game data,
/// miles and settings aren't read yet), on every page while it's open.
fn unsupported_game_warning(session: &Entity<Session>, cx: &App) -> Option<Alert> {
    (session.read(cx).game() != save::Game::Ets2).then(|| {
        Alert::warning("unsupported-game", t!(cx, "ats-warning"))
            .banner()
            .text_color(cx.theme().foreground)
            .icon(Icon::new(IconName::TriangleAlert).text_color(cx.theme().warning))
    })
}

/// Says so when the profile file (the company's name) couldn't be written after the slot was.
/// Returns whether it was written (or didn't need to be).
fn report_profile(
    result: Option<save::Result<save::Backup>>,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    match result {
        None | Some(Ok(_)) => true,
        Some(Err(error)) => {
            window.push_notification(
                Notification::new()
                    .with_type(NotificationType::Error)
                    .title(t!(cx, "save-profile-failed"))
                    .message(error.to_string()),
                cx,
            );
            false
        }
    }
}

/// Writes the session's changes in the background: back up the slot, then write it.
fn start_save(session: Entity<Session>, window: &mut Window, cx: &mut App) {
    let prepared = session.read(cx).prepared();
    let slot = match prepared {
        Ok(slot) => slot,
        Err(error) => {
            window.push_notification(
                crate::report::error_notification(
                    t!(cx, "save-prepare-failed"),
                    &error,
                    "Couldn't prepare a save",
                ),
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
    let (game, profile_folder, name, profile, profile_dir) = {
        let session = session.read(cx);
        (
            session.game(),
            session.profile_folder(),
            labels::slot_name(cx, session.slot()),
            session.prepared_profile(),
            session.profile().dir.clone(),
        )
    };
    session.update(cx, |session, cx| session.set_saving(true, cx));

    window
        .spawn(cx, async move |cx| {
            let result = cx
                .background_spawn(async move {
                    let backup = save::save(game, &slot, &backups_root, &profile_folder)?;
                    // The slot first; then the profile file, only if the company's name changed.
                    let profile = profile.map(|document| {
                        save::save_profile_file(
                            game,
                            &profile_dir,
                            &document,
                            &backups_root,
                            &profile_folder,
                        )
                    });
                    Ok::<_, save::Error>((slot, backup, profile))
                })
                .await;
            let _ = cx.update(|window, cx| match result {
                Ok((written, _backup, profile)) => {
                    let profile_written = report_profile(profile, window, cx);
                    session.update(cx, |session, cx| {
                        session.mark_saved(written, profile_written, cx)
                    });
                    window.push_notification(
                        Notification::new()
                            .with_type(NotificationType::Success)
                            .title(t!(cx, "saved-title", name = &name))
                            // Short: the Backups panel lists it, with Show in folder.
                            .message(t!(cx, "saved-backup")),
                        cx,
                    );
                }
                Err(error) => {
                    session.update(cx, |session, cx| session.set_saving(false, cx));
                    window.push_notification(
                        crate::report::error_notification(
                            t!(cx, "save-failed", name = &name),
                            &error,
                            "Couldn't save",
                        ),
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
                crate::report::error_notification(
                    t!(cx, "save-prepare-failed"),
                    &error,
                    "Couldn't prepare a save",
                ),
                cx,
            );
            return;
        }
    };
    let (game, original_name, profile, profile_dir, profile_folder) = {
        let session = session.read(cx);
        (
            session.game(),
            labels::slot_name(cx, session.slot()),
            session.prepared_profile(),
            session.profile().dir.clone(),
            session.profile_folder(),
        )
    };
    let backups_root = backups_root();
    let new_name = t!(cx, "new-slot-name", name = &original_name).to_string();
    session.update(cx, |session, cx| session.set_saving(true, cx));

    window
        .spawn(cx, async move |cx| {
            let name = new_name.clone();
            let result = cx
                .background_spawn(async move {
                    let dir = save::save_as_new_slot(game, &slot, &name)?;
                    let written = save::load(&dir)?;
                    // The company's name is the profile's, so it's written for this one too.
                    let profile = match (profile, backups_root) {
                        (Some(document), Some(root)) => Some(save::save_profile_file(
                            game,
                            &profile_dir,
                            &document,
                            &root,
                            &profile_folder,
                        )),
                        _ => None,
                    };
                    Ok::<_, save::Error>((dir, written, profile))
                })
                .await;
            let _ = cx.update(|window, cx| match result {
                Ok((dir, written, profile)) => {
                    let profile_written = report_profile(profile, window, cx);
                    session.update(cx, |session, cx| {
                        session.moved_to(dir, written, profile_written, cx)
                    });
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
                        crate::report::error_notification(
                            t!(cx, "save-failed", name = &new_name),
                            &error,
                            "Couldn't save",
                        ),
                        cx,
                    );
                }
            });
        })
        .detach();
}

/// The open profile's other slots, newest first: the ones the changes can be saved over.
fn other_slots(session: &Entity<Session>, cx: &App) -> Vec<save::Slot> {
    let session = session.read(cx);
    let open = session.slot().dir.clone();
    let mut slots: Vec<save::Slot> = save::profile_slots(&session.profile().dir)
        .into_iter()
        .filter(|slot| slot.dir != open)
        .collect();
    slots.sort_by_key(|slot| std::cmp::Reverse(slot.modified));
    slots
}

/// Asks which of the profile's other slots to save over, then to confirm it.
/// A slot with its place in the list it came from (for its button's id).
type NumberedSlot<'a> = (usize, &'a save::Slot);

fn choose_slot_to_save_over(session: Entity<Session>, window: &mut Window, cx: &mut App) {
    let slots = other_slots(&session, cx);
    let original = labels::slot_name(cx, session.read(cx).slot());
    let currency = session.read(cx).game().currency_symbol();
    window.open_alert_dialog(cx, move |alert, _, cx| {
        // The player's own saves first, then the game's (which it overwrites as it goes), each
        // with what tells them apart: the slot, the money, when.
        let (own, game_made): (Vec<NumberedSlot>, Vec<NumberedSlot>) = slots
            .iter()
            .enumerate()
            .partition(|(_, slot)| slot.kind.is_manual());
        let muted = cx.theme().muted_foreground;
        let row = |index: usize, slot: &save::Slot, cx: &mut App| {
            let name = labels::slot_name(cx, slot);
            let mut detail: Vec<String> = Vec::new();
            if let Some(money) = slot.info.money {
                detail.push(format!("{currency} {}", crate::format::thousands(money)));
            }
            if let Some(modified) = slot.modified {
                detail.push(labels::ago(cx, modified).to_string());
            }
            let number = match slot.kind {
                save::SlotKind::Manual(number) => {
                    Some(t!(cx, "start-slot-number", number = number))
                }
                _ => None,
            };
            let session = session.clone();
            let slot = slot.clone();
            let original = original.clone();
            Button::new(("save-over-slot", index))
                .ghost()
                .w_full()
                .child(
                    h_flex()
                        .w_full()
                        .justify_between()
                        .gap_4()
                        .child(
                            h_flex()
                                .min_w_0()
                                .gap_2()
                                .child(div().truncate().child(name.clone()))
                                .children(number.map(|number| {
                                    div().flex_shrink_0().text_color(muted).child(number)
                                })),
                        )
                        .child(
                            div()
                                .flex_shrink_0()
                                .text_color(muted)
                                .child(detail.join(" · ")),
                        ),
                )
                .on_click(move |_, window, cx| {
                    window.close_dialog(cx);
                    confirm_save_over(
                        session.clone(),
                        slot.clone(),
                        name.clone(),
                        original.clone(),
                        window,
                        cx,
                    );
                })
                .into_any_element()
        };
        let heading = |title: SharedString| {
            div()
                .px_3()
                .pt_2()
                .text_xs()
                .font_weight(gpui_kit::FontWeight::MEDIUM)
                .text_color(muted)
                .child(title)
                .into_any_element()
        };
        let mut rows: Vec<gpui_kit::AnyElement> = Vec::new();
        if !own.is_empty() {
            rows.push(heading(t!(cx, "start-your-saves")));
            for (index, slot) in own {
                rows.push(row(index, slot, cx));
            }
        }
        if !game_made.is_empty() {
            rows.push(heading(t!(cx, "start-game-saves")));
            for (index, slot) in game_made {
                rows.push(row(index, slot, cx));
            }
        }
        alert
            .title(t!(cx, "save-over-pick-title"))
            .description(
                v_flex()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(t!(cx, "save-over-pick-body", original = &original)),
                    )
                    .child(
                        v_flex()
                            .id("save-over-slots")
                            .max_h(rems(18.))
                            .overflow_y_scroll()
                            .gap_1()
                            .children(rows),
                    ),
            )
            .width(px(560.))
            .footer(
                h_flex().w_full().justify_end().child(
                    Button::new("cancel-save-over")
                        .outline()
                        .label(t!(cx, "cancel"))
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                ),
            )
    });
}

fn confirm_save_over(
    session: Entity<Session>,
    target: save::Slot,
    target_name: SharedString,
    original: SharedString,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let session = session.clone();
        let target = target.clone();
        let target_name = target_name.clone();
        alert
            .title(t!(cx, "save-over-confirm-title", name = &target_name))
            .description(t!(
                cx,
                "save-over-confirm-body",
                name = &target_name,
                original = &original
            ))
            .confirm()
            // The kit's own Cancel is English; ours follows the app's language.
            .cancel_text(t!(cx, "cancel"))
            .ok_text(t!(cx, "save-over-confirm"))
            .on_ok(move |_, window, cx| {
                start_save_over(
                    session.clone(),
                    target.clone(),
                    target_name.clone(),
                    window,
                    cx,
                );
                true
            })
    });
}

/// Writes the session's changes over another slot of the profile (backed up first); the open
/// slot stays as it was, and the editor carries on with the other one.
fn start_save_over(
    session: Entity<Session>,
    target: save::Slot,
    target_name: SharedString,
    window: &mut Window,
    cx: &mut App,
) {
    let slot = match session.read(cx).prepared() {
        Ok(slot) => slot,
        Err(error) => {
            window.push_notification(
                crate::report::error_notification(
                    t!(cx, "save-prepare-failed"),
                    &error,
                    "Couldn't prepare a save",
                ),
                cx,
            );
            return;
        }
    };
    let Some(backups_root) = backups_root() else {
        window.push_notification(
            Notification::new()
                .with_type(NotificationType::Error)
                .title(t!(cx, "save-failed", name = &target_name))
                .message(t!(cx, "save-over-no-backups")),
            cx,
        );
        return;
    };
    let (game, original_name, profile, profile_dir, profile_folder) = {
        let session = session.read(cx);
        (
            session.game(),
            labels::slot_name(cx, session.slot()),
            session.prepared_profile(),
            session.profile().dir.clone(),
            session.profile_folder(),
        )
    };
    session.update(cx, |session, cx| session.set_saving(true, cx));
    let dir = target.dir.clone();
    window
        .spawn(cx, async move |cx| {
            let result = cx
                .background_spawn(async move {
                    save::save_over(game, &slot, &dir, &backups_root, &profile_folder)?;
                    let written = save::load(&dir)?;
                    // The company's name is the profile's, so it's written for this one too.
                    let profile = profile.map(|document| {
                        save::save_profile_file(
                            game,
                            &profile_dir,
                            &document,
                            &backups_root,
                            &profile_folder,
                        )
                    });
                    Ok::<_, save::Error>((dir, written, profile))
                })
                .await;
            let _ = cx.update(|window, cx| match result {
                Ok((dir, written, profile)) => {
                    let profile_written = report_profile(profile, window, cx);
                    session.update(cx, |session, cx| {
                        session.moved_to(dir, written, profile_written, cx)
                    });
                    window.push_notification(
                        Notification::new()
                            .with_type(NotificationType::Success)
                            .title(t!(cx, "saved-over-title", name = &target_name))
                            .message(t!(cx, "saved-over-body", original = &original_name)),
                        cx,
                    );
                }
                Err(error) => {
                    session.update(cx, |session, cx| session.set_saving(false, cx));
                    window.push_notification(
                        crate::report::error_notification(
                            t!(cx, "save-failed", name = &target_name),
                            &error,
                            "Couldn't save",
                        ),
                        cx,
                    );
                }
            });
        })
        .detach();
}

/// Asks before restoring a backup (what's in the slot now is backed up first, and unsaved changes
/// are dropped), then restores it in the background and opens the slot again.
fn confirm_restore(
    workspace: gpui_kit::WeakEntity<Workspace>,
    session: Entity<Session>,
    backup: save::Backup,
    window: &mut Window,
    cx: &mut App,
) {
    let (name, changes) = {
        let session = session.read(cx);
        (
            labels::slot_name(cx, session.slot()),
            session.change_count(),
        )
    };
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let workspace = workspace.clone();
        let session = session.clone();
        let backup = backup.clone();
        alert
            .title(t!(cx, "restore-title", name = &name))
            .description(t!(cx, "restore-body", changes = changes))
            .confirm()
            // The kit's own Cancel is English; ours follows the app's language.
            .cancel_text(t!(cx, "cancel"))
            .ok_text(t!(cx, "backup-restore"))
            .on_ok(move |_, window, cx| {
                start_restore(
                    workspace.clone(),
                    session.clone(),
                    backup.clone(),
                    window,
                    cx,
                );
                true
            })
    });
}

fn start_restore(
    workspace: gpui_kit::WeakEntity<Workspace>,
    session: Entity<Session>,
    backup: save::Backup,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(root) = backups_root() else {
        return;
    };
    let (game, profile_folder, slot_dir, name) = {
        let session = session.read(cx);
        (
            session.game(),
            session.profile_folder(),
            session.slot().dir.clone(),
            labels::slot_name(cx, session.slot()),
        )
    };
    window
        .spawn(cx, async move |cx| {
            let result = cx
                .background_spawn(async move {
                    save::restore(game, &backup, &slot_dir, &root, &profile_folder)
                })
                .await;
            let _ = cx.update(|window, cx| match result {
                Ok(_) => {
                    window.close_sheet(cx);
                    let _ = workspace.update(cx, |workspace, cx| workspace.reopen(window, cx));
                    window.push_notification(
                        Notification::new()
                            .with_type(NotificationType::Success)
                            .title(t!(cx, "restored-title", name = &name))
                            .message(t!(cx, "restored-body")),
                        cx,
                    );
                }
                Err(error) => window.push_notification(
                    crate::report::error_notification(
                        t!(cx, "restore-failed", name = &name),
                        &error,
                        "Couldn't restore a backup",
                    ),
                    cx,
                ),
            });
        })
        .detach();
}

/// Where backups go: the app's own data folder (outside the game's folders, which Steam Cloud
/// uploads).
pub(crate) fn backups_root() -> Option<PathBuf> {
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

/// "Version 0.2.0 is available", when a newer version is out: opens its release page.
fn update_notice(cx: &App) -> Option<impl IntoElement> {
    let (version, url) = crate::updates::available(cx)?;
    Some(
        h_flex().px_2().child(
            Button::new("update-available")
                .ghost()
                .small()
                .icon(Icon::new(IconName::ArrowDown))
                .label(t!(cx, "updates-banner", version = version))
                .tooltip(t!(cx, "updates-download"))
                .on_click(move |_, _, cx| cx.open_url(&url)),
        ),
    )
}
