//! Development builds only: `TSE_SCRIPT=<file>` drives the open save through a list of steps, one
//! per line, once the game's data and map are read. It's how `scripts/screenshots.sh` takes every
//! picture in one run, without keystrokes or clicks:
//!
//! ```text
//! page gallery            go to a page, by its slug (`TSE_PAGE`'s names)
//! theme dark              light or dark, for this run only (the settings stay as they are)
//! run Visit every city    a palette command, by its label as the palette shows it
//! open truck              open the first truck's panel (or `trailer`, `coach`)
//! close                   close the open panel or dialog
//! wait 1500               wait that many milliseconds (World map draws its tiles meanwhile)
//! shot gallery-dark       ask for a picture (see below), and wait until it's taken
//! ```
//!
//! The app can't capture its own window, so `shot <name>` writes `<name>` into `request` in the
//! folder `TSE_SHOT_DIR` names, and waits until the script capturing the window deletes it.
//! Lines starting with `#` are notes. When the steps are done, `done` is written there too.

use std::path::PathBuf;
use std::time::Duration;

use gpui_kit::component::WindowExt as _;
use gpui_kit::{Context, Window};

use crate::pages::Page;
use crate::settings::Appearance;
use crate::workspace::Workspace;

/// One step of a script.
#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    Page(Page),
    Theme(Appearance),
    Run(String),
    Open(save::VehicleKind),
    Close,
    Wait(u64),
    Shot(String),
}

/// The steps in a script's text; a line that isn't a step is reported and skipped.
pub fn parse(text: &str) -> Vec<Step> {
    let mut steps = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
        let rest = rest.trim();
        let step = match word {
            "page" => Page::ALL
                .into_iter()
                .find(|page| page.slug() == rest)
                .map(Step::Page),
            "theme" => match rest {
                "light" => Some(Step::Theme(Appearance::Light)),
                "dark" => Some(Step::Theme(Appearance::Dark)),
                _ => None,
            },
            "run" if !rest.is_empty() => Some(Step::Run(rest.to_string())),
            "close" => Some(Step::Close),
            "wait" => rest.parse().ok().map(Step::Wait),
            "open" => match rest {
                "truck" => Some(Step::Open(save::VehicleKind::Truck)),
                "trailer" => Some(Step::Open(save::VehicleKind::Trailer)),
                "coach" => Some(Step::Open(save::VehicleKind::Coach)),
                _ => None,
            },
            "shot" if !rest.is_empty() => Some(Step::Shot(rest.to_string())),
            _ => None,
        };
        match step {
            Some(step) => steps.push(step),
            None => eprintln!("TSE_SCRIPT: skipped a line it doesn't know: {line}"),
        }
    }
    steps
}

/// Starts the script `TSE_SCRIPT` names, if any (development builds only).
pub fn start(window: &mut Window, cx: &mut Context<Workspace>) {
    if !cfg!(debug_assertions) {
        return;
    }
    let Some(script) = std::env::var_os("TSE_SCRIPT") else {
        return;
    };
    let steps = match std::fs::read_to_string(&script) {
        Ok(text) => parse(&text),
        Err(error) => {
            eprintln!("TSE_SCRIPT: can't read it: {error}");
            return;
        }
    };
    let shots = std::env::var_os("TSE_SHOT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    cx.spawn_in(window, async move |this, cx| {
        // A copy of the executor: the closure can't hold on to `cx`, which each step needs.
        let executor = cx.background_executor().clone();
        let pause = |ms: u64| executor.timer(Duration::from_millis(ms));
        // Until the save is open and the game's data and map are read.
        loop {
            let is_ready = this
                .update(cx, |this, cx| this.is_ready_for_script(cx))
                .unwrap_or(false);
            if is_ready {
                break;
            }
            pause(100).await;
        }
        for step in steps {
            match step {
                Step::Wait(ms) => pause(ms).await,
                Step::Shot(name) => {
                    // A moment to draw what the last step changed.
                    pause(300).await;
                    let request = shots.join("request");
                    if std::fs::write(&request, &name).is_err() {
                        eprintln!("TSE_SCRIPT: can't ask for a picture in {}", shots.display());
                        continue;
                    }
                    // Taken when the capturing script deletes the request (ten seconds at most).
                    for _ in 0..200 {
                        if !request.exists() {
                            break;
                        }
                        pause(50).await;
                    }
                }
                step => {
                    let _ = this.update_in(cx, |this, window, cx| this.run_step(step, window, cx));
                }
            }
        }
        let _ = std::fs::write(shots.join("done"), "done");
    })
    .detach();
}

/// Carries out a step that acts on the app (not a wait or a picture).
pub fn act(
    workspace: &mut Workspace,
    step: Step,
    has_coaches: bool,
    session: Option<gpui_kit::Entity<crate::session::Session>>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    match step {
        Step::Page(page) => workspace.go_to(page, cx),
        Step::Theme(appearance) => {
            crate::theme::apply(appearance, Some(window), cx);
            window.refresh();
        }
        Step::Run(name) => match crate::palette::command_named(&name, has_coaches, cx) {
            Some(command) => workspace.run(command, window, cx),
            None => eprintln!("TSE_SCRIPT: no palette command is called {name}"),
        },
        Step::Open(kind) => {
            let vehicle = session.as_ref().and_then(|session| {
                session
                    .read(cx)
                    .vehicles()
                    .into_iter()
                    .find(|vehicle| vehicle.kind == kind)
            });
            match (session, vehicle) {
                (Some(session), Some(vehicle)) => {
                    crate::pages::details::open_vehicle(session, &vehicle, window, cx)
                }
                _ => eprintln!("TSE_SCRIPT: the save has no {kind:?} to open"),
            }
        }
        Step::Close => {
            window.close_dialog(cx);
            window.close_sheet(cx);
        }
        Step::Wait(_) | Step::Shot(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_script() {
        let steps = parse(
            "# a note\npage gallery\ntheme dark\nrun Visit every city\nopen trailer\nclose\nwait 500\nshot gallery-dark\nfly away\n",
        );
        assert_eq!(
            steps,
            [
                Step::Page(Page::Gallery),
                Step::Theme(Appearance::Dark),
                Step::Run("Visit every city".to_string()),
                Step::Open(save::VehicleKind::Trailer),
                Step::Close,
                Step::Wait(500),
                Step::Shot("gallery-dark".to_string()),
            ]
        );
    }
}
