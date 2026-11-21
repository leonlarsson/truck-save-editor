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

/// A job in progress reads, and cancelling it (where the editor can) leaves nothing pointing at
/// the job or its company trailer, as the game leaves a delivered job.
#[test]
fn real_jobs_read_and_cancel_cleanly() {
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let mut slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let Some(job) = save::current_job(&slot.game) else {
            continue;
        };
        assert!(!job.cargo.is_empty(), "slot {index}: the job has cargo");
        assert!(
            !job.to_city.is_empty(),
            "slot {index}: the job has a destination"
        );
        if !job.kind.can_cancel() {
            continue;
        }
        let trailer = slot
            .game
            .unit(&job.id)
            .and_then(|unit| unit.single("company_trailer"))
            .unwrap_or("null")
            .to_string();
        assert!(save::cancel_job(&mut slot.game), "slot {index}: cancels");
        assert!(save::current_job(&slot.game).is_none(), "slot {index}");
        let text = sii::to_text(&slot.game);
        let references = |id: &str| {
            text.lines()
                .filter(|line| line.ends_with(&format!(": {id}")))
                .count()
        };
        assert_eq!(
            references(&job.id),
            0,
            "slot {index}: nothing points at the job"
        );
        // Only the job itself (now unreferenced) still names its trailer.
        assert_eq!(
            references(&trailer),
            1,
            "slot {index}: nothing else points at the trailer"
        );
        // And it still reads back as written.
        let again = sii::parse_text(&text).expect("reads back");
        assert_eq!(sii::to_text(&again), text, "slot {index}");
    }
}

/// A truck moves to another garage with its driver, and the company keeps every truck and driver.
#[test]
fn real_trucks_move_between_garages() {
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let mut slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let totals = |document: &sii::Document| {
            save::garages(document)
                .iter()
                .fold((0, 0), |(trucks, drivers), garage| {
                    (trucks + garage.trucks, drivers + garage.drivers)
                })
        };
        let before = totals(&slot.game);
        let garages = save::garages(&slot.game);
        let trucks = save::vehicles(&slot.game);
        // The first truck that can move somewhere, and where.
        let found = trucks.iter().find_map(|truck| {
            garages
                .iter()
                .find(|garage| save::can_move_to_garage(&slot.game, &truck.id, &garage.id).is_ok())
                .map(|garage| (truck.id.clone(), garage.id.clone()))
        });
        let Some((truck, target)) = found else {
            continue;
        };
        assert_eq!(
            save::move_to_garage(&mut slot.game, &truck, &target),
            Ok(()),
            "slot {index}"
        );
        assert_eq!(
            save::garage_of(&slot.game, &truck),
            Some(target),
            "slot {index}"
        );
        assert_eq!(
            totals(&slot.game),
            before,
            "slot {index}: nothing lost or doubled"
        );
    }
}
