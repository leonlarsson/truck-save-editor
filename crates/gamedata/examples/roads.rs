//! Counts the roads and junctions on the installed game's map (what "Discover every road" adds),
//! and its cities and garages: `cargo run -p gamedata --example roads -- <install folder>`.

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        eprintln!("usage: roads <install folder>");
        return;
    };
    let started = std::time::Instant::now();
    let map = gamedata::map_contents(save::Game::Ets2, std::path::Path::new(&dir));
    let cities: std::collections::BTreeSet<&str> = map
        .places
        .iter()
        .filter(|place| place.kind == gamedata::MapPlaceKind::CityArea)
        .map(|place| place.city.as_str())
        .collect();
    let garages = map
        .places
        .iter()
        .filter(|place| place.kind == gamedata::MapPlaceKind::Garage)
        .count();
    println!(
        "{} roads and junctions, {} cities, {} garages, in {:.1?}",
        map.roads.len(),
        cities.len(),
        garages,
        started.elapsed()
    );
    // The game's cities without an area on the map.
    if let Ok(data) =
        gamedata::GameData::load(save::Game::Ets2, std::path::Path::new(&dir), "en_us")
    {
        let missing: Vec<String> = data
            .cities()
            .filter(|(token, _)| !cities.contains(token))
            .map(|(token, dlc)| format!("{token} ({})", dlc.unwrap_or("base")))
            .collect();
        println!(
            "{} defined cities have no area on the map: {missing:?}",
            missing.len()
        );
        // `--list`: every defined city, its DLC and whether it has a map area, tab separated.
        if std::env::args().any(|arg| arg == "--list") {
            for (token, dlc) in data.cities() {
                println!(
                    "city\t{token}\t{}\t{}",
                    dlc.unwrap_or("base"),
                    cities.contains(token)
                );
            }
        }
    }
}
