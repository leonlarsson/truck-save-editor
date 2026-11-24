//! Counts the cities with a recruitment agency, read from the game's map and prefab definitions:
//! `cargo run -p gamedata --example agencies -- <install folder> [ets2|ats]`. Read-only.

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(dir) = args.next() else {
        eprintln!("usage: agencies <install folder> [ets2|ats]");
        return;
    };
    let game = match args.next().as_deref() {
        Some("ats") => save::Game::Ats,
        _ => save::Game::Ets2,
    };
    let dir = std::path::Path::new(&dir);
    let Ok(data) = gamedata::GameData::load(game, dir, "en_us") else {
        eprintln!("couldn't read the game's data");
        return;
    };
    let map = gamedata::map_contents(game, dir);
    let cities = gamedata::agency_cities(&map, &data.agency_models);
    println!(
        "{} agency models, {} cities with an agency",
        data.agency_models.len(),
        cities.len()
    );
    if let Some(listed) = data.recruitment_cities() {
        let missing: Vec<&String> = listed.difference(&cities).collect();
        let extra: Vec<&String> = cities.difference(listed).collect();
        println!("against the shipped list: missing {missing:?}, not in it {extra:?}");
    }
}
