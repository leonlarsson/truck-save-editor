//! Moving the player's truck, on the Map page: asking the game for a free quick travel when the
//! save loads (the player picks any visited city in the game), or to where it stands in another
//! save of the same profile.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _,
    button::Button,
    h_flex,
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Task, Window, div, rems,
};

use crate::i18n::t;
use crate::labels;
use crate::session::Session;
use crate::ui::Tip as _;

pub struct PositionPicker {
    session: Entity<Session>,
    /// The profile's other saves.
    slots: Vec<save::Slot>,
    select: Entity<SelectState<SearchableVec<SlotOption>>>,
    /// What's happening or what went wrong, if anything.
    status: Option<(SharedString, bool)>,
    _task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl PositionPicker {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (profile_dir, open_dir) = {
            let session = session.read(cx);
            (session.profile().dir.clone(), session.slot().dir.clone())
        };
        let slots: Vec<save::Slot> = save::profile_slots(&profile_dir)
            .into_iter()
            .filter(|slot| slot.dir != open_dir)
            .collect();
        let items: Vec<SlotOption> = slots
            .iter()
            .enumerate()
            .map(|(index, slot)| SlotOption {
                index: index.to_string().into(),
                name: labels::slot_name(cx, slot),
            })
            .collect();
        let select = cx.new(|cx| {
            SelectState::new(SearchableVec::new(items), None, window, cx).searchable(true)
        });
        let _subscriptions = vec![
            cx.subscribe_in(
                &select,
                window,
                |this, _, event: &SelectEvent<SearchableVec<SlotOption>>, window, cx| {
                    if let SelectEvent::Confirm(Some(index)) = event
                        && let Ok(index) = index.parse::<usize>()
                    {
                        this.move_to(index, cx);
                        // An action, not a setting: the list goes back to its prompt.
                        this.select
                            .update(cx, |select, cx| select.set_selected_index(None, window, cx));
                    }
                },
            ),
            cx.observe(&session, |_, _, cx| cx.notify()),
        ];
        Self {
            session,
            slots,
            select,
            status: None,
            _task: None,
            _subscriptions,
        }
    }

    /// Reads the chosen save in the background and moves the player to its position.
    fn move_to(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(slot) = self.slots.get(index).cloned() else {
            return;
        };
        let name = labels::slot_name(cx, &slot).to_string();
        self.status = Some((t!(cx, "player-move-reading", save = &name), false));
        cx.notify();
        self._task = Some(cx.spawn(async move |this, cx| {
            let dir = slot.dir.clone();
            let position = cx
                .background_spawn(async move {
                    save::load(&dir)
                        .ok()
                        .and_then(|loaded| save::position(&loaded.game))
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                match position {
                    Some(position) => {
                        this.status = None;
                        this.session
                            .update(cx, |session, cx| session.move_player(position, name, cx));
                    }
                    None => {
                        this.status = Some((t!(cx, "player-move-no-position", save = &name), true));
                    }
                }
                cx.notify();
            });
        }));
    }
}

impl Render for PositionPicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let session = self.session.read(cx);
        let travel_problem = session.can_travel().err();
        let is_travel_asked = session.quick_travel();
        let move_problem = session.can_move_player().err();
        let why = |problem: save::PlayerProblem, cx: &Context<Self>| match problem {
            save::PlayerProblem::OnJob => t!(cx, "player-why-on-job"),
            save::PlayerProblem::TrailerHitched => t!(cx, "player-move-hitched"),
            save::PlayerProblem::NoOwnTruck => t!(cx, "player-why-no-truck"),
            _ => t!(cx, "player-quick-travel-unavailable"),
        };
        let muted = cx.theme().muted_foreground;
        // Asked for: what happens when the save loads. Otherwise why it can't be, if it can't.
        let note = if is_travel_asked {
            Some(t!(cx, "player-quick-travel-asked"))
        } else {
            travel_problem.map(|problem| why(problem, cx))
        };
        v_flex()
            .gap_1()
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        Button::new("quick-travel")
                            .small()
                            .outline()
                            .label(t!(cx, "player-quick-travel"))
                            .tip(t!(cx, "player-quick-travel-tooltip"))
                            .disabled(travel_problem.is_some() || is_travel_asked)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.session
                                    .update(cx, |session, cx| session.set_quick_travel(true, cx));
                            })),
                    )
                    .child(
                        div().w(rems(14.)).child(
                            Select::new(&self.select)
                                .small()
                                .placeholder(t!(cx, "player-move"))
                                .disabled(move_problem.is_some() || self.slots.is_empty())
                                .accessibility_label(t!(cx, "player-move")),
                        ),
                    ),
            )
            .children(note.map(|note| div().text_xs().text_color(muted).child(note)))
            .children(self.status.clone().map(|(text, is_problem)| {
                div()
                    .text_xs()
                    .text_color(if is_problem { cx.theme().danger } else { muted })
                    .child(text)
            }))
    }
}

/// Another save, by its place in the list.
#[derive(Clone)]
struct SlotOption {
    index: SharedString,
    name: SharedString,
}

impl SelectItem for SlotOption {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.index
    }
}
