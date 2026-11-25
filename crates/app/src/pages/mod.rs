//! The app's pages, one per sidebar entry.

pub mod all_data;
pub mod buy;
pub mod cities;
pub mod coach_career;
pub mod details;
pub mod drivers;
pub mod fleet;
pub mod gallery;
pub mod game_settings;
pub mod garages;
pub mod jobs;
pub mod licences;
pub mod map_view;
pub mod new_job;
pub mod open_save;
pub mod overview;
pub mod position;
pub mod profile_tools;
pub mod saved_colours;
pub mod settings;
pub mod statistics;
pub mod world_map;

use gpui_kit::{App, SharedString};

use crate::i18n::t;

use gpui_kit::assets::IconName;

/// Every page, in the sidebar's order: the save's pages in its groups, then Settings (the
/// editor's own), at its foot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Overview,
    Statistics,
    CoachCareer,
    Fleet,
    Garages,
    Drivers,
    Jobs,
    Cities,
    WorldMap,
    Gallery,
    AllData,
    GameSettings,
    Settings,
}

impl Page {
    pub const ALL: [Page; 13] = [
        Page::Overview,
        Page::Statistics,
        Page::CoachCareer,
        Page::Fleet,
        Page::Garages,
        Page::Drivers,
        Page::Jobs,
        Page::Cities,
        Page::WorldMap,
        Page::Gallery,
        Page::AllData,
        Page::GameSettings,
        Page::Settings,
    ];
    /// Coach career shows only for a save with a coach career (see `is_shown`).
    pub const CAREER: [Page; 3] = [Page::Overview, Page::Statistics, Page::CoachCareer];
    pub const COMPANY: [Page; 4] = [Page::Fleet, Page::Garages, Page::Drivers, Page::Jobs];
    /// The map's pages: the cities, the map itself (the game's World map), the Gallery.
    pub const MAP: [Page; 3] = [Page::Cities, Page::WorldMap, Page::Gallery];
    pub const ADVANCED: [Page; 2] = [Page::AllData, Page::GameSettings];

    /// Whether the sidebar and the palette offer the page: Coach career only with a coach career.
    pub fn is_shown(self, has_coach_career: bool) -> bool {
        self != Page::CoachCareer || has_coach_career
    }

    pub fn title(self, cx: &App) -> SharedString {
        match self {
            Page::Overview => t!(cx, "page-overview"),
            Page::Statistics => t!(cx, "page-statistics"),
            Page::CoachCareer => t!(cx, "page-coach-career"),
            Page::Fleet => t!(cx, "page-fleet"),
            Page::Garages => t!(cx, "page-garages"),
            Page::Drivers => t!(cx, "page-drivers"),
            Page::Cities => t!(cx, "page-cities"),
            Page::WorldMap => t!(cx, "page-world-map"),
            Page::Gallery => t!(cx, "page-gallery"),
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
            Page::CoachCareer => "coach-career",
            Page::Fleet => "fleet",
            Page::Garages => "garages",
            Page::Drivers => "drivers",
            Page::Cities => "cities",
            Page::WorldMap => "world-map",
            Page::Gallery => "gallery",
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
            Page::CoachCareer => IconName::Bus,
            Page::Fleet => IconName::Truck,
            Page::Garages => IconName::Warehouse,
            Page::Drivers => IconName::Users,
            Page::Cities => IconName::Building2,
            Page::WorldMap => IconName::Map,
            Page::Gallery => IconName::Camera,
            Page::Jobs => IconName::Container,
            Page::AllData => IconName::Database,
            Page::GameSettings => IconName::Gamepad2,
            Page::Settings => IconName::Settings,
        }
    }
}
