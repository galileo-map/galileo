//! Tile index types.

use serde::{Deserialize, Serialize};

/// Tile index with additional `virtual_x` index that can be used to wrap tiles
/// over 180 longitude line.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct WrappingTileIndex {
    /// Z index.
    pub z: u32,
    /// X index.
    pub x: i32,
    /// Y index.
    pub y: i32,
    /// Virtual wrapping X index.
    pub virtual_x: i32,
}

impl WrappingTileIndex {
    /// Create a new index instance without wrapping.
    pub fn new(x: i32, y: i32, z: u32) -> Self {
        Self {
            x,
            y,
            z,
            virtual_x: x,
        }
    }
}

/// Tile index.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
pub struct TileIndex {
    /// X index.
    pub x: i32,
    /// Y index.
    pub y: i32,
    /// Z index.
    pub z: u32,
}

impl TileIndex {
    /// Create a new index instance.
    pub fn new(x: i32, y: i32, z: u32) -> Self {
        Self { x, y, z }
    }

    /// Converts the tile index into a wrapping tile index by setting `display_x` equal to `x`.
    pub fn into_wrapping(self) -> WrappingTileIndex {
        WrappingTileIndex {
            x: self.x,
            y: self.y,
            z: self.z,
            virtual_x: self.x,
        }
    }
}

impl From<WrappingTileIndex> for TileIndex {
    fn from(value: WrappingTileIndex) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
