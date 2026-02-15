use std::borrow::Cow;

use gpui::{AssetSource, SharedString};
use rust_embed::RustEmbed;

pub struct VeilAssetSource;

impl AssetSource for VeilAssetSource {
    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        if path.starts_with("embedded://") {
            EmbeddedAssets.list(path)
        } else {
            DiskAssets.list(path)
        }
    }

    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        if let Some(stripped) = path.strip_prefix("embedded://") {
            EmbeddedAssets.load(stripped)
        } else {
            DiskAssets.load(path)
        }
    }
}

struct DiskAssets;

impl AssetSource for DiskAssets {
    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }

    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(Cow::Owned(bytes))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(RustEmbed)]
#[folder = "../../../assets"]
#[include = "fonts/*"]
#[include = "icons/*"]
#[include = "images/*"]
struct EmbeddedAssets;

impl AssetSource for EmbeddedAssets {
    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let files: Vec<SharedString> = Self::iter()
            .filter(|file| file.starts_with(path))
            .map(|file| file.into())
            .collect();

        Ok(files)
    }

    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        match Self::get(path) {
            Some(file) => Ok(Some(file.data)),
            None => Ok(None),
        }
    }
}
