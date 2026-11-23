//! Lists what companies ship and take, from the installed game (read-only):
//! `cargo run -p gamedata --example companies -- agronord`.

fn main() {
    let Some(dir) = gamedata::find_install(save::Game::Ets2) else {
        println!("The game wasn't found");
        return;
    };
    let cargo = gamedata::company_cargo(&dir);
    println!(
        "{} companies ship, {} take",
        cargo.ships.len(),
        cargo.takes.len()
    );
    if let Some(company) = std::env::args().nth(1) {
        println!("{company} ships: {:?}", cargo.ships.get(&company));
        println!("{company} takes: {:?}", cargo.takes.get(&company));
    }
}
