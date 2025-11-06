use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;

use crate::render::PackedBundle;
use crate::tile_schema::{TileIndex, WrappingTileIndex};
use crate::TileSchema;

const DEFAULT_FADE_IN_DURATION: Duration = Duration::from_millis(300);

#[derive(Clone)]
pub(crate) struct DisplayedTile<StyleId: Copy> {
    pub(crate) index: WrappingTileIndex,
    pub(crate) bundle: Arc<dyn PackedBundle>,
    style_id: StyleId,
    pub(crate) opacity: f32,
    displayed_at: web_time::Instant,
}

impl<StyleId: Copy> DisplayedTile<StyleId> {
    pub(crate) fn is_opaque(&self) -> bool {
        self.opacity >= 0.999
    }
}

pub(crate) trait TileProvider<StyleId> {
    fn get_tile(&self, index: TileIndex, style_id: StyleId) -> Option<Arc<dyn PackedBundle>>;
}

pub(crate) struct TilesContainer<StyleId, Provider>
where
    StyleId: Copy,
    Provider: TileProvider<StyleId>,
{
    pub(crate) tiles: Mutex<Vec<DisplayedTile<StyleId>>>,
    tile_schema: TileSchema,
    pub(crate) tile_provider: Provider,
    pub fade_in_duration: AtomicU64,
}

impl<StyleId, Provider> TilesContainer<StyleId, Provider>
where
    StyleId: Copy + PartialEq,
    Provider: TileProvider<StyleId>,
{
    pub(crate) fn new(tile_schema: TileSchema, tile_provider: Provider) -> Self {
        Self {
            tiles: Default::default(),
            tile_schema,
            tile_provider,
            fade_in_duration: AtomicU64::new(DEFAULT_FADE_IN_DURATION.as_millis() as u64),
        }
    }

    pub(crate) fn update_displayed_tiles(
        &self,
        needed_indices: impl IntoIterator<Item = WrappingTileIndex>,
        style_id: StyleId,
    ) -> bool {
        let mut displayed_tiles = self.tiles.lock();

        let mut needed_tiles = vec![];
        let mut to_substitute = vec![];

        let now = web_time::Instant::now();
        let fade_in_time = self.fade_in_duration();
        let mut requires_redraw = false;

        for index in needed_indices {
            if let Some(displayed) = displayed_tiles
                .iter_mut()
                .find(|displayed| displayed.index == index && displayed.style_id == style_id)
            {
                if !displayed.is_opaque() {
                    to_substitute.push(index);
                    let fade_in_secs = fade_in_time.as_secs_f64();
                    displayed.opacity = if fade_in_secs > 0.001 {
                        ((now.duration_since(displayed.displayed_at)).as_secs_f64() / fade_in_secs)
                            .min(1.0) as f32
                    } else {
                        1.0
                    };
                    requires_redraw = true;
                }

                needed_tiles.push(displayed.clone());
            } else {
                match self.tile_provider.get_tile(index.into(), style_id) {
                    None => to_substitute.push(index),
                    Some(bundle) => {
                        let opacity = if self.requires_animation() { 0.0 } else { 1.0 };
                        needed_tiles.push(DisplayedTile {
                            index,
                            bundle,
                            style_id,
                            opacity,
                            displayed_at: now,
                        });
                        to_substitute.push(index);
                        requires_redraw = true;
                    }
                }
            }
        }

        let mut new_displayed = vec![];
        for displayed in displayed_tiles.iter() {
            if needed_tiles
                .iter()
                .any(|new| new.index == displayed.index && new.style_id == displayed.style_id)
            {
                continue;
            }

            let Some(displayed_bbox) = self.tile_schema.tile_bbox(displayed.index) else {
                continue;
            };

            for subst in &to_substitute {
                let Some(subst_bbox) = self.tile_schema.tile_bbox(*subst) else {
                    continue;
                };

                if displayed_bbox.intersects(subst_bbox) {
                    new_displayed.push(displayed.clone());
                    break;
                }
            }
        }

        new_displayed.append(&mut needed_tiles);
        *displayed_tiles = new_displayed;

        requires_redraw
    }

    pub fn fade_in_duration(&self) -> Duration {
        Duration::from_millis(self.fade_in_duration.load(Ordering::Relaxed))
    }

    pub fn set_fade_in_duration(&self, duration: Duration) {
        self.fade_in_duration
            .store(duration.as_millis() as u64, Ordering::Relaxed);
    }

    fn requires_animation(&self) -> bool {
        self.fade_in_duration.load(Ordering::Relaxed) > 1
    }
}
