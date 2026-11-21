//! What can go wrong with a save, said so a player knows what happened and what to do.

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sii(#[from] sii::Error),

    #[error("the save has no `{0}` unit")]
    MissingUnit(&'static str),

    #[error("`{unit}` has no `{field}`")]
    MissingField { unit: String, field: &'static str },

    #[error("`{unit}.{field}` is `{value}`, which isn't what was expected")]
    BadValue {
        unit: String,
        field: &'static str,
        value: String,
    },

    #[error("{action} {}: {source}", path.display())]
    Io {
        action: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("the game is running. Close it first: it overwrites saves when it exits")]
    GameRunning,

    #[error("{} isn't a save slot (it has no game.sii)", .0.display())]
    NotASlot(PathBuf),

    #[error("the save didn't read back the way it was written, so nothing was kept: {0}")]
    NotVerified(String),
}

#[cfg(feature = "native")]
pub(crate) fn io(
    action: &'static str,
    path: &std::path::Path,
) -> impl FnOnce(std::io::Error) -> Error {
    let path = path.to_path_buf();
    move |source| Error::Io {
        action,
        path,
        source,
    }
}
