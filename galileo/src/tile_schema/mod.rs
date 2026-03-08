//! [`TileSchema`] is used by tile layers to calculate [tile indices](`TileIndex`) needed
//! for a given [`MapView`](crate::view::MapView).

mod builder;
mod schema;
mod tile_index;

pub use builder::{TileSchemaBuilder, TileSchemaError};
pub use schema::{TileSchema, VerticalDirection};
pub use tile_index::{TileIndex, WrappingTileIndex};
