//! A truck's or trailer's paint, in its panel: the paint job, from those the game has for the
//! model and that fit the vehicle (one per name: some come once per cabin), and the colours the
//! paint job lets the player change, with the saved colours as quick picks. Every change goes
//! through the session (`VehicleEdit::PaintJob`, `VehicleEdit::PaintColour`).

use std::sync::Arc;

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, Hsla, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, rems,
};
use save::PaintColour;

use super::fact;
use crate::game_data;
use crate::i18n::t;
use crate::session::{Session, paint_colour_label};
use crate::ui::{to_hsla, to_rgb8};

pub struct PaintEditor {
    session: Entity<Session>,
    vehicle: String,
    /// The model folder the paint jobs are in (`volvo.fh16`).
    model: String,
    job: Entity<SelectState<SearchableVec<PaintJobOption>>>,
    colours: Vec<(PaintColour, Entity<ColorPickerState>)>,
    /// The paint jobs that fit this vehicle, one per name, cheapest first (once read).
    choices: Vec<gamedata::PaintJob>,
    is_filled: bool,
    _subscriptions: Vec<Subscription>,
}

/// One paint job in the list: shown by its name, chosen by its definition.
#[derive(Clone)]
struct PaintJobOption {
    path: SharedString,
    title: SharedString,
}

impl SelectItem for PaintJobOption {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.path
    }
}

/// The paint jobs that fit a vehicle with these parts: one per name, the first that fits (a
/// paint job that names no parts fits every vehicle of its model).
fn fitting(jobs: &[gamedata::PaintJob], part_ids: &[String]) -> Vec<gamedata::PaintJob> {
    let mut chosen: Vec<gamedata::PaintJob> = Vec::new();
    for job in jobs {
        let fits = job.suitable_for.is_empty()
            || job.suitable_for.iter().any(|part| part_ids.contains(part));
        if fits && !chosen.iter().any(|other| other.name == job.name) {
            chosen.push(job.clone());
        }
    }
    chosen
}

impl PaintEditor {
    /// The editor for a vehicle's paint, whose paint jobs are in `model`'s folder.
    pub fn new(
        session: Entity<Session>,
        vehicle: String,
        model: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let paint = save::vehicle_paint(&session.read(cx).preview(), &vehicle);
        let job = cx.new(|cx| {
            SelectState::new(SearchableVec::new(Vec::new()), None, window, cx).searchable(true)
        });
        let colours: Vec<(PaintColour, Entity<ColorPickerState>)> = PaintColour::ALL
            .into_iter()
            .map(|which| {
                let shown = paint
                    .as_ref()
                    .and_then(|paint| paint.colour(which))
                    .map(|colour| to_hsla(save::to_display(colour)));
                let state = cx.new(|cx| {
                    let state = ColorPickerState::new(window, cx);
                    match shown {
                        Some(colour) => state.default_value(colour),
                        None => state,
                    }
                });
                (which, state)
            })
            .collect();

        let mut _subscriptions = vec![
            cx.subscribe_in(
                &job,
                window,
                |this, _, event: &SelectEvent<SearchableVec<PaintJobOption>>, _, cx| {
                    if let SelectEvent::Confirm(Some(path)) = event {
                        this.choose_job(path, cx);
                    }
                },
            ),
            // Undo, Discard and the like change what this shows.
            cx.observe_in(&session, window, |this, _, window, cx| {
                this.sync(window, cx)
            }),
        ];
        for (which, state) in &colours {
            let which = *which;
            _subscriptions.push(cx.subscribe(
                state,
                move |this, _, event: &ColorPickerEvent, cx| {
                    // A colour cleared (no colour at all) isn't paint: left as it is.
                    let ColorPickerEvent::Change(Some(colour)) = event else {
                        return;
                    };
                    let edit = save::VehicleEdit::PaintColour {
                        vehicle: this.vehicle.clone(),
                        colour: which,
                        linear: save::to_linear(to_rgb8(*colour)),
                    };
                    this.session
                        .update(cx, |session, cx| session.edit_vehicle(edit, cx));
                },
            ));
        }
        Self {
            session,
            vehicle,
            model,
            job,
            colours,
            choices: Vec::new(),
            is_filled: false,
            _subscriptions,
        }
    }

    /// Puts the chosen paint job on, in its own colours.
    fn choose_job(&mut self, path: &str, cx: &mut Context<Self>) {
        let Some(job) = self.choices.iter().find(|job| job.path == path) else {
            return;
        };
        let edit = save::VehicleEdit::PaintJob {
            vehicle: self.vehicle.clone(),
            job: save::NewPaintJob {
                data_path: job.path.clone(),
                price: job.price,
                base: job.base,
                masks: job.masks,
                flake: job.flake,
                flip: job.flip,
            },
        };
        self.session
            .update(cx, |session, cx| session.edit_vehicle(edit, cx));
    }

    /// The vehicle's paint job as listed: its own definition, or the one of the same name that
    /// fits (a paint job made for another cabin).
    fn listed_path(&self, data_path: &str, all: &[gamedata::PaintJob]) -> Option<SharedString> {
        if self.choices.iter().any(|job| job.path == data_path) {
            return Some(data_path.to_string().into());
        }
        let name = &all.iter().find(|job| job.path == data_path)?.name;
        self.choices
            .iter()
            .find(|job| &job.name == name)
            .map(|job| job.path.clone().into())
    }

    /// Fills the list once the model's paint jobs are read, with the vehicle's own selected.
    fn fill(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_filled {
            return;
        }
        let Some(all) = game_data::model_paint_jobs(cx, &self.model) else {
            return;
        };
        let preview = self.session.read(cx).preview();
        let parts = save::vehicle_part_ids(&preview, &self.vehicle);
        self.choices = fitting(&all, &parts);
        let items: Vec<PaintJobOption> = self
            .choices
            .iter()
            .map(|job| PaintJobOption {
                path: job.path.clone().into(),
                title: game_data::paint_job_name(cx, &job.path).into(),
            })
            .collect();
        let current = save::vehicle_paint(&preview, &self.vehicle)
            .and_then(|paint| self.listed_path(&paint.data_path, &all));
        self.job.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(items), window, cx);
            if let Some(current) = &current {
                select.set_selected_value(current, window, cx);
            }
        });
        self.is_filled = true;
    }

    /// Shows the paint as it'll be after saving: the paint job selected, the colours set.
    fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(paint) = save::vehicle_paint(&self.session.read(cx).preview(), &self.vehicle)
        else {
            return;
        };
        for (which, state) in &self.colours {
            let Some(colour) = paint.colour(*which) else {
                continue;
            };
            let shown = to_hsla(save::to_display(colour));
            let is_shown = state
                .read(cx)
                .value()
                .is_some_and(|value| to_rgb8(value) == to_rgb8(shown));
            if !is_shown {
                state.update(cx, |state, cx| state.set_value(shown, window, cx));
            }
        }
        if self.is_filled
            && let Some(all) = game_data::model_paint_jobs(cx, &self.model)
            && let Some(current) = self.listed_path(&paint.data_path, &all)
        {
            self.job.update(cx, |select, cx| {
                select.set_selected_value(&current, window, cx)
            });
        }
        cx.notify();
    }

    /// Which colours the vehicle's paint job lets the player change: by its definition, or, while
    /// that isn't known, the base colour when it has one.
    fn open_colours(&self, cx: &mut Context<Self>) -> Vec<PaintColour> {
        let Some(paint) = save::vehicle_paint(&self.session.read(cx).preview(), &self.vehicle)
        else {
            return Vec::new();
        };
        let definition = game_data::model_paint_jobs(cx, &self.model)
            .and_then(|all| all.iter().find(|job| job.path == paint.data_path).cloned());
        match definition {
            Some(job) => PaintColour::ALL
                .into_iter()
                .filter(|which| match which {
                    PaintColour::Base => !job.base_locked,
                    PaintColour::MaskR => job.has_pattern && !job.masks_locked[0],
                    PaintColour::MaskG => job.has_pattern && !job.masks_locked[1],
                    PaintColour::MaskB => job.has_pattern && !job.masks_locked[2],
                })
                .collect(),
            None => paint
                .base
                .map(|_| vec![PaintColour::Base])
                .unwrap_or_default(),
        }
    }
}

impl Render for PaintEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.fill(window, cx);
        let placeholder = if self.is_filled {
            t!(cx, "paint-job-pick")
        } else {
            t!(cx, "truck-part-loading")
        };
        let open = self.open_colours(cx);
        // The saved colours as quick picks in every colour picker.
        let saved: Vec<Hsla> = self
            .session
            .read(cx)
            .saved_colours()
            .into_iter()
            .flatten()
            .map(to_hsla)
            .collect();
        let saved = Arc::new(saved);
        let rows: Vec<_> = self
            .colours
            .iter()
            .filter(|(which, _)| open.contains(which))
            .map(|(which, state)| {
                let label = paint_colour_label(cx, *which);
                fact(
                    label.clone(),
                    ColorPicker::new(state)
                        .small()
                        .featured_colors(saved.as_ref().clone())
                        .accessibility_label(label),
                    cx,
                )
            })
            .collect();
        v_flex()
            .gap_1()
            .child(fact(
                t!(cx, "paint-job"),
                div().w(rems(15.)).child(
                    Select::new(&self.job)
                        .small()
                        .placeholder(placeholder)
                        .disabled(!self.is_filled),
                ),
                cx,
            ))
            .children(rows)
            .children(open.is_empty().then(|| {
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "paint-locked"))
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(path: &str, name: &str, fits: &[&str]) -> gamedata::PaintJob {
        gamedata::PaintJob {
            path: path.into(),
            name: name.into(),
            price: 0,
            unlock: 0,
            suitable_for: fits.iter().map(|part| part.to_string()).collect(),
            base_locked: false,
            masks_locked: [true; 3],
            has_pattern: false,
            base: None,
            masks: [None; 3],
            flake: None,
            flip: None,
        }
    }

    #[test]
    fn lists_each_paint_job_once_in_the_version_that_fits() {
        let jobs = [
            job("plain.sii", "@@plain@@", &[]),
            job("flag_a.sii", "@@flag@@", &["l2h1.volvo.fh16.cabin"]),
            job("flag_b.sii", "@@flag@@", &["l2h2.volvo.fh16.cabin"]),
            job("other_a.sii", "@@other@@", &["l2h3.volvo.fh16.cabin"]),
        ];
        let parts = ["l2h2.volvo.fh16.cabin".to_string()];
        let paths: Vec<String> = fitting(&jobs, &parts)
            .into_iter()
            .map(|job| job.path)
            .collect();
        assert_eq!(paths, ["plain.sii", "flag_b.sii"]);
    }
}
