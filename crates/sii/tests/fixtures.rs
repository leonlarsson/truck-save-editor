//! Reads every real save in `$ETS2_FIXTURES` (local copies, never committed), writes it as text,
//! reads that back and checks nothing changed. Skips quietly when there are no fixtures, as on CI.

use std::path::{Path, PathBuf};
use std::time::Instant;

fn sii_files(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sii_files(&path, found);
        } else if path.extension().is_some_and(|e| e == "sii")
            && path.file_name().is_some_and(|n| n != "profile.sii")
        {
            found.push(path);
        }
    }
}

#[test]
fn real_saves_round_trip_through_text() {
    let Some(root) = std::env::var_os("ETS2_FIXTURES").map(PathBuf::from) else {
        eprintln!("ETS2_FIXTURES isn't set; skipping");
        return;
    };
    let mut files = Vec::new();
    sii_files(&root, &mut files);
    files.sort();
    if files.is_empty() {
        eprintln!("no fixtures in {}; skipping", root.display());
        return;
    }

    for path in files {
        let bytes = std::fs::read(&path).expect("reads the file");
        let started = Instant::now();
        let document = sii::read(&bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let read_time = started.elapsed();

        let text = sii::to_text(&document);
        let again = sii::parse_text(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        // (The documents can differ in one harmless way: an empty array from binary is written
        // as just its count, `trucks: 0`, which reads back as a single value. So compare text.)
        assert!(
            sii::to_text(&again) == text,
            "{} changed going through text",
            path.display()
        );
        assert_eq!(again.len(), document.len());

        if path.file_name().is_some_and(|n| n == "game.sii") {
            assert!(
                document.first_of_kind("economy").is_some(),
                "{}: no economy",
                path.display()
            );
            assert!(
                document.first_of_kind("bank").is_some(),
                "{}: no bank",
                path.display()
            );
        }
        eprintln!(
            "{}: {} units, read in {:?}",
            path.display(),
            document.len(),
            read_time
        );
    }
}
