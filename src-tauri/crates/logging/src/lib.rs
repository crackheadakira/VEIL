use anyhow::bail;
use std::fmt::Display;
use tracing_subscriber::EnvFilter;

pub use tracing::{debug, error, info, trace, warn};

pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

pub fn log_if_err<T, E>(result: Result<T, E>, context: &str) -> Option<T>
where
    E: Display,
{
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            error!("{context}: {e}");
            None
        }
    }
}

#[macro_export]
/// Takes a result and passes it through logging before returning result.
macro_rules! try_with_log {
    ($name:expr, $op:expr) => {{
        match $op() {
            Ok(ok) => {
                ::logging::info!(target: module_path!(), "{} succeeded", $name);
                Ok(ok)
            }
            Err(e) => {
                ::logging::error!(target: module_path!(), "{} failed: {e}", $name);
                Err(anyhow::anyhow!("{} failed: {e}", $name))
            }
        }
    }};
}

/// Attempt locking, and if fails log it and bail.
pub fn lock_or_log<T>(
    guard: Result<T, std::sync::PoisonError<T>>,
    lock_name: &str,
) -> Result<T, anyhow::Error> {
    match guard {
        Ok(g) => Ok(g),
        Err(poisoned) => {
            bail!("{lock_name} lock poisoned: {poisoned:?}");
        }
    }
}

pub mod async_impl {
    use anyhow::bail;

    pub async fn try_with_log<T, F>(name: &str, op: impl FnOnce() -> F) -> anyhow::Result<T>
    where
        F: std::future::Future<Output = anyhow::Result<T>>,
    {
        match op().await {
            Ok(ok) => {
                tracing::info!("{name} succeeded");
                Ok(ok)
            }
            Err(e) => {
                tracing::error!("{name} failed: {e}");
                bail!("{name} failed: {e}");
            }
        }
    }
}
