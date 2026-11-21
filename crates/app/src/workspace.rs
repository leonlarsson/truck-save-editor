//! The window's layout: the sidebar on the leading edge, the open page beside it, and the page's
//! header with what's changed and the Save button.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle},
    h_flex,
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem},
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, StatefulInteractiveElement as _, Styled as _, Window, div, px, rems,
};

use crate::APP_NAME;
use crate::pages::{Page, fleet::FleetPage, overview::OverviewPage};
use crate::theme;
use crate::ui::PageHeader;

pub struct Workspace {
    page: Page,
    overview: Entity<OverviewPage>,
    fleet: Entity<FleetPage>,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let overview = cx.new(|cx| OverviewPage::new(window, cx));
        let fleet = cx.new(|cx| FleetPage::new(window, cx));
        // Redraw the header (the change count) whenever a page changes.
        cx.observe(&overview, |_, _, cx| cx.notify()).detach();
        cx.observe(&fleet, |_, _, cx| cx.notify()).detach();
        Self {
            page: dev_start_page().unwrap_or(Page::Overview),
            overview,
            fleet,
        }
    }

    fn changes(&self, cx: &Context<Self>) -> usize {
        self.overview.read(cx).changes() + self.fleet.read(cx).changes(cx)
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_dark = cx.theme().is_dark();
        Sidebar::new("sidebar")
            .w(px(232.))
            .header(
                // Plain content, not `SidebarHeader`: that one is a menu trigger and lights up on hover.
                h_flex()
                    .p_2()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size_8()
                            .flex_shrink_0()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().sidebar_primary)
                            .text_color(cx.theme().sidebar_primary_foreground)
                            .child(Icon::new(IconName::Truck)),
                    )
                    .child(
                        v_flex()
                            .min_w_0()
                            .child(div().text_sm().font_semibold().child(APP_NAME))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("John · Autosave"),
                            ),
                    ),
            )
            .child(SidebarGroup::new("Career").child(self.menu(&Page::CAREER, cx)))
            .child(SidebarGroup::new("Company").child(self.menu(&Page::COMPANY, cx)))
            .child(SidebarGroup::new("Advanced").child(self.menu(&Page::ADVANCED, cx)))
            .footer(
                // Plain content, not `SidebarFooter` (a menu trigger that lights up on hover).
                h_flex().p_2().child(
                    Button::new("toggle-theme")
                        .ghost()
                        .small()
                        .icon(Icon::new(if is_dark {
                            IconName::Sun
                        } else {
                            IconName::Moon
                        }))
                        .label(if is_dark { "Light mode" } else { "Dark mode" })
                        .on_click(|_, window, cx| theme::toggle_mode(window, cx)),
                ),
            )
    }

    fn menu(&self, pages: &[Page], cx: &mut Context<Self>) -> SidebarMenu {
        SidebarMenu::new().children(pages.iter().map(|&page| {
            SidebarMenuItem::new(page.title())
                .icon(Icon::new(page.icon()))
                .active(page == self.page)
                .disable(!page.is_built())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.page = page;
                    cx.notify();
                }))
        }))
    }

    fn render_header(&self, cx: &mut Context<Self>) -> PageHeader {
        let changes = self.changes(cx);
        let summary = match changes {
            0 => "No changes".to_string(),
            1 => "1 change".to_string(),
            n => format!("{n} changes"),
        };

        PageHeader::new(self.page.title())
            .subtitle("Sample data: real saves load in a later step")
            .action(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(summary),
            )
            .action(
                Button::new("discard")
                    .ghost()
                    .icon(Icon::new(IconName::Undo2))
                    .label("Discard")
                    .disabled(changes == 0)
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.overview
                            .update(cx, |page, cx| page.discard_changes(window, cx));
                        this.fleet.update(cx, |page, cx| page.discard_changes(cx));
                    })),
            )
            .action(
                Button::new("save")
                    .primary()
                    .icon(Icon::new(IconName::Save))
                    .label("Save")
                    .disabled(changes == 0)
                    .on_click(|_, window, cx| {
                        window.push_notification(
                            "Saving comes with real saves. This preview runs on sample data.",
                            cx,
                        );
                    }),
            )
    }

    fn render_page(&self) -> impl IntoElement {
        match self.page {
            Page::Overview => div()
                .id("overview-scroll")
                .size_full()
                .overflow_y_scroll()
                .child(self.overview.clone())
                .into_any_element(),
            Page::Fleet => div()
                .size_full()
                .child(self.fleet.clone())
                .into_any_element(),
            other => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Empty::new().header(
                        EmptyHeader::new()
                            .media(EmptyMedia::new().child(Icon::new(other.icon())))
                            .title(
                                EmptyTitle::new()
                                    .child(format!("{} isn't built yet", other.title())),
                            )
                            .description(EmptyDescription::new().child("It's on the way.")),
                    ),
                )
                .into_any_element(),
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_sidebar(cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .min_w(rems(30.))
                    .child(self.render_header(cx))
                    .child(div().flex_1().min_h_0().child(self.render_page())),
            )
    }
}

/// Development builds only: `TSE_PAGE=fleet` opens that page on launch, so a screenshot can show
/// any page without clicking. Release builds always open on Overview.
fn dev_start_page() -> Option<Page> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let wanted = std::env::var("TSE_PAGE").ok()?.to_lowercase();
    Page::ALL
        .into_iter()
        .find(|page| page.title().to_lowercase().replace(' ', "-") == wanted)
}
