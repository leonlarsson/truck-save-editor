//! What the player has discovered of the map: the cities they've visited (and how often), and the
//! truck dealers and recruitment agencies they've found.
//!
//! `economy.unlocked_dealers[]` and `economy.unlocked_recruitments[]` list city tokens: a dealer or
//! agency is unlocked by driving past it. Which cities have one isn't in the save; the game's data
//! knows (see the `gamedata` crate).
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
    visit_cities(document, &missing)
}

/// Marks the cities in `tokens` as visited once, leaving those already visited as they are.
/// Returns how many were added.
pub fn visit_cities(document: &mut Document, tokens: &[String]) -> usize {
    let visited_before: Vec<String> = cities(document)
        .into_iter()
        .filter(|city| city.visits > 0)
        .map(|city| city.token)
        .collect();
    let missing: Vec<&String> = tokens
        .iter()
        .filter(|token| !visited_before.contains(token))
        .collect();
    if missing.is_empty() {
        return 0;
    }
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
        match visited.iter().position(|city| city == *token) {
            // Listed with a count of 0: counted once now.
            Some(index) => counts[index] = "1".to_string(),
            None => {
                visited.push((*token).clone());
                counts.push("1".to_string());
            }
        }
    }
    economy.set("visited_cities", Value::Array(visited));
    economy.set("visited_cities_count", Value::Array(counts));
    missing.len()
}

/// Something on the map the player unlocks by finding it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    Dealer,
    Recruitment,
}

impl Place {
    fn field(self) -> &'static str {
        match self {
            Place::Dealer => "unlocked_dealers",
            Place::Recruitment => "unlocked_recruitments",
        }
    }
}

/// The cities where the player has unlocked a dealer or an agency, as the save lists them.
pub fn unlocked(document: &Document, place: Place) -> Vec<String> {
    document
        .first_of_kind("economy")
        .map(|economy| economy.items(place.field()).to_vec())
        .unwrap_or_default()
}

/// Unlocks the dealers or agencies in `cities`, keeping those already unlocked in their order.
/// Returns how many were added.
pub fn unlock(document: &mut Document, place: Place, cities: &[String]) -> usize {
    let mut list = unlocked(document, place);
    let before = list.len();
    for city in cities {
        if !list.contains(city) {
            list.push(city.clone());
        }
    }
    let added = list.len() - before;
    if added == 0 {
        return 0;
    }
    let Some(id) = document
        .first_of_kind("economy")
        .map(|unit| unit.id.clone())
    else {
        return 0;
    };
    let Some(mut economy) = document.unit_mut(&id) else {
        return 0;
    };
    economy.set(place.field(), Value::Array(list));
    added
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n unlocked_dealers: 1\r\n unlocked_dealers[0]: lyon\r\n unlocked_recruitments: 0\r\n visited_cities: 1\r\n visited_cities[0]: berlin\r\n visited_cities_count: 1\r\n visited_cities_count[0]: 4\r\n}\r\n\r\ngarage : garage.berlin {\r\n status: 0\r\n}\r\n\r\ngarage : garage.lyon {\r\n status: 0\r\n}\r\n\r\n}\r\n";

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

    #[test]
    fn visits_one_city_leaving_the_others() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        let lyon = vec!["lyon".to_string()];
        assert_eq!(visit_cities(&mut document, &lyon), 1);
        let economy = document.first_of_kind("economy").expect("economy");
        assert_eq!(economy.items("visited_cities"), ["berlin", "lyon"]);
        assert_eq!(economy.items("visited_cities_count"), ["4", "1"]);
        // Already visited: Berlin keeps its count.
        let both = vec!["berlin".to_string(), "lyon".to_string()];
        assert_eq!(visit_cities(&mut document, &both), 0);
        assert_eq!(cities(&document)[0].visits, 4);
    }

    #[test]
    fn unlocks_dealers_and_agencies_once() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        let cities = vec!["berlin".to_string(), "lyon".to_string()];
        assert_eq!(unlock(&mut document, Place::Dealer, &cities), 1);
        assert_eq!(unlocked(&document, Place::Dealer), ["lyon", "berlin"]);
        assert_eq!(unlock(&mut document, Place::Dealer, &cities), 0);

        assert_eq!(unlock(&mut document, Place::Recruitment, &cities), 2);
        assert_eq!(unlocked(&document, Place::Recruitment), ["berlin", "lyon"]);
    }
}
