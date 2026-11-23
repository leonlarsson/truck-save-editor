//! What the player has discovered of the map: the cities they've visited (and how often), and the
//! truck dealers and recruitment agencies they've found.
//!
//! `economy.unlocked_dealers[]` and `economy.unlocked_recruitments[]` list city tokens: a dealer or
//! agency is unlocked by driving past it, and locked again by leaving the list. Which cities have one isn't in the save; the game's data
//! knows (see the `gamedata` crate).
//!
//! `economy.visited_cities[]` lists city tokens and `economy.visited_cities_count[]` how many times
//! each was visited, item by item. The garages (a unit for every city that has one, owned or
//! not) and the visited list name the cities the save knows; smaller towns have no garage, so
//! the game's own city list (the `gamedata` crate) adds the rest: see `cities_including`.

use sii::{Document, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct City {
    pub token: String,
    /// How many times the player has been there (0 if never).
    pub visits: u32,
}

/// Every city the save knows (the garages', then visited ones without a garage), with how often
/// it was visited.
pub fn cities(document: &Document) -> Vec<City> {
    cities_including(document, &[])
}

/// The save's cities (see `cities`) and the ones in `more` it doesn't know (never visited): the
/// game's whole city list, for the towns without a garage.
pub fn cities_including(document: &Document, more: &[String]) -> Vec<City> {
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
    let mut cities: Vec<City> = document
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
        .collect();
    let extra: Vec<&String> = visited.iter().chain(more).collect();
    for token in extra {
        if !cities.iter().any(|city| &city.token == token) {
            cities.push(City {
                visits: visits(token),
                token: token.clone(),
            });
        }
    }
    cities
}

/// The cities the save has a company or a garage in: the game counts a city only when it has one
/// of those (ferry ports and villages without either aren't counted, as its Career Progress shows).
pub fn cities_with_business(document: &Document) -> std::collections::BTreeSet<String> {
    let companies = document.units_of_kind("company").filter_map(|unit| {
        let (_, city) = unit.id.strip_prefix("company.volatile.")?.split_once('.')?;
        Some(city.to_string())
    });
    let garages = document
        .units_of_kind("garage")
        .filter_map(|unit| unit.id.strip_prefix("garage.").map(str::to_string));
    companies.chain(garages).collect()
}

/// Marks every city as visited (once, for those never visited): the save's, and those in `more`
/// (the game's whole list). Returns how many were added.
pub fn visit_all_cities(document: &mut Document, more: &[String]) -> usize {
    let missing: Vec<String> = cities_including(document, more)
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

/// Locks the dealers or agencies in `cities` again, as if never found: they leave the list, the
/// others keep their order. Returns how many were removed.
pub fn lock(document: &mut Document, place: Place, cities: &[String]) -> usize {
    let mut list = unlocked(document, place);
    let before = list.len();
    list.retain(|city| !cities.contains(city));
    let removed = before - list.len();
    if removed == 0 {
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
    removed
}

/// Marks the cities in `tokens` as never visited: they leave the visited list, with their counts
/// (the two lists go item by item). Returns how many were removed.
pub fn unvisit_cities(document: &mut Document, tokens: &[String]) -> usize {
    let Some(id) = document
        .first_of_kind("economy")
        .map(|unit| unit.id.clone())
    else {
        return 0;
    };
    let Some(mut economy) = document.unit_mut(&id) else {
        return 0;
    };
    let visited = economy
        .get("visited_cities")
        .map(|v| v.items().to_vec())
        .unwrap_or_default();
    let counts = economy
        .get("visited_cities_count")
        .map(|v| v.items().to_vec())
        .unwrap_or_default();
    let mut kept_cities = Vec::new();
    let mut kept_counts = Vec::new();
    for (index, city) in visited.iter().enumerate() {
        if tokens.contains(city) {
            continue;
        }
        kept_cities.push(city.clone());
        kept_counts.push(
            counts
                .get(index)
                .cloned()
                .unwrap_or_else(|| "1".to_string()),
        );
    }
    let removed = visited.len() - kept_cities.len();
    if removed > 0 {
        economy.set("visited_cities", Value::Array(kept_cities));
        economy.set("visited_cities_count", Value::Array(kept_counts));
    }
    removed
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
    fn names_the_cities_with_a_company_or_a_garage() {
        let save = SAVE.replace(
            "garage : garage.lyon {",
            "company : company.volatile.acme.harbourtown {\r\n}\r\n\r\ngarage : garage.lyon {",
        );
        let document = sii::parse_text(&save).expect("parses");
        let cities: Vec<String> = cities_with_business(&document).into_iter().collect();
        assert_eq!(cities, ["berlin", "harbourtown", "lyon"]);
    }

    #[test]
    fn knows_towns_without_a_garage() {
        // Harbourtown has no garage: known from its visit; Hilltown only from the game's list.
        let save = SAVE.replace(
            " visited_cities: 1\r\n visited_cities[0]: berlin\r\n visited_cities_count: 1\r\n visited_cities_count[0]: 4\r\n",
            " visited_cities: 2\r\n visited_cities[0]: berlin\r\n visited_cities[1]: harbourtown\r\n visited_cities_count: 2\r\n visited_cities_count[0]: 4\r\n visited_cities_count[1]: 2\r\n",
        );
        let mut document = sii::parse_text(&save).expect("parses");
        let tokens: Vec<String> = cities(&document).into_iter().map(|c| c.token).collect();
        assert_eq!(tokens, ["berlin", "lyon", "harbourtown"]);
        let more = ["hilltown".to_string(), "lyon".to_string()];
        let all = cities_including(&document, &more);
        assert_eq!(all.len(), 4);
        assert_eq!(all[2].visits, 2);
        assert_eq!(
            all[3],
            City {
                token: "hilltown".into(),
                visits: 0
            }
        );
        // Every city: Lyon and Hilltown.
        assert_eq!(visit_all_cities(&mut document, &more), 2);
        assert!(
            cities_including(&document, &more)
                .iter()
                .all(|city| city.visits > 0)
        );
    }

    #[test]
    fn visits_every_city_keeping_counts_in_step() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert_eq!(visit_all_cities(&mut document, &[]), 1);
        assert!(cities(&document).iter().all(|city| city.visits > 0));
        let economy = document.first_of_kind("economy").expect("economy");
        assert_eq!(economy.items("visited_cities"), ["berlin", "lyon"]);
        assert_eq!(economy.items("visited_cities_count"), ["4", "1"]);
        assert_eq!(visit_all_cities(&mut document, &[]), 0);
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
    fn unvisits_a_city_with_its_count() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        visit_all_cities(&mut document, &[]);
        let berlin = vec!["berlin".to_string()];
        assert_eq!(unvisit_cities(&mut document, &berlin), 1);
        let economy = document.first_of_kind("economy").expect("economy");
        assert_eq!(economy.items("visited_cities"), ["lyon"]);
        assert_eq!(economy.items("visited_cities_count"), ["1"]);
        assert_eq!(unvisit_cities(&mut document, &berlin), 0);
    }

    #[test]
    fn locks_a_dealer_again() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        let lyon = vec!["lyon".to_string()];
        assert_eq!(lock(&mut document, Place::Dealer, &lyon), 1);
        assert!(unlocked(&document, Place::Dealer).is_empty());
        assert_eq!(lock(&mut document, Place::Dealer, &lyon), 0);
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
