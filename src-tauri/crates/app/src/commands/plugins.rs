use crate::{error::FrontendError, systems::utils};

#[tauri::command]
#[specta::specta]
pub fn open_url(url: &str) -> Result<(), FrontendError> {
    utils::open_url(url)
}
