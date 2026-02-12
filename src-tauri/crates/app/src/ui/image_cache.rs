use gpui::{Global, Image};
use logging::debug;
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::Arc,
};

pub struct AlbumCoverCache {
    max_items: usize,
    map: HashMap<PathBuf, Arc<Image>>,
    usage_queue: VecDeque<PathBuf>,
}

impl Global for AlbumCoverCache {}

impl AlbumCoverCache {
    pub fn new(max_items: usize) -> Self {
        Self {
            max_items,
            map: HashMap::new(),
            usage_queue: VecDeque::new(),
        }
    }

    pub fn get_or_load(&mut self, path: impl Into<PathBuf>) -> Arc<Image> {
        let path = path.into();

        if let Some(image) = self.map.get(&path) {
            self.usage_queue.retain(|p| p != &path);
            self.usage_queue.push_front(path.clone());

            return image.clone();
        }

        debug!("Reading {path:?} from disk");

        let bytes = std::fs::read(&path)
            .unwrap_or_else(|_| panic!("Failed to read album cover: {:?}", path));
        let image = Arc::new(Image::from_bytes(gpui::ImageFormat::Jpeg, bytes));

        self.map.insert(path.clone(), image.clone());
        self.usage_queue.push_front(path.clone());

        while self.map.len() > self.max_items {
            if let Some(oldest) = self.usage_queue.pop_back() {
                debug!("Removing {oldest:?} from image cache");
                self.map.remove(&oldest);
            }
        }

        image
    }
}
