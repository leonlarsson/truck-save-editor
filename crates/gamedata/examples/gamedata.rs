//! Shows what the editor reads from the installed game: `cargo run -p gamedata --example gamedata`.

fn main() {
    let game = save::Game::Ets2;
    let Some(dir) = gamedata::find_install(game) else {
        println!("{} wasn't found", game.name());
        return;
    };
    println!("{}: {}", game.name(), dir.display());
    match gamedata::GameData::load(game, &dir, "en_us") {
        Ok(data) => {
            let mut by_dlc: std::collections::BTreeMap<&str, usize> = Default::default();
            for (_, dlc) in data.cities() {
                *by_dlc.entry(dlc.unwrap_or("base")).or_default() += 1;
            }
            println!("{} cities: {by_dlc:?}", data.cities().count());
            let album = &data.photo_album;
            println!(
                "{} viewpoints, {} photo trophies; first: {:?}",
                album.viewpoints.len(),
                album.landmarks.len(),
                album.landmarks.first()
            );
            for level in [0, 5, 30, 100, 182] {
                println!(
                    "level {level}: starts at {} XP, rank {:?}, coach rank {:?}",
                    data.truck_levels.experience_for(level),
                    data.truck_rank(level),
                    data.coach_rank(level)
                );
            }
            for skill in [
                "long_dist",
                "adr",
                "long_duty",
                "satisfaction_increase",
                "ecodriving",
            ] {
                println!("{skill}: up to {:?}", data.max_skill_level(skill));
            }
            for city in ["malmo", "krakow", "zurich", "berlin"] {
                println!("{city}: {:?}", data.city_name(city));
            }
            for model in [
                "volvo.fh16_2012",
                "scania.s_2016",
                "man.lions_24",
                "scs.box",
            ] {
                println!("{model}: {:?}", data.model_name(model));
            }
            println!(
                "renar: {:?}, battery: {:?}",
                data.company_name("renar"),
                data.cargo_name("battery")
            );
            println!(
                "{} cities with a dealer, {} with a recruitment agency",
                data.dealer_cities().len(),
                data.recruitment_cities().len()
            );
        }
        Err(error) => println!("couldn't read it: {error}"),
    }
}
