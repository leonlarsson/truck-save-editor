//! Research tool: finds map items by id in the installed game's map sectors and prints each one's
//! type, archive and sector, and the bytes around it (read-only):
//! `cargo run -p gamedata --example find_items -- <install folder> <ids file>`, the ids one per line.
//! With `--type <n>` in place of the ids file: every item of that type (with a plausible box), its
//! id and the bytes after its box.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let [_, dir, flag, kind] = args.as_slice()
        && flag == "--type"
    {
        let kind: u32 = kind.parse().unwrap_or(0);
        list_type(std::path::Path::new(dir), kind);
        return;
    }
    let [_, dir, ids_file] = args.as_slice() else {
        eprintln!("usage: find_items <install folder> <ids file> | --type <n>");
        return;
    };
    let ids: Vec<u64> = std::fs::read_to_string(ids_file)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();
    let dir = std::path::Path::new(dir);
    let mut archives = vec![dir.join("base_map.scs")];
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut dlcs: Vec<_> = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                name.starts_with("dlc_") && name.ends_with(".scs")
            })
            .collect();
        dlcs.sort();
        archives.extend(dlcs);
    }
    let mut found = 0;
    for path in archives {
        let Ok(mut archive) = hashfs::Archive::open(&path) else {
            continue;
        };
        let Ok(Some(listing)) = archive.list("/map/europe") else {
            continue;
        };
        for file in listing.files.iter().filter(|name| name.ends_with(".base")) {
            let Ok(Some(bytes)) = archive.read(&format!("/map/europe/{file}")) else {
                continue;
            };
            for id in &ids {
                let needle = id.to_le_bytes();
                let mut at = 0;
                while let Some(offset) = bytes[at..].windows(8).position(|w| w == needle) {
                    let start = at + offset;
                    let kind = start
                        .checked_sub(4)
                        .and_then(|from| bytes.get(from..start))
                        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
                    let around: Vec<String> = bytes
                        [start.saturating_sub(4)..(start + 60).min(bytes.len())]
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect();
                    println!(
                        "{id}\ttype {kind:?}\t{}\t{file}\t{}",
                        path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                        around.join("")
                    );
                    found += 1;
                    at = start + 8;
                }
            }
        }
    }
    eprintln!("{found} matches");
}

/// Every item of `kind` whose head looks like an item's: a non-zero id and a plausible box.
fn list_type(dir: &std::path::Path, kind: u32) {
    let mut archives = vec![dir.join("base_map.scs")];
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut dlcs: Vec<_> = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                name.starts_with("dlc_") && name.ends_with(".scs")
            })
            .collect();
        dlcs.sort();
        archives.extend(dlcs);
    }
    let mut seen = std::collections::HashSet::new();
    let f32_at = |bytes: &[u8], at: usize| -> Option<f32> {
        Some(f32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
    };
    for path in archives {
        let Ok(mut archive) = hashfs::Archive::open(&path) else {
            continue;
        };
        let Ok(Some(listing)) = archive.list("/map/europe") else {
            continue;
        };
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        for file in listing.files.iter().filter(|name| name.ends_with(".base")) {
            let Ok(Some(bytes)) = archive.read(&format!("/map/europe/{file}")) else {
                continue;
            };
            let needle = kind.to_le_bytes();
            let mut at = 20;
            while at + 44 <= bytes.len() {
                if bytes[at..at + 4] == needle {
                    let id =
                        u64::from_le_bytes(bytes[at + 4..at + 12].try_into().unwrap_or([0; 8]));
                    let low = [
                        f32_at(&bytes, at + 12),
                        f32_at(&bytes, at + 16),
                        f32_at(&bytes, at + 20),
                    ];
                    let high = [
                        f32_at(&bytes, at + 32),
                        f32_at(&bytes, at + 36),
                        f32_at(&bytes, at + 40),
                    ];
                    let is_box = (0..3).all(|axis| match (low[axis], high[axis]) {
                        (Some(l), Some(h)) => {
                            l.is_finite()
                                && h.is_finite()
                                && l <= h
                                && h - l < 5000.0
                                && l.abs() < 2e6
                        }
                        _ => false,
                    });
                    if id != 0 && is_box && seen.insert(id) {
                        let after: Vec<String> = bytes[at + 44..(at + 44 + 40).min(bytes.len())]
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect();
                        println!(
                            "{id}\t{name}\t{file}\t{:.0}\t{:.0}\t{}",
                            low[0].unwrap_or(0.0),
                            low[2].unwrap_or(0.0),
                            after.join("")
                        );
                    }
                }
                at += 1;
            }
        }
    }
    eprintln!("{} items of type {kind}", seen.len());
}
