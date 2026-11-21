//! What the player has discovered of the map: the cities they've visited (and how often).
//!
//! `economy.visited_cities[]` lists city tokens and `economy.visited_cities_count[]` how many times
//! each was visited, item by item. Every city has a garage unit (owned or not), so the garages
//! list every city there is.

use sii::{Document, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct City {
    pub token: String,
    /// How many times the player has been there (0 if never).
    pub visits: u32,
}

/// Every city (from the garages), with how often it was visited, in the save's garage order.
pub fn cities(document: &Document) -> Vec<City> {
    let Some(economy) = document.first_of_kind("economy") else {
        return Vec::new();
    };
    let visited = economy.items("visited_cities");
    let counts = economy.items("visited_cities_count");
    let visits = |token: &str| -> u32 {
        visited
            .iter()
            .position(|city| city == token)
            .and_then(|index| counts.get(index))
            .and_then(|count| count.parse().ok())
            .unwrap_or(0)
    };
    document
        .units_of_kind("garage")
        .map(|garage| {
            let token = garage
                .id
                .strip_prefix("garage.")
                .unwrap_or(&garage.id)
                .to_string();
            City {
                visits: visits(&token),
                token,
            }
        })
        .collect()
}

/// Marks every city as visited (once, for those never visited). Returns how many were added.
pub fn visit_all_cities(document: &mut Document) -> usize {
    let missing: Vec<String> = cities(document)
        .into_iter()
        .filter(|city| city.visits == 0)
        .map(|city| city.token)
        .collect();
    let Some(id) = document
        .first_of_kind("economy")
        .map(|unit| unit.id.clone())
    else {
        return 0;
    };
    let Some(mut economy) = document.unit_mut(&id) else {
        return 0;
    };
    let mut visited = economy
        .get("visited_cities")
        .map(|v| v.items().to_vec())
        .unwrap_or_default();
    let mut counts = economy
        .get("visited_cities_count")
        .map(|v| v.items().to_vec())
        .unwrap_or_default();
    // The two lists go item by item; keep them the same length.
    counts.resize(visited.len(), "1".to_string());
    for token in &missing {
        if !visited.contains(token) {
            visited.push(token.clone());
            counts.push("1".to_string());
        }
    }
    economy.set("visited_cities", Value::Array(visited));
    economy.set("visited_cities_count", Value::Array(counts));
    missing.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n visited_cities: 1\r\n visited_cities[0]: berlin\r\n visited_cities_count: 1\r\n visited_cities_count[0]: 4\r\n}\r\n\r\ngarage : garage.berlin {\r\n status: 0\r\n}\r\n\r\ngarage : garage.lyon {\r\n status: 0\r\n}\r\n\r\n}\r\n";

    #[test]
    fn lists_cities_and_visits() {
        let document = sii::parse_text(SAVE).expect("parses");
        let cities = cities(&document);
        assert_eq!(cities.len(), 2);
        assert_eq!(
            cities[0],
            City {
                token: "berlin".into(),
                visits: 4
            }
        );
        assert_eq!(cities[1].visits, 0);
    }

    #[test]
    fn visits_every_city_keeping_counts_in_step() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert_eq!(visit_all_cities(&mut document), 1);
        assert!(cities(&document).iter().all(|city| city.visits > 0));
        let economy = document.first_of_kind("economy").expect("economy");
        assert_eq!(economy.items("visited_cities"), ["berlin", "lyon"]);
        assert_eq!(economy.items("visited_cities_count"), ["4", "1"]);
        assert_eq!(visit_all_cities(&mut document), 0);
    }
}
