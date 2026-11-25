//! The running game, live: what the editor's plugin (`crates/live-plugin`, loaded by the game
//! through its own telemetry SDK) says about it ten times a second. Read-only.
//!
//! A thread keeps looking for the plugin on this computer (`live::listen`); the newest snapshot
//! is put in a global the Live page and the map read. When the game isn't running, or runs
//! without the plugin, there's simply no snapshot. The connection never leaves this computer
//! (`127.0.0.1`), and the editor sends nothing over it.
//!
//! Also here: putting the plugin into the game's folder, and taking it out again.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use gpui_kit::{App, Context, Global, Subscription};

use live::Snapshot;

/// What the plugin's file is called in the game's folder.
const PLUGIN_FILE: &str = "tse_live.dll";

/// The plugin itself, carried inside the editor when it was built with it (see `build.rs`):
/// Windows builds made by `scripts/windows.sh` and CI. Other builds have none.
#[cfg(has_live_plugin)]
const EMBEDDED_PLUGIN: Option<&[u8]> = Some(include_bytes!(env!("TSE_LIVE_PLUGIN")));
#[cfg(not(has_live_plugin))]
const EMBEDDED_PLUGIN: Option<&[u8]> = None;

/// How often the newest snapshot is shown (the plugin sends ten a second).
const SHOWN_EVERY: Duration = Duration::from_millis(100);

/// What a removed plugin is called while the running game still holds on to its file: Windows
/// won't delete a library in use, but lets it be renamed, and the game only loads `.dll` files.
const REMOVED_PLUGIN_FILE: &str = "tse_live.dll.removed";
/// What a plugin is called while it's being written, before it takes its place.
const NEW_PLUGIN_FILE: &str = "tse_live.dll.new";

/// How far the truck moves before the trail gets a new point, in metres, and how many points
/// the trail keeps (the oldest half goes when it's full: a very long session).
const TRAIL_STEP: f32 = 20.0;
const TRAIL_POINTS: usize = 50_000;

/// The game as last heard (`None` when it isn't heard), and where the truck has been since it
/// was first heard: the trail on the map, in the game's metres (east, south).
#[derive(Default)]
struct Live {
    snapshot: Option<Arc<Snapshot>>,
    trail: Arc<Vec<[f32; 2]>>,
}

/// Takes the newest snapshot: the trail grows when the truck has moved, and starts over when the
/// game goes quiet (the next time it's heard is another session).
fn hear(cx: &mut App, snapshot: Option<Snapshot>) {
    let mut trail = cx
        .try_global::<Live>()
        .map(|live| live.trail.clone())
        .unwrap_or_default();
    match &snapshot {
        Some(snapshot) if snapshot.has_position => {
            let at = [snapshot.x as f32, snapshot.z as f32];
            let has_moved = trail
                .last()
                .is_none_or(|last| (at[0] - last[0]).hypot(at[1] - last[1]) >= TRAIL_STEP);
            if has_moved {
                // The map holds a copy only while it draws, so this rarely copies the trail.
                let points = Arc::make_mut(&mut trail);
                if points.len() >= TRAIL_POINTS {
                    points.drain(..TRAIL_POINTS / 2);
                }
                points.push(at);
            }
        }
        Some(_) => {}
        None => trail = Arc::default(),
    }
    cx.set_global(Live {
        snapshot: snapshot.map(Arc::new),
        trail,
    });
}

impl Global for Live {}

/// What the listening thread heard last, and a count that goes up with every change, so the
/// window only redraws when there's something new.
#[derive(Default)]
struct Heard {
    count: u64,
    snapshot: Option<Snapshot>,
}

/// Starts looking for the game.
pub fn init(cx: &mut App) {
    cx.set_global(Live::default());
    // Tests never talk to a game that happens to be running on the machine.
    if cfg!(test) {
        return;
    }
    // Development builds: `TSE_LIVE_FAKE=1` shows an invented drive, to see the page and the
    // map's marker without the game.
    if cfg!(debug_assertions) && std::env::var_os("TSE_LIVE_FAKE").is_some() {
        // Two snapshots, so there's a trail to see behind the truck.
        let mut before = invented();
        (before.x, before.z) = (before.x - 400.0, before.z + 260.0);
        hear(cx, Some(before));
        hear(cx, Some(invented()));
        return;
    }

    let heard = Arc::new(Mutex::new(Heard::default()));
    let listening = {
        let heard = heard.clone();
        // Never set: the thread looks for the game for as long as the editor runs.
        let should_stop = Arc::new(AtomicBool::new(false));
        live::listen(live::PORT, should_stop, move |snapshot| {
            let mut heard = heard.lock().unwrap_or_else(PoisonError::into_inner);
            heard.count += 1;
            heard.snapshot = snapshot;
        })
    };
    if listening.is_err() {
        // No thread to be had: Live just stays empty.
        return;
    }
    cx.spawn(async move |cx| {
        let mut shown = 0;
        loop {
            cx.background_executor().timer(SHOWN_EVERY).await;
            let newer = {
                let heard = heard.lock().unwrap_or_else(PoisonError::into_inner);
                (heard.count != shown).then(|| (heard.count, heard.snapshot.clone()))
            };
            if let Some((count, snapshot)) = newer {
                shown = count;
                cx.update(|cx| hear(cx, snapshot));
            }
        }
    })
    .detach();
}

/// The game as last heard, if it's running with the plugin.
pub fn snapshot(cx: &App) -> Option<Arc<Snapshot>> {
    cx.try_global::<Live>()
        .and_then(|live| live.snapshot.clone())
}

/// Where the truck has been since the game was first heard, oldest first.
pub fn trail(cx: &App) -> Arc<Vec<[f32; 2]>> {
    cx.try_global::<Live>()
        .map(|live| live.trail.clone())
        .unwrap_or_default()
}

/// Whether the plugin in the game is older than the one this editor carries (it sends less:
/// no session tally from the first one).
pub fn is_plugin_older(snapshot: &Snapshot) -> bool {
    snapshot.plugin_version < live::PLUGIN_VERSION
}

/// Redraws a view whenever the game says something new (or goes quiet).
pub fn observe<V: 'static>(cx: &mut Context<V>) -> Subscription {
    cx.observe_global::<Live>(|_, cx| cx.notify())
}

/// Calls `on_change` whenever the game says something new (or goes quiet), for a view that does
/// more than redraw.
pub fn observe_with<V: 'static>(
    cx: &mut Context<V>,
    on_change: impl Fn(&mut V, &mut Context<V>) + 'static,
) -> Subscription {
    cx.observe_global::<Live>(on_change)
}

/// Tests only: the game says this now (an invented drive), or goes quiet.
#[cfg(test)]
pub fn set_for_test(cx: &mut App, is_heard: bool) {
    hear(cx, is_heard.then(invented));
}

/// Which game a snapshot is from (the SDK's ids: `eut2`, `ats`).
pub fn game_of(snapshot: &Snapshot) -> save::Game {
    if snapshot.game == "ats" {
        save::Game::Ats
    } else {
        save::Game::Ets2
    }
}

/// Where the plugin goes in a game's folder.
pub fn plugin_path(install_dir: &Path) -> PathBuf {
    install_dir
        .join("bin")
        .join("win_x64")
        .join("plugins")
        .join(PLUGIN_FILE)
}

/// Whether this build of the editor carries the plugin.
pub fn has_plugin() -> bool {
    EMBEDDED_PLUGIN.is_some()
}

/// Whether the plugin is in a game's folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginState {
    NotAdded,
    Added,
    /// Removed while the game was running: the game keeps using the file (renamed out of the
    /// way) until it closes, and won't load it again.
    RemovedButInUse,
}

/// Where a removed plugin waits to be deleted (see `REMOVED_PLUGIN_FILE`).
fn removed_plugin_path(install_dir: &Path) -> PathBuf {
    plugin_path(install_dir).with_file_name(REMOVED_PLUGIN_FILE)
}

/// Whether the plugin is in the game's folder. Also finishes an earlier removal: a plugin
/// removed while the game ran is deleted here once the game has let go of it.
pub fn plugin_state(install_dir: &Path) -> PluginState {
    let removed = removed_plugin_path(install_dir);
    if removed.is_file() {
        // Fails while the game still runs: tried again the next time anyone looks.
        let _ = std::fs::remove_file(&removed);
    }
    if plugin_path(install_dir).is_file() {
        PluginState::Added
    } else if removed.is_file() {
        PluginState::RemovedButInUse
    } else {
        PluginState::NotAdded
    }
}

/// Moves a plugin the game is using out of the way (it can't be deleted or written over while
/// loaded, but it can be renamed).
fn set_aside(install_dir: &Path) -> std::io::Result<()> {
    let removed = removed_plugin_path(install_dir);
    // One from before, if the game has let go of it since.
    let _ = std::fs::remove_file(&removed);
    std::fs::rename(plugin_path(install_dir), removed)
}

/// Writes the plugin into the game's folder (making its `plugins` folder if it has none), over
/// an older one if there is one. The game loads it the next time it starts.
pub fn install_plugin(install_dir: &Path) -> std::io::Result<()> {
    let plugin = EMBEDDED_PLUGIN.ok_or(std::io::ErrorKind::NotFound)?;
    let target = plugin_path(install_dir);
    if let Some(folder) = target.parent() {
        std::fs::create_dir_all(folder)?;
    }
    // Written whole under another name first, then put in place in one step: a write that fails
    // halfway (a full disk) never leaves half a plugin where the game looks.
    let fresh = target.with_file_name(NEW_PLUGIN_FILE);
    if let Err(error) = std::fs::write(&fresh, plugin) {
        let _ = std::fs::remove_file(&fresh);
        return Err(error);
    }
    if std::fs::rename(&fresh, &target).is_ok() {
        return Ok(());
    }
    // The running game holds the old one: set it aside, then put the new one in its place.
    let placed = set_aside(install_dir).and_then(|()| std::fs::rename(&fresh, &target));
    if placed.is_err() {
        let _ = std::fs::remove_file(&fresh);
    }
    placed
}

/// Takes the plugin out of the game's folder, at any time: deleted, or while the game is using
/// it, set aside so the game doesn't load it again (and deleted once the game has closed, see
/// `plugin_state`).
pub fn remove_plugin(install_dir: &Path) -> std::io::Result<()> {
    let target = plugin_path(install_dir);
    if !target.is_file() {
        return Ok(());
    }
    if std::fs::remove_file(&target).is_ok() {
        return Ok(());
    }
    set_aside(install_dir)
}

/// An invented drive, for `TSE_LIVE_FAKE`: no value is from a real save.
fn invented() -> Snapshot {
    Snapshot {
        plugin_version: live::PLUGIN_VERSION,
        game: "eut2".to_string(),
        session_km: 412.0,
        session_deliveries: 1,
        session_earned: 21_300,
        session_xp: 640,
        session_fines: 550,
        session_travel_costs: 35,
        game_minutes: 3 * 1440 + 14 * 60 + 25,
        rest_stop_minutes: 312,
        has_position: true,
        x: -3400.0,
        z: -12500.0,
        heading: 0.6,
        speed: 22.0,
        rpm: 1250.0,
        gear: 11,
        cruise_control: 22.2,
        speed_limit: 25.0,
        fuel: 612.0,
        fuel_capacity: 1000.0,
        fuel_range_km: 1840.0,
        odometer_km: 48_210.0,
        navigation_metres: 312_000.0,
        navigation_seconds: 980.0,
        wear_engine: 0.03,
        wear_transmission: 0.02,
        wear_cabin: 0.01,
        wear_chassis: 0.04,
        wear_wheels: 0.12,
        is_trailer_connected: true,
        cargo_damage: 0.02,
        truck_brand: "Scania".to_string(),
        truck_name: "S".to_string(),
        job: Some(live::Job {
            cargo_id: "apples".to_string(),
            cargo: "Apples".to_string(),
            source_city_id: "hamburg".to_string(),
            source_city: "Hamburg".to_string(),
            source_company: "Tradeaux".to_string(),
            destination_city_id: "berlin".to_string(),
            destination_city: "Berlin".to_string(),
            destination_company: "Posped".to_string(),
            income: 18_400,
            delivery_minutes: 3 * 1440 + 23 * 60,
            planned_distance_km: 480,
            cargo_mass_kg: 18_000.0,
            is_cargo_loaded: true,
            ..live::Job::default()
        }),
        events: vec![
            live::TimedEvent {
                game_minutes: 3 * 1440 + 9 * 60,
                event: live::Event::Delivered {
                    revenue: 21_300,
                    earned_xp: 640,
                    distance_km: 512.0,
                    cargo_damage: 0.0,
                },
            },
            live::TimedEvent {
                game_minutes: 3 * 1440 + 13 * 60 + 2,
                event: live::Event::Toll { amount: 35 },
            },
            live::TimedEvent {
                game_minutes: 3 * 1440 + 13 * 60 + 40,
                event: live::Event::Fined {
                    offence: "speeding".to_string(),
                    amount: 550,
                },
            },
        ],
        ..Snapshot::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plugin_goes_in_the_game_s_plugins_folder() {
        let path = plugin_path(Path::new("game"));
        assert!(path.ends_with(Path::new("bin/win_x64/plugins").join(PLUGIN_FILE)));
    }

    #[test]
    fn the_plugin_can_be_removed_and_a_set_aside_one_is_cleaned_up() {
        let game = std::env::temp_dir().join(format!("tse-live-plugin-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&game);
        assert_eq!(plugin_state(&game), PluginState::NotAdded);

        let plugin = plugin_path(&game);
        let folder = plugin.parent().expect("the plugins folder");
        std::fs::create_dir_all(folder).expect("makes the plugins folder");
        std::fs::write(&plugin, b"plugin").expect("writes a stand-in plugin");
        assert_eq!(plugin_state(&game), PluginState::Added);

        // Nothing holds the file here, so removing deletes it.
        remove_plugin(&game).expect("removes it");
        assert_eq!(plugin_state(&game), PluginState::NotAdded);

        // One set aside while the game ran is deleted the next time anyone looks.
        std::fs::write(&plugin, b"plugin").expect("writes a stand-in plugin");
        set_aside(&game).expect("sets it aside");
        assert!(!plugin.is_file() && removed_plugin_path(&game).is_file());
        assert_eq!(plugin_state(&game), PluginState::NotAdded);
        assert!(!removed_plugin_path(&game).is_file());
        let _ = std::fs::remove_dir_all(&game);
    }

    #[test]
    fn knows_which_game_is_talking() {
        let mut snapshot = invented();
        assert_eq!(game_of(&snapshot), save::Game::Ets2);
        snapshot.game = "ats".to_string();
        assert_eq!(game_of(&snapshot), save::Game::Ats);
    }
}
