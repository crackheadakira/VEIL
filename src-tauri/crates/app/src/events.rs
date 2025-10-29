use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_specta::{Event, TypedEvent};

use crate::error::FrontendError;

pub trait EventSystemHandler: Event + Sized + Send + Sync + 'static + Serialize {
    /// Take in the corresponding events and pass them to the appropriate
    /// private handler.
    ///
    /// Will also emit the required updates to the frontend.
    fn handle(
        event: TypedEvent<Self>,
        handle: &AppHandle,
    ) -> impl Future<Output = Result<(), FrontendError>> + Send;

    /// Attaches a listener to the given event on it's own async task.
    fn attach_listener(handle: &AppHandle)
    where
        for<'de> Self: Deserialize<'de>,
    {
        let handle_clone = handle.clone();
        Self::listen(&handle_clone.clone(), move |event| {
            let handle = handle_clone.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = Self::handle(event, &handle).await {
                    if let Err(e) = e.emit(&handle) {
                        logging::error!("Failed to emit error to Frontend: {:?}", e);
                    }
                }
            });
        });
    }
}
