use crate::{VeilState, error::VeilError};
use std::{future::Future, sync::Arc};
use tokio::sync::broadcast;

pub trait EventSystemHandler: Sized + Send + Sync + 'static {
    /// Take in the corresponding events and pass them to the appropriate
    /// private handler.
    ///
    /// Will also emit the required updates to the frontend.
    fn handle(
        event: Self,
        state: &VeilState,
    ) -> impl Future<Output = Result<(), VeilError>> + Send;

    /// Attaches a listener to the given event on it's own async task.
    fn attach_listener(event_bus: EventBus<Self>, state: Arc<VeilState>)
    where
        Self: Clone,
    {
        tokio::spawn(async move {
            let mut rx = event_bus.subscribe();

            while let Ok(event) = rx.recv().await {
                if let Err(e) = Self::handle(event, &state).await {
                    logging::error!("Error handling event: {:?}", e);
                }
            }
        });
    }
}

#[derive(Clone)]
pub struct EventBus<T: Clone> {
    sender: broadcast::Sender<T>,
}

impl<T: Clone> EventBus<T> {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }

    pub fn emit(&self, event: T) {
        let _ = self.sender.send(event);
    }
}
