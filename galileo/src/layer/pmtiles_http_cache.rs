//! An HTTP range-request backend with persistent on-disk cache for PMTiles.
//!
//! This module provides [`CachedHttpBackend`], an implementation of the
//! [`pmtiles::AsyncBackend`] trait that fetches byte ranges over HTTP and stores
//! them on disk for reuse. It also includes a small helper to prefetch tiles.

use std::path::{Path, PathBuf};

use bytes::Bytes;
use pmtiles::{reqwest::header::{HeaderValue, RANGE}, reqwest::{Client, Method, Request, StatusCode, Url}};
use pmtiles::{AsyncBackend, AsyncPmTilesReader, DirectoryCache, PmtError, PmtResult};
use sled::{Db, Tree};
use tokio::sync::Semaphore;
use std::sync::Arc;

use crate::tile_schema::{TileIndex, TileSchema, VerticalDirection};
use galileo_types::cartesian::CartesianPoint2d;

/// A persistent file-backed HTTP range cache for PMTiles AsyncBackend.
/// Stores each requested range as a file named "{offset}-{length}" under a folder derived from the URL.
#[derive(Clone)]
pub struct CachedHttpBackend {
    client: Client,
    url: Url,
    url_folder: PathBuf,
    /// Sled tree used to index cached byte ranges for this URL.
    tree: Tree,
    /// Keep the sled database handle alive for the lifetime of the backend.
    ///
    /// The handle is intentionally not read after initialization, but it must be
    /// held so that the opened tree remains valid. See sled documentation for
    /// details.
    #[allow(dead_code)]
    db: Db,
}

impl CachedHttpBackend {
    /// Creates a new cached HTTP backend for the given URL and cache root folder.
    ///
    /// The cache stores each requested byte range as a file under a URL-specific
    /// directory inside `cache_root`. A small sled index tracks which ranges are
    /// present to minimize network requests.
    pub fn try_from(client: Client, url: impl pmtiles::reqwest::IntoUrl, cache_root: impl AsRef<Path>) -> PmtResult<Self> {
        let url = url.into_url()?;
        let cache_root = cache_root.as_ref().to_path_buf();
        let url_folder = cache_root.join(sanitize_for_fs(url.as_str()));
        std::fs::create_dir_all(&url_folder)?;
        let db = sled::open(cache_root.join("index"))
            .map_err(|e| PmtError::Reading(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        let tree = db
            .open_tree(sanitize_for_fs(url.as_str()))
            .map_err(|e| PmtError::Reading(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        Ok(Self { client, url, url_folder, tree, db })
    }

    fn range_path(&self, offset: usize, length: usize) -> PathBuf {
        self.url_folder.join(format!("{}-{}", offset, length))
    }

    fn get_next_range_at_or_after(&self, at: usize) -> Option<(usize, usize)> {
        let key = encode_u64(at as u64);
        if let Some(Ok((k, v))) = self.tree.range(..=key).rev().next() {
            let (start, len) = (decode_u64(&k) as usize, decode_len(&v) as usize);
            if start + len > at {
                return Some((start, len));
            }
        }
        if let Some(Ok((k, v))) = self.tree.range(key..).next() {
            let (start, len) = (decode_u64(&k) as usize, decode_len(&v) as usize);
            return Some((start, len));
        }
        None
    }

    fn record_range(&self, offset: usize, length: usize) -> PmtResult<()> {
        self.tree
            .insert(encode_u64(offset as u64), encode_len(length as u64))
            .map_err(|e| PmtError::Reading(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        Ok(())
    }

    async fn fetch_and_store(&self, offset: usize, length: usize) -> PmtResult<Bytes> {
        let end = offset + length - 1;
        let range = format!("bytes={offset}-{end}");
        let range = HeaderValue::try_from(range)?;

        let mut req = Request::new(Method::GET, self.url.clone());
        req.headers_mut().insert(RANGE, range);

        let response = self.client.execute(req).await?.error_for_status()?;
        if response.status() != StatusCode::PARTIAL_CONTENT {
            return Err(PmtError::RangeRequestsUnsupported);
        }
        let response_bytes = response.bytes().await?;
        if response_bytes.len() > length {
            return Err(PmtError::ResponseBodyTooLong(response_bytes.len(), length));
        }
        let path = self.range_path(offset, length);
        if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
        std::fs::write(&path, &response_bytes)?;
        self.record_range(offset, length)?;
        Ok(response_bytes)
    }
}

impl AsyncBackend for CachedHttpBackend {
    async fn read(&self, offset: usize, length: usize) -> PmtResult<Bytes> {
        let end = offset + length;
        let mut cursor = offset;
        let mut out = vec![0u8; length];

        while cursor < end {
            if let Some((start, len)) = self.get_next_range_at_or_after(cursor) {
                let start_end = start + len;
                if start > cursor {
                    let gap_len = (start - cursor).min(end - cursor);
                    let fetched = self.fetch_and_store(cursor, gap_len).await?;
                    out[(cursor - offset)..(cursor - offset + gap_len)].copy_from_slice(&fetched);
                    cursor += gap_len;
                    continue;
                }
                let file_path = self.range_path(start, len);
                match std::fs::read(&file_path) {
                    Ok(bytes) => {
                        let take = (start_end.min(end)) - cursor;
                        let rel = cursor - start;
                        out[(cursor - offset)..(cursor - offset + take)]
                            .copy_from_slice(&bytes[rel..rel + take]);
                        cursor += take;
                    }
                    Err(_) => {
                        // Stale index entry: remove and fetch missing span
                        let _ = self.tree.remove(encode_u64(start as u64));
                        let take = (start_end.min(end)) - cursor;
                        let fetched = self.fetch_and_store(cursor, take).await?;
                        out[(cursor - offset)..(cursor - offset + take)].copy_from_slice(&fetched);
                        cursor += take;
                    }
                }
            } else {
                let gap_len = end - cursor;
                let fetched = self.fetch_and_store(cursor, gap_len).await?;
                out[(cursor - offset)..(cursor - offset + gap_len)].copy_from_slice(&fetched);
                cursor = end;
            }
        }

        Ok(Bytes::from(out))
    }
}

fn sanitize_for_fs(input: &str) -> String {
    input
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn encode_u64(v: u64) -> [u8; 8] { v.to_be_bytes() }
fn decode_u64(b: &[u8]) -> u64 { let mut arr = [0u8; 8]; arr.copy_from_slice(&b[..8]); u64::from_be_bytes(arr) }
fn encode_len(v: u64) -> Vec<u8> { v.to_be_bytes().to_vec() }
fn decode_len(b: &[u8]) -> u64 { decode_u64(b) }

/// Configuration for background tile prefetch.
pub struct PrefetchConfig {
    /// Zoom levels to prefetch (e.g. 0..=5).
    pub zoom_levels: Vec<u8>,
    /// Optional bounding box to limit prefetch area; if `None` the full schema bounds are used.
    pub bbox: Option<galileo_types::cartesian::Rect>,
    /// Maximum number of concurrent tile requests.
    pub concurrency: usize,
}

impl Default for PrefetchConfig {
    fn default() -> Self {
        // Prefetch zoom levels 0..=5 by default
        Self { zoom_levels: vec![0, 1, 2, 3, 4, 5], bbox: None, concurrency: 16 }
    }
}

/// Spawns background prefetch of specified zoom levels over optional bbox using a fresh reader
/// backed by [`CachedHttpBackend`]. The app remains responsive.
pub async fn spawn_prefetch_for_url<C: DirectoryCache + Send + Sync + 'static>(
    cache_root: impl AsRef<Path>,
    client: Client,
    url: impl pmtiles::reqwest::IntoUrl,
    schema: TileSchema,
    config: PrefetchConfig,
) -> PmtResult<tokio::task::JoinHandle<()>> {
    let backend = CachedHttpBackend::try_from(client, url, cache_root)?;
    let reader = AsyncPmTilesReader::try_from_cached_source(backend, pmtiles::NoCache).await?;
    Ok(tokio::spawn(async move {
        prefetch_tiles(reader, schema, config).await;
    }))
}

async fn prefetch_tiles<B, C>(
    reader: AsyncPmTilesReader<B, C>,
    schema: TileSchema,
    config: PrefetchConfig,
) where
    B: AsyncBackend + Send + Sync + 'static,
    C: DirectoryCache + Send + Sync + 'static,
{
    let bbox = config.bbox.unwrap_or(schema.bounds);
    let sem = Arc::new(Semaphore::new(config.concurrency.max(1)));
    let reader = Arc::new(reader);

    for &z in &config.zoom_levels {
        if let Some(resolution) = schema.lod_resolution(z as u32) {
            let tiles = tiles_for_bbox_and_res(&schema, bbox, resolution, z as u32);
            for idx in tiles {
                let sem_cloned = Arc::clone(&sem);
                let reader_ref = Arc::clone(&reader);
                tokio::spawn(async move {
                    let _permit = sem_cloned.acquire_owned().await.unwrap();
                    if let Some(coord) = pmtiles::TileCoord::new(idx.z as u8, idx.x as u32, idx.y as u32) {
                        let _ = reader_ref.get_tile(coord).await; // Ignore errors; best-effort prefetch
                    }
                });
            }
        }
    }
}

fn tiles_for_bbox_and_res(schema: &TileSchema, bbox: galileo_types::cartesian::Rect, resolution: f64, z: u32) -> Vec<TileIndex> {
    let tile_w = resolution * schema.tile_width() as f64;
    let tile_h = resolution * schema.tile_height() as f64;

    let x_adj = |x: f64| x - schema.origin.x();
    let y_adj = |y: f64| match schema.y_direction { VerticalDirection::TopToBottom => schema.origin.y() - y, VerticalDirection::BottomToTop => y - schema.origin.y() };

    let x_min = (x_adj(bbox.x_min()) / tile_w).floor() as i32;
    let x_max_adj = x_adj(bbox.x_max());
    let x_add_one = if (x_max_adj % tile_w) < 0.001 { -1 } else { 0 };
    let x_max = (x_max_adj / tile_w) as i32 + x_add_one;

    let (top, bottom) = if schema.y_direction == VerticalDirection::TopToBottom { (bbox.y_min(), bbox.y_max()) } else { (bbox.y_max(), bbox.y_min()) };
    let y_min = (y_adj(bottom) / tile_h) as i32;
    let y_max_adj = y_adj(top);
    let y_add_one = if (y_max_adj % tile_h) < 0.001 { -1 } else { 0 };
    let y_max = (y_max_adj / tile_h) as i32 + y_add_one;

    let mut out = Vec::new();
    for x in x_min..=x_max { for y in y_min..=y_max { out.push(TileIndex::new(x, y, z)); } }
    out
}
