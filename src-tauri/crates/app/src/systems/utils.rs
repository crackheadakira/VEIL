use anyhow::Context;
use rfd::{AsyncFileDialog, FileHandle};

use crate::{TauriState, error::FrontendError};

pub fn open_url(url: &str) -> Result<(), FrontendError> {
    if !url.contains("http") {
        return Err(FrontendError::Standard(
            "URL does not contain HTTP".to_owned(),
        ));
    }

    webbrowser::open(url).context("Failed to open URL in browser")?;

    Ok(())
}

pub async fn open_folder_picker(
    starting_path: Option<&str>,
    picker_title: &str,
) -> Option<FileHandle> {
    let mut dialog = AsyncFileDialog::new();

    if let Some(path) = starting_path {
        dialog = dialog.set_directory(path);
    }

    dialog.set_title(picker_title).pick_folder().await
}

pub fn sanitize_string(string: &str) -> String {
    string
        .trim()
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "")
}

pub async fn get_handle_to_music_folder(
    state: &TauriState<'_>,
) -> Result<Option<FileHandle>, FrontendError> {
    let music_dir = {
        let config_path = logging::lock_or_log(state.config.read(), "Config Read")?;
        config_path.library.music_dir.clone()
    };

    let handle = open_folder_picker(music_dir.as_deref(), "Select your music folder").await;
    Ok(handle)
}

pub fn data_path() -> std::path::PathBuf {
    let home_dir = dirs::data_local_dir().unwrap();
    home_dir.join("com.veil")
}

#[cfg(test)]
mod tests {
    use super::sanitize_string;

    #[test]
    fn sanitize_strings() {
        assert_eq!(sanitize_string("h/e/l/l/o"), "hello");

        assert_eq!(sanitize_string("h//e/\\/l////l// //o"), "hell o");

        assert_eq!(sanitize_string("h:*/e\\/*:ll?\"o"), "hello");
    }
}
