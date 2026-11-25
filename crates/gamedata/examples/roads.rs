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
    let with_line: std::collections::HashSet<u64> = map.lines.iter().map(|line| line.id).collect();
    println!(
        "{} lines; {} of the roads and junctions have one",
        map.lines.len(),
        with_line.len()
    );
    let junctions: std::collections::HashSet<u64> =
        map.prefabs.iter().map(|prefab| prefab.id).collect();
    let (mut roads_without, mut junctions_without) = (0, 0);
    for road in map
        .roads
        .iter()
        .filter(|road| !with_line.contains(&road.id))
    {
        if junctions.contains(&road.id) {
            junctions_without += 1;
        } else {
            roads_without += 1;
        }
    }
    println!(
        "without a line: {roads_without} roads, {junctions_without} of {} junctions",
        junctions.len()
    );
    // With a city's token after the folder: the lines within 3 km of it, as points along each
    // (`east south` a line, a blank line between lines), to plot and compare with the game's map.
    if let Some(city) = std::env::args().nth(2) {
        let Some(area) = map.places.iter().find(|place| place.city == city) else {
            eprintln!("no city {city} on the map");
            return;
        };
        for line in &map.lines {
            let near =
                |at: [f32; 2]| (at[0] - area.x).abs() < 3000.0 && (at[1] - area.z).abs() < 3000.0;
            if near(line.from) || near(line.to) {
                for step in 0..=16 {
                    let at = line.point(step as f32 / 16.0);
                    println!("{} {}", at[0] - area.x, at[1] - area.z);
                }
                println!();
            }
        }
        return;
    }
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
        // The photo album's viewpoints and photo trophies the scan placed on the map.
        let album = &data.photo_album;
        let viewpoints = album
            .viewpoints
            .iter()
            .filter(|item| {
                map.markers.iter().any(|marker| {
                    matches!(marker, gamedata::MapMarker::Viewpoint { item: id, .. } if item.map_items.contains(id))
                })
            })
            .count();
        let landmarks = album
            .landmarks
            .iter()
            .filter(|item| {
                map.markers.iter().any(|marker| {
                    matches!(marker, gamedata::MapMarker::Landmark { id, .. } if *id == item.id)
                })
            })
            .count();
        println!(
            "on the map: {viewpoints} of {} viewpoints, {landmarks} of {} photo trophies ({} markers found)",
            album.viewpoints.len(),
            album.landmarks.len(),
            map.markers.len()
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
