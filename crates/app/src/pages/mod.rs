//! The app's pages, one per sidebar entry.

pub mod all_data;
pub mod details;
pub mod drivers;
pub mod fleet;
pub mod game_settings;
pub mod garages;
pub mod jobs;
pub mod licences;
pub mod map;
pub mod map_view;
pub mod open_save;
pub mod overview;
pub mod position;
pub mod profile_tools;
pub mod saved_colours;
pub mod settings;
pub mod statistics;

use gpui_kit::{App, SharedString};

use crate::i18n::t;

use gpui_kit::assets::IconName;

/// Every page, in the sidebar's order: the save's pages in its groups, then Settings (the
/// editor's own), at its foot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Overview,
    Statistics,
    Fleet,
    Garages,
    Drivers,
    Map,
    Jobs,
    AllData,
    GameSettings,
    Settings,
}

impl Page {
    pub const ALL: [Page; 10] = [
        Page::Overview,
        Page::Statistics,
        Page::Fleet,
        Page::Garages,
        Page::Drivers,
        Page::Map,
        Page::Jobs,
        Page::AllData,
        Page::GameSettings,
        Page::Settings,
    ];
    pub const CAREER: [Page; 2] = [Page::Overview, Page::Statistics];
    pub const COMPANY: [Page; 5] = [
        Page::Fleet,
        Page::Garages,
        Page::Drivers,
        Page::Map,
        Page::Jobs,
    ];
    pub const ADVANCED: [Page; 2] = [Page::AllData, Page::GameSettings];

    pub fn title(self, cx: &App) -> SharedString {
        match self {
            Page::Overview => t!(cx, "page-overview"),
            Page::Statistics => t!(cx, "page-statistics"),
            Page::Fleet => t!(cx, "page-fleet"),
            Page::Garages => t!(cx, "page-garages"),
            Page::Drivers => t!(cx, "page-drivers"),
            Page::Map => t!(cx, "page-map"),
            Page::Jobs => t!(cx, "page-jobs"),
            Page::AllData => t!(cx, "page-all-data"),
            Page::GameSettings => t!(cx, "page-game-settings"),
            Page::Settings => t!(cx, "page-settings"),
        }
    }

    /// A stable name for the page, for `TSE_PAGE` (not shown, not translated).
    pub fn slug(self) -> &'static str {
        match self {
            Page::Overview => "overview",
            Page::Statistics => "statistics",
            Page::Fleet => "fleet",
            Page::Garages => "garages",
            Page::Drivers => "drivers",
            Page::Map => "map",
            Page::Jobs => "jobs",
            Page::AllData => "all-data",
            Page::GameSettings => "game-settings",
            Page::Settings => "settings",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            Page::Overview => IconName::Gauge,
            Page::Statistics => IconName::ChartNoAxesColumn,
            Page::Fleet => IconName::Truck,
            Page::Garages => IconName::Warehouse,
            Page::Drivers => IconName::Users,
            Page::Map => IconName::Map,
            Page::Jobs => IconName::Container,
            Page::AllData => IconName::Database,
            Page::GameSettings => IconName::Gamepad2,
            Page::Settings => IconName::Settings,
        }
    }
}
