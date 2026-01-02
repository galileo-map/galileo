//! Builder for [`TileSchema`].

use core::f64;
use std::sync::Arc;

use ahash::HashMap;
use galileo_types::cartesian::{Point2, Rect};

use super::schema::{TileSchema, VerticalDirection};

/// Builder for [`TileSchema`].
///
/// The builder validates all the input parameters and guarantees that the created schema is valid.
#[derive(Debug)]
pub struct TileSchemaBuilder {
    origin: Point2,
    world_bounds: Rect,
    tile_bounds: Rect,
    lods: Lods,
    tile_width: u32,
    tile_height: u32,
    y_direction: VerticalDirection,
    wrap_x: bool,
}

#[derive(Debug)]
enum Lods {
    Logarithmic(Vec<u32>),
    Custom(HashMap<u32, f64>),
}

/// Errors that can occur during building a [`TileSchema`].
#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum TileSchemaError {
    /// No zoom levels provided
    #[error("no zoom levels provided")]
    NoZLevelsProvided,

    /// Invalid tile size
    #[error("invalid tile size: {width}x{height}")]
    InvalidTileSize {
        /// Tile width
        width: u32,
        /// Tile height
        height: u32,
    },

    /// Resolution too small.
    ///
    /// If the resolution is too small, it means that the tile indices would exceed the maximum
    /// representable value (u64::MAX).
    #[error("resolution too small at z-level {z_level}: {resolution}")]
    ResolutionTooSmall {
        /// Z-level where resolution is too small
        z_level: u32,
        /// The resolution value that is too small
        resolution: f64,
    },

    /// Z-level resolutions are not decreasing
    #[error("resolution at z-level {upper_level} ({upper_resolution}) cannot be smaller than resolution at z-level {lower_level} ({lower_resolution})")]
    NotSortedZLevels {
        /// Smaller z-level value
        upper_level: u32,
        /// Resolution of the `upper_level`
        upper_resolution: f64,
        /// Larger z-level value
        lower_level: u32,
        /// Resolution of the `lower_level`
        lower_resolution: f64,
    },

    /// Tile bounds have invalid value
    #[error("tile bounds have zero size or not finite: {0:?}")]
    InvalidTileBounds(Rect),

    /// World bounds have invalid value
    #[error("world bounds have zero size or not finite: {0:?}")]
    InvalidWorldBounds(Rect),
}

impl TileSchemaBuilder {
    /// Create a new builder with default parameters.
    pub fn build(self) -> Result<TileSchema, TileSchemaError> {
        if !self.tile_bounds.width().is_normal() || !self.tile_bounds.height().is_normal() {
            return Err(TileSchemaError::InvalidTileBounds(self.tile_bounds));
        }

        if self.wrap_x
            && matches!(self.lods, Lods::Logarithmic(_))
            && (!self.world_bounds.width().is_normal() || !self.world_bounds.height().is_normal())
        {
            return Err(TileSchemaError::InvalidWorldBounds(self.world_bounds));
        }

        // Resolution is bound by the maximum tile index that can be represented
        let min_resolution = f64::min(
            self.world_bounds.width() / self.tile_width as f64 / i64::MAX as f64,
            self.world_bounds.height() / self.tile_height as f64 / i64::MAX as f64,
        );

        let lods = match self.lods {
            Lods::Logarithmic(z_levels) => {
                if z_levels.is_empty() {
                    return Err(TileSchemaError::NoZLevelsProvided);
                }

                let top_resolution = self.world_bounds.width() / self.tile_width as f64;

                let max_z_level = *z_levels.iter().max().unwrap_or(&0);
                let mut lods = vec![f64::MAX; max_z_level as usize + 1];

                for z in z_levels {
                    let resolution = top_resolution / f64::powi(2.0, z as i32);

                    if resolution < min_resolution {
                        return Err(TileSchemaError::ResolutionTooSmall {
                            z_level: z,
                            resolution,
                        });
                    }

                    lods[z as usize] = resolution;
                }

                for i in 1..lods.len() {
                    if lods[i] == f64::MAX {
                        lods[i] = lods[i - 1];
                    }
                }

                lods
            }
            Lods::Custom(z_levels) => {
                if z_levels.is_empty() {
                    return Err(TileSchemaError::NoZLevelsProvided);
                }

                let max_z_level = *z_levels.keys().max().unwrap_or(&0);
                let mut lods = vec![f64::MAX; max_z_level as usize + 1];

                for i in 0..lods.len() {
                    match z_levels.get(&(i as u32)) {
                        Some(&resolution) => {
                            if resolution < min_resolution {
                                return Err(TileSchemaError::ResolutionTooSmall {
                                    z_level: i as u32,
                                    resolution,
                                });
                            }

                            if i > 0 && lods[i - 1] < resolution {
                                return Err(TileSchemaError::NotSortedZLevels {
                                    upper_level: i as u32 - 1,
                                    upper_resolution: lods[i - 1],
                                    lower_level: i as u32,
                                    lower_resolution: resolution,
                                });
                            }

                            lods[i] = resolution
                        }
                        None => {
                            if i > 0 {
                                lods[i] = lods[i - 1];
                            }
                        }
                    }
                }

                lods
            }
        };

        if self.tile_width == 0 || self.tile_height == 0 {
            return Err(TileSchemaError::InvalidTileSize {
                width: self.tile_width,
                height: self.tile_height,
            });
        }

        Ok(TileSchema {
            origin: self.origin,
            tile_bounds: self.tile_bounds,
            world_bounds: self.world_bounds,
            lods: Arc::new(lods),
            tile_width: self.tile_width,
            tile_height: self.tile_height,
            y_direction: self.y_direction,
            wrap_x: self.wrap_x,
        })
    }

    /// Standard Web Mercator based tile scheme (used, for example, by OSM and Google maps).
    pub fn web_mercator(z_levels: impl IntoIterator<Item = u32>) -> Self {
        const TILE_SIZE: u32 = 256;

        Self::web_mercator_base()
            .logarithmic_z_levels(z_levels)
            .rect_tile_size(TILE_SIZE)
    }

    fn web_mercator_base() -> Self {
        const MAX_COORD_VALUE: f64 = 20037508.342787;

        Self {
            origin: Point2::new(-MAX_COORD_VALUE, MAX_COORD_VALUE),
            world_bounds: Rect::new(
                -MAX_COORD_VALUE,
                -MAX_COORD_VALUE,
                MAX_COORD_VALUE,
                MAX_COORD_VALUE,
            ),
            tile_bounds: Rect::new(
                -MAX_COORD_VALUE,
                -MAX_COORD_VALUE,
                MAX_COORD_VALUE,
                MAX_COORD_VALUE,
            ),
            lods: Lods::Logarithmic(Vec::new()),
            tile_width: 0,
            tile_height: 0,
            y_direction: VerticalDirection::TopToBottom,
            wrap_x: true,
        }
    }

    /// Set both tile width and height to `tile_size` in pixels.
    pub fn rect_tile_size(mut self, tile_size: u32) -> Self {
        self.tile_width = tile_size;
        self.tile_height = tile_size;

        self
    }

    /// Set width of the tiles in pixels.
    pub fn tile_width(mut self, width: u32) -> Self {
        self.tile_width = width;
        self
    }

    /// Set height of the tiles in pixels.
    pub fn tile_height(mut self, height: u32) -> Self {
        self.tile_height = height;
        self
    }

    /// Set z-levels of the tile schema with the logarithmic scale with base 2.
    ///
    /// This is the usual approach for most tile schemas to approach the levels of detail. It
    /// considers the top level with z-index `0` to be a single tile that includes the whole
    /// world map. Then the next level divides this tile into 4 parts (2x2 along x and y axis).
    /// Next level further divides every tile into 4 and so on.
    ///
    /// With this approach the whole world map is divided into `2^z_level` tiles along each axis.
    /// The resolution for each level is calculated accordingly.
    ///
    /// For the builder to be able to calculate logarithmic scale, it must have world bounds
    /// specified correctly.
    ///
    /// The max z-level that can be set with logarithmic scale is limited by the number of X and Y
    /// indices that can be expressed with `i64`, which for tiles of size 256 pixels in Web Mercator
    /// projection equals `63`. If you have tiles with z-indices larger than this value, use `z-levels`
    /// method instead to set resolutions manually.
    pub fn logarithmic_z_levels(mut self, z_levels: impl IntoIterator<Item = u32>) -> Self {
        self.lods = Lods::Logarithmic(z_levels.into_iter().collect());

        self
    }

    /// Sets the z-levels with specified resolution from the iterator.
    ///
    /// Z-levels are given as tuples of `(z-index, resolution)`. Smaller z-indexes must correspond
    /// to larger resolution values. If z-levels are not sorted correctly, building the tile schema
    /// would result in [`TileSchemaError::NotSortedZLevels`] error.
    pub fn z_levels(mut self, z_levels: impl IntoIterator<Item = (u32, f64)>) -> Self {
        self.lods = Lods::Custom(z_levels.into_iter().collect());
        self
    }

    /// If set to true, tiles will wrap around x axis of the schema bounds.
    ///
    /// This means, that if a tile requested for x coordinate that is larger than the maximum x of the
    /// bounding box or smaller than the minimum x value, the tile index will be calculated by reducing
    /// or increasing x coordinate by the whole number of bounding box widths. This produces an effect of
    /// horizontally infinite map, where a user can pan as log as they want to the right or left.
    ///
    /// Note, that for wrapping to work property, world bounds of the tile schema should cover the whole globe.
    /// This is not enforced in `.build()` method validatation since tile schema is agnostic to the CRS
    /// it will be used for.
    pub fn wrap_x(mut self, shall_wrap: bool) -> Self {
        self.wrap_x = shall_wrap;
        self
    }

    /// Sets the origin point of the tiles.
    ///
    /// Origin point is set in projection coordinates (for example, in Mercator meters for Mercator projection).
    ///
    /// It is the point where the tile with index `(X: 0, Y: 0)` is located (for every Z index). If the schema
    /// uses direction of Y indices from top to bottom, the origin point will be at the left top angle of the
    /// tile. If the direction of Y indices is from bottom to top, the origin point will be at the left bottom
    /// point of the tile.
    ///
    /// ```
    /// # use galileo::tile_schema::TileSchemaBuilder;
    /// # use galileo::galileo_types::cartesian::Point2;
    /// let tile_schema = TileSchemaBuilder::web_mercator(0..23)
    ///     .origin(Point2::new(-20_037_508.342787, 20_037_508.342787))
    ///     .build()
    ///     .expect("tile schema is properly defined");
    /// ```
    ///
    /// Note that origin point doesn't have to be inside the tile bounds. For example, the origin may point to
    /// the top left angle of the world map, but tiles might only be available for a specific region, and the
    /// bounds will only contain that region. In this case tiles may have indices starting not from 0.
    pub fn origin(mut self, origin: Point2) -> Self {
        self.origin = origin;
        self
    }

    /// Sets the rectangle in projected coordinates for which tiles are available.
    ///
    /// Tiles that lie outside of the bounds will not be requested from the source.
    ///
    /// ```
    /// # use galileo::tile_schema::TileSchemaBuilder;
    /// # use galileo::galileo_types::cartesian::Rect;
    /// let tile_schema = TileSchemaBuilder::web_mercator(0..23)
    ///     // only show tiles for Angola
    ///     .tile_bounds(Rect::new(1282761., -1975899., 2674573., -590691.))
    ///     .build()
    ///     .expect("tile schema is properly defined");
    /// ```
    ///
    /// # Errors
    ///
    /// If either width or height of the bounds rectangle is `0`, `NaN` or `Infinity`, building the tile schema
    /// will return an error [`TileSchemaError::InvalidTileBounds`].
    pub fn tile_bounds(mut self, bounds: Rect) -> Self {
        self.tile_bounds = bounds;
        self
    }

    /// Sets the rectangle in projected coordinates, which includes the whole globe as defined by the target
    /// projection.
    ///
    /// World bounds are used to calculate x coordinate of tiles when wrapping around 180 parallel, and to
    /// calculate resolution levels for logarithmic z-levels. If wrapping is not used and z-levels are set
    /// manually, this parameter is not required for correct calculations of the tile indices.
    ///
    /// ```
    /// # use galileo::tile_schema::TileSchemaBuilder;
    /// # use galileo::galileo_types::cartesian::Rect;
    /// let tile_schema = TileSchemaBuilder::web_mercator(0..23)
    ///     // square WebMercator projetion bounds
    ///     .world_bounds(Rect::new(-20037508.342787, -20037508.342787, 20037508.342787, 20037508.342787))
    ///     .build()
    ///     .expect("tile schema is properly defined");
    /// ```
    ///
    /// # Errors
    ///
    /// If either width or height of the bounds rectangle is `0`, `NaN` or `Infinity`, building the tile schema
    /// will return an error `TileSchemaError::InvalidWorldBounds`. This check is skipped if neither wrapping nor
    /// logarithmic z-levels are used for the schema.
    pub fn world_bounds(mut self, bounds: Rect) -> Self {
        self.world_bounds = bounds;
        self
    }

    /// Sets direction of Y-indices of the tiles.
    ///
    /// The direction is specified relative to the projected coordinates direction. We consider negative Y
    /// coordinates of the projection to be at the bottom and positive at the top. So if
    /// `VerticalDirection::TopToBottom` is set, tiles with Y index 0 will be at the very top of the map.
    pub fn y_direction(mut self, direction: VerticalDirection) -> Self {
        self.y_direction = direction;
        self
    }
}

#[cfg(test)]
mod tests {
    use core::f64;

    use approx::assert_abs_diff_eq;

    use super::*;
    use crate::tile_schema::VerticalDirection;

    const TOP_RESOLUTION: f64 = 156543.03392802345;

    #[test]
    fn schema_builder_normal_web_mercator() {
        let schema = TileSchemaBuilder::web_mercator(0..=20).build().unwrap();
        assert_eq!(schema.lods.len(), 21);

        assert_abs_diff_eq!(schema.lods[0], TOP_RESOLUTION);

        for z in 1..=20 {
            let expected = TOP_RESOLUTION / 2f64.powi(z);
            assert_abs_diff_eq!(schema.lods[z as usize], expected);
        }

        assert_eq!(schema.tile_width, 256);
        assert_eq!(schema.tile_height, 256);
        assert_eq!(
            schema.origin,
            Point2::new(-20037508.342787, 20037508.342787)
        );
        assert_eq!(
            schema.tile_bounds,
            Rect::new(
                -20037508.342787,
                -20037508.342787,
                20037508.342787,
                20037508.342787
            )
        );
        assert_eq!(schema.y_direction, VerticalDirection::TopToBottom);
    }

    #[test]
    fn schema_builder_no_z_levels() {
        let result = TileSchemaBuilder::web_mercator(std::iter::empty()).build();
        assert!(
            matches!(result, Err(TileSchemaError::NoZLevelsProvided)),
            "Got {:?}",
            result
        );
    }

    #[test]
    fn skipping_first_z_levels() {
        let schema = TileSchemaBuilder::web_mercator(5..=10).build().unwrap();
        assert_eq!(schema.lods.len(), 11);

        assert_abs_diff_eq!(schema.lods[5], TOP_RESOLUTION / 2f64.powi(5));
        assert_abs_diff_eq!(schema.lods[10], TOP_RESOLUTION / 2f64.powi(10));
    }

    #[test]
    fn zero_tile_size() {
        let result = TileSchemaBuilder::web_mercator(0..=20)
            .rect_tile_size(0)
            .build();
        assert!(
            matches!(
                result,
                Err(TileSchemaError::InvalidTileSize {
                    width: 0,
                    height: 0
                })
            ),
            "Got {:?}",
            result
        );
    }

    #[test]
    fn skipped_first_z_levels_have_f64_max() {
        let schema = TileSchemaBuilder::web_mercator(5..=10).build().unwrap();
        assert_eq!(schema.lods.len(), 11);

        for z in 0..5 {
            assert_eq!(schema.lods[z], f64::MAX, "Level {} should have f64::MAX", z);
        }

        for z in 5..=10 {
            let expected = 156543.03392802345 / 2f64.powi(z);
            assert_abs_diff_eq!(schema.lods[z as usize], expected);
        }
    }

    #[test]
    fn skipped_middle_z_levels_use_previous_value() {
        let schema = TileSchemaBuilder::web_mercator([1, 2, 3, 5])
            .build()
            .unwrap();
        assert_eq!(schema.lods.len(), 6);

        assert_eq!(schema.lods[0], f64::MAX);

        assert_abs_diff_eq!(schema.lods[1], TOP_RESOLUTION / 2f64.powi(1));
        assert_abs_diff_eq!(schema.lods[2], TOP_RESOLUTION / 2f64.powi(2));
        assert_abs_diff_eq!(schema.lods[3], TOP_RESOLUTION / 2f64.powi(3));

        let expected_level_3 = TOP_RESOLUTION / 2f64.powi(3);
        assert_abs_diff_eq!(schema.lods[4], expected_level_3);

        assert_abs_diff_eq!(schema.lods[5], TOP_RESOLUTION / 2f64.powi(5));
    }

    #[test]
    fn skipped_multiple_middle_z_levels_use_previous_value() {
        let schema = TileSchemaBuilder::web_mercator([0, 1, 5]).build().unwrap();
        assert_eq!(schema.lods.len(), 6);

        assert_abs_diff_eq!(schema.lods[0], TOP_RESOLUTION / 2f64.powi(0));
        assert_abs_diff_eq!(schema.lods[1], TOP_RESOLUTION / 2f64.powi(1));

        let expected_level_1 = TOP_RESOLUTION / 2f64.powi(1);
        for z in 2..5 {
            assert_abs_diff_eq!(schema.lods[z], expected_level_1);
        }

        assert_abs_diff_eq!(schema.lods[5], TOP_RESOLUTION / 2f64.powi(5));
    }

    #[test]
    fn resolution_at_boundary_of_precision() {
        let result = TileSchemaBuilder::web_mercator(0..=63).build();
        assert!(
            result.is_ok(),
            "Expected z=0..=64 to be valid, got {:?}",
            result
        );

        let result = TileSchemaBuilder::web_mercator(0..=64).build();

        assert!(
            matches!(
                result,
                Err(TileSchemaError::ResolutionTooSmall { z_level: 64, .. })
            ),
            "Expected ResolutionTooSmall error, got {:?}",
            result
        );
    }

    #[test]
    fn custom_z_levels_equivalent_to_logarithmic() {
        let mut lods = vec![];
        const LEVELS: u32 = 32;

        for i in 0..=LEVELS {
            lods.push((i, TOP_RESOLUTION / 2f64.powi(i as i32)));
        }

        let tile_schema = TileSchemaBuilder::web_mercator(0..0)
            .z_levels(lods)
            .build()
            .unwrap();

        assert_eq!(
            tile_schema.lods,
            TileSchemaBuilder::web_mercator(0..=LEVELS)
                .build()
                .unwrap()
                .lods,
        );
    }

    #[test]
    fn custom_z_levels_check_for_min_resolution() {
        let result = TileSchemaBuilder::web_mercator(0..0)
            .z_levels([(0, TOP_RESOLUTION), (1, TOP_RESOLUTION / 2f64.powi(65))])
            .build();
        assert!(
            matches!(
                result,
                Err(TileSchemaError::ResolutionTooSmall { z_level: 1, .. })
            ),
            "Unexpected schema build result: {result:?}"
        );
    }

    #[test]
    fn custom_z_levels_must_be_sorted() {
        let mut lods = vec![];
        const LEVELS: u32 = 32;

        for i in 0..=LEVELS {
            lods.push((i, TOP_RESOLUTION / 2f64.powi(i as i32)));
        }

        lods.swap(1, 2);
        lods[1].0 = 1;
        lods[2].0 = 2;

        let result = TileSchemaBuilder::web_mercator(0..0).z_levels(lods).build();

        assert!(
            matches!(
                result,
                Err(TileSchemaError::NotSortedZLevels {
                    upper_level: 1,
                    lower_level: 2,
                    ..
                })
            ),
            "Unexpected schema build result: {result:?}"
        )
    }

    #[test]
    fn invalid_tile_bounds_return_error() {
        let to_check = [
            Rect::new(0.0, 0.0, 0.0, 1000.0),
            Rect::new(0.0, 0.0, 1000.0, 0.0),
            Rect::new(0.0, 0.0, f64::NAN, 1000.0),
            Rect::new(0.0, 0.0, 0.0, f64::INFINITY),
            Rect::new(f64::NEG_INFINITY, 0.0, 1000.0, 1000.0),
        ];

        for bounds in to_check {
            let result = TileSchemaBuilder::web_mercator(0..18)
                .tile_bounds(bounds)
                .build();
            assert!(
                matches!(result, Err(TileSchemaError::InvalidTileBounds(_))),
                "Error not returned for tile bounds: {bounds:?}"
            );
        }
    }

    #[test]
    fn invalid_world_bounds_return_error() {
        let to_check = [
            Rect::new(0.0, 0.0, 0.0, 1000.0),
            Rect::new(0.0, 0.0, 1000.0, 0.0),
            Rect::new(0.0, 0.0, f64::NAN, 1000.0),
            Rect::new(0.0, 0.0, 0.0, f64::INFINITY),
            Rect::new(f64::NEG_INFINITY, 0.0, 1000.0, 1000.0),
        ];

        for bounds in to_check {
            let result = TileSchemaBuilder::web_mercator(0..18)
                .world_bounds(bounds)
                .build();
            assert!(
                matches!(result, Err(TileSchemaError::InvalidWorldBounds(_))),
                "Error not returned for world bounds: {bounds:?}"
            );
        }
    }

    #[test]
    fn invalid_world_bounds_skipped_if_not_needed() {
        let to_check = [
            Rect::new(0.0, 0.0, 0.0, 1000.0),
            Rect::new(0.0, 0.0, 1000.0, 0.0),
            Rect::new(0.0, 0.0, f64::NAN, 1000.0),
            Rect::new(0.0, 0.0, 0.0, f64::INFINITY),
            Rect::new(f64::NEG_INFINITY, 0.0, 1000.0, 1000.0),
        ];

        for bounds in to_check {
            let result = TileSchemaBuilder::web_mercator(0..18)
                .world_bounds(bounds)
                .wrap_x(false)
                .z_levels([(0, 1000.0), (1, 500.0)])
                .build();
            assert!(
                result.is_ok(),
                "Error returned for world bounds: {bounds:?}"
            );
        }
    }

    #[test]
    fn setting_y_direction() {
        let schema = TileSchemaBuilder::web_mercator(0..18)
            .y_direction(VerticalDirection::BottomToTop)
            .build()
            .unwrap();
        assert_eq!(schema.y_direction(), VerticalDirection::BottomToTop);
    }
}
