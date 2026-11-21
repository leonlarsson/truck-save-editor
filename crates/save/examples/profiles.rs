//! Lists the profiles and slots found on this computer: `cargo run -p save --example profiles`.
//! Read-only; for checking profile detection while developing.

fn main() {
    for game in save::Game::ALL {
        for profile in save::find_profiles(game) {
            let slots = save::profile_slots(&profile.dir);
            println!(
                "{} · {} ({:?}) · {} slots",
                game.name(),
                profile.name,
                profile.location,
                slots.len()
            );
            for slot in slots.iter().take(6) {
                println!(
                    "    {:<20} {:<28} saved {}",
                    slot.folder,
                    slot.display_name(),
                    slot.info
                        .saved_at
                        .map(|t| {
                            let (y, mo, d, h, mi, _) = save::utc_parts(t);
                            format!("{y}-{mo:02}-{d:02} {h:02}:{mi:02} UTC")
                        })
                        .unwrap_or_default()
                );
            }
        }
    }
}
