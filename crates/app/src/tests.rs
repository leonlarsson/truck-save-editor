//! Whole-app tests: the real workspace on a small made-up save, driven by keyboard and clicks.

use std::path::PathBuf;

use gpui_kit::component::WindowExt as _;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, TestAppContext, WindowBounds, WindowOptions,
    px, size,
};
use save::{Game, Location, Profile, Slot, SlotKind};

use crate::workspace::Workspace;

/// A made-up save: a career, two cities (one visited), one half-full truck, as plain text.
const SAVE: &str = "SiiNunit\r\n{\r\neconomy : _nameless.1 {\r\n bank: _nameless.2\r\n player: _nameless.3\r\n experience_points: 5000\r\n adr: 0\r\n long_dist: 0\r\n heavy: 0\r\n fragile: 0\r\n urgent: 0\r\n mechanical: 0\r\n visited_cities: 1\r\n visited_cities[0]: lyon\r\n visited_cities_count: 1\r\n visited_cities_count[0]: 2\r\n}\r\n\r\nbank : _nameless.2 {\r\n money_account: 1000\r\n loans: 0\r\n}\r\n\r\nplayer : _nameless.3 {\r\n hq_city: lyon\r\n trucks: 1\r\n trucks[0]: _nameless.4\r\n}\r\n\r\nvehicle : _nameless.4 {\r\n engine_wear: 0\r\n transmission_wear: 0\r\n cabin_wear: 0\r\n chassis_wear: 0\r\n fuel_relative: &3f000000\r\n wheels_wear: 0\r\n odometer: 1000\r\n license_plate: \"AB 123|france\"\r\n}\r\n\r\ngarage : garage.lyon {\r\n status: 1\r\n}\r\n\r\ngarage : garage.berlin {\r\n status: 0\r\n}\r\n\r\n}\r\n";

/// A made-up profile file. The login fields hold placeholders, nothing real.
const PROFILE: &str = "SiiNunit\r\n{\r\nuser_profile : _nameless.9 {\r\n company_name: \"Northwind Haulage\"\r\n cached_experience: 5000\r\n online_user_name: \"placeholder\"\r\n online_password: \"placeholder\"\r\n}\r\n\r\n}\r\n";

/// A slot folder in the system's temporary folder, removed when dropped.
struct TempSlot(PathBuf);

impl TempSlot {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("tse-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("makes the slot folder");
        std::fs::write(dir.join("game.sii"), SAVE).expect("writes the save");
        // The profile folder is the slot folder here; its profile file has made-up login fields.
        std::fs::write(dir.join("profile.sii"), PROFILE).expect("writes the profile");
        Self(dir)
    }
}

impl Drop for TempSlot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Opens the workspace in a window and the made-up save in it.
fn open(cx: &mut TestAppContext, slot: &TempSlot) -> (AnyWindowHandle, Entity<Workspace>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::i18n::init(cx);
        crate::theme::init(cx);
        crate::bind_keys(cx);
    });
    let profile = Profile {
        game: Game::Ets2,
        name: "Tester".into(),
        dir: slot.0.clone(),
        location: Location::Local,
        last_saved: None,
    };
    let slot = Slot {
        dir: slot.0.clone(),
        folder: "1".into(),
        kind: SlotKind::Manual(1),
        info: Default::default(),
        modified: None,
    };
    let (handle, workspace) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    Default::default(),
                    size(px(1200.), px(800.)),
                ))),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| Workspace::new(window, cx)),
        )
        .expect("opens the window")
    });
    cx.update_window(handle, |_, window, cx| {
        // A headless window starts inactive, and an inactive window gets no focus events.
        window.activate_window();
        workspace.update(cx, |workspace, cx| {
            workspace.open(profile, slot, window, cx)
        });
    })
    .expect("opens the save");
    cx.run_until_parked();
    (handle, workspace)
}

fn change_count(workspace: &Entity<Workspace>, cx: &mut TestAppContext) -> usize {
    cx.update(|cx| {
        workspace
            .read(cx)
            .session()
            .map(|session| session.read(cx).change_count())
            .expect("a save is open")
    })
}

/// One user action, then let GPUI deliver its events (they arrive between updates).
fn act(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    action: impl FnOnce(&mut gpui_kit::Window, &mut gpui_kit::App),
) {
    cx.update_window(handle, |_, window, cx| {
        window.render_frame(cx);
        action(window, cx);
    })
    .expect("the window is open");
    cx.run_until_parked();
}

#[gpui_kit::test]
fn the_palette_runs_commands_and_undo_takes_them_back(cx: &mut TestAppContext) {
    let slot = TempSlot::new("palette");
    let (handle, workspace) = open(cx, &slot);
    assert_eq!(change_count(&workspace, cx), 0);

    act(cx, handle, |window, cx| window.press("ctrl-k", cx));
    act(cx, handle, |window, cx| window.input("visit", cx));
    act(cx, handle, |window, cx| window.press("enter", cx));
    assert_eq!(change_count(&workspace, cx), 1, "Visit every city ran");

    act(cx, handle, |window, cx| window.press("ctrl-z", cx));
    assert_eq!(change_count(&workspace, cx), 0, "Undo took it back");

    act(cx, handle, |window, cx| window.press("ctrl-shift-z", cx));
    assert_eq!(change_count(&workspace, cx), 1, "Redo brought it back");
}

#[gpui_kit::test]
fn the_undo_button_is_off_until_there_is_something_to_undo(cx: &mut TestAppContext) {
    let slot = TempSlot::new("undo-button");
    let (handle, workspace) = open(cx, &slot);

    // Nothing to undo: clicking Undo does nothing.
    act(cx, handle, |window, cx| window.click("undo", cx));
    assert_eq!(change_count(&workspace, cx), 0);

    act(cx, handle, |window, cx| window.press("ctrl-k", cx));
    act(cx, handle, |window, cx| window.input("max all truck", cx));
    act(cx, handle, |window, cx| window.press("enter", cx));
    // Five skills went up: five changes in the list, one step of Undo.
    assert_eq!(change_count(&workspace, cx), 5);

    act(cx, handle, |window, cx| window.click("undo", cx));
    assert_eq!(change_count(&workspace, cx), 0);
}

/// Runs a palette command by typing part of its name.
fn run_command(cx: &mut TestAppContext, handle: AnyWindowHandle, name: &'static str) {
    act(cx, handle, |window, cx| window.press("ctrl-k", cx));
    act(cx, handle, |window, cx| window.input(name, cx));
    act(cx, handle, |window, cx| window.press("enter", cx));
}

#[gpui_kit::test]
fn without_the_game_dealers_stay_locked_but_cities_can_be_visited(cx: &mut TestAppContext) {
    let slot = TempSlot::new("map");
    let (handle, workspace) = open(cx, &slot);
    run_command(cx, handle, "map");

    // No game data in tests: nobody knows where dealers are, so the button is off.
    act(cx, handle, |window, cx| window.click("unlock-dealers", cx));
    assert_eq!(change_count(&workspace, cx), 0);

    act(cx, handle, |window, cx| window.click("visit-all", cx));
    assert_eq!(change_count(&workspace, cx), 1);
}

#[gpui_kit::test]
fn quick_start_is_one_undo_step(cx: &mut TestAppContext) {
    let slot = TempSlot::new("quick-start");
    let (handle, workspace) = open(cx, &slot);
    run_command(cx, handle, "quick start");
    act(cx, handle, |window, cx| {
        window.click("quick-start-apply", cx)
    });
    // Rich start: money, five skills, ADR, and a large garage in Lyon, the headquarters (this
    // save has no coach career, so no coach skills).
    assert_eq!(change_count(&workspace, cx), 8);
    cx.update(|cx| {
        let session = workspace.read(cx).session().expect("a save is open");
        assert_eq!(session.read(cx).career().money, 10_000_000);
    });

    act(cx, handle, |window, cx| window.press("ctrl-z", cx));
    assert_eq!(change_count(&workspace, cx), 0);
}

#[gpui_kit::test]
fn a_detail_panel_edits_one_vehicle(cx: &mut TestAppContext) {
    let slot = TempSlot::new("details");
    let (handle, workspace) = open(cx, &slot);
    run_command(cx, handle, "fleet");
    act(cx, handle, |window, cx| {
        window.click(("details", 0usize), cx)
    });
    // The panel slides in; let it arrive before clicking inside it.
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(1));
    cx.run_until_parked();
    act(cx, handle, |window, cx| window.click("fill-tank", cx));
    assert_eq!(change_count(&workspace, cx), 1);
    cx.update(|cx| {
        let session = workspace.read(cx).session().expect("a save is open");
        let fuel = session.read(cx).vehicles()[0].fuel;
        assert_eq!(fuel, Some(1.0));
    });

    act(cx, handle, |window, cx| window.press("ctrl-z", cx));
    assert_eq!(change_count(&workspace, cx), 0);
}

#[gpui_kit::test]
fn renaming_the_company_changes_only_its_name(cx: &mut TestAppContext) {
    let slot = TempSlot::new("company");
    let (_, workspace) = open(cx, &slot);
    let session = cx.update(|cx| workspace.read(cx).session().expect("a save is open"));
    cx.update(|cx| {
        assert_eq!(
            session.read(cx).company_name().as_deref(),
            Some("Northwind Haulage")
        )
    });

    session.update(cx, |session, cx| {
        session.set_company_name("Fjord Freight".into(), cx)
    });
    assert_eq!(change_count(&workspace, cx), 1);
    cx.update(|cx| {
        let profile = session
            .read(cx)
            .prepared_profile()
            .expect("a profile to write");
        assert_eq!(
            save::company_name(&profile).as_deref(),
            Some("Fjord Freight")
        );
        let unit = profile.first_of_kind("user_profile").expect("the profile");
        assert_eq!(unit.single("online_user_name"), Some("\"placeholder\""));
        assert_eq!(unit.single("online_password"), Some("\"placeholder\""));
    });

    session.update(cx, |session, cx| session.undo(cx));
    assert_eq!(change_count(&workspace, cx), 0);
}

#[gpui_kit::test]
fn ctrl_and_a_number_goes_to_a_page_and_enter_opens_a_row(cx: &mut TestAppContext) {
    let slot = TempSlot::new("keyboard");
    let (handle, _) = open(cx, &slot);
    act(cx, handle, |window, cx| window.press("ctrl-2", cx));
    act(cx, handle, |window, _| {
        assert!(
            window.try_find("refuel-all").is_some(),
            "Ctrl+2 shows Fleet"
        );
    });

    // Pick the first row, then Enter opens its panel.
    act(cx, handle, |window, cx| window.click("table-cell-0:0", cx));
    act(cx, handle, |window, cx| window.press("enter", cx));
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(1));
    cx.run_until_parked();
    act(cx, handle, |window, _| {
        assert!(
            window.try_find("fill-tank").is_some(),
            "Enter opened the truck's panel"
        );
    });
}

#[gpui_kit::test]
fn escape_leaves_a_field_then_closes_its_panel(cx: &mut TestAppContext) {
    let slot = TempSlot::new("escape");
    let (handle, _) = open(cx, &slot);
    act(cx, handle, |window, cx| window.press("ctrl-2", cx));
    act(cx, handle, |window, cx| {
        window.click(("details", 0usize), cx)
    });
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(1));
    cx.run_until_parked();

    act(cx, handle, |window, cx| window.click("amount-input", cx));
    act(cx, handle, |window, cx| window.press("escape", cx));
    act(cx, handle, |window, _| {
        assert!(
            window.try_find("fill-tank").is_some(),
            "the first Esc only leaves the field"
        );
    });
    act(cx, handle, |window, cx| window.press("escape", cx));
    cx.executor()
        .advance_clock(std::time::Duration::from_secs(1));
    cx.run_until_parked();
    act(cx, handle, |window, _| {
        assert!(
            window.try_find("fill-tank").is_none(),
            "the second Esc closes the panel"
        );
    });
}

#[gpui_kit::test]
fn a_search_with_no_matches_offers_to_clear_it(cx: &mut TestAppContext) {
    let slot = TempSlot::new("no-matches");
    let (handle, _) = open(cx, &slot);
    act(cx, handle, |window, cx| window.press("ctrl-2", cx));
    act(cx, handle, |window, cx| window.click("filter-search", cx));
    act(cx, handle, |window, cx| window.input("no such truck", cx));
    act(cx, handle, |window, _| {
        assert!(window.try_find("empty-clear-filters").is_some());
    });
    act(cx, handle, |window, cx| {
        window.click("empty-clear-filters", cx)
    });
    act(cx, handle, |window, _| {
        assert!(window.try_find("empty-clear-filters").is_none());
        assert!(
            window.try_find(("details", 0usize)).is_some(),
            "the truck is back"
        );
    });
}

/// Whether a dialog is open in the window.
fn has_dialog(cx: &mut TestAppContext, handle: AnyWindowHandle) -> bool {
    cx.update_window(handle, |_, window, cx| window.has_active_dialog(cx))
        .expect("the window is open")
}

#[gpui_kit::test]
fn ctrl_s_opens_the_save_dialog_only_when_there_is_something_to_save(cx: &mut TestAppContext) {
    let slot = TempSlot::new("ctrl-s");
    let (handle, workspace) = open(cx, &slot);

    act(cx, handle, |window, cx| window.press("ctrl-s", cx));
    assert!(!has_dialog(cx, handle), "nothing to save, no dialog");

    run_command(cx, handle, "visit");
    act(cx, handle, |window, cx| window.press("ctrl-s", cx));
    assert!(has_dialog(cx, handle), "the save dialog opened");

    // Escape leaves it without saving: the change is still waiting.
    act(cx, handle, |window, cx| window.press("escape", cx));
    assert!(!has_dialog(cx, handle));
    assert_eq!(change_count(&workspace, cx), 1);
}
