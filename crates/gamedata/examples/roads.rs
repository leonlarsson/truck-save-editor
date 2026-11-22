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
}
