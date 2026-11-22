//! Removing things from the company: a hired driver, a truck, a trailer, a garage, and forgetting
//! the roads driven. The game would pay for a sold truck or garage; these pay nothing.
//!
//! The player unit lists drivers, trucks and trailers, each with lists that run alongside it,
//! one entry per item: `drivers` with `driver_readiness_timer` and, by game version,
//! `driver_flags` and `driver_undrivable_truck_timers` or `driver_quit_warned`; `trucks` with
//! `truck_profit_logs`; `trailers` with `trailer_utilization_logs` (`trailer_defs` is not one of
//! them, whatever its length). Removing an item removes its entry from each. A list of the same
//! length named like one of them that isn't known stops the removal: guessing would put the lists
//! out of step.
//!
//! Garages hold their trucks and drivers in slots (`vehicles[i]`, `drivers[i]`) and trailers in a
//! plain list. A removed truck's or driver's slot becomes `null`. A driver's scheduled events
//! (`economy_event`s in the economy's event queue, linked by `unit_link`) go too. Whatever is left
//! with nothing pointing at it (the driver's job and logs, a truck's parts) is removed by
//! `drop_unreachable`, as the game reads a save as one tree.
//!
//! A removed garage takes the shape of the garages the company doesn't own: no slots, no
//! trailers, `status: 0`, `productivity: 0`, its profit log kept.

use std::collections::HashSet;

use sii::{Document, Value};

/// Lists alongside `drivers`, `trucks` and `trailers` (whichever a save has).
const DRIVER_LISTS: [&str; 5] = [
    "drivers",
    "driver_flags",
    "driver_readiness_timer",
    "driver_undrivable_truck_timers",
    "driver_quit_warned",
];
const TRUCK_LISTS: [&str; 2] = ["trucks", "truck_profit_logs"];
const TRAILER_LISTS: [&str; 2] = ["trailers", "trailer_utilization_logs"];
/// Named like a trailer list but not one: the trailer models in use, not one per trailer.
const NOT_TRAILER_LISTS: [&str; 1] = ["trailer_defs"];

/// Why something can't be removed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoveProblem {
    /// The company has no such driver, truck, trailer or (owned) garage.
    NotFound,
    /// It's the player, or what the player drives or pulls.
    IsPlayers,
    /// A truck with a driver: remove or move the driver first.
    HasDriver,
    /// A trailer in use by the job in progress.
    InUse,
    /// The headquarters' garage.
    IsHeadquarters,
    /// A garage with trucks, drivers or trailers in it.
    NotEmpty,
    /// The save's lists aren't in a shape the editor knows (a newer game version).
    UnknownShape,
}

/// What can be removed.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Removal {
    Driver(String),
    Truck(String),
    Trailer(String),
    Garage(String),
}

fn player_id(document: &Document) -> Option<String> {
    document
        .first_of_kind("economy")?
        .single("player")
        .map(str::to_string)
}

/// The player's lists that run alongside `main` (`drivers` …), and whether there's an unknown one
/// that seems to (same prefix, same length).
fn parallel_lists(
    document: &Document,
    player: &str,
    known: &[&str],
    prefix: &str,
    not_parallel: &[&str],
) -> Result<Vec<String>, RemoveProblem> {
    let unit = document.unit(player).ok_or(RemoveProblem::NotFound)?;
    let length = unit.items(known[0]).len();
    let mut lists = Vec::new();
    for field in &unit.fields {
        let Value::Array(items) = &field.value else {
            continue;
        };
        let name = field.name.as_str();
        // An empty list (a newer save keeps `driver_quit_warned: 0`) isn't one entry per item:
        // leave it alone.
        if items.is_empty() && length > 0 {
            continue;
        }
        if known.contains(&name) {
            if items.len() != length {
                return Err(RemoveProblem::UnknownShape);
            }
            lists.push(name.to_string());
        } else if name.starts_with(prefix)
            && !not_parallel.contains(&name)
            && items.len() == length
            && length > 1
        {
            return Err(RemoveProblem::UnknownShape);
        }
    }
    Ok(lists)
}

/// Removes entry `index` from each list.
fn remove_entry(document: &mut Document, player: &str, lists: &[String], index: usize) {
    if let Some(mut unit) = document.unit_mut(player) {
        for list in lists {
            let mut items = unit
                .get(list)
                .map(|value| value.items().to_vec())
                .unwrap_or_default();
            if index < items.len() {
                items.remove(index);
            }
            unit.set(list, Value::Array(items));
        }
    }
}

/// Puts `null` wherever a garage's slots hold `id`, and takes it out of garages' trailer lists.
fn clear_from_garages(document: &mut Document, id: &str) {
    let garages: Vec<String> = document
        .units_of_kind("garage")
        .filter(|garage| {
            ["vehicles", "drivers", "trailers"]
                .iter()
                .any(|field| garage.items(field).iter().any(|item| item == id))
        })
        .map(|garage| garage.id.clone())
        .collect();
    for garage in garages {
        let Some(mut unit) = document.unit_mut(&garage) else {
            continue;
        };
        for field in ["vehicles", "drivers"] {
            let slots: Vec<String> = unit
                .get(field)
                .map(|value| value.items().to_vec())
                .unwrap_or_default();
            if slots.iter().any(|item| item == id) {
                let cleared = slots
                    .into_iter()
                    .map(|item| if item == id { "null".to_string() } else { item })
                    .collect();
                unit.set(field, Value::Array(cleared));
            }
        }
        let trailers: Vec<String> = unit
            .get("trailers")
            .map(|value| value.items().to_vec())
            .unwrap_or_default();
        if trailers.iter().any(|item| item == id) {
            let kept = trailers.into_iter().filter(|item| item != id).collect();
            unit.set("trailers", Value::Array(kept));
        }
    }
}

/// The vehicles and trailers the player drives or pulls: `assigned_*` and `my_*` on the player
/// (older saves), `vehicle` and `trailer` of `player_vehicles` (newer ones).
fn players_own(document: &Document, player: &str) -> HashSet<String> {
    let mut own = HashSet::new();
    if let Some(unit) = document.unit(player) {
        for field in [
            "assigned_truck",
            "my_truck",
            "assigned_trailer",
            "my_trailer",
        ] {
            if let Some(id) = unit.single(field) {
                own.insert(id.to_string());
            }
        }
    }
    for unit in document.units_of_kind("player_vehicles") {
        for field in ["vehicle", "trailer"] {
            if let Some(id) = unit.single(field) {
                own.insert(id.to_string());
            }
        }
    }
    own.remove("null");
    own
}

/// Whether something can be removed, and if not, why.
pub fn can_remove(document: &Document, removal: &Removal) -> Result<(), RemoveProblem> {
    let player = player_id(document).ok_or(RemoveProblem::NotFound)?;
    let player_unit = document.unit(&player).ok_or(RemoveProblem::NotFound)?;
    match removal {
        Removal::Driver(id) => {
            if !player_unit.items("drivers").contains(id) {
                return Err(RemoveProblem::NotFound);
            }
            if document
                .unit(id)
                .is_none_or(|unit| unit.kind != "driver_ai")
            {
                return Err(RemoveProblem::IsPlayers);
            }
            parallel_lists(document, &player, &DRIVER_LISTS, "driver_", &[]).map(|_| ())
        }
        Removal::Truck(id) => {
            if !player_unit.items("trucks").contains(id) {
                return Err(RemoveProblem::NotFound);
            }
            if players_own(document, &player).contains(id) {
                return Err(RemoveProblem::IsPlayers);
            }
            if crate::move_options(document, id).is_ok_and(|options| options.partner.is_some()) {
                return Err(RemoveProblem::HasDriver);
            }
            parallel_lists(document, &player, &TRUCK_LISTS, "truck_", &[]).map(|_| ())
        }
        Removal::Trailer(id) => {
            if !player_unit.items("trailers").contains(id) {
                return Err(RemoveProblem::NotFound);
            }
            if players_own(document, &player).contains(id) {
                return Err(RemoveProblem::IsPlayers);
            }
            let in_job = player_unit
                .single("current_job")
                .and_then(|job| document.unit(job))
                .and_then(|job| job.single("company_trailer"))
                == Some(id.as_str());
            if in_job {
                return Err(RemoveProblem::InUse);
            }
            parallel_lists(
                document,
                &player,
                &TRAILER_LISTS,
                "trailer_",
                &NOT_TRAILER_LISTS,
            )
            .map(|_| ())
        }
        Removal::Garage(id) => {
            let garage = document
                .unit(id)
                .filter(|unit| unit.kind == "garage")
                .ok_or(RemoveProblem::NotFound)?;
            if garage.single("status").unwrap_or("0") == "0" {
                return Err(RemoveProblem::NotFound);
            }
            let city = id.strip_prefix("garage.").unwrap_or(id);
            if player_unit.single("hq_city") == Some(city) {
                return Err(RemoveProblem::IsHeadquarters);
            }
            let is_empty = ["vehicles", "drivers", "trailers"]
                .iter()
                .all(|field| garage.items(field).iter().all(|item| item == "null"));
            if !is_empty {
                return Err(RemoveProblem::NotEmpty);
            }
            Ok(())
        }
    }
}

/// Removes something from the company (see the module docs), or says why it can't.
pub fn remove(document: &mut Document, removal: &Removal) -> Result<(), RemoveProblem> {
    can_remove(document, removal)?;
    let player = player_id(document).ok_or(RemoveProblem::NotFound)?;
    match removal {
        Removal::Driver(id) => {
            let lists = parallel_lists(document, &player, &DRIVER_LISTS, "driver_", &[])?;
            remove_listed(document, &player, &lists, "drivers", id);
            clear_from_garages(document, id);
            remove_events_of(document, id);
        }
        Removal::Truck(id) => {
            let lists = parallel_lists(document, &player, &TRUCK_LISTS, "truck_", &[])?;
            remove_listed(document, &player, &lists, "trucks", id);
            clear_from_garages(document, id);
        }
        Removal::Trailer(id) => {
            let lists = parallel_lists(
                document,
                &player,
                &TRAILER_LISTS,
                "trailer_",
                &NOT_TRAILER_LISTS,
            )?;
            remove_listed(document, &player, &lists, "trailers", id);
            clear_from_garages(document, id);
        }
        Removal::Garage(id) => {
            if let Some(mut unit) = document.unit_mut(id) {
                for field in ["vehicles", "drivers", "trailers"] {
                    unit.set(field, Value::Array(Vec::new()));
                }
                unit.set("status", Value::single("0"));
                if unit.get("productivity").is_some() {
                    unit.set("productivity", Value::single("0"));
                }
            }
        }
    }
    crate::drop_unreachable(document);
    Ok(())
}

/// Removes `id`'s entry from `main` and the lists alongside it.
fn remove_listed(document: &mut Document, player: &str, lists: &[String], main: &str, id: &str) {
    let index = document
        .unit(player)
        .and_then(|unit| unit.items(main).iter().position(|item| item == id));
    if let Some(index) = index {
        remove_entry(document, player, lists, index);
    }
}

/// Takes a driver's scheduled events out of the economy's event queue.
fn remove_events_of(document: &mut Document, id: &str) {
    let queues: Vec<String> = document
        .units_of_kind("economy_event_queue")
        .map(|queue| queue.id.clone())
        .collect();
    for queue in queues {
        let Some(events) = document
            .unit(&queue)
            .map(|unit| unit.items("data").to_vec())
        else {
            continue;
        };
        let kept: Vec<String> = events
            .into_iter()
            .filter(|event| {
                document
                    .unit(event)
                    .and_then(|unit| unit.single("unit_link"))
                    != Some(id)
            })
            .collect();
        if let Some(mut unit) = document.unit_mut(&queue) {
            unit.set("data", Value::Array(kept));
        }
    }
}

/// How many road pieces the player has discovered (the map's explored roads).
pub fn discovered_roads(document: &Document) -> usize {
    document
        .first_of_kind("economy")
        .map_or(0, |economy| economy.items("discovered_items").len())
}

/// Forgets every road driven: the map starts undiscovered again. The summary the game's load
/// menu shows (`info.sii`) says so too.
pub fn forget_roads(document: &mut Document, info: Option<&mut Document>) {
    let economy = document
        .first_of_kind("economy")
        .map(|unit| unit.id.clone());
    if let Some(mut unit) = economy.and_then(|id| document.unit_mut(&id))
        && unit.get("discovered_items").is_some()
    {
        unit.set("discovered_items", Value::Array(Vec::new()));
    }
    if let Some(info) = info {
        let id = info.units().first().map(|unit| unit.id.clone());
        if let Some(mut unit) = id.and_then(|id| info.unit_mut(&id))
            && unit.get("info_explored_ratio").is_some()
        {
            unit.set("info_explored_ratio", Value::single("0"));
        }
    }
}

/// Marks every road in `ids` as driven (the game's map, read by the caller from its files), as well
/// as those already driven: the list is sorted, each id once. The load menu's summary says the
/// map is explored. Returns how many roads were added.
pub fn discover_roads(document: &mut Document, ids: &[u64], info: Option<&mut Document>) -> usize {
    let Some(economy) = document.first_of_kind("economy") else {
        return 0;
    };
    if economy.get("discovered_items").is_none() {
        return 0;
    }
    let economy_id = economy.id.clone();
    let mut all: Vec<u64> = economy
        .items("discovered_items")
        .iter()
        .filter_map(|id| id.parse().ok())
        .collect();
    let Some(mut unit) = document.unit_mut(&economy_id) else {
        return 0;
    };
    let before = all.len();
    all.extend_from_slice(ids);
    all.sort_unstable();
    all.dedup();
    let added = all.len().saturating_sub(before);
    unit.set(
        "discovered_items",
        Value::Array(all.iter().map(u64::to_string).collect()),
    );
    if let Some(info) = info {
        let id = info.units().first().map(|unit| unit.id.clone());
        if let Some(mut unit) = id.and_then(|id| info.unit_mut(&id))
            && unit.get("info_explored_ratio").is_some()
        {
            unit.set("info_explored_ratio", Value::single("1"));
        }
    }
    added
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A newer save's shape: two hired drivers and the player, trucks with profit logs, a trailer,
    /// garages, and events for one driver.
    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n event_queue: q\r\n garages: 3\r\n garages[0]: garage.lyon\r\n garages[1]: garage.rome\r\n garages[2]: garage.oslo\r\n discovered_items: 2\r\n discovered_items[0]: 1234\r\n discovered_items[1]: 5678\r\n}\r\n\r\nplayer : p {\r\n hq_city: lyon\r\n trucks: 3\r\n trucks[0]: truck.me\r\n trucks[1]: truck.a\r\n trucks[2]: truck.b\r\n truck_profit_logs: 3\r\n truck_profit_logs[0]: log.me\r\n truck_profit_logs[1]: log.a\r\n truck_profit_logs[2]: log.b\r\n trailers: 1\r\n trailers[0]: trailer.t\r\n trailer_utilization_logs: 1\r\n trailer_utilization_logs[0]: tlog.t\r\n trailer_defs: 1\r\n trailer_defs[0]: def.t\r\n drivers: 3\r\n drivers[0]: me\r\n drivers[1]: driver.a\r\n drivers[2]: driver.c\r\n driver_flags: 3\r\n driver_flags[0]: 0\r\n driver_flags[1]: 1\r\n driver_flags[2]: 2\r\n driver_readiness_timer: 3\r\n driver_readiness_timer[0]: 10\r\n driver_readiness_timer[1]: 11\r\n driver_readiness_timer[2]: 12\r\n assigned_truck: truck.me\r\n current_job: null\r\n}\r\n\r\ndriver_player : me {\r\n}\r\n\r\ndriver_ai : driver.a {\r\n driver_job: job.a\r\n}\r\n\r\ndriver_job : job.a {\r\n}\r\n\r\ndriver_ai : driver.c {\r\n}\r\n\r\nvehicle : truck.me {\r\n}\r\n\r\nvehicle : truck.a {\r\n}\r\n\r\nvehicle : truck.b {\r\n}\r\n\r\nprofit_log : log.me {\r\n}\r\n\r\nprofit_log : log.a {\r\n}\r\n\r\nprofit_log : log.b {\r\n}\r\n\r\ntrailer : trailer.t {\r\n trailer_definition: def.t\r\n}\r\n\r\ntrailer_def : def.t {\r\n}\r\n\r\ntrailer_utilization_log : tlog.t {\r\n}\r\n\r\ngarage : garage.lyon {\r\n vehicles: 2\r\n vehicles[0]: truck.me\r\n vehicles[1]: truck.a\r\n drivers: 2\r\n drivers[0]: me\r\n drivers[1]: driver.a\r\n trailers: 1\r\n trailers[0]: trailer.t\r\n status: 3\r\n productivity: 0\r\n}\r\n\r\ngarage : garage.rome {\r\n vehicles: 2\r\n vehicles[0]: truck.b\r\n vehicles[1]: null\r\n drivers: 2\r\n drivers[0]: null\r\n drivers[1]: driver.c\r\n trailers: 0\r\n status: 3\r\n productivity: 5\r\n}\r\n\r\ngarage : garage.oslo {\r\n vehicles: 1\r\n vehicles[0]: null\r\n drivers: 1\r\n drivers[0]: null\r\n trailers: 0\r\n status: 1\r\n profit_log: glog.oslo\r\n productivity: 2\r\n}\r\n\r\nprofit_log : glog.oslo {\r\n}\r\n\r\neconomy_event_queue : q {\r\n data: 2\r\n data[0]: ev.1\r\n data[1]: ev.2\r\n}\r\n\r\neconomy_event : ev.1 {\r\n unit_link: driver.a\r\n}\r\n\r\neconomy_event : ev.2 {\r\n unit_link: company.x\r\n}\r\n\r\n}\r\n";

    fn document() -> Document {
        sii::parse_text(SAVE).expect("parses")
    }

    #[test]
    fn discovers_roads_keeping_the_ones_driven_once_each() {
        let mut document = document();
        assert_eq!(discover_roads(&mut document, &[5678, 9, 99999], None), 2);
        assert_eq!(
            document
                .unit("e")
                .expect("economy")
                .items("discovered_items"),
            ["9", "1234", "5678", "99999"]
        );
        assert_eq!(discovered_roads(&document), 4);
    }

    #[test]
    fn removes_a_driver_with_their_lists_slot_events_and_job() {
        let mut document = document();
        assert_eq!(
            remove(&mut document, &Removal::Driver("driver.a".into())),
            Ok(())
        );
        let player = document.unit("p").expect("player");
        assert_eq!(player.items("drivers"), ["me", "driver.c"]);
        assert_eq!(player.items("driver_flags"), ["0", "2"]);
        assert_eq!(player.items("driver_readiness_timer"), ["10", "12"]);
        let lyon = document.unit("garage.lyon").expect("lyon");
        assert_eq!(lyon.items("drivers"), ["me", "null"]);
        assert_eq!(document.unit("q").expect("queue").items("data"), ["ev.2"]);
        for gone in ["driver.a", "job.a", "ev.1"] {
            assert!(document.unit(gone).is_none(), "{gone} removed");
        }
        // Everything else stays.
        assert!(document.unit("driver.c").is_some() && document.unit("ev.2").is_some());
    }

    #[test]
    fn removes_a_truck_without_a_driver_with_its_profit_log() {
        let mut document = document();
        assert_eq!(
            remove(&mut document, &Removal::Truck("truck.b".into())),
            Ok(())
        );
        let player = document.unit("p").expect("player");
        assert_eq!(player.items("trucks"), ["truck.me", "truck.a"]);
        assert_eq!(player.items("truck_profit_logs"), ["log.me", "log.a"]);
        assert_eq!(
            document
                .unit("garage.rome")
                .expect("rome")
                .items("vehicles"),
            ["null", "null"]
        );
        assert!(document.unit("truck.b").is_none() && document.unit("log.b").is_none());
    }

    #[test]
    fn removes_a_trailer_but_not_its_model_list() {
        let mut document = document();
        assert_eq!(
            remove(&mut document, &Removal::Trailer("trailer.t".into())),
            Ok(())
        );
        let player = document.unit("p").expect("player");
        assert!(player.items("trailers").is_empty());
        assert!(player.items("trailer_utilization_logs").is_empty());
        // Same length as the trailers, but not one per trailer: left alone.
        assert_eq!(player.items("trailer_defs"), ["def.t"]);
        assert!(
            document
                .unit("garage.lyon")
                .expect("lyon")
                .items("trailers")
                .is_empty()
        );
        assert!(document.unit("trailer.t").is_none() && document.unit("tlog.t").is_none());
        assert!(
            document.unit("def.t").is_some(),
            "still listed in trailer_defs"
        );
    }

    #[test]
    fn removes_an_empty_garage_into_the_unowned_shape() {
        let mut document = document();
        assert_eq!(
            remove(&mut document, &Removal::Garage("garage.oslo".into())),
            Ok(())
        );
        let oslo = document.unit("garage.oslo").expect("oslo stays, unowned");
        assert!(oslo.items("vehicles").is_empty() && oslo.items("drivers").is_empty());
        assert_eq!(oslo.single("status"), Some("0"));
        assert_eq!(oslo.single("productivity"), Some("0"));
        assert_eq!(oslo.single("profit_log"), Some("glog.oslo"));
        assert!(document.unit("glog.oslo").is_some());
    }

    #[test]
    fn says_why_something_stays() {
        let document = document();
        let problem = |removal: Removal| can_remove(&document, &removal);
        assert_eq!(
            problem(Removal::Driver("me".into())),
            Err(RemoveProblem::IsPlayers)
        );
        assert_eq!(
            problem(Removal::Truck("truck.me".into())),
            Err(RemoveProblem::IsPlayers)
        );
        assert_eq!(
            problem(Removal::Truck("truck.a".into())),
            Err(RemoveProblem::HasDriver)
        );
        assert_eq!(
            problem(Removal::Driver("nobody".into())),
            Err(RemoveProblem::NotFound)
        );
        assert_eq!(
            problem(Removal::Garage("garage.lyon".into())),
            Err(RemoveProblem::IsHeadquarters)
        );
        assert_eq!(
            problem(Removal::Garage("garage.rome".into())),
            Err(RemoveProblem::NotEmpty)
        );
    }

    #[test]
    fn leaves_an_empty_list_alone() {
        let text = SAVE.replace(
            " assigned_truck: truck.me\r\n",
            " driver_quit_warned: 0\r\n assigned_truck: truck.me\r\n",
        );
        let mut document = sii::parse_text(&text).expect("parses");
        // Make it an empty array, as a binary save decodes it.
        if let Some(mut player) = document.unit_mut("p") {
            player.set("driver_quit_warned", Value::Array(Vec::new()));
        }
        assert_eq!(
            remove(&mut document, &Removal::Driver("driver.a".into())),
            Ok(())
        );
        let player = document.unit("p").expect("player");
        assert!(player.items("driver_quit_warned").is_empty());
        assert_eq!(player.items("drivers"), ["me", "driver.c"]);
    }

    #[test]
    fn refuses_lists_it_does_not_know() {
        // A driver list of the same length the editor doesn't know: don't guess.
        let text = SAVE.replace(
            " driver_readiness_timer: 3\r\n",
            " driver_mood: 3\r\n driver_mood[0]: 1\r\n driver_mood[1]: 1\r\n driver_mood[2]: 1\r\n driver_readiness_timer: 3\r\n",
        );
        let document = sii::parse_text(&text).expect("parses");
        assert_eq!(
            can_remove(&document, &Removal::Driver("driver.a".into())),
            Err(RemoveProblem::UnknownShape)
        );
    }

    #[test]
    fn forgets_the_roads() {
        let mut document = document();
        assert_eq!(discovered_roads(&document), 2);
        let mut info =
            sii::parse_text("SiiNunit\r\n{\r\nsave_container : _nameless.1 {\r\n info_explored_ratio: &3f000000\r\n}\r\n\r\n}\r\n")
                .expect("parses");
        forget_roads(&mut document, Some(&mut info));
        assert_eq!(discovered_roads(&document), 0);
        assert_eq!(info.units()[0].single("info_explored_ratio"), Some("0"));
    }
}
