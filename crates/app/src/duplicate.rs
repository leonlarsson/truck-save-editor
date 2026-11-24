//! Duplicate a save from the start screen, unchanged: a copy in a new slot next to the original,
//! named "<name> (copy)", written and checked as any save is (`save::save_as_new_slot`). The
//! original stays as it is.
//!
//! Only for local profiles. A Steam Cloud profile lists only the save files Steam knows, and Steam
//! learns of files only when the game writes them, so a slot the editor makes there never shows in
//! the game (see `docs/decisions.md`, 2026-11-22).

use std::rc::Rc;

use gpui_kit::component::{
    WindowExt as _,
    notification::{Notification, NotificationType},
};
use gpui_kit::{App, AppContext as _, SharedString, Window};

use crate::i18n::t;
use crate::labels;

/// Called when a copy has been written, to show the profile's saves again.
pub type OnDone = Rc<dyn Fn(&mut Window, &mut App)>;

/// Why a save can't be duplicated, if it can't: a copy wouldn't show in the game on Steam Cloud,
/// and a save on its own has no profile to put a copy in.
pub fn why_not(profile: &save::Profile, cx: &App) -> Option<SharedString> {
    match profile.location {
        save::Location::Local => None,
        save::Location::SteamCloud => Some(t!(cx, "duplicate-cloud")),
        save::Location::Loose => Some(t!(cx, "duplicate-loose")),
    }
}

/// Asks to confirm, then writes the copy.
pub fn open(
    profile: save::Profile,
    slot: save::Slot,
    on_done: OnDone,
    window: &mut Window,
    cx: &mut App,
) {
    if why_not(&profile, cx).is_some() {
        return;
    }
    let name = labels::slot_name(cx, &slot);
    let copy_name: SharedString = t!(cx, "duplicate-name", name = &name);
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let (game, slot, name, copy_name, on_done) = (
            profile.game,
            slot.clone(),
            name.clone(),
            copy_name.clone(),
            on_done.clone(),
        );
        alert
            .title(t!(cx, "duplicate-title", name = &name))
            .description(t!(cx, "duplicate-body", name = &name, copy = &copy_name))
            .confirm()
            // The kit's own Cancel is English; ours follows the app's language.
            .cancel_text(t!(cx, "cancel"))
            .ok_text(t!(cx, "duplicate-confirm"))
            .on_ok(move |_, window, cx| {
                write(
                    game,
                    slot.clone(),
                    name.clone(),
                    copy_name.clone(),
                    on_done.clone(),
                    window,
                    cx,
                );
                true
            })
    });
}

/// Reads the save as it is on disk and writes it into a new slot, in the background.
fn write(
    game: save::Game,
    slot: save::Slot,
    name: SharedString,
    copy_name: SharedString,
    on_done: OnDone,
    window: &mut Window,
    cx: &mut App,
) {
    window
        .spawn(cx, async move |cx| {
            let new_name = copy_name.clone();
            let result = cx
                .background_spawn(async move {
                    let loaded = save::load(&slot.dir)?;
                    save::save_as_new_slot(game, &loaded, &new_name)
                })
                .await;
            let _ = cx.update(|window, cx| match result {
                Ok(_) => {
                    window.push_notification(
                        Notification::new()
                            .with_type(NotificationType::Success)
                            .title(t!(cx, "duplicated-title", name = &copy_name))
                            .message(t!(cx, "duplicated-body", original = &name)),
                        cx,
                    );
                    on_done(window, cx);
                }
                Err(error) => window.push_notification(
                    crate::report::error_notification(
                        t!(cx, "duplicate-failed", name = &name),
                        &error,
                        "Couldn't duplicate a save",
                    ),
                    cx,
                ),
            });
        })
        .detach();
}
