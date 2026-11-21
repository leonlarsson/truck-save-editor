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
        let trucks = save::vehicles(&slot.game);
        assert!(career.experience >= 0);
        assert!(!career.hq_city.is_empty());
        assert!(
            trucks
                .iter()
                .all(|truck| (0.0..=1.0).contains(&truck.wear.worst()))
        );

        // Every field the editor writes is there.
        assert_eq!(
            save::missing_edited_fields(&slot.game),
            Vec::<(&str, &str)>::new(),
            "{}: fields the editor writes are missing",
            dir.display()
        );

        // Every truck's wear is in a format the editor knows.
        assert!(
            trucks
                .iter()
                .all(|truck| save::wear_is_known(&slot.game, &truck.id)),
            "{}: a truck's wear is in an unknown format",
            dir.display()
        );

        // Every garage reads, and owned ones have slots.
        let garages = save::garages(&slot.game);
        assert!(!garages.is_empty(), "{}: no garages", dir.display());
        assert!(garages.iter().filter(|g| g.is_owned).all(|g| g.slots > 0));

        // Hired drivers read, with skills in range.
        assert!(save::drivers(&slot.game).iter().all(|d| {
            d.skills
                .levels()
                .iter()
                .all(|(_, l)| *l <= save::MAX_SKILL_LEVEL)
        }));

        // Every visited city is one the garages know.
        let cities = save::cities(&slot.game);
        let visited = slot
            .game
            .first_of_kind("economy")
            .map(|e| e.items("visited_cities").len())
            .unwrap_or(0);
        assert!(cities.iter().filter(|c| c.visits > 0).count() <= visited);

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

/// What the editor writes reads back exactly the same: the check after every save passes on real
/// saves. Written to a scratch folder; the fixtures aren't touched.
#[test]
fn real_saves_read_back_the_way_they_were_written() {
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let scratch =
            std::env::temp_dir().join(format!("tse-verify-{}-{index}", std::process::id()));
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(&scratch).expect("makes the scratch folder");
        std::fs::write(scratch.join("game.sii"), sii::to_text(&slot.game)).expect("writes");
        if let Some(info) = &slot.info {
            std::fs::write(scratch.join("info.sii"), sii::to_text(info)).expect("writes");
        }
        let result = save::verify(&scratch, &slot.game, slot.info.as_ref());
        let _ = std::fs::remove_dir_all(&scratch);
        assert!(result.is_ok(), "slot {index}: {result:?}");
    }
}
