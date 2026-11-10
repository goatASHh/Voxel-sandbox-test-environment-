use crate::block::BlockId;

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_HEIGHT: usize = 128;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * WORLD_HEIGHT;

/// CPU-side voxel data. GPU meshes are deliberately stored elsewhere: a resident
/// chunk does not have to be meshed or rendered.
#[derive(Clone)]
pub struct Chunk {
    pub x: i32,
    pub z: i32,
    blocks: Vec<u16>,
    pub dirty: bool,
    pub generated: bool,
}

impl Chunk {
    pub fn empty(x: i32, z: i32) -> Self {
        Self { x, z, blocks: vec![BlockId::Air as u16; CHUNK_VOLUME], dirty: false, generated: false }
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        (y * CHUNK_SIZE + z) * CHUNK_SIZE + x
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        if x >= CHUNK_SIZE || z >= CHUNK_SIZE || y >= WORLD_HEIGHT { return BlockId::Air; }
        BlockId::from_raw(self.blocks[Self::index(x, y, z)])
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) -> bool {
        if x >= CHUNK_SIZE || z >= CHUNK_SIZE || y >= WORLD_HEIGHT { return false; }
        let index = Self::index(x, y, z);
        let value = block as u16;
        if self.blocks[index] != value {
            self.blocks[index] = value;
            self.dirty = true;
        }
        true
    }

    pub fn raw_blocks(&self) -> &[u16] { &self.blocks }

    pub fn highest_solid_y(&self, x: usize, z: usize) -> Option<usize> {
        (0..WORLD_HEIGHT).rev().find(|&y| self.get(x, y, z).is_solid())
    }

    /// Highest terrain block, excluding tree trunks and foliage. This is used
    /// by the first editing pass, which targets the visible ground surface.
    pub fn highest_terrain_y(&self, x: usize, z: usize) -> Option<usize> {
        (0..WORLD_HEIGHT).rev().find(|&y| {
            !matches!(self.get(x, y, z), BlockId::Air | BlockId::Water |
                BlockId::OakLog | BlockId::OakLeaves | BlockId::PineLog | BlockId::PineLeaves)
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_surface_ignores_tree_blocks() {
        let mut chunk = Chunk::empty(0, 0);
        chunk.set(2, 4, 3, BlockId::Stone);
        chunk.set(2, 5, 3, BlockId::OakLog);
        chunk.set(2, 6, 3, BlockId::OakLeaves);
        assert_eq!(chunk.highest_terrain_y(2, 3), Some(4));
    }
}
