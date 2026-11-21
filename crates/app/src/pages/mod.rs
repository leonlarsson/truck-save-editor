//! The app's pages, one per sidebar entry.

pub mod all_data;
pub mod details;
pub mod drivers;
pub mod fleet;
pub mod garages;
pub mod jobs;
pub mod map;
pub mod open_save;
pub mod overview;
pub mod settings;

use gpui_kit::{App, SharedString};

use crate::i18n::t;

use gpui_kit::assets::IconName;

/// Every page the sidebar lists, in order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Overview,
    Fleet,
    Garages,
    Drivers,
    Map,
    Jobs,
    AllData,
    Settings,
}

impl Page {
    pub const ALL: [Page; 8] = [
        Page::Overview,
        Page::Fleet,
        Page::Garages,
        Page::Drivers,
        Page::Map,
        Page::Jobs,
        Page::AllData,
        Page::Settings,
    ];
    pub const CAREER: [Page; 1] = [Page::Overview];
    pub const COMPANY: [Page; 5] = [
        Page::Fleet,
        Page::Garages,
        Page::Drivers,
        Page::Map,
        Page::Jobs,
    ];
    pub const ADVANCED: [Page; 2] = [Page::AllData, Page::Settings];

    pub fn title(self, cx: &App) -> SharedString {
        match self {
            Page::Overview => t!(cx, "page-overview"),
            Page::Fleet => t!(cx, "page-fleet"),
            Page::Garages => t!(cx, "page-garages"),
            Page::Drivers => t!(cx, "page-drivers"),
            Page::Map => t!(cx, "page-map"),
            Page::Jobs => t!(cx, "page-jobs"),
            Page::AllData => t!(cx, "page-all-data"),
            Page::Settings => t!(cx, "page-settings"),
        }
    }

    /// A stable name for the page, for `TSE_PAGE` (not shown, not translated).
    pub fn slug(self) -> &'static str {
        match self {
            Page::Overview => "overview",
            Page::Fleet => "fleet",
            Page::Garages => "garages",
            Page::Drivers => "drivers",
            Page::Map => "map",
            Page::Jobs => "jobs",
            Page::AllData => "all-data",
            Page::Settings => "settings",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            Page::Overview => IconName::Gauge,
            Page::Fleet => IconName::Truck,
            Page::Garages => IconName::Warehouse,
            Page::Drivers => IconName::Users,
            Page::Map => IconName::Map,
            Page::Jobs => IconName::Container,
            Page::AllData => IconName::Database,
            Page::Settings => IconName::Settings,
        }
    }
}
