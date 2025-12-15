//! Builder for [`TileSchema`].

use core::f64;
use std::sync::Arc;

use galileo_types::cartesian::{Point2, Rect};

use super::schema::{TileSchema, VerticalDirection};

/// Builder for [`TileSchema`].
///
/// The builder validates all the input parameters and guarantees that the created schema is valid.
#[derive(Debug)]
pub struct TileSchemaBuilder {
    origin: Point2,
    bounds: Rect,
    lods: Lods,
    tile_width: u32,
    tile_height: u32,
    y_direction: VerticalDirection,
}

#[derive(Debug)]
enum Lods {
    Logarithmic(Vec<u32>),
}

/// Errors that can occur during building a [`TileSchema`].
#[derive(Debug, thiserror::Error)]
pub enum TileSchemaError {
    /// No zoom levels provided
    #[error("No zoom levels provided")]
    NoZLevelsProvided,

    /// Invalid tile size
    #[error("Invalid tile size: {width}x{height}")]
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
    #[error("Resolution too small at z-level {z_level}: {resolution}")]
    ResolutionTooSmall {
        /// Z-level where resolution is too small
        z_level: u32,
        /// The resolution value that is too small
        resolution: f64,
    },
}

impl TileSchemaBuilder {
    /// Create a new builder with default parameters.
    pub fn build(self) -> Result<TileSchema, TileSchemaError> {
        let lods = match self.lods {
            Lods::Logarithmic(z_levels) => {
                if z_levels.is_empty() {
                    return Err(TileSchemaError::NoZLevelsProvided);
                }

                let top_resolution = self.bounds.width() / self.tile_width as f64;

                // Resolution is bound by the maximum tile index that can be represented
                let min_resolution = f64::min(
                    self.bounds.width() / self.tile_width as f64 / u64::MAX as f64,
                    self.bounds.height() / self.tile_height as f64 / u64::MAX as f64,
                );

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
        };

        if self.tile_width == 0 || self.tile_height == 0 {
            return Err(TileSchemaError::InvalidTileSize {
                width: self.tile_width,
                height: self.tile_height,
            });
        }

        Ok(TileSchema {
            origin: self.origin,
            bounds: self.bounds,
            lods: Arc::new(lods),
            tile_width: self.tile_width,
            tile_height: self.tile_height,
            y_direction: self.y_direction,
        })
    }

    /// Standard Web Mercator based tile scheme (used, for example, by OSM and Google maps).
    pub fn web_mercator(z_levels: impl IntoIterator<Item = u32>) -> Self {
        const TILE_SIZE: u32 = 256;

        Self::web_mercator_base()
            .with_logarithmic_z_levels(z_levels)
            .with_rect_tile_size(TILE_SIZE)
    }

    fn web_mercator_base() -> Self {
        const MAX_COORD_VALUE: f64 = 20037508.342787;

        Self {
            origin: Point2::new(-MAX_COORD_VALUE, MAX_COORD_VALUE),
            bounds: Rect::new(
                -MAX_COORD_VALUE,
                -MAX_COORD_VALUE,
                MAX_COORD_VALUE,
                MAX_COORD_VALUE,
            ),
            lods: Lods::Logarithmic(Vec::new()),
            tile_width: 0,
            tile_height: 0,
            y_direction: VerticalDirection::TopToBottom,
        }
    }

    /// Set both tile width and height to `tile_size`.
    pub fn with_rect_tile_size(mut self, tile_size: u32) -> Self {
        self.tile_width = tile_size;
        self.tile_height = tile_size;

        self
    }

    fn with_logarithmic_z_levels(mut self, z_levels: impl IntoIterator<Item = u32>) -> Self {
        self.lods = Lods::Logarithmic(z_levels.into_iter().collect());

        self
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use super::*;
    use crate::tile_schema::VerticalDirection;

    #[test]
    fn schema_builder_normal_web_mercator() {
        let schema = TileSchemaBuilder::web_mercator(0..=20).build().unwrap();
        assert_eq!(schema.lods.len(), 21);

        assert_abs_diff_eq!(schema.lods[0], 156543.03392802345);

        for z in 1..=20 {
            let expected = 156543.03392802345 / 2f64.powi(z);
            assert_abs_diff_eq!(schema.lods[z as usize], expected);
        }

        assert_eq!(schema.tile_width, 256);
        assert_eq!(schema.tile_height, 256);
        assert_eq!(
            schema.origin,
            Point2::new(-20037508.342787, 20037508.342787)
        );
        assert_eq!(
            schema.bounds,
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

        assert_abs_diff_eq!(schema.lods[5], 156543.03392802345 / 2f64.powi(5));
        assert_abs_diff_eq!(schema.lods[10], 156543.03392802345 / 2f64.powi(10));
    }

    #[test]
    fn zero_tile_size() {
        let result = TileSchemaBuilder::web_mercator(0..=20)
            .with_rect_tile_size(0)
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

        assert_abs_diff_eq!(schema.lods[1], 156543.03392802345 / 2f64.powi(1));
        assert_abs_diff_eq!(schema.lods[2], 156543.03392802345 / 2f64.powi(2));
        assert_abs_diff_eq!(schema.lods[3], 156543.03392802345 / 2f64.powi(3));

        let expected_level_3 = 156543.03392802345 / 2f64.powi(3);
        assert_abs_diff_eq!(schema.lods[4], expected_level_3);

        assert_abs_diff_eq!(schema.lods[5], 156543.03392802345 / 2f64.powi(5));
    }

    #[test]
    fn skipped_multiple_middle_z_levels_use_previous_value() {
        let schema = TileSchemaBuilder::web_mercator([0, 1, 5]).build().unwrap();
        assert_eq!(schema.lods.len(), 6);

        assert_abs_diff_eq!(schema.lods[0], 156543.03392802345 / 2f64.powi(0));
        assert_abs_diff_eq!(schema.lods[1], 156543.03392802345 / 2f64.powi(1));

        let expected_level_1 = 156543.03392802345 / 2f64.powi(1);
        for z in 2..5 {
            assert_abs_diff_eq!(schema.lods[z], expected_level_1);
        }

        assert_abs_diff_eq!(schema.lods[5], 156543.03392802345 / 2f64.powi(5));
    }

    #[test]
    fn resolution_at_boundary_of_precision() {
        let result = TileSchemaBuilder::web_mercator(0..=64).build();
        assert!(
            result.is_ok(),
            "Expected z=0..=64 to be valid, got {:?}",
            result
        );

        let result = TileSchemaBuilder::web_mercator(0..=65).build();

        assert!(
            matches!(
                result,
                Err(TileSchemaError::ResolutionTooSmall { z_level: 65, .. })
            ),
            "Expected ResolutionTooSmall error, got {:?}",
            result
        );
    }
}
