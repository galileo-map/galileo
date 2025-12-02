//! Builder for [`TileSchema`].

use core::f64;

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

                let max_z_level = *z_levels.iter().max().unwrap_or(&0);
                let mut lods = vec![f64::NAN; max_z_level as usize + 1];

                for z in z_levels {
                    let resolution = top_resolution / f64::powi(2.0, z as i32);
                    lods[z as usize] = resolution;
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
            lods,
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
}
