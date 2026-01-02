//! Tile schema definition.

use std::sync::Arc;

use galileo_types::cartesian::{CartesianPoint2d, Point2, Rect};
use serde::{Deserialize, Serialize};

use super::tile_index::WrappingTileIndex;
use crate::view::MapView;

const RESOLUTION_TOLERANCE: f64 = 0.01;

/// Direction of the Y index of tiles.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum VerticalDirection {
    /// Tiles with `Y == 0` are at the top of the map.
    TopToBottom,
    /// Tiles with `Y == 0` are at the bottom of the map.
    BottomToTop,
}

/// Tile schema specifies how tile indices are calculated based on the map position and resolution.
///
/// # Additional notes
///
/// ## Equality
///
/// This type implements `PartialEq`, but internals rely heavily on floating numbers so all the
/// limitation of floating numbers comparison apply to `TileSchema` also. It is usually fine to
/// compare two instances created by the same code, but be aware of comparing schemas after
/// seraializing/deserializing.
///
/// ## Deserialization
///
/// `TileSchema` supports serialization and deserialization as we need to be able to transfer
/// it between workers/processes. But be aware that the type internal logic relies on the all
/// parameters to be correctly difined. If constructed incorrectly, it may return nonsensical
/// results when iterating tiles, possibly resulting in infinite iteration. So it is recommended
/// not to use deserialization to construct `TileSchema` from any external sources or long-term
/// storage. Use [`super::TileSchemaBuilder`] instead which does all necessary validation before
/// created a schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TileSchema {
    /// Position where all tiles have `X == 0, Y == 0` indices.
    pub(super) origin: Point2,
    /// Rectangle that contains all tiles of the tile scheme.
    pub(super) tile_bounds: Rect,
    /// Rectangle that tiles should wrap around.
    pub(super) world_bounds: Rect,
    /// Sorted set of levels of detail that specify resolutions for each z-level.
    pub(super) lods: Arc<Vec<f64>>,
    /// Width of a single tile in pixels.
    pub(super) tile_width: u32,
    /// Height of a single tile in pixels.
    pub(super) tile_height: u32,
    /// Direction of the Y-axis.
    pub(super) y_direction: VerticalDirection,
    /// Specifies whether tiles should be wrapped over x index.
    pub(super) wrap_x: bool,
}

pub struct Lod {
    resolution: f64,
    z_index: u32,
}

impl TileSchema {
    /// Resolution of the given z-level, if exists.
    pub fn lod_resolution(&self, z: u32) -> Option<f64> {
        let resolution = *self.lods.get(z as usize)?;
        if resolution.is_finite() && resolution > 0.0 {
            Some(resolution)
        } else {
            None
        }
    }

    /// Origin point of the tiles.
    ///
    /// Origin point is set in projection coordinates (for example, in Mercator meters for Mercator projection).
    ///
    /// It is the point where the tile with index `(X: 0, Y: 0)` is located (for every Z index). If the schema
    /// uses direction of Y indices from top to bottom, the origin point will be at the left top angle of the
    /// tile. If the direction of Y indices is from bottom to top, the origin point will be at the left bottom
    /// point of the tile.
    pub fn origin(&self) -> Point2 {
        self.origin
    }

    /// Rectangle in projected coordinates for which tiles are available.
    ///
    /// Tiles that lie outside of the bounds will not be requested from the source.
    pub fn tile_bounds(&self) -> Rect {
        self.tile_bounds
    }

    /// Rectangle in projected coordinates, which includes the whole globe as defined by the target
    /// projection.
    ///
    /// World bounds are used to calculate x coordinate of tiles when wrapping around 180 parallel, and to
    /// calculate resolution levels for logarithmic z-levels. If wrapping is not used and z-levels are set
    /// manually, this parameter is not required for correct calculations of the tile indices.
    pub fn world_bounds(&self) -> Rect {
        self.world_bounds
    }

    /// If set to true, tiles will wrap around x axis of the schema bounds.
    ///
    /// This means, that if a tile requested for x coordinate that is larger than the maximum x of the
    /// bounding box or smaller than the minimum x value, the tile index will be calculated by reducing
    /// or increasing x coordinate by the whole number of bounding box widths. This produces an effect of
    /// horizontally infinite map, where a user can pan as log as they want to the right or left.
    pub fn wrap_x(&self) -> bool {
        self.wrap_x
    }

    /// Width of a single tile.
    pub fn tile_width(&self) -> u32 {
        self.tile_width
    }

    /// Height of a single tile.
    pub fn tile_height(&self) -> u32 {
        self.tile_height
    }

    /// Direction of Y-indices of the tiles.
    ///
    /// The direction is specified relative to the projected coordinates direction. We consider negative Y
    /// coordinates of the projection to be at the bottom and positive at the top. So if
    /// `VerticalDirection::TopToBottom` is set, tiles with Y index 0 will be at the very top of the map.
    pub fn y_direction(&self) -> VerticalDirection {
        self.y_direction
    }

    /// Iterate over tile indices that should be displayed for the given map view.
    pub fn iter_tiles(&self, view: &MapView) -> Option<impl Iterator<Item = WrappingTileIndex>> {
        let resolution = view.resolution();
        let bounding_box = view.get_bbox()?;
        self.iter_tiles_over_bbox(resolution, bounding_box)
    }

    /// Returns the bounding rectangle of the given tile index, if the index is valid.
    pub fn tile_bbox(&self, index: WrappingTileIndex) -> Option<Rect> {
        let x_index = index.virtual_x;
        let y_index = index.y;

        let resolution = self.lod_resolution(index.z)?;
        let x_min = self.origin.x() + (x_index as f64) * self.tile_width as f64 * resolution;
        let y_min = match self.y_direction {
            VerticalDirection::TopToBottom => {
                self.origin.y() - (y_index + 1) as f64 * self.tile_height as f64 * resolution
            }
            VerticalDirection::BottomToTop => {
                self.origin.y() + (y_index as f64) * self.tile_height as f64 * resolution
            }
        };

        Some(Rect::new(
            x_min,
            y_min,
            x_min + self.tile_width as f64 * resolution,
            y_min + self.tile_height as f64 * resolution,
        ))
    }

    /// Select a level of detail for the given resolution.
    fn select_lod(&self, resolution: f64) -> Option<Lod> {
        if !resolution.is_finite() || self.lods.is_empty() {
            return None;
        }

        let adj_resolution = resolution * (1.0 + RESOLUTION_TOLERANCE);
        let index = self
            .lods
            .partition_point(|&resolution| resolution >= adj_resolution);
        let index = index.min(self.lods.len() - 1);
        Some(Lod {
            resolution: self.lods[index],
            z_index: index as u32,
        })
    }

    fn iter_tiles_over_bbox(
        &self,
        resolution: f64,
        bounding_box: Rect,
    ) -> Option<impl Iterator<Item = WrappingTileIndex>> {
        let lod = self.select_lod(resolution)?;

        let tile_w = lod.resolution * self.tile_width as f64;
        let tile_h = lod.resolution * self.tile_height as f64;

        let x_min = (self.x_adj(bounding_box.x_min()) / tile_w).floor() as i32;
        let x_min = x_min.max(self.min_x_displayed_index(lod.resolution));

        let x_max_adj = self.x_adj(bounding_box.x_max());
        let x_add_one = if (x_max_adj % tile_w) < 0.001 { -1 } else { 0 };

        let x_max = (x_max_adj / tile_w) as i32 + x_add_one;
        let x_max = x_max.min(self.max_x_displayed_index(lod.resolution));

        let (top, bottom) = if self.y_direction == VerticalDirection::TopToBottom {
            (bounding_box.y_min(), bounding_box.y_max())
        } else {
            (bounding_box.y_max(), bounding_box.y_min())
        };

        let y_min = (self.y_adj(bottom) / tile_h) as i32;
        let y_min = y_min.max(self.min_y_index(lod.resolution));

        let y_max_adj = self.y_adj(top);
        let y_add_one = if (y_max_adj % tile_h) < 0.001 { -1 } else { 0 };

        let y_max = (y_max_adj / tile_h) as i32 + y_add_one;
        let y_max = y_max.min(self.max_y_index(lod.resolution));

        let world_x_min = self.world_min_x_index(lod.resolution);
        let world_x_max = self.world_max_x_index(lod.resolution);
        let index_range = world_x_max - world_x_min + 1;

        let actual_x =
            move |display_x: i32| (display_x - world_x_min).rem_euclid(index_range) + world_x_min;

        let tile_x_min = self.min_x_index(lod.resolution);
        let tile_x_max = self.max_x_index(lod.resolution);

        Some(
            (x_min..=x_max)
                .filter(move |x| actual_x(*x) >= tile_x_min && actual_x(*x) <= tile_x_max)
                .flat_map(move |x| {
                    (y_min..=y_max).map(move |y| WrappingTileIndex {
                        x: actual_x(x),
                        y,
                        z: lod.z_index,
                        virtual_x: x,
                    })
                }),
        )
    }

    fn x_adj(&self, x: f64) -> f64 {
        x - self.origin.x()
    }

    fn y_adj(&self, y: f64) -> f64 {
        match self.y_direction {
            VerticalDirection::TopToBottom => self.origin.y() - y,
            VerticalDirection::BottomToTop => y - self.origin.y(),
        }
    }

    fn min_x_displayed_index(&self, resolution: f64) -> i32 {
        if self.wrap_x() {
            i32::MIN
        } else {
            self.min_x_index(resolution)
        }
    }

    fn max_x_displayed_index(&self, resolution: f64) -> i32 {
        if self.wrap_x() {
            i32::MAX
        } else {
            self.max_x_index(resolution)
        }
    }

    fn min_x_index(&self, resolution: f64) -> i32 {
        ((self.tile_bounds.x_min() - self.origin.x()) / resolution / self.tile_width as f64).floor()
            as i32
    }

    fn max_x_index(&self, resolution: f64) -> i32 {
        let pix_bound = (self.tile_bounds.x_max() - self.origin.x()) / resolution;
        let floored = pix_bound.floor();
        if (pix_bound - floored).abs() < 0.1 {
            (pix_bound / self.tile_width as f64) as i32 - 1
        } else {
            (pix_bound / self.tile_width as f64) as i32
        }
    }

    fn world_min_x_index(&self, resolution: f64) -> i32 {
        if self.wrap_x {
            ((self.world_bounds.x_min() - self.origin.x()) / resolution / self.tile_width as f64)
                .floor() as i32
        } else {
            self.min_x_index(resolution)
        }
    }

    fn world_max_x_index(&self, resolution: f64) -> i32 {
        if self.wrap_x {
            let pix_bound = (self.world_bounds.x_max() - self.origin.x()) / resolution;
            let floored = pix_bound.floor();
            if (pix_bound - floored).abs() < 0.1 {
                (pix_bound / self.tile_width as f64) as i32 - 1
            } else {
                (pix_bound / self.tile_width as f64) as i32
            }
        } else {
            self.max_x_index(resolution)
        }
    }

    fn min_y_index(&self, resolution: f64) -> i32 {
        match self.y_direction {
            VerticalDirection::TopToBottom => ((self.origin.y() - self.tile_bounds.y_max())
                / resolution
                / self.tile_height as f64)
                .floor() as i32,
            VerticalDirection::BottomToTop => ((self.tile_bounds.y_min() - self.origin.y())
                / resolution
                / self.tile_height as f64)
                .floor() as i32,
        }
    }

    fn max_y_index(&self, resolution: f64) -> i32 {
        let pix_bound = match self.y_direction {
            VerticalDirection::TopToBottom => {
                (self.origin.y() - self.tile_bounds.y_min()) / resolution
            }
            VerticalDirection::BottomToTop => {
                (self.tile_bounds.y_max() - self.origin.y()) / resolution
            }
        };

        let floored = pix_bound.floor();
        if (pix_bound - floored).abs() < 0.1 {
            (pix_bound / self.tile_width as f64) as i32 - 1
        } else {
            (pix_bound / self.tile_width as f64) as i32
        }
    }
}

#[cfg(test)]
mod tests {
    use galileo_types::cartesian::Size;

    use super::*;
    use crate::tile_schema::WrappingTileIndex;

    fn simple_schema() -> TileSchema {
        schema_with_lods(vec![8.0, 4.0, 2.0])
    }

    fn schema_with_lods(lods: Vec<f64>) -> TileSchema {
        TileSchema {
            origin: Point2::default(),
            tile_bounds: Rect::new(0.0, 0.0, 2048.0, 2048.0),
            world_bounds: Rect::new(0.0, 0.0, 2048.0, 2048.0),
            lods: Arc::new(lods),
            tile_width: 256,
            tile_height: 256,
            y_direction: VerticalDirection::BottomToTop,
            wrap_x: true,
        }
    }

    fn get_view(resolution: f64, bbox: Rect) -> MapView {
        MapView::new_projected(&bbox.center(), resolution).with_size(Size::new(
            bbox.width() / resolution,
            bbox.height() / resolution,
        ))
    }

    #[test]
    fn select_lod() {
        let schema = simple_schema();
        assert_eq!(schema.select_lod(8.0).unwrap().z_index, 0);
        assert_eq!(schema.select_lod(9.0).unwrap().z_index, 0);
        assert_eq!(schema.select_lod(16.0).unwrap().z_index, 0);
        assert_eq!(schema.select_lod(7.99).unwrap().z_index, 0);
        assert_eq!(schema.select_lod(7.5).unwrap().z_index, 1);
        assert_eq!(schema.select_lod(4.1).unwrap().z_index, 1);
        assert_eq!(schema.select_lod(4.0).unwrap().z_index, 1);
        assert_eq!(schema.select_lod(1.5).unwrap().z_index, 2);
        assert_eq!(schema.select_lod(1.0).unwrap().z_index, 2);
    }

    #[test]
    fn select_lod_skipped_levels() {
        let schema = schema_with_lods(vec![f64::MAX, f64::MAX, 8.0, 4.0, 2.0]);
        assert_eq!(schema.select_lod(8.0).unwrap().z_index, 2);
        assert_eq!(schema.select_lod(9.0).unwrap().z_index, 2);
        assert_eq!(schema.select_lod(16.0).unwrap().z_index, 2);
        assert_eq!(schema.select_lod(7.99).unwrap().z_index, 2);
        assert_eq!(schema.select_lod(7.5).unwrap().z_index, 3);
        assert_eq!(schema.select_lod(4.1).unwrap().z_index, 3);
        assert_eq!(schema.select_lod(4.0).unwrap().z_index, 3);
        assert_eq!(schema.select_lod(1.5).unwrap().z_index, 4);
        assert_eq!(schema.select_lod(1.0).unwrap().z_index, 4);
    }

    #[test]
    fn select_lod_duplicate_levels() {
        let schema = schema_with_lods(vec![16.0, 8.0, 8.0, 8.0, 8.0, 2.0]);
        assert_eq!(schema.select_lod(16.0).unwrap().z_index, 0);
        assert_eq!(schema.select_lod(17.0).unwrap().z_index, 0);
        assert_eq!(schema.select_lod(15.99).unwrap().z_index, 0);
        assert_eq!(schema.select_lod(8.1).unwrap().z_index, 1);
        assert_eq!(schema.select_lod(8.0).unwrap().z_index, 1);
        assert_eq!(schema.select_lod(7.99).unwrap().z_index, 1);
        assert_eq!(schema.select_lod(2.1).unwrap().z_index, 5);
        assert_eq!(schema.select_lod(2.0).unwrap().z_index, 5);
        assert_eq!(schema.select_lod(1.0).unwrap().z_index, 5);
    }

    #[test]
    fn iter_indices_full_bbox() {
        let schema = simple_schema();
        let bbox = Rect::new(0.0, 0.0, 2048.0, 2048.0);
        let view = get_view(8.0, bbox);
        assert_eq!(schema.iter_tiles(&view).unwrap().count(), 1);
        for tile in schema.iter_tiles(&view).unwrap() {
            assert_eq!(tile.x, 0);
            assert_eq!(tile.y, 0);
            assert_eq!(tile.z, 0);
        }

        let view = get_view(4.0, bbox);
        let mut tiles: Vec<WrappingTileIndex> = schema.iter_tiles(&view).unwrap().collect();
        tiles.dedup();
        assert_eq!(tiles.len(), 4);
        for tile in tiles {
            assert!(tile.x >= 0 && tile.x <= 1);
            assert!(tile.y >= 0 && tile.y <= 1);
            assert_eq!(tile.z, 1);
        }

        let view = get_view(2.0, bbox);
        let mut tiles: Vec<WrappingTileIndex> = schema.iter_tiles(&view).unwrap().collect();
        tiles.dedup();
        assert_eq!(tiles.len(), 16);
        for tile in tiles {
            assert!(tile.x >= 0 && tile.x <= 3);
            assert!(tile.y >= 0 && tile.y <= 3);
            assert_eq!(tile.z, 2);
        }
    }

    #[test]
    fn iter_indices_part_bbox() {
        let schema = simple_schema();
        let bbox = Rect::new(200.0, 700.0, 1200.0, 1100.0);
        let view = get_view(8.0, bbox);
        assert_eq!(schema.iter_tiles(&view).unwrap().count(), 1);
        for tile in schema.iter_tiles(&view).unwrap() {
            assert_eq!(tile.x, 0);
            assert_eq!(tile.y, 0);
            assert_eq!(tile.z, 0);
        }

        let view = get_view(4.0, bbox);
        let mut tiles: Vec<WrappingTileIndex> = schema.iter_tiles(&view).unwrap().collect();
        tiles.dedup();
        assert_eq!(tiles.len(), 4);
        for tile in tiles {
            assert!(tile.x >= 0 && tile.x <= 1);
            assert!(tile.y >= 0 && tile.y <= 1);
            assert_eq!(tile.z, 1);
        }

        let view = get_view(2.0, bbox);
        let mut tiles: Vec<WrappingTileIndex> = schema.iter_tiles(&view).unwrap().collect();
        tiles.dedup();
        assert_eq!(tiles.len(), 6);
        for tile in tiles {
            assert!(tile.x >= 0 && tile.x <= 2);
            assert!(tile.y >= 1 && tile.y <= 2);
            assert_eq!(tile.z, 2);
        }
    }

    #[test]
    fn iter_tiles_outside_of_bbox() {
        let schema = simple_schema();
        let bbox = Rect::new(-100.0, -100.0, -50.0, -50.0);
        let view = get_view(8.0, bbox);
        assert_eq!(schema.iter_tiles(&view).unwrap().count(), 0);
        let view = get_view(2.0, bbox);
        assert_eq!(schema.iter_tiles(&view).unwrap().count(), 0);

        let bbox = Rect::new(2100.0, 0.0, 2500.0, 2048.0);
        let view = get_view(8.0, bbox);
        assert_eq!(schema.iter_tiles(&view).unwrap().count(), 1);
        assert_eq!(
            schema.iter_tiles(&view).unwrap().next().unwrap().virtual_x,
            1
        );
        let view = get_view(2.0, bbox);
        assert_eq!(schema.iter_tiles(&view).unwrap().count(), 4);
        assert_eq!(
            schema.iter_tiles(&view).unwrap().next().unwrap().virtual_x,
            4
        );
    }

    #[test]
    fn iter_tiles_origin_out_of_bounds() {
        let schema = TileSchema {
            origin: Point2::new(0.0, 0.0),
            tile_bounds: Rect::new(1000.0, 1000.0, 2000.0, 2000.0),
            world_bounds: Rect::new(1000.0, 1000.0, 2000.0, 2000.0),
            lods: Arc::new(vec![30.0, 10.0, 1.0]),
            tile_width: 10,
            tile_height: 10,
            y_direction: VerticalDirection::BottomToTop,
            wrap_x: false,
        };

        let tiles: Vec<_> = schema
            .iter_tiles_over_bbox(10.0, Rect::new(0.0, 0.0, 500.0, 500.0))
            .unwrap()
            .collect();
        assert!(
            tiles.is_empty(),
            "Expected empty tiles iter, but got: {tiles:?}"
        );

        let tiles: Vec<_> = schema
            .iter_tiles_over_bbox(10.0, Rect::new(900.0, 900.0, 1100.0, 1100.0))
            .unwrap()
            .collect();

        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0], WrappingTileIndex::new(10, 10, 1));

        let tiles: Vec<_> = schema
            .iter_tiles_over_bbox(30.0, Rect::new(900.0, 900.0, 950.0, 950.0))
            .unwrap()
            .collect();

        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0], WrappingTileIndex::new(3, 3, 0));
    }

    #[test]
    fn iter_tiles_origin_out_of_bounds_top_to_bottom() {
        let schema = TileSchema {
            origin: Point2::new(0.0, 3000.0),
            tile_bounds: Rect::new(1000.0, 1000.0, 2000.0, 2000.0),
            world_bounds: Rect::new(1000.0, 1000.0, 2000.0, 2000.0),
            lods: Arc::new(vec![30.0, 10.0, 1.0]),
            tile_width: 10,
            tile_height: 10,
            y_direction: VerticalDirection::TopToBottom,
            wrap_x: false,
        };

        let tiles: Vec<_> = schema
            .iter_tiles_over_bbox(10.0, Rect::new(0.0, 0.0, 500.0, 500.0))
            .unwrap()
            .collect();
        assert!(
            tiles.is_empty(),
            "Expected empty tiles iter, but got: {tiles:?}"
        );

        let tiles: Vec<_> = schema
            .iter_tiles_over_bbox(10.0, Rect::new(900.0, 900.0, 1099.0, 1099.0))
            .unwrap()
            .collect();

        println!("{tiles:?}");
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0], WrappingTileIndex::new(10, 19, 1));

        let tiles: Vec<_> = schema
            .iter_tiles_over_bbox(30.0, Rect::new(900.0, 900.0, 950.0, 950.0))
            .unwrap()
            .collect();

        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0], WrappingTileIndex::new(3, 6, 0));
    }

    #[test]
    fn tile_bbox_origin_out_of_bounds() {
        let schema = TileSchema {
            origin: Point2::new(0.0, 0.0),
            tile_bounds: Rect::new(1000.0, 1000.0, 2000.0, 2000.0),
            world_bounds: Rect::new(1000.0, 1000.0, 2000.0, 2000.0),
            lods: Arc::new(vec![300.0, 100.0, 10.0, 1.0]),
            tile_width: 10,
            tile_height: 10,
            y_direction: VerticalDirection::BottomToTop,
            wrap_x: false,
        };

        let bbox = schema.tile_bbox(WrappingTileIndex::new(10, 10, 2)).unwrap();
        assert_eq!(bbox, Rect::new(1000.0, 1000.0, 1100.0, 1100.0));
    }

    #[test]
    fn wrapping_is_done_around_world_bounds() {
        let schema = TileSchema {
            origin: Point2::default(),
            tile_bounds: Rect::new(0.0, 0.0, 1000.0, 1000.0),
            world_bounds: Rect::new(0.0, 0.0, 2000.0, 2000.0),
            lods: Arc::new(vec![10.0, 5.0, 1.0]),
            tile_width: 100,
            tile_height: 100,
            y_direction: VerticalDirection::BottomToTop,
            wrap_x: true,
        };

        let bbox = Rect::new(0.0, 0.0, 3000.0, 1000.0);
        let tiles: Vec<_> = schema.iter_tiles_over_bbox(10.0, bbox).unwrap().collect();

        assert_eq!(tiles.len(), 2, "Unexpected tiles set: {tiles:?}");
        assert_eq!(
            tiles[0],
            WrappingTileIndex {
                z: 0,
                x: 0,
                y: 0,
                virtual_x: 0
            }
        );
        assert_eq!(
            tiles[1],
            WrappingTileIndex {
                z: 0,
                x: 0,
                y: 0,
                virtual_x: 2
            }
        );
    }
}
