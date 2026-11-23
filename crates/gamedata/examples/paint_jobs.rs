//! Lists the paint jobs a model can have, from the installed game (read-only):
//! `cargo run -p gamedata --example paint_jobs -- volvo.fh16`.

fn main() {
    let model = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "volvo.fh16".to_string());
    let Some(dir) = gamedata::find_install(save::Game::Ets2) else {
        println!("The game wasn't found");
        return;
    };
    let jobs = gamedata::model_paint_jobs(&dir, &model);
    println!("{model}: {} paint jobs", jobs.len());
    for job in &jobs {
        let open: Vec<&str> = [
            (!job.base_locked).then_some("base"),
            (job.has_pattern && !job.masks_locked[0]).then_some("1"),
            (job.has_pattern && !job.masks_locked[1]).then_some("2"),
            (job.has_pattern && !job.masks_locked[2]).then_some("3"),
        ]
        .into_iter()
        .flatten()
        .collect();
        println!(
            "{:>7} {:<40} {:<32} colours: {:<14} fits: {}",
            job.price,
            job.path.rsplit('/').next().unwrap_or_default(),
            job.name,
            open.join(" "),
            job.suitable_for.join(", ")
        );
    }
}
