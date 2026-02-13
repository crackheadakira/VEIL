// Based off https://github.com/zed-industries/zed/blob/44015e0379b6c868258318d9c159134adbf801d9/crates/gpui/src/elements/uniform_list.rs

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ContentMask, Div, Element, ElementId, Entity,
    GlobalElementId, Hitbox, InspectorElementId, InteractiveElement, Interactivity, IntoElement,
    IsZero, LayoutId, Overflow, Pixels, Point, Rems, ScrollHandle, Size, Stateful,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, div, point, rems, size,
};
use smallvec::SmallVec;
use std::{cell::RefCell, cmp, ops::Range, rc::Rc};

/// uniform_grid provides lazy rendering for a set of items with uniform width and height.
/// Items are arranged in a grid that wraps based on available width.
/// When rendered into a container with overflow-y: hidden and a fixed (or max) height,
/// uniform_grid will only render the visible subset of items.
#[track_caller]
pub fn uniform_grid<R>(
    id: impl Into<ElementId>,
    item_count: usize,
    f: impl 'static + Fn(Range<usize>, &mut Window, &mut App) -> Vec<R>,
) -> UniformGrid
where
    R: IntoElement,
{
    let id = id.into();
    let mut base_style = StyleRefinement::default();
    let scroll_handle = UniformGridScrollHandle::new();
    base_style.overflow.y = Some(Overflow::Scroll);

    let render_range = move |range: Range<usize>, window: &mut Window, cx: &mut App| {
        f(range, window, cx)
            .into_iter()
            .map(|component| component.into_any_element())
            .collect()
    };

    UniformGrid {
        item_count,
        item_to_measure_index: 0,
        render_items: Box::new(render_range),
        base: div()
            .id(id.clone())
            .size_full()
            .overflow_scroll()
            .track_scroll(&scroll_handle),
        id: id.into(),
        scroll_handle,
        gap: rems(0.0),
        preload_rows: 0,
    }
}

/// A grid element for efficiently laying out and displaying items in a wrapping grid.
pub struct UniformGrid {
    item_count: usize,
    item_to_measure_index: usize,
    render_items: Box<
        dyn for<'a> Fn(Range<usize>, &'a mut Window, &'a mut App) -> SmallVec<[AnyElement; 64]>,
    >,
    base: Stateful<Div>,
    id: ElementId,
    scroll_handle: UniformGridScrollHandle,
    gap: Rems,
    preload_rows: usize,
}

/// Frame state used by the UniformGrid.
pub struct UniformGridFrameState {
    items: SmallVec<[AnyElement; 32]>,
}

/// A handle for controlling the scroll position of a uniform grid.
#[derive(Clone, Debug, Default)]
pub struct UniformGridScrollHandle {
    pub state: Rc<RefCell<UniformGridScrollState>>,
    pub base_handle: ScrollHandle,
}

/// Where to place the element scrolled to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollStrategy {
    /// Place the element at the top of the grid's viewport.
    Top,
    /// Attempt to place the element in the middle of the grid's viewport.
    Center,
    /// Attempt to place the element at the bottom of the grid's viewport.
    Bottom,
    /// If the element is not visible, scroll minimally to make it visible.
    Nearest,
}

#[derive(Clone, Copy, Debug)]
pub struct DeferredScrollToItem {
    pub item_index: usize,
    pub strategy: ScrollStrategy,
    pub scroll_strict: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UniformGridScrollState {
    pub deferred_scroll_to_item: Option<DeferredScrollToItem>,
    pub last_item_size: Option<Size<Pixels>>,
    pub last_content_size: Option<Size<Pixels>>,
}

impl UniformGridScrollHandle {
    /// Create a new scroll handle to bind to a uniform grid.
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(UniformGridScrollState {
                deferred_scroll_to_item: None,
                last_item_size: None,
                last_content_size: None,
            })),
            base_handle: ScrollHandle::new(),
        }
    }

    /// Scroll the grid so that the given item index is visible.
    pub fn scroll_to_item(&self, ix: usize, strategy: ScrollStrategy) {
        self.state.borrow_mut().deferred_scroll_to_item = Some(DeferredScrollToItem {
            item_index: ix,
            strategy,
            scroll_strict: false,
        });
    }

    /// Scroll the grid so that the given item index is at scroll strategy position.
    pub fn scroll_to_item_strict(&self, ix: usize, strategy: ScrollStrategy) {
        self.state.borrow_mut().deferred_scroll_to_item = Some(DeferredScrollToItem {
            item_index: ix,
            strategy,
            scroll_strict: true,
        });
    }

    /// Checks if the grid can be scrolled vertically.
    pub fn is_scrollable(&self) -> bool {
        if let (Some(item_size), Some(content_size)) = (
            self.state.borrow().last_item_size,
            self.state.borrow().last_content_size,
        ) {
            content_size.height > item_size.height
        } else {
            false
        }
    }

    /// Scroll to the bottom of the grid.
    pub fn scroll_to_bottom(&self) {
        self.scroll_to_item(usize::MAX, ScrollStrategy::Bottom);
    }
}

impl From<ScrollHandle> for UniformGridScrollHandle {
    fn from(handle: ScrollHandle) -> Self {
        let mut this = UniformGridScrollHandle::new();
        this.base_handle = handle;
        this
    }
}

impl AsRef<ScrollHandle> for UniformGridScrollHandle {
    fn as_ref(&self) -> &ScrollHandle {
        &self.base_handle
    }
}

impl std::ops::Deref for UniformGridScrollHandle {
    type Target = ScrollHandle;

    fn deref(&self) -> &Self::Target {
        &self.base_handle
    }
}

impl UniformGridScrollHandle {
    fn offset(&self) -> Point<Pixels> {
        self.base_handle.offset()
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        self.base_handle.set_offset(offset);
    }
}

impl Styled for UniformGrid {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl Element for UniformGrid {
    type RequestLayoutState = UniformGridFrameState;
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let layout_id = self.base.interactivity().request_layout(
            global_id,
            inspector_id,
            window,
            cx,
            |style, window, cx| {
                window.with_text_style(style.text_style().cloned(), |window| {
                    window.request_layout(style, None, cx)
                })
            },
        );

        (
            layout_id,
            UniformGridFrameState {
                items: SmallVec::new(),
            },
        )
    }

    fn prepaint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        frame_state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Hitbox> {
        let style = self
            .base
            .interactivity()
            .compute_style(global_id, None, window, cx);
        let border = style.border_widths.to_pixels(window.rem_size());
        let padding = style
            .padding
            .to_pixels(bounds.size.into(), window.rem_size());

        let padded_bounds = Bounds::from_corners(
            bounds.origin + point(border.left + padding.left, border.top + padding.top),
            bounds.bottom_right()
                - point(border.right + padding.right, border.bottom + padding.bottom),
        );

        let item_size = self.measure_item(None, window, cx);

        let gap_px = self.gap.to_pixels(window.rem_size());

        let available_width = padded_bounds.size.width;
        let items_per_row = if item_size.width.is_zero() {
            1
        } else {
            let total_gap_width = if self.gap.is_zero() {
                Pixels::ZERO
            } else {
                let max_items = (available_width / item_size.width).floor() as usize;
                if max_items == 0 {
                    return self.base.interactivity().prepaint(
                        global_id,
                        inspector_id,
                        bounds,
                        Size::default(),
                        window,
                        cx,
                        |_style, _scroll_offset, hitbox, _window, _cx| hitbox,
                    );
                }
                gap_px * (max_items - 1)
            };

            let width_for_items = available_width - total_gap_width;
            cmp::max(1, (width_for_items / item_size.width).floor() as usize)
        };

        let total_rows = if self.item_count == 0 {
            0
        } else {
            (self.item_count + items_per_row - 1) / items_per_row
        };

        let row_height = item_size.height;
        let total_gap_height = if total_rows > 1 {
            gap_px * (total_rows - 1)
        } else {
            Pixels::ZERO
        };

        let content_size = Size {
            width: padded_bounds.size.width,
            height: row_height * total_rows + total_gap_height,
        };

        let shared_scroll_to_item = {
            let mut handle = self.scroll_handle.state.borrow_mut();
            handle.last_item_size = Some(padded_bounds.size);
            handle.last_content_size = Some(content_size);
            handle.deferred_scroll_to_item.take()
        };

        let mut updated_scroll_offset = self.scroll_handle.base_handle.offset();

        self.base.interactivity().prepaint(
            global_id,
            inspector_id,
            bounds,
            content_size,
            window,
            cx,
            |_style, mut scroll_offset, hitbox, window, cx| {
                if self.item_count == 0 {
                    return hitbox;
                }

                let content_height = content_size.height;
                let is_scrolled_vertically = !scroll_offset.y.is_zero();
                let max_scroll_offset = padded_bounds.size.height - content_height;

                if is_scrolled_vertically && scroll_offset.y < max_scroll_offset {
                    updated_scroll_offset.y = max_scroll_offset;
                    scroll_offset.y = max_scroll_offset;
                }

                if let Some(DeferredScrollToItem {
                    item_index,
                    mut strategy,
                    scroll_strict,
                }) = shared_scroll_to_item
                {
                    let list_height = padded_bounds.size.height;

                    let item_row = item_index / items_per_row;
                    let row_top = row_height * item_row + gap_px * item_row;
                    let row_bottom = row_top + row_height;
                    let scroll_top = -updated_scroll_offset.y;

                    let is_above = row_top < scroll_top;
                    let is_below = row_bottom > scroll_top + list_height;

                    if scroll_strict || is_above || is_below {
                        if strategy == ScrollStrategy::Nearest {
                            if is_above {
                                strategy = ScrollStrategy::Top;
                            } else if is_below {
                                strategy = ScrollStrategy::Bottom;
                            }
                        }

                        let max_scroll_offset = (content_height - list_height).max(Pixels::ZERO);
                        match strategy {
                            ScrollStrategy::Top => {
                                updated_scroll_offset.y =
                                    -row_top.clamp(Pixels::ZERO, max_scroll_offset);
                            }
                            ScrollStrategy::Center => {
                                let row_center = row_top + row_height / 2.0;
                                let viewport_center = list_height / 2.0;
                                let target_scroll_top = row_center - viewport_center;
                                updated_scroll_offset.y =
                                    -target_scroll_top.clamp(Pixels::ZERO, max_scroll_offset);
                            }
                            ScrollStrategy::Bottom => {
                                updated_scroll_offset.y = -(row_bottom - list_height)
                                    .clamp(Pixels::ZERO, max_scroll_offset);
                            }
                            ScrollStrategy::Nearest => {}
                        }
                    }

                    self.scroll_handle
                        .base_handle
                        .set_offset(updated_scroll_offset);
                    scroll_offset = updated_scroll_offset;
                }

                let first_visible_row = if row_height.is_zero() || (row_height + gap_px).is_zero() {
                    0
                } else {
                    let row_plus_gap = row_height + gap_px;
                    ((-scroll_offset.y) / row_plus_gap).floor() as usize
                };

                let last_visible_row = if row_height.is_zero() {
                    total_rows
                } else {
                    let row_plus_gap = row_height + gap_px;
                    (((-scroll_offset.y + padded_bounds.size.height) / row_plus_gap).ceil()
                        as usize)
                        .min(total_rows)
                };

                let first_rendered_row = first_visible_row.saturating_sub(self.preload_rows);
                let last_rendered_row = (last_visible_row + self.preload_rows).min(total_rows);

                let first_visible_item = first_rendered_row * items_per_row;
                let last_visible_item =
                    cmp::min(last_rendered_row * items_per_row, self.item_count);

                let visible_range = first_visible_item..last_visible_item;

                let items = (self.render_items)(visible_range.clone(), window, cx);

                let content_mask = ContentMask { bounds };
                window.with_content_mask(Some(content_mask), |window| {
                    for (mut item, ix) in items.into_iter().zip(visible_range.clone()) {
                        let row = ix / items_per_row;
                        let col = ix % items_per_row;

                        let item_origin = padded_bounds.origin
                            + scroll_offset
                            + point(
                                (item_size.width + gap_px) * col,
                                (row_height + gap_px) * row,
                            );

                        let available_space = size(
                            AvailableSpace::Definite(item_size.width),
                            AvailableSpace::Definite(item_size.height),
                        );
                        item.layout_as_root(available_space, window, cx);
                        item.prepaint_at(item_origin, window, cx);
                        frame_state.items.push(item);
                    }
                });

                hitbox
            },
        )
    }

    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        hitbox: &mut Option<Hitbox>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.base.interactivity().paint(
            global_id,
            inspector_id,
            bounds,
            hitbox.as_ref(),
            window,
            cx,
            |_, window, cx| {
                for item in &mut request_layout.items {
                    item.paint(window, cx);
                }
            },
        )
    }
}

impl IntoElement for UniformGrid {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl UniformGrid {
    /// Selects a specific item for measurement.
    pub fn with_width_from_item(mut self, item_index: Option<usize>) -> Self {
        self.item_to_measure_index = item_index.unwrap_or(0);
        self
    }

    /// Sets the gap between items in the grid.
    pub fn gap(mut self, gap: impl Into<Rems>) -> Self {
        self.gap = gap.into();
        self
    }

    // Sets the number of extra rows to preload above and below the visible area.
    /// This can reduce visual popping during fast scrolling at the cost of rendering more items.
    /// For example, `preload_rows(2)` will render 2 extra rows above and 2 below what's visible.
    pub fn preload_rows(mut self, amount: usize) -> Self {
        self.preload_rows = amount;
        self
    }

    fn measure_item(
        &self,
        _list_width: Option<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> Size<Pixels> {
        if self.item_count == 0 {
            return Size::default();
        }

        let item_ix = cmp::min(self.item_to_measure_index, self.item_count - 1);
        let mut items = (self.render_items)(item_ix..item_ix + 1, window, cx);
        let Some(mut item_to_measure) = items.pop() else {
            return Size::default();
        };
        let available_space = size(AvailableSpace::MinContent, AvailableSpace::MinContent);
        item_to_measure.layout_as_root(available_space, window, cx)
    }

    /// Track and render scroll state of this grid with reference to the given scroll handle.
    pub fn track_scroll(mut self, handle: &UniformGridScrollHandle) -> Self {
        self.base = self.base.track_scroll(handle);
        self.scroll_handle = handle.clone();
        self
    }
}

impl InteractiveElement for UniformGrid {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}
