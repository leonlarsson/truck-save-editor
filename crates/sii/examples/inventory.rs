//! Lists what a SII file is made of, or what changed between two:
//!
//! ```text
//! cargo run -p sii --example inventory -- game.sii              # every unit kind and field
//! cargo run -p sii --example inventory -- old/game.sii new/game.sii   # what changed
//! ```
//!
//! For checking a new game version (`docs/compatibility.md`). Prints structure only, never values.

use sii::inventory::{diff, inventory};

fn read(path: &str) -> sii::Document {
    let result = std::fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| sii::read(&bytes).map_err(|e| e.to_string()));
    result.unwrap_or_else(|error| {
        eprintln!("{path}: {error}");
        std::process::exit(1);
    })
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [path] => {
            for (kind, info) in inventory(&read(path)).kinds {
                println!("{kind} ({} units)", info.count);
                for (field, shape) in info.fields {
                    println!("    {field}: {shape}");
                }
            }
        }
        [before, after] => {
            let changes = diff(&inventory(&read(before)), &inventory(&read(after)));
            if changes.is_empty() {
                println!("No changes in units, fields or value shapes.");
                return;
            }
            for kind in &changes.added_kinds {
                println!("+ unit kind {kind}");
            }
            for kind in &changes.removed_kinds {
                println!("- unit kind {kind}");
            }
            for (kind, field, shape) in &changes.added_fields {
                println!("+ {kind}.{field}: {shape}");
            }
            for (kind, field, shape) in &changes.removed_fields {
                println!("- {kind}.{field}: {shape}");
            }
            for (kind, field, before, after) in &changes.changed_fields {
                println!("~ {kind}.{field}: {before} → {after}");
            }
        }
        _ => {
            eprintln!("usage: inventory <file.sii> [<newer file.sii>]");
            std::process::exit(2);
        }
    }
}
