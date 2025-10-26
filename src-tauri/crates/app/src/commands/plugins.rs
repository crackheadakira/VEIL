use rfd::{AsyncFileDialog, FileHandle};

use crate::error::FrontendError;

#[tauri::command]
#[specta::specta]
pub async fn open_url(url: &str) -> Result<(), FrontendError> {
    if !url.contains("http") {
        return Err(FrontendError::Standard(
            "URL does not contain HTTP".to_string(),
        ));
    }

    webbrowser::open(url)?;

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
