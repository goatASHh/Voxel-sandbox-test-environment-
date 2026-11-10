//! Versioned, checksummed manual world persistence.
//! Writes are synchronous by design; background saving remains opt-in/disabled.
use crate::chunk::{Chunk, CHUNK_VOLUME, CHUNK_SIZE, WORLD_HEIGHT};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}};

pub const SAVE_FORMAT_VERSION: u32 = 1;
const MAX_SAVED_CHUNKS: usize = 4096;

#[derive(Debug, Serialize, Deserialize)]
struct SaveFile {
    format_version: u32,
    world_seed: u64,
    chunks: Vec<SavedChunk>,
    checksum: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedChunk { x: i32, z: i32, blocks: Vec<u16> }

#[derive(Debug)]
pub enum SaveError {
    Io(io::Error),
    Json(serde_json::Error),
    UnsupportedVersion(u32),
    InvalidChunkData,
    ChecksumMismatch,
}
impl From<io::Error> for SaveError { fn from(e: io::Error) -> Self { Self::Io(e) } }
impl From<serde_json::Error> for SaveError { fn from(e: serde_json::Error) -> Self { Self::Json(e) } }

fn checksum(seed: u64, chunks: &[SavedChunk]) -> u64 {
    // Stable FNV-1a over metadata and little-endian block IDs.
    let mut h = 0xcbf29ce484222325u64;
    let mut feed = |byte: u8| { h ^= byte as u64; h = h.wrapping_mul(0x100000001b3); };
    for b in seed.to_le_bytes() { feed(b); }
    for c in chunks {
        for b in c.x.to_le_bytes().into_iter().chain(c.z.to_le_bytes()) { feed(b); }
        for id in &c.blocks { for b in id.to_le_bytes() { feed(b); } }
    }
    drop(feed);
    h
}

fn validate(file: &SaveFile) -> Result<(), SaveError> {
    if file.format_version != SAVE_FORMAT_VERSION { return Err(SaveError::UnsupportedVersion(file.format_version)); }
    if file.chunks.len() > MAX_SAVED_CHUNKS { return Err(SaveError::InvalidChunkData); }
    for c in &file.chunks {
        if c.blocks.len() != CHUNK_VOLUME || c.blocks.iter().any(|id| *id > 14) {
            return Err(SaveError::InvalidChunkData);
        }
    }
    if checksum(file.world_seed, &file.chunks) != file.checksum { return Err(SaveError::ChecksumMismatch); }
    Ok(())
}

/// Save to `path`; previous valid bytes are retained in `path.bak`.
pub fn save_world(path: impl AsRef<Path>, seed: u64, chunks: &[Chunk]) -> Result<(), SaveError> {
    let path = path.as_ref();
    if chunks.len() > MAX_SAVED_CHUNKS { return Err(SaveError::InvalidChunkData); }
    let saved: Vec<SavedChunk> = chunks.iter().map(|c| SavedChunk {
        x: c.x, z: c.z, blocks: c.raw_blocks().to_vec(),
    }).collect();
    let file = SaveFile { format_version: SAVE_FORMAT_VERSION, world_seed: seed,
        checksum: checksum(seed, &saved), chunks: saved };
    validate(&file)?;
    let bytes = serde_json::to_vec(&file)?;
    let temp = suffix_path(path, ".tmp");
    let backup = suffix_path(path, ".bak");
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() { fs::create_dir_all(parent)?; }
    }
    fs::write(&temp, bytes)?;
    if path.exists() { fs::copy(path, &backup)?; }
    if let Err(e) = fs::rename(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(SaveError::Io(e));
    }
    Ok(())
}

/// Load primary save; if it is invalid or missing, try its backup.
pub fn load_world(path: impl AsRef<Path>) -> Result<(u64, Vec<Chunk>), SaveError> {
    let path = path.as_ref();
    match load_one(path) {
        Ok(world) => Ok(world),
        Err(primary_error) => match load_one(&suffix_path(path, ".bak")) {
            Ok(world) => Ok(world),
            Err(_) => Err(primary_error),
        },
    }
}

fn load_one(path: &Path) -> Result<(u64, Vec<Chunk>), SaveError> {
    let bytes = fs::read(path)?;
    let file: SaveFile = serde_json::from_slice(&bytes)?;
    validate(&file)?;
    let chunks = file.chunks.into_iter().map(|saved| {
        let mut chunk = Chunk::empty(saved.x, saved.z);
        for y in 0..WORLD_HEIGHT { for z in 0..CHUNK_SIZE { for x in 0..CHUNK_SIZE {
            let i = (y * CHUNK_SIZE + z) * CHUNK_SIZE + x;
            chunk.set(x, y, z, crate::block::BlockId::from_raw(saved.blocks[i]));
        }}}
        chunk.dirty = false;
        chunk.generated = true;
        chunk
    }).collect();
    Ok((file.world_seed, chunks))
}

fn suffix_path(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string(); name.push(suffix); PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_preserves_edits_and_seed() {
        let path = std::env::temp_dir().join(format!("poco-save-{}.json", std::process::id()));
        let mut c = Chunk::empty(-2, 3); c.set(4, 17, 9, crate::block::BlockId::Stone);
        save_world(&path, 0x1234, &[c]).unwrap();
        let (seed, chunks) = load_world(&path).unwrap();
        assert_eq!(seed, 0x1234);
        assert_eq!(chunks[0].get(4, 17, 9), crate::block::BlockId::Stone);
        let _ = fs::remove_file(&path); let _ = fs::remove_file(suffix_path(&path, ".bak"));
    }
}
