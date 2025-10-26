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
