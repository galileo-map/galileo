#![allow(dead_code)] // This module is only used in wasm32 target and in unit tests

use std::fmt::Display;
use std::sync::atomic::{AtomicU64, Ordering};

use bytes::Bytes;
use galileo_mvt::MvtTile;
use serde::{Deserialize, Serialize};
use serde_with::json::JsonString;
use serde_with::serde_as;

use crate::TileSchema;
use crate::layer::vector_tile_layer::style::VectorTileStyle;
use crate::layer::vector_tile_layer::tile_provider::processor::TileProcessingError;
use crate::render::render_bundle::RenderBundle;
use crate::tile_schema::TileIndex;

const INVALID_ID: u64 = 0;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct WebWorkerRequestId(u64);

impl Display for WebWorkerRequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl WebWorkerRequestId {
    pub(crate) fn next() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub(crate) fn empty() -> Self {
        Self(INVALID_ID)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct WebWorkerRequest {
    pub(crate) request_id: WebWorkerRequestId,
    pub(crate) payload: WebWorkerRequestPayload,
}

#[serde_as]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum WebWorkerRequestPayload {
    ProcessVtTile {
        tile: MvtTile,
        index: TileIndex,
        #[serde_as(as = "JsonString")]
        style: VectorTileStyle,
        tile_schema: TileSchema,
    },
    LoadFont {
        font_data: Bytes,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct WebWorkerResponse {
    pub(crate) request_id: WebWorkerRequestId,
    pub(crate) payload: WebWorkerResponsePayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum WebWorkerResponsePayload {
    Ready,
    ProcessVtTile {
        result: Box<Result<RenderBundle, TileProcessingError>>,
    },
    LoadFont,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum WebWorkerError {}

impl TryFrom<Result<WebWorkerResponsePayload, WebWorkerError>> for RenderBundle {
    type Error = TileProcessingError;

    fn try_from(
        value: Result<WebWorkerResponsePayload, WebWorkerError>,
    ) -> Result<Self, Self::Error> {
        match value {
            Ok(WebWorkerResponsePayload::ProcessVtTile { result }) => *result,
            _ => {
                log::error!("Unexpected response type for tile processing request: {value:?}");
                Err(TileProcessingError::Internal)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;
    use crate::tile_schema::TileSchemaBuilder;

    #[test]
    fn serialize_request_payload() {
        let style = VectorTileStyle {
            rules: vec![],
            background: Color::BLACK.into(),
        };

        let payload = WebWorkerRequestPayload::ProcessVtTile {
            tile: MvtTile { layers: vec![] },
            index: TileIndex::new(0, 1, 2),
            style,
            tile_schema: TileSchemaBuilder::web_mercator(0..5).build().unwrap(),
        };

        let serialized =
            bincode::serde::encode_to_vec(&payload, bincode::config::standard()).unwrap();
        let (deserialized, _) =
            bincode::serde::decode_from_slice(&serialized, bincode::config::standard()).unwrap();

        assert_eq!(payload, deserialized);
    }
}
