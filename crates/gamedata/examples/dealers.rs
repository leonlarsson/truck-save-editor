//! Lists the trucks and trailers the game's dealers sell ready-made, with what each costs (its parts' prices
//! together): `cargo run -p gamedata --example dealers -- <install folder> [ets2|ats]`. Read-only.

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(dir) = args.next() else {
        eprintln!("usage: dealers <install folder> [ets2|ats]");
        return;
    };
    let game = match args.next().as_deref() {
        Some("ats") => save::Game::Ats,
        _ => save::Game::Ets2,
    };
    let dir = std::path::Path::new(&dir);
    let data = gamedata::GameData::load(game, dir, "en_us").ok();
    let trucks = gamedata::dealer_trucks(dir);
    println!("{} trucks", trucks.len());
    for truck in &trucks {
        let name = |raw: &str| match &data {
            Some(data) => data.resolve(raw),
            None => raw.to_string(),
        };
        let model = data
            .as_ref()
            .and_then(|data| data.model_name(&truck.model))
            .unwrap_or_else(|| truck.model.clone());
        println!(
            "{model} · {} · {} · {} parts · {} · {:?}",
            name(&truck.chassis),
            name(&truck.engine),
            truck.accessories.len(),
            truck.price,
            truck.dlc
        );
    }
    let trailers = gamedata::dealer_trailers(dir);
    println!("{} trailers", trailers.len());
    for trailer in &trailers {
        let chassis = match &data {
            Some(data) => data.resolve(&trailer.chassis),
            None => trailer.chassis.clone(),
        };
        let count = trailer
            .units
            .iter()
            .filter(|unit| unit.kind == "trailer")
            .count();
        println!(
            "{} {} · {chassis} · {count} trailer(s) · {} · {:?}",
            trailer.model, trailer.body_type, trailer.price, trailer.dlc
        );
    }
}
