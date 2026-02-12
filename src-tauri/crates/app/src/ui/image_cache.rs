use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use futures::FutureExt;
use gpui::{
    App, AppContext, Asset, AssetLogger, ElementId, Entity, ImageAssetLoader, ImageCache,
    ImageCacheItem, ImageCacheProvider, ImageSource, RenderImage, Resource, hash,
};
use logging::error;

pub struct AlbumCoverCacheProvider {
    id: ElementId,
    max_items: usize,
}

impl AlbumCoverCacheProvider {
    pub fn new(id: impl Into<ElementId>, max_items: usize) -> Self {
        Self {
            id: id.into(),
            max_items,
        }
    }
}

impl ImageCacheProvider for AlbumCoverCacheProvider {
    fn provide(&mut self, window: &mut gpui::Window, cx: &mut App) -> gpui::AnyImageCache {
        window
            .with_global_id(self.id.clone(), |id, window| {
                window.with_element_state(id, |cache, _| {
                    let cache = cache.unwrap_or_else(|| AlbumCoverCache::new(self.max_items, cx));
                    (cache.clone(), cache)
                })
            })
            .into()
    }
}

pub struct AlbumCoverCache {
    max_items: usize,
    usage_list: VecDeque<u64>,
    cache: HashMap<u64, (ImageCacheItem, Resource)>,
}

impl AlbumCoverCache {
    pub fn new(max_items: usize, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            cx.on_release(|this: &mut Self, cx| {
                for (_idx, (mut item, resource)) in std::mem::take(&mut this.cache) {
                    if let Some(Ok(image)) = item.get() {
                        cx.drop_image(image, None);
                    }
                    ImageSource::Resource(resource).remove_asset(cx);
                }
            })
            .detach();

            AlbumCoverCache {
                max_items,
                usage_list: VecDeque::with_capacity(max_items),
                cache: HashMap::with_capacity(max_items),
            }
        })
    }
}

impl ImageCache for AlbumCoverCache {
    fn load(
        &mut self,
        resource: &Resource,
        window: &mut gpui::Window,
        cx: &mut App,
    ) -> Option<Result<Arc<RenderImage>, gpui::ImageCacheError>> {
        let h = hash(resource);

        if let Some(item) = self.cache.get_mut(&h) {
            let idx = self
                .usage_list
                .iter()
                .position(|x| *x == h)
                .expect("usage_list inconsistent with cache");
            self.usage_list.remove(idx);
            self.usage_list.push_front(h);

            return item.0.get();
        }

        let load_future = AssetLogger::<ImageAssetLoader>::load(resource.clone(), cx);
        let task = cx.background_executor().spawn(load_future).shared();

        if self.usage_list.len() >= self.max_items {
            if let Some(old) = self.usage_list.pop_back() {
                if let Some((mut old_item, old_resource)) = self.cache.remove(&old) {
                    if let Some(Ok(image)) = old_item.get() {
                        cx.drop_image(image, Some(window));
                    }
                    ImageSource::Resource(old_resource).remove_asset(cx);
                }
            }
        }

        self.cache
            .insert(h, (ImageCacheItem::Loading(task.clone()), resource.clone()));
        self.usage_list.push_front(h);

        let current_entity = window.current_view();

        window
            .spawn(cx, async move |cx| {
                if let Err(e) = task.await {
                    error!("Failed to load image: {:?}", e);
                }
                cx.on_next_frame(move |_, cx| {
                    cx.notify(current_entity);
                });
            })
            .detach();

        None
    }
}
