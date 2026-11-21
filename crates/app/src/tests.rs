//! Whole-app tests: the real workspace on a small made-up save, driven by keyboard and clicks.

use std::path::PathBuf;

use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, TestAppContext, WindowBounds, WindowOptions,
    px, size,
};
use save::{Game, Location, Profile, Slot, SlotKind};

use crate::workspace::Workspace;

/// A made-up save: a career, two cities (one visited), as plain text.
const SAVE: &str = "SiiNunit\r\n{\r\neconomy : _nameless.1 {\r\n bank: _nameless.2\r\n player: _nameless.3\r\n experience_points: 5000\r\n adr: 0\r\n long_dist: 0\r\n heavy: 0\r\n fragile: 0\r\n urgent: 0\r\n mechanical: 0\r\n visited_cities: 1\r\n visited_cities[0]: lyon\r\n visited_cities_count: 1\r\n visited_cities_count[0]: 2\r\n}\r\n\r\nbank : _nameless.2 {\r\n money_account: 1000\r\n loans: 0\r\n}\r\n\r\nplayer : _nameless.3 {\r\n hq_city: lyon\r\n}\r\n\r\ngarage : garage.lyon {\r\n status: 1\r\n}\r\n\r\ngarage : garage.berlin {\r\n status: 0\r\n}\r\n\r\n}\r\n";

/// A slot folder in the system's temporary folder, removed when dropped.
struct TempSlot(PathBuf);

impl TempSlot {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("tse-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("makes the slot folder");
        std::fs::write(dir.join("game.sii"), SAVE).expect("writes the save");
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
