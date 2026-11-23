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
    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n bus_job_log: log\r\n bus_stops: 3\r\n bus_stops[0]: stop.a\r\n bus_stops[1]: stop.b\r\n bus_stops[2]: stop.c\r\n total_distances_by_mode: 3\r\n total_distances_by_mode[0]: 9000\r\n total_distances_by_mode[1]: 450\r\n total_distances_by_mode[2]: 0\r\n}\r\n\r\nbus_job_log : log {\r\n entries: 2\r\n entries[0]: job.1\r\n entries[1]: job.2\r\n}\r\n\r\nbus_job_log_entry : job.1 {\r\n total_passengers: 30\r\n satisfaction: 0.5\r\n}\r\n\r\nbus_job_log_entry : job.2 {\r\n total_passengers: 10\r\n satisfaction: 1\r\n}\r\n\r\nbus_stop : stop.a {\r\n discovered: true\r\n}\r\n\r\nbus_stop : stop.b {\r\n discovered: false\r\n}\r\n\r\nbus_stop : stop.c {\r\n discovered: false\r\n}\r\n\r\n}\r\n";

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
