//! The command palette (Ctrl+K): every page and every bulk action, searchable, from the keyboard.

use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Icon, IndexPath, WindowExt as _,
    command::{Command, CommandGroup, CommandItem, CommandState},
};
use gpui_kit::{App, Entity, Focusable as _, ParentElement as _, Styled as _, WeakEntity, Window};
use save::VehicleKind;

use crate::i18n::t;
use crate::pages::Page;
use crate::workspace::Workspace;

gpui_kit::actions!(palette, [OpenPalette]);

/// Something the palette can do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaletteCommand {
    Go(Page),
    QuickStart,
    MaxTruckSkills,
    MaxCoachSkills,
    AllAdr,
    Repair(VehicleKind),
    BuyAllGarages,
    UpgradeOwnedGarages,
    MaxAllDrivers,
    VisitEveryCity,
    RefuelAll,
    KeepOffers,
    UnlockDealers,
    UnlockRecruitments,
    Save,
    Undo,
    Redo,
    Discard,
    Backups,
    SwitchSave,
    ToggleTheme,
}

/// The palette's commands, in groups: pages, then actions. `has_coaches` hides coach commands for
/// saves without them.
fn groups(has_coaches: bool) -> Vec<Vec<PaletteCommand>> {
    let pages = Page::ALL
        .iter()
        .map(|page| PaletteCommand::Go(*page))
        .collect();
    let mut actions = vec![
        PaletteCommand::QuickStart,
        PaletteCommand::MaxTruckSkills,
        PaletteCommand::AllAdr,
        PaletteCommand::Repair(VehicleKind::Truck),
        PaletteCommand::Repair(VehicleKind::Trailer),
        PaletteCommand::RefuelAll,
        PaletteCommand::BuyAllGarages,
        PaletteCommand::UpgradeOwnedGarages,
        PaletteCommand::MaxAllDrivers,
        PaletteCommand::VisitEveryCity,
        PaletteCommand::UnlockDealers,
        PaletteCommand::UnlockRecruitments,
        PaletteCommand::KeepOffers,
    ];
    if has_coaches {
        actions.insert(2, PaletteCommand::MaxCoachSkills);
        actions.push(PaletteCommand::Repair(VehicleKind::Coach));
    }
    let app = vec![
        PaletteCommand::Save,
        PaletteCommand::Undo,
        PaletteCommand::Redo,
        PaletteCommand::Discard,
        PaletteCommand::Backups,
        PaletteCommand::SwitchSave,
        PaletteCommand::ToggleTheme,
    ];
    vec![pages, actions, app]
}

fn item(command: PaletteCommand, cx: &App) -> CommandItem {
    let (label, icon, keywords): (_, IconName, &[&str]) = match command {
        PaletteCommand::Go(page) => (
            t!(cx, "palette-go", page = page.title(cx)),
            page.icon(),
            &["page", "open", "show"],
        ),
        PaletteCommand::QuickStart => (
            t!(cx, "quick-start-open"),
            IconName::Rocket,
            &["preset", "fresh", "new", "rich", "unlock"],
        ),
        PaletteCommand::MaxTruckSkills => (
            t!(cx, "palette-max-truck-skills"),
            IconName::Award,
            &["skills", "level"],
        ),
        PaletteCommand::MaxCoachSkills => (
            t!(cx, "palette-max-coach-skills"),
            IconName::Award,
            &["skills", "bus"],
        ),
        PaletteCommand::AllAdr => (
            t!(cx, "palette-all-adr"),
            IconName::FlaskConical,
            &["adr", "dangerous"],
        ),
        PaletteCommand::Repair(VehicleKind::Truck) => (
            t!(cx, "palette-repair-trucks"),
            IconName::Wrench,
            &["wear", "fix", "damage"],
        ),
        PaletteCommand::Repair(VehicleKind::Coach) => (
            t!(cx, "palette-repair-coaches"),
            IconName::Wrench,
            &["wear", "fix", "bus"],
        ),
        PaletteCommand::Repair(VehicleKind::Trailer) => (
            t!(cx, "palette-repair-trailers"),
            IconName::Wrench,
            &["wear", "fix", "trailer"],
        ),
        PaletteCommand::RefuelAll => (
            t!(cx, "palette-refuel-all"),
            IconName::Truck,
            &["fuel", "tank", "fill", "diesel"],
        ),
        PaletteCommand::KeepOffers => (
            t!(cx, "jobs-keep"),
            IconName::Container,
            &["jobs", "offers", "cargo", "expire"],
        ),
        PaletteCommand::BuyAllGarages => (
            t!(cx, "palette-buy-all-garages"),
            IconName::Warehouse,
            &["garage", "buy"],
        ),
        PaletteCommand::UpgradeOwnedGarages => (
            t!(cx, "palette-upgrade-owned-garages"),
            IconName::Warehouse,
            &["garage", "large"],
        ),
        PaletteCommand::MaxAllDrivers => (
            t!(cx, "drivers-max-all"),
            IconName::Users,
            &["drivers", "skills"],
        ),
        PaletteCommand::VisitEveryCity => (
            t!(cx, "map-visit-all"),
            IconName::Map,
            &["cities", "map", "discover"],
        ),
        PaletteCommand::UnlockDealers => (
            t!(cx, "map-unlock-dealers"),
            IconName::Map,
            &["dealer", "truck", "buy"],
        ),
        PaletteCommand::UnlockRecruitments => (
            t!(cx, "map-unlock-recruitments"),
            IconName::Map,
            &["recruitment", "hire", "drivers"],
        ),
        PaletteCommand::Save => (t!(cx, "palette-save"), IconName::Save, &["write"]),
        PaletteCommand::Undo => (t!(cx, "undo"), IconName::Undo2, &["back", "revert"]),
        PaletteCommand::Redo => (t!(cx, "redo"), IconName::Redo2, &["again", "forward"]),
        PaletteCommand::Discard => (
            t!(cx, "palette-discard"),
            IconName::Undo2,
            &["undo", "revert"],
        ),
        PaletteCommand::Backups => (
            t!(cx, "backups"),
            IconName::ArchiveRestore,
            &["restore", "undo"],
        ),
        PaletteCommand::SwitchSave => (
            t!(cx, "switch-save"),
            IconName::FolderOpen,
            &["open", "slot"],
        ),
        PaletteCommand::ToggleTheme => (
            t!(cx, "palette-theme"),
            IconName::Moon,
            &["dark", "light", "theme"],
        ),
    };
    CommandItem::new()
        .label(label)
        .icon(Icon::new(icon))
        .keywords(keywords.iter().copied())
}

/// Opens the palette over the window; confirming a command runs it on the workspace.
pub fn open(
    workspace: WeakEntity<Workspace>,
    state: Entity<CommandState>,
    has_coaches: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let groups = groups(has_coaches);
    // Every opening starts from an empty search.
    state.update(cx, |state, cx| state.set_query("", window, cx));
    let focus_on_mount = Rc::new(Cell::new(true));
    window.open_dialog(cx, move |dialog, _, _| {
        let workspace = workspace.clone();
        let state = state.clone();
        let groups = groups.clone();
        let focus_on_mount = focus_on_mount.clone();
        dialog
            .close_button(false)
            .p_0()
            .content(move |content, window, cx| {
                // The search field takes focus once, when the palette opens.
                if focus_on_mount.replace(false) {
                    let state = state.clone();
                    window.defer(cx, move |window, cx| {
                        state.read(cx).focus_handle(cx).focus(window, cx);
                    });
                }
                let titles = [
                    t!(cx, "palette-group-pages"),
                    t!(cx, "palette-group-actions"),
                    t!(cx, "palette-group-app"),
                ];
                let mut command = Command::new(&state)
                    .bordered(false)
                    .rounded(cx.theme().radius_lg)
                    .placeholder(t!(cx, "palette-placeholder"));
                for (group, title) in groups.iter().zip(titles) {
                    command = command.group(
                        CommandGroup::new()
                            .label(title)
                            .items(group.iter().map(|c| item(*c, cx))),
                    );
                }
                let workspace = workspace.clone();
                let groups = groups.clone();
                content.child(command.on_confirm(move |index: IndexPath, window, cx| {
                    let chosen = groups
                        .get(index.section)
                        .and_then(|group| group.get(index.row))
                        .copied();
                    window.close_dialog(cx);
                    if let Some(chosen) = chosen {
                        let _ =
                            workspace.update(cx, |workspace, cx| workspace.run(chosen, window, cx));
                    }
                }))
            })
    });
}
