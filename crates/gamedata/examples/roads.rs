//! Counts the roads and junctions on the installed game's map (what "Discover every road" adds):
//! `cargo run -p gamedata --example roads -- <install folder>`.

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        eprintln!("usage: roads <install folder>");
        return;
    };
    let started = std::time::Instant::now();
    let ids = gamedata::road_ids(save::Game::Ets2, std::path::Path::new(&dir));
    println!(
        "{} roads and junctions in {:.1?}",
        ids.len(),
        started.elapsed()
    );
}
