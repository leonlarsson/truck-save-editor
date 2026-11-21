//! Reads the career and fleet of every real save in `$ETS2_FIXTURES` (local copies, never
//! committed), and checks writing the career back changes nothing else. Skips without fixtures.

use std::path::PathBuf;

fn slot_dirs() -> Vec<PathBuf> {
    let Some(root) = std::env::var_os("ETS2_FIXTURES").map(PathBuf::from) else {
        return Vec::new();
    };
    let Ok(games) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = games
        .flatten()
        .filter_map(|game| std::fs::read_dir(game.path()).ok())
        .flat_map(|slots| slots.flatten().map(|slot| slot.path()))
        .filter(|dir| dir.join("game.sii").is_file())
        .collect();
    dirs.sort();
    dirs
}

#[test]
fn real_saves_have_a_career_and_a_fleet() {
    for dir in slot_dirs() {
        let mut slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let career =
            save::Career::read(&slot.game).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let trucks = save::trucks(&slot.game);
        assert!(career.experience >= 0);
        assert!(!career.hq_city.is_empty());
        assert!(
            trucks
                .iter()
                .all(|truck| (0.0..=1.0).contains(&truck.wear.worst()))
        );

        // Writing the unchanged career back leaves the save exactly as it was.
        let before = sii::to_text(&slot.game);
        career.write(&mut slot.game).expect("writes");
        assert!(
            sii::to_text(&slot.game) == before,
            "{}: writing changed the save",
            dir.display()
        );

        let named = trucks.iter().filter(|t| t.model_id.is_some()).count();
        eprintln!(
            "{}: level data ok, {} trucks ({} with a known model)",
            dir.display(),
            trucks.len(),
            named
        );
    }
}
