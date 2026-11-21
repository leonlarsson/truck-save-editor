//! Writes one of the game's textures as a picture, to check texture decoding:
//! `cargo run -p gamedata --example icon -- <archive.scs> <path.tobj> <out.ppm> [sdf]`.
//! With `sdf`, draws a distance-field icon's shape instead of its raw channels. With a size
//! instead (`48`), and a `.mat` path, draws the icon as the app does, in its own colours, on grey.
//! Read-only.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [archive, path, out, rest @ ..] = args.as_slice() else {
        eprintln!("usage: icon <archive.scs> <path.tobj> <out.ppm> [sdf]");
        std::process::exit(2);
    };
    if let Some(size) = rest.first().and_then(|size| size.parse::<u32>().ok()) {
        let (folder, name) = path.rsplit_once('/').unwrap_or(("", path));
        let name = name.trim_end_matches(".mat");
        let icon = hashfs::Archive::open(archive)
            .ok()
            .and_then(|mut archive| gamedata::GameIcon::load(&mut archive, folder, name));
        let Some(icon) = icon else {
            eprintln!("{path}: couldn't read that icon");
            std::process::exit(1);
        };
        let bgra = icon.render(size, [None; 4], Some([18, 18, 18, 255]));
        // On mid grey, so every colour shows.
        let pixels: Vec<u8> = bgra
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| {
                let a = u32::from(p[3]);
                let mix = |c: u8| ((u32::from(c) * a + 128 * (255 - a)) / 255) as u8;
                [mix(p[2]), mix(p[1]), mix(p[0])]
            })
            .collect();
        let mut file = format!("P6\n{size} {size}\n255\n").into_bytes();
        file.extend_from_slice(&pixels);
        if let Err(error) = std::fs::write(out, file) {
            eprintln!("{out}: {error}");
            std::process::exit(1);
        }
        return;
    }
    let texture = hashfs::Archive::open(archive)
        .and_then(|mut archive| archive.read_texture(path))
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(1)
        });
    let Some(texture) = texture else {
        eprintln!("{path}: no texture there");
        std::process::exit(1);
    };
    println!("{}×{} {:?}", texture.width, texture.height, texture.format);
    let pixels: Vec<u8> = if rest.first().map(String::as_str) == Some("sdf") {
        let coverage = gamedata::distance_field_coverage(&texture, 0).unwrap_or_default();
        coverage
            .iter()
            .flat_map(|c| [255 - c, 255 - c, 255 - c])
            .collect()
    } else {
        // The four channels side by side, each in grey: red, green, blue, alpha.
        let rgba = gamedata::decode_rgba(&texture).unwrap_or_default();
        let width = texture.width as usize;
        let mut panels = Vec::new();
        for row in rgba.chunks_exact(width * 4) {
            for channel in 0..4 {
                panels.extend(row.as_chunks::<4>().0.iter().flat_map(|p| [p[channel]; 3]));
            }
        }
        panels
    };
    let panel_count = if rest.is_empty() { 4 } else { 1 };
    // A plain PPM: no image library needed.
    let mut file = format!(
        "P6\n{} {}\n255\n",
        texture.width * panel_count,
        texture.height
    )
    .into_bytes();
    file.extend_from_slice(&pixels);
    if let Err(error) = std::fs::write(out, file) {
        eprintln!("{out}: {error}");
        std::process::exit(1);
    }
}
