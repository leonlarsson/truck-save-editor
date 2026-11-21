//! The app's look: GPUI Kit's default theme with our amber accent (`themes/truck.json`), in light
//! and dark. Every colour in the UI comes from these tokens, through `cx.theme()`.

use gpui_kit::{
    App, Window,
    component::{Theme, ThemeMode, ThemeRegistry},
};

use crate::settings::Appearance;

const THEMES: &str = include_str!("../themes/truck.json");
const LIGHT: &str = "Truck Light";
const DARK: &str = "Truck Dark";

/// Registers our themes and follows the system's light or dark setting.
pub fn init(cx: &mut App) {
    let registry = ThemeRegistry::global_mut(cx);
    if let Err(error) = registry.load_themes_from_str(THEMES) {
        // The JSON is compiled in, so this only fails if it was edited badly; the defaults still work.
        eprintln!("Couldn't load the app theme, using the default one: {error}");
        return;
    }
    let light = registry.themes().get(LIGHT).cloned();
    let dark = registry.themes().get(DARK).cloned();

    Theme::update(cx, |theme| {
        if let Some(light) = light {
            theme.light_theme = light;
        }
        if let Some(dark) = dark {
            theme.dark_theme = dark;
        }
    });
    apply(crate::settings::get(cx).appearance, None, cx);

    // Development builds only: `TSE_THEME=dark` or `light` overrides the system setting, so a
    // screenshot can show either theme.
    if cfg!(debug_assertions) {
        match std::env::var("TSE_THEME").as_deref() {
            Ok("dark") => Theme::change(ThemeMode::Dark, None, cx),
            Ok("light") => Theme::change(ThemeMode::Light, None, cx),
            _ => {}
        }
    }
}

/// Shows the app light, dark, or as the system is.
pub fn apply(appearance: Appearance, window: Option<&mut Window>, cx: &mut App) {
    match appearance {
        Appearance::System => Theme::sync_system_appearance(window, cx),
        Appearance::Light => Theme::change(ThemeMode::Light, window, cx),
        Appearance::Dark => Theme::change(ThemeMode::Dark, window, cx),
    }
}

/// Chooses an appearance and keeps it for next time.
pub fn set_appearance(appearance: Appearance, window: &mut Window, cx: &mut App) {
    crate::settings::update(cx, |settings| settings.appearance = appearance);
    apply(appearance, Some(window), cx);
}

/// Switches between light and dark (and keeps the choice).
pub fn toggle_mode(window: &mut Window, cx: &mut App) {
    let next = if cx.theme_is_dark() {
        Appearance::Light
    } else {
        Appearance::Dark
    };
    set_appearance(next, window, cx);
}

/// Small helper so callers don't need to import `ActiveTheme` just to ask this.
trait ThemeIsDark {
    fn theme_is_dark(&self) -> bool;
}

impl ThemeIsDark for App {
    fn theme_is_dark(&self) -> bool {
        Theme::global(self).is_dark()
    }
}
