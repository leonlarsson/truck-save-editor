//! The coach career's services (Coaches DLC): what the game's statistics screen shows under Coach
//! Services, and the bus stops found.
//!
//! The game doesn't keep coach totals: it works them out from its coach job log.
//! `economy.bus_job_log` points at a `bus_job_log` whose `entries[]` are `bus_job_log_entry`
//! units, one per service, each with `total_passengers`, `satisfaction` (0 to 1),
//! `travelled_distance` (km), `reward`, `exp_delta` and the coach (`assigned_bus`). The number of
//! services is how many entries there are, the passengers their sum, and so on.
//!
//! The coach distance is the second item of `economy.total_distances_by_mode[]` (the first is
//! the truck's): it's a little more than the logged services add up to, the driving to and from
//! them included.
//!
//! `economy.bus_stops[]` points at every `bus_stop` on the map, each `discovered: true` once the
//! player has been there.

use sii::{Document, Value, value};

/// Which item of `total_distances_by_mode[]` is the coach's.
const COACH_DISTANCE_MODE: usize = 1;

/// The coach career in numbers, as the game's statistics screen works them out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoachStats {
    pub services: usize,
    pub passengers: i64,
    /// The average of the services' satisfaction, 0 to 1 (`None` without services).
    pub average_satisfaction: Option<f32>,
    /// The best service's satisfaction, 0 to 1.
    pub best_satisfaction: Option<f32>,
    pub distance_km: i64,
    pub stops_found: usize,
    pub stops_total: usize,
}

/// One coach service from the log, as the game's statistics list it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoachService {
    /// The game day it ended (0 is the career's first).
    pub day: i64,
    /// The cities of its first and last stop, by token (`berlin`).
    pub from_city: String,
    pub to_city: String,
    /// How many legs it had (stops after the first).
    pub stages: i64,
    pub passengers: i64,
    /// How happy the passengers were, 0 to 1.
    pub satisfaction: Option<f32>,
    pub distance_km: i64,
    pub reward: i64,
    pub xp: i64,
}

/// Every service in the coach job log, oldest first.
pub fn coach_services(document: &Document) -> Vec<CoachService> {
    log_entries(document)
        .iter()
        .filter_map(|id| document.unit(id))
        .map(|unit| {
            let number = |field: &str| unit.single(field).and_then(value::parse_i64).unwrap_or(0);
            // `"city.berlin"`: a stop is named for its city.
            let city = |field: &str| {
                unit.single(field)
                    .map(value::parse_string)
                    .map(|name| name.strip_prefix("city.").unwrap_or(&name).to_string())
                    .unwrap_or_default()
            };
            CoachService {
                day: number("end_time") / crate::DAY_MINUTES,
                from_city: city("source_bus_stop_name"),
                to_city: city("target_bus_stop_name"),
                stages: number("stage_count"),
                passengers: number("total_passengers"),
                satisfaction: unit
                    .single("satisfaction")
                    .and_then(value::parse_f32)
                    .map(|satisfaction| satisfaction.clamp(0.0, 1.0)),
                distance_km: unit
                    .single("travelled_distance")
                    .and_then(value::parse_f32)
                    .map(|km| km.round() as i64)
                    .unwrap_or(0),
                reward: number("reward"),
                xp: number("exp_delta"),
            }
        })
        .collect()
}

/// The coach service being driven: `player.current_bus_job` points at a `player_bus_job`, whose
/// `stages[]` are `player_bus_job_stage` units (one per leg, each from a `source_bus_stop` to a
/// `target_bus_stop`), `stage_idx` the leg being driven, and `passengers[]` who's on board.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurrentCoachService {
    /// The leg being driven, counting from 1, and how many there are.
    pub stage: usize,
    pub stages: usize,
    /// The stops of the leg being driven, as the save names them (`bus_stop.city.berlin`, or
    /// `bus_stop.viewpoints.<name>` for a stop at a viewpoint).
    pub from_stop: String,
    pub to_stop: String,
    /// The leg's planned length.
    pub planned_distance_km: i64,
    /// When it should arrive, in the game's minutes; `None` when the game hasn't planned it yet.
    pub planned_arrival: Option<i64>,
    pub passengers: usize,
    /// How happy the passengers on board are, 0 to 1: the average of each one's own
    /// (`player_bus_passenger.satisfaction`), which is what the game shows. `None` without
    /// passengers.
    pub satisfaction: Option<f32>,
}

impl CurrentCoachService {
    /// The city a stop is in, by token, when it's a city's stop (`bus_stop.city.berlin`).
    pub fn stop_city(stop: &str) -> Option<&str> {
        stop.strip_prefix("bus_stop.city.")
    }
}

/// The coach service being driven, if there is one.
pub fn current_coach_service(document: &Document) -> Option<CurrentCoachService> {
    let player = document
        .first_of_kind("economy")?
        .single("player")
        .and_then(|id| document.unit(id))?;
    let job = document
        .unit(player.single("current_bus_job")?)
        .filter(|unit| unit.kind == "player_bus_job")?;
    let stages = job.items("stages");
    let index = job
        .single("stage_idx")
        .and_then(value::parse_i64)
        .and_then(|index| usize::try_from(index).ok())
        .unwrap_or(0);
    let stage = document.unit(stages.get(index)?)?;
    let text = |field: &str| stage.single(field).unwrap_or_default().to_string();
    let passengers: Vec<&String> = job
        .items("passengers")
        .iter()
        .filter(|id| id.as_str() != "null")
        .collect();
    let moods: Vec<f32> = passengers
        .iter()
        .filter_map(|id| document.unit(id))
        .filter_map(|passenger| passenger.single("satisfaction").and_then(value::parse_f32))
        .collect();
    let satisfaction = (!moods.is_empty())
        .then(|| (moods.iter().sum::<f32>() / moods.len() as f32).clamp(0.0, 1.0));
    Some(CurrentCoachService {
        stage: index + 1,
        stages: stages.len(),
        from_stop: text("source_bus_stop"),
        to_stop: text("target_bus_stop"),
        planned_distance_km: stage
            .single("planned_distance")
            .and_then(value::parse_i64)
            .unwrap_or(0),
        // The game writes the largest 32-bit number for "not planned yet".
        planned_arrival: stage
            .single("planned_arrival_time")
            .and_then(value::parse_i64)
            .filter(|minutes| *minutes < i64::from(u32::MAX)),
        passengers: passengers.len(),
        satisfaction,
    })
}

/// The ids of the coach job log's entries, in the log's order.
fn log_entries(document: &Document) -> Vec<String> {
    let Some(log) = document
        .first_of_kind("economy")
        .and_then(|economy| economy.single("bus_job_log"))
        .and_then(|id| document.unit(id))
    else {
        return Vec::new();
    };
    log.items("entries")
        .iter()
        .filter(|id| id.as_str() != "null")
        .cloned()
        .collect()
}

/// The ids of every bus stop on the map.
fn bus_stops(document: &Document) -> Vec<String> {
    document
        .first_of_kind("economy")
        .map(|economy| {
            economy
                .items("bus_stops")
                .iter()
                .filter(|id| id.as_str() != "null")
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

/// The coach career's numbers, or `None` for a save without one (no coach log, no bus stops).
pub fn coach_stats(document: &Document) -> Option<CoachStats> {
    let entries = log_entries(document);
    let stops = bus_stops(document);
    if entries.is_empty() && stops.is_empty() {
        return None;
    }
    let mut stats = CoachStats {
        services: entries.len(),
        stops_total: stops.len(),
        ..CoachStats::default()
    };
    let mut satisfactions = Vec::new();
    for unit in entries.iter().filter_map(|id| document.unit(id)) {
        stats.passengers += unit
            .single("total_passengers")
            .and_then(value::parse_i64)
            .unwrap_or(0);
        if let Some(satisfaction) = unit.single("satisfaction").and_then(value::parse_f32) {
            satisfactions.push(satisfaction.clamp(0.0, 1.0));
        }
    }
    if !satisfactions.is_empty() {
        stats.average_satisfaction =
            Some(satisfactions.iter().sum::<f32>() / satisfactions.len() as f32);
        stats.best_satisfaction = satisfactions.iter().copied().reduce(f32::max);
    }
    stats.distance_km = document
        .first_of_kind("economy")
        .and_then(|economy| {
            economy
                .items("total_distances_by_mode")
                .get(COACH_DISTANCE_MODE)
                .and_then(|distance| value::parse_i64(distance))
        })
        .unwrap_or(0);
    stats.stops_found = stops
        .iter()
        .filter_map(|id| document.unit(id))
        .filter(|stop| stop.single("discovered") == Some("true"))
        .count();
    Some(stats)
}

/// Every logged service's passengers left as happy as can be (satisfaction 1). Returns how many
/// services changed.
pub fn perfect_satisfaction(document: &mut Document) -> usize {
    let mut changed = 0;
    for id in log_entries(document) {
        let Some(mut entry) = document.unit_mut(&id) else {
            continue;
        };
        let is_perfect = entry
            .get("satisfaction")
            .and_then(|value| value.as_single())
            .and_then(value::parse_f32)
            .is_some_and(|satisfaction| satisfaction >= 1.0);
        if !is_perfect {
            entry.set("satisfaction", Value::single("1"));
            changed += 1;
        }
    }
    changed
}

/// Every bus stop on the map marked as found. Returns how many weren't before.
pub fn discover_bus_stops(document: &mut Document) -> usize {
    let mut changed = 0;
    for id in bus_stops(document) {
        let Some(mut stop) = document.unit_mut(&id) else {
            continue;
        };
        if stop.get("discovered").and_then(|value| value.as_single()) != Some("true") {
            stop.set("discovered", Value::single("true"));
            changed += 1;
        }
    }
    changed
}

/// Spreads `total` passengers over the logged services, in proportion to how many each carried
/// (evenly when none did), so the statistics' total and average follow. Whole passengers only:
/// what doesn't divide evenly goes to the first services. Returns false without services to
/// spread them over.
pub fn set_coach_passengers(document: &mut Document, total: i64) -> bool {
    let entries = log_entries(document);
    if entries.is_empty() {
        return false;
    }
    let total = total.max(0);
    let carried: Vec<i64> = entries
        .iter()
        .map(|id| {
            document
                .unit(id)
                .and_then(|unit| unit.single("total_passengers"))
                .and_then(value::parse_i64)
                .unwrap_or(0)
                .max(0)
        })
        .collect();
    let carried_total: i64 = carried.iter().sum();
    // Each service's share, rounded down; the passengers left over go one each to the first.
    let mut shares: Vec<i64> = if carried_total == 0 {
        vec![total / entries.len() as i64; entries.len()]
    } else {
        carried
            .iter()
            .map(|&count| {
                (i128::from(total) * i128::from(count) / i128::from(carried_total)) as i64
            })
            .collect()
    };
    let mut left = total - shares.iter().sum::<i64>();
    for share in shares.iter_mut() {
        if left <= 0 {
            break;
        }
        *share += 1;
        left -= 1;
    }
    for (id, share) in entries.iter().zip(shares) {
        if let Some(mut entry) = document.unit_mut(id) {
            entry.set("total_passengers", Value::single(share.to_string()));
        }
    }
    true
}

/// Sets the coach distance, in km. Returns false when the save keeps no distances by mode.
pub fn set_coach_distance(document: &mut Document, km: i64) -> bool {
    let Some(id) = document
        .first_of_kind("economy")
        .map(|economy| economy.id.clone())
    else {
        return false;
    };
    let Some(mut economy) = document.unit_mut(&id) else {
        return false;
    };
    let mut distances = economy
        .get("total_distances_by_mode")
        .map(|value| value.items().to_vec())
        .unwrap_or_default();
    let Some(distance) = distances.get_mut(COACH_DISTANCE_MODE) else {
        return false;
    };
    *distance = km.max(0).to_string();
    economy.set("total_distances_by_mode", Value::Array(distances));
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two services (30 and 10 passengers), three bus stops, one of them found.
    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n bus_job_log: log\r\n bus_stops: 3\r\n bus_stops[0]: stop.a\r\n bus_stops[1]: stop.b\r\n bus_stops[2]: stop.c\r\n total_distances_by_mode: 3\r\n total_distances_by_mode[0]: 9000\r\n total_distances_by_mode[1]: 450\r\n total_distances_by_mode[2]: 0\r\n}\r\n\r\nbus_job_log : log {\r\n entries: 2\r\n entries[0]: job.1\r\n entries[1]: job.2\r\n}\r\n\r\nbus_job_log_entry : job.1 {\r\n source_bus_stop_name: \"city.harbourtown\"\r\n target_bus_stop_name: \"city.hilltown\"\r\n stage_count: 2\r\n exp_delta: 120\r\n reward: 3400\r\n travelled_distance: 211.6\r\n end_time: 3000\r\n total_passengers: 30\r\n satisfaction: 0.5\r\n}\r\n\r\nbus_job_log_entry : job.2 {\r\n total_passengers: 10\r\n satisfaction: 1\r\n}\r\n\r\nbus_stop : stop.a {\r\n discovered: true\r\n}\r\n\r\nbus_stop : stop.b {\r\n discovered: false\r\n}\r\n\r\nbus_stop : stop.c {\r\n discovered: false\r\n}\r\n\r\n}\r\n";

    #[test]
    fn reads_the_service_being_driven() {
        const DRIVING: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n}\r\n\r\nplayer : p {\r\n current_bus_job: bus.job\r\n}\r\n\r\nplayer_bus_job : bus.job {\r\n stages: 2\r\n stages[0]: stage.1\r\n stages[1]: stage.2\r\n stage_idx: 1\r\n passengers: 2\r\n passengers[0]: rider.1\r\n passengers[1]: rider.2\r\n}\r\n\r\nplayer_bus_passenger : rider.1 {\r\n satisfaction: 1\r\n}\r\n\r\nplayer_bus_passenger : rider.2 {\r\n satisfaction: 0.5\r\n}\r\n\r\nplayer_bus_job_stage : stage.1 {\r\n source_bus_stop: bus_stop.city.harbourtown\r\n target_bus_stop: bus_stop.viewpoints.oldmill\r\n planned_distance: 120\r\n planned_arrival_time: 5000\r\n}\r\n\r\nplayer_bus_job_stage : stage.2 {\r\n source_bus_stop: bus_stop.viewpoints.oldmill\r\n target_bus_stop: bus_stop.city.hilltown\r\n planned_distance: 90\r\n planned_arrival_time: 4294967295\r\n}\r\n\r\n}\r\n";
        let document = sii::parse_text(DRIVING).expect("parses");
        let service = current_coach_service(&document).expect("a service is being driven");
        assert_eq!((service.stage, service.stages), (2, 2));
        assert_eq!(service.from_stop, "bus_stop.viewpoints.oldmill");
        assert_eq!(
            CurrentCoachService::stop_city(&service.to_stop),
            Some("hilltown")
        );
        assert_eq!(CurrentCoachService::stop_city(&service.from_stop), None);
        assert_eq!(service.planned_distance_km, 90);
        assert_eq!(service.planned_arrival, None);
        assert_eq!(service.passengers, 2);
        assert_eq!(service.satisfaction, Some(0.75));

        // No service: the save in the other tests has none.
        let document = sii::parse_text(SAVE).expect("parses");
        assert_eq!(current_coach_service(&document), None);
    }

    #[test]
    fn lists_the_services() {
        let document = sii::parse_text(SAVE).expect("parses");
        let services = coach_services(&document);
        assert_eq!(services.len(), 2);
        assert_eq!(
            services[0],
            CoachService {
                day: 2,
                from_city: "harbourtown".to_string(),
                to_city: "hilltown".to_string(),
                stages: 2,
                passengers: 30,
                satisfaction: Some(0.5),
                distance_km: 212,
                reward: 3400,
                xp: 120,
            }
        );
    }

    #[test]
    fn reads_the_services_as_the_game_counts_them() {
        let document = sii::parse_text(SAVE).expect("parses");
        let stats = coach_stats(&document).expect("a coach career");
        assert_eq!(stats.services, 2);
        assert_eq!(stats.passengers, 40);
        assert_eq!(stats.average_satisfaction, Some(0.75));
        assert_eq!(stats.best_satisfaction, Some(1.0));
        assert_eq!(stats.distance_km, 450);
        assert_eq!((stats.stops_found, stats.stops_total), (1, 3));
    }

    #[test]
    fn a_save_without_coaches_has_no_stats() {
        let document =
            sii::parse_text("SiiNunit\r\n{\r\neconomy : e {\r\n}\r\n\r\n}\r\n").expect("parses");
        assert_eq!(coach_stats(&document), None);
    }

    #[test]
    fn perfects_satisfaction_and_finds_every_stop() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert_eq!(perfect_satisfaction(&mut document), 1);
        assert_eq!(discover_bus_stops(&mut document), 2);
        let stats = coach_stats(&document).expect("a coach career");
        assert_eq!(stats.average_satisfaction, Some(1.0));
        assert_eq!((stats.stops_found, stats.stops_total), (3, 3));
        assert_eq!(perfect_satisfaction(&mut document), 0);
        assert_eq!(discover_bus_stops(&mut document), 0);
    }

    #[test]
    fn spreads_passengers_in_proportion() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        // 30:10 of 101 is 75.75:25.25, so 75 and 25, and the one left over goes to the first.
        assert!(set_coach_passengers(&mut document, 101));
        let carried: Vec<String> = ["job.1", "job.2"]
            .iter()
            .filter_map(|id| document.unit(id))
            .filter_map(|unit| unit.single("total_passengers").map(str::to_string))
            .collect();
        assert_eq!(carried, ["76", "25"]);
        assert_eq!(
            coach_stats(&document).map(|stats| stats.passengers),
            Some(101)
        );
    }

    #[test]
    fn sets_the_coach_distance_only() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert!(set_coach_distance(&mut document, 1234));
        let economy = document.first_of_kind("economy").expect("economy");
        assert_eq!(
            economy.items("total_distances_by_mode"),
            ["9000", "1234", "0"]
        );
    }
}
