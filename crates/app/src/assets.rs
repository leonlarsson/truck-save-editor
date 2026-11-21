//! Where the app's icons come from.
//!
//! GPUI Kit's `Assets` only embeds the ~100 icons its own components use. The Lucide icons the app
//! uses on top of those are listed in `AppIcons`, so only they end up in the binary.

use std::borrow::Cow;

use gpui_kit::{AssetSource, Result, SharedString};

gpui_kit::assets::icon_assets!(
    AppIcons,
    [
        Container,
        Database,
        FolderOpen,
        FlaskConical,
        Gauge,
        ArchiveRestore,
        Map,
        Moon,
        Save,
        Sun,
        Truck,
        Undo2,
        Users,
        Wallet,
        Warehouse,
        Wrench,
        Award
    ]
);

/// GPUI Kit's own icons, plus `AppIcons`.
pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = AppIcons.load(path)? {
            return Ok(Some(bytes));
        }
        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = gpui_kit::assets::Assets.list(path)?;
        paths.extend(AppIcons.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
