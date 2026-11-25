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

/// A job in progress reads, and cancelling it (where the editor can) removes the job and its
/// company trailer and leaves the save one tree, as the game leaves a delivered job.
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
        // Gone with nothing pointing at them, as when the game finishes a job, and the save
        // is still one tree.
        assert!(
            slot.game.unit(&job.id).is_none(),
            "slot {index}: the job is gone"
        );
        assert!(
            slot.game.unit(&trailer).is_none(),
            "slot {index}: the trailer is gone"
        );
        let root = slot
            .game
            .first_of_kind("economy")
            .map(|unit| unit.id.clone())
            .expect("an economy unit");
        assert_eq!(
            slot.game.reachable_from(&root).len(),
            slot.game.len(),
            "slot {index}: one tree"
        );
        let text = sii::to_text(&slot.game);
        // And it still reads back as written.
        let again = sii::parse_text(&text).expect("reads back");
        assert_eq!(sii::to_text(&again), text, "slot {index}");
    }
}

/// A truck moves to another garage with its driver, and the company keeps every truck and driver.
/// In real saves: a truck and a trailer copy into a garage with room, linked into the one tree
/// from the economy (nothing is cut off), and the save writes and reads back the same.
#[test]
fn real_trucks_and_trailers_copy_into_a_garage() {
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let mut slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let units_before = slot.game.len();
        let mut copied = 0;
        for kind in [save::VehicleKind::Truck, save::VehicleKind::Trailer] {
            let Some(vehicle) = save::vehicles(&slot.game)
                .into_iter()
                .find(|vehicle| vehicle.kind == kind)
            else {
                continue;
            };
            let Some(garage) = save::copy_targets(&slot.game, &vehicle.id)
                .unwrap_or_else(|problem| panic!("slot {index}: {problem:?}"))
                .into_iter()
                .next()
            else {
                continue;
            };
            let copy = save::copy_vehicle(&mut slot.game, &vehicle.id, &garage)
                .unwrap_or_else(|problem| panic!("slot {index}: {problem:?}"));
            assert!(
                save::vehicles(&slot.game).iter().any(|v| v.id == copy),
                "slot {index}: the copy is in the fleet"
            );
            copied += 1;
        }
        if copied == 0 {
            continue;
        }
        assert!(slot.game.len() > units_before, "slot {index}");
        assert_eq!(
            save::drop_unreachable(&mut slot.game),
            0,
            "slot {index}: the copies hang off the economy"
        );
        let text = sii::to_text(&slot.game);
        assert_eq!(
            sii::to_text(&sii::parse_text(&text).expect("reads back")),
            text,
            "slot {index}"
        );
    }
}

/// In real saves, with or without a truck of the company's: a truck bought into a garage with a
/// free slot is in the fleet, new (no wear, a full tank), hangs off the economy, and the save
/// writes and reads back the same. A used truck's accessories stand in for a dealer's (the test
/// has no game data).
#[test]
fn real_saves_buy_a_truck() {
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let mut slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let Some(garage) = save::buy_targets(&slot.game).into_iter().next() else {
            continue;
        };
        let Some(parts) = slot
            .game
            .units_of_kind("used_truck_offer")
            .filter_map(|offer| offer.single("truck"))
            .find_map(|id| slot.game.unit(id))
            .map(|truck| {
                truck
                    .items("accessories")
                    .iter()
                    .filter_map(|id| slot.game.unit(id).cloned())
                    .collect::<Vec<sii::Unit>>()
            })
        else {
            continue;
        };
        let id = save::buy_truck(&mut slot.game, &parts, &garage)
            .unwrap_or_else(|problem| panic!("slot {index}: {problem:?}"));
        let truck = save::vehicles(&slot.game)
            .into_iter()
            .find(|vehicle| vehicle.id == id)
            .unwrap_or_else(|| panic!("slot {index}: the truck is in the fleet"));
        assert_eq!(truck.wear.worst(), 0.0, "slot {index}");
        assert_eq!(
            save::drop_unreachable(&mut slot.game),
            0,
            "slot {index}: the truck hangs off the economy"
        );
        let text = sii::to_text(&slot.game);
        assert_eq!(
            sii::to_text(&sii::parse_text(&text).expect("reads back")),
            text,
            "slot {index}"
        );
    }
}

/// In real saves with a trailer: a trailer (a stand-in for a dealer's, made from the save's own
/// as a dealer file has it) bought into a garage with room is the company's, shares the save's
/// type of that name, hangs off the economy, and the save writes and reads back the same.
#[test]
fn real_saves_buy_a_trailer() {
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let mut slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let Some(garage) = save::trailer_targets(&slot.game).into_iter().next() else {
            continue;
        };
        let Some(trailer) = save::vehicles(&slot.game)
            .into_iter()
            .find(|vehicle| vehicle.kind == save::VehicleKind::Trailer)
        else {
            continue;
        };
        // The front trailer and its accessories, its type named as a dealer file names it.
        let Some(front) = slot.game.unit(&trailer.id).cloned() else {
            continue;
        };
        let Some(kind) = front
            .single("trailer_definition")
            .and_then(|id| slot.game.unit(id))
            .cloned()
        else {
            continue;
        };
        let name =
            sii::value::parse_string(kind.single("source_name").unwrap_or("\"\"")).to_string();
        let mut dealer_front = front.clone();
        if let Some(field) = dealer_front
            .fields
            .iter_mut()
            .find(|field| field.name == "trailer_definition")
        {
            field.value = sii::Value::single(name.clone());
        }
        if let Some(field) = dealer_front
            .fields
            .iter_mut()
            .find(|field| field.name == "slave_trailer")
        {
            field.value = sii::Value::single("null");
        }
        let mut units = vec![dealer_front];
        units.extend(
            front
                .items("accessories")
                .iter()
                .filter_map(|id| slot.game.unit(id).cloned()),
        );
        let mut definition = sii::Unit::new("trailer_def", name);
        definition.fields = kind.fields.clone();
        let types_before = slot.game.units_of_kind("trailer_def").count();
        let id = save::buy_trailer(&mut slot.game, &units, &[definition], &garage)
            .unwrap_or_else(|problem| panic!("slot {index}: {problem:?}"));
        assert!(
            save::vehicles(&slot.game)
                .iter()
                .any(|vehicle| vehicle.id == id),
            "slot {index}: the trailer is the company's"
        );
        assert_eq!(
            slot.game.units_of_kind("trailer_def").count(),
            types_before,
            "slot {index}: the type is shared"
        );
        assert_eq!(save::drop_unreachable(&mut slot.game), 0, "slot {index}");
        let text = sii::to_text(&slot.game);
        assert_eq!(
            sii::to_text(&sii::parse_text(&text).expect("reads back")),
            text,
            "slot {index}"
        );
    }
}

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

/// Every rotation (a four-number group) in real saves is written as the game reads it, with a
/// semicolon after the first number: `(w; x, y, z)`. The game refuses `(w, x, y, z)`.
#[test]
fn real_saves_write_rotations_the_way_the_game_reads_them() {
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let text = sii::to_text(&slot.game);
        let mut rotations = 0;
        for line in text.lines() {
            let Some((_, value)) = line.split_once(": ") else {
                continue;
            };
            for group in value.split('(').skip(1) {
                let group = group.split(')').next().unwrap_or_default();
                let numbers = group.split([',', ';']).count();
                if numbers == 4 {
                    rotations += 1;
                    let first = group.split([',', ';']).next().unwrap_or_default();
                    assert!(
                        group[first.len()..].starts_with(';'),
                        "slot {index}: a rotation written as ({group})"
                    );
                }
            }
        }
        assert!(rotations > 0, "slot {index}: a real save has rotations");
    }
}

/// A save is one tree: every unit in a save the game wrote is reachable from the economy unit.
/// (The game refuses a file with units nothing points at: "There are multiple unit trees".)
#[test]
fn real_saves_are_one_tree_from_the_economy() {
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let root = slot
            .game
            .first_of_kind("economy")
            .map(|unit| unit.id.clone())
            .expect("an economy unit");
        let reached = slot.game.reachable_from(&root);
        let stray: Vec<&str> = slot
            .game
            .units()
            .iter()
            .filter(|unit| !reached.contains(&unit.id))
            .map(|unit| unit.kind.as_str())
            .collect();
        assert!(
            stray.is_empty(),
            "slot {index}: {} units not reachable, kinds {:?}",
            stray.len(),
            stray.iter().take(5).collect::<Vec<_>>()
        );
    }
}

/// Removing a hired driver, a truck without a driver and a trailer from a real save keeps the
/// player's lists in step and the save one tree; forgetting the roads empties them.
#[test]
fn real_removals_keep_the_save_whole() {
    use save::Removal;
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let mut slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let game = &slot.game;
        let player = game
            .first_of_kind("economy")
            .and_then(|economy| economy.single("player"))
            .and_then(|id| game.unit(id))
            .expect("a player");
        let pick = |list: &str, make: fn(String) -> Removal| {
            player
                .items(list)
                .iter()
                .map(|id| make(id.clone()))
                .find(|removal| save::can_remove(game, removal).is_ok())
        };
        let removals: Vec<Removal> = [
            pick("drivers", Removal::Driver),
            pick("trucks", Removal::Truck),
            pick("trailers", Removal::Trailer),
        ]
        .into_iter()
        .flatten()
        .collect();
        // Each kind the save has a candidate for can be removed: here, a hired driver.
        let has_hired = player
            .items("drivers")
            .iter()
            .any(|id| game.unit(id).is_some_and(|unit| unit.kind == "driver_ai"));
        assert!(
            !has_hired || matches!(removals.first(), Some(Removal::Driver(_))),
            "slot {index}: a hired driver can be removed"
        );
        let lengths = |document: &sii::Document| {
            let player = document
                .first_of_kind("economy")
                .and_then(|economy| economy.single("player"))
                .and_then(|id| document.unit(id))
                .expect("a player");
            [
                "drivers",
                "driver_readiness_timer",
                "trucks",
                "truck_profit_logs",
                "trailers",
                "trailer_utilization_logs",
            ]
            .map(|list| player.items(list).len())
        };
        let before = lengths(&slot.game);
        for removal in &removals {
            assert_eq!(
                save::remove(&mut slot.game, removal),
                Ok(()),
                "slot {index}: {removal:?}"
            );
        }
        let after = lengths(&slot.game);
        // Each list lost exactly one entry when its kind was removed, and they stay in step.
        assert_eq!(after[0], after[1], "slot {index}: drivers in step");
        assert_eq!(after[2], after[3], "slot {index}: trucks in step");
        assert_eq!(after[4], after[5], "slot {index}: trailers in step");
        assert!(
            after
                .iter()
                .zip(before)
                .all(|(a, b)| *a == b || *a + 1 == b)
        );

        save::forget_roads(&mut slot.game, slot.info.as_mut());
        assert_eq!(save::discovered_roads(&slot.game), 0);
        let root = slot
            .game
            .first_of_kind("economy")
            .map(|unit| unit.id.clone())
            .expect("an economy unit");
        assert_eq!(
            slot.game.reachable_from(&root).len(),
            slot.game.len(),
            "slot {index}: one tree"
        );
        let text = sii::to_text(&slot.game);
        assert_eq!(
            sii::to_text(&sii::parse_text(&text).expect("reads back")),
            text
        );
    }
}

/// Changing a real trailer's cargo, weight and countries, a truck's fuel and its badge keeps the
/// save whole and reads back.
#[test]
fn real_vehicle_edits_keep_the_save_whole() {
    use save::VehicleEdit;
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let mut slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let vehicles = save::vehicles(&slot.game);
        let mut edits = Vec::new();
        if let Some(trailer) = vehicles
            .iter()
            .find(|v| v.kind == save::VehicleKind::Trailer)
        {
            let load = save::trailer_load(&slot.game, &trailer.id).expect("a trailer's load");
            edits.push(VehicleEdit::CargoMass {
                trailer: trailer.id.clone(),
                kg: 20000.0,
            });
            edits.push(VehicleEdit::EmptyMass {
                trailer: trailer.id.clone(),
                kg: load.empty_kg / 2.0,
            });
            edits.push(VehicleEdit::Unrestrict {
                trailer: trailer.id.clone(),
            });
        }
        if let Some(truck) = vehicles.iter().find(|v| {
            v.kind == save::VehicleKind::Truck && save::badge(&slot.game, &v.id).is_some()
        }) {
            edits.push(VehicleEdit::Fuel {
                vehicle: truck.id.clone(),
                ratio: 3.0,
            });
            edits.push(VehicleEdit::RemoveBadge {
                vehicle: truck.id.clone(),
            });
        }
        for edit in &edits {
            assert!(
                save::apply_vehicle_edit(&mut slot.game, edit),
                "slot {index}: {edit:?}"
            );
        }
        for edit in &edits {
            match edit {
                VehicleEdit::CargoMass { trailer, .. } => {
                    let load = save::trailer_load(&slot.game, trailer).expect("a trailer's load");
                    assert!((load.cargo_kg - 20000.0).abs() < 1.0, "slot {index}");
                    assert!(load.countries.is_empty(), "slot {index}");
                }
                VehicleEdit::RemoveBadge { vehicle } => {
                    assert!(save::badge(&slot.game, vehicle).is_none(), "slot {index}");
                }
                _ => {}
            }
        }
        let root = slot
            .game
            .first_of_kind("economy")
            .map(|unit| unit.id.clone())
            .expect("economy");
        assert_eq!(
            slot.game.reachable_from(&root).len(),
            slot.game.len(),
            "slot {index}: one tree"
        );
        let text = sii::to_text(&slot.game);
        assert_eq!(
            sii::to_text(&sii::parse_text(&text).expect("reads back")),
            text
        );
    }
}

/// Every real truck has an engine and a transmission the editor finds, and swapping one keeps
/// the save whole.
#[test]
fn real_trucks_have_parts_to_swap() {
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let mut slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let trucks: Vec<save::Vehicle> = save::vehicles(&slot.game)
            .into_iter()
            .filter(|v| v.kind == save::VehicleKind::Truck)
            .collect();
        for truck in &trucks {
            for part in ["engine", "transmission"] {
                assert!(
                    save::vehicle_part(&slot.game, &truck.id, part).is_some(),
                    "slot {index}: a truck without its {part}"
                );
            }
        }
        if let Some(truck) = trucks.first() {
            let (_, path) = save::vehicle_part(&slot.game, &truck.id, "engine").expect("an engine");
            let edit = save::VehicleEdit::Part {
                vehicle: truck.id.clone(),
                part: "engine".into(),
                data_path: path,
                price: 1,
            };
            assert!(save::apply_vehicle_edit(&mut slot.game, &edit));
            let text = sii::to_text(&slot.game);
            assert_eq!(
                sii::to_text(&sii::parse_text(&text).expect("reads back")),
                text
            );
        }
    }
}

/// In real saves: the player's truck and position are found, another truck can be driven without
/// losing a driver, and a position can be moved to.
#[test]
fn real_saves_know_the_players_truck_and_position() {
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let mut slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let position = save::position(&slot.game);
        // A new career has no truck of its own yet (it drives quick jobs), so no position (the
        // game keeps zeros) and nothing more to try.
        let has_trucks = slot
            .game
            .first_of_kind("player")
            .is_some_and(|player| !player.items("trucks").is_empty());
        if !has_trucks {
            assert!(
                position.is_none(),
                "slot {index}: no position without a truck"
            );
            continue;
        }
        assert!(position.is_some(), "slot {index}: the player's position");
        let own = save::own_truck(&slot.game);
        assert!(own.is_some(), "slot {index}: the player's truck");
        let drivers = |document: &sii::Document| {
            save::garages(document)
                .iter()
                .map(|garage| garage.drivers)
                .sum::<usize>()
        };
        let before = drivers(&slot.game);
        let other = save::vehicles(&slot.game).into_iter().find(|v| {
            v.kind == save::VehicleKind::Truck && save::can_drive(&slot.game, &v.id).is_ok()
        });
        if let Some(other) = other {
            assert_eq!(
                save::drive_truck(&mut slot.game, &other.id),
                Ok(()),
                "slot {index}"
            );
            assert_eq!(
                save::own_truck(&slot.game),
                Some(other.id.clone()),
                "slot {index}"
            );
            assert_eq!(drivers(&slot.game), before, "slot {index}: nobody lost");
        }
        if save::can_move_player(&slot.game).is_ok()
            && let Some(position) = position
        {
            assert_eq!(
                save::move_player(&mut slot.game, &position),
                Ok(()),
                "slot {index}"
            );
        }
        let text = sii::to_text(&slot.game);
        assert_eq!(
            sii::to_text(&sii::parse_text(&text).expect("reads back")),
            text
        );
    }
}

/// Resting, clearing the record and moving the clock on a real save keep it whole.
#[test]
fn real_saves_take_the_players_day_edits() {
    use save::PlayerEdit;
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let mut slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let before = save::time_of_day(&slot.game).expect("a clock");
        for edit in [
            PlayerEdit::Rest,
            PlayerEdit::ClearOffences,
            PlayerEdit::SkipTo { hour: 8 },
        ] {
            save::apply_player_edit(&mut slot.game, &edit);
        }
        assert_eq!(save::offences(&slot.game).0, 0, "slot {index}");
        let after = save::time_of_day(&slot.game).expect("a clock");
        assert_eq!(after.1, 8 * 60, "slot {index}");
        assert!(after != before, "slot {index}: the clock moved");
        let root = slot
            .game
            .first_of_kind("economy")
            .map(|unit| unit.id.clone())
            .expect("economy");
        assert_eq!(
            slot.game.reachable_from(&root).len(),
            slot.game.len(),
            "slot {index}: one tree"
        );
        let text = sii::to_text(&slot.game);
        assert_eq!(
            sii::to_text(&sii::parse_text(&text).expect("reads back")),
            text
        );
    }
}

#[test]
fn real_saves_have_readable_statistics() {
    for (index, dir) in slot_dirs().into_iter().enumerate() {
        let slot = save::load(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let deliveries = save::deliveries(&slot.game);
        assert!(
            deliveries
                .iter()
                .all(|delivery| !delivery.cargo.is_empty() && delivery.distance_km >= 0),
            "save {index}: every delivery has a cargo"
        );
        let totals = save::totals(&slot.game);
        if !deliveries.is_empty() {
            assert!(
                totals.distance_km > 0,
                "save {index}: it has driven somewhere"
            );
        }
        // Only counts, nothing personal.
        println!("save {index}: {} deliveries", deliveries.len());
    }
}
