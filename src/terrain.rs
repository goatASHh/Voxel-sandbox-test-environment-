//! Seeded procedural terrain. Generation is deterministic and independent of rendering.
use crate::{block::BlockId, chunk::{Chunk, CHUNK_SIZE, WORLD_HEIGHT}};

const SEA_LEVEL: i32 = 42;
const WORLD_FLOOR: usize = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome { Meadow, Forest, Desert, Alpine, Tundra, Wetland }

#[derive(Debug, Clone, Copy)]
pub struct ColumnInfo { pub height: i32, pub biome: Biome }

pub struct TerrainGenerator { pub seed: u64 }

impl TerrainGenerator {
    pub fn new(seed: u64) -> Self { Self { seed } }

    pub fn column_info(&self, wx: i32, wz: i32) -> ColumnInfo {
        let continental = self.noise2(wx as f32 * 0.004, wz as f32 * 0.004);
        let hills = self.noise2(wx as f32 * 0.018, wz as f32 * 0.018);
        let detail = self.noise2(wx as f32 * 0.065, wz as f32 * 0.065);
        let climate = self.noise2(wx as f32 * 0.0025 + 71.0, wz as f32 * 0.0025 - 19.0);
        let moisture = self.noise2(wx as f32 * 0.003 + 211.0, wz as f32 * 0.003 + 37.0);
        let raw = 38.0 + continental * 19.0 + hills * 13.0 + detail * 3.0;
        let height = raw.round().clamp(5.0, 112.0) as i32;
        let biome = if height > 82 { Biome::Alpine }
            else if climate < -0.48 { Biome::Tundra }
            else if moisture < -0.38 && climate > 0.30 { Biome::Desert }
            else if moisture > 0.52 && height < 50 { Biome::Wetland }
            else if moisture > 0.12 { Biome::Forest }
            else { Biome::Meadow };
        ColumnInfo { height, biome }
    }

    pub fn generate_chunk(&self, chunk_x: i32, chunk_z: i32) -> Chunk {
        let mut chunk = Chunk::empty(chunk_x, chunk_z);
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let wx = chunk_x * CHUNK_SIZE as i32 + x as i32;
                let wz = chunk_z * CHUNK_SIZE as i32 + z as i32;
                let info = self.column_info(wx, wz);
                let surface = info.height as usize;
                for y in WORLD_FLOOR..=surface.min(WORLD_HEIGHT - 1) {
                    let yi = y as i32;
                    let depth = info.height - yi;
                    let mut block = if y == WORLD_FLOOR { BlockId::Bedrock }
                        else if depth == 0 {
                            match info.biome {
                                Biome::Desert | Biome::Wetland => BlockId::Sand,
                                Biome::Alpine | Biome::Tundra if info.height > SEA_LEVEL + 25 => BlockId::Snow,
                                _ => BlockId::Grass,
                            }
                        } else if depth < 4 {
                            match info.biome {
                                Biome::Desert => BlockId::Sand,
                                Biome::Wetland => if depth == 1 { BlockId::Clay } else { BlockId::Dirt },
                                _ => BlockId::Dirt,
                            }
                        } else if info.biome == Biome::Desert && depth < 9 { BlockId::Sandstone }
                        else { BlockId::Stone };

                    // 3D cave field is sampled only underground; the surface skin stays intact.
                    if depth > 5 && depth < 48 && self.cave_field(wx, yi, wz) > 0.63 {
                        block = BlockId::Air;
                    }
                    chunk.set(x, y, z, block);
                }
                // Fill ocean basins after terrain/cave carving. Water is a separate transparent material.
                if surface < SEA_LEVEL as usize {
                    for y in surface + 1..=SEA_LEVEL as usize {
                        chunk.set(x, y, z, BlockId::Water);
                    }
                }
            }
        }
        // Trees are generated from world coordinates, not chunk-local randomness.
        // Scan a small border so trunks/canopies remain consistent across chunk edges.
        let min_x = chunk_x * CHUNK_SIZE as i32 - 3;
        let min_z = chunk_z * CHUNK_SIZE as i32 - 3;
        let max_x = chunk_x * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 + 2;
        let max_z = chunk_z * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 + 2;
        for tz in min_z..=max_z {
            for tx in min_x..=max_x {
                if let Some((tree_biome, trunk_height)) = self.tree_candidate(tx, tz) {
                    let ground = self.column_info(tx, tz).height;
                    let log = if tree_biome == 1 { BlockId::PineLog } else { BlockId::OakLog };
                    let leaves = if tree_biome == 1 { BlockId::PineLeaves } else { BlockId::OakLeaves };
                    for dy in 1..=trunk_height {
                        self.set_world_block_if_inside(&mut chunk, tx, ground + dy, tz, log);
                    }
                    let crown_y = ground + trunk_height;
                    let crown_radius = if tree_biome == 1 { 2 } else { 2 };
                    for dy in -1..=2 {
                        let radius: i32 = if dy == 2 { 1 } else { crown_radius };
                        for dz in -radius..=radius {
                            for dx in -radius..=radius {
                                // Rounded-ish canopy corners, but retain a chunky silhouette.
                                if dx.abs() == radius && dz.abs() == radius && dy != -1 { continue; }
                                self.set_world_block_if_inside(&mut chunk, tx + dx, crown_y + dy, tz + dz, leaves);
                            }
                        }
                    }
                }
            }
        }
        chunk.generated = true;
        chunk.dirty = false;
        chunk
    }

    fn set_world_block_if_inside(&self, chunk: &mut Chunk, wx: i32, y: i32, wz: i32, block: BlockId) {
        let ox = chunk.x * CHUNK_SIZE as i32;
        let oz = chunk.z * CHUNK_SIZE as i32;
        let lx = wx - ox;
        let lz = wz - oz;
        if lx >= 0 && lz >= 0 && lx < CHUNK_SIZE as i32 && lz < CHUNK_SIZE as i32
            && y >= 0 && y < WORLD_HEIGHT as i32 {
            let existing = chunk.get(lx as usize, y as usize, lz as usize);
            // Never replace terrain or water with foliage; canopy can overlap its own trunk.
            if existing == BlockId::Air || existing == BlockId::OakLog || existing == BlockId::PineLog {
                chunk.set(lx as usize, y as usize, lz as usize, block);
            }
        }
    }

    /// Returns (tree type, trunk height). Placement uses a coarse jittered grid
    /// to avoid clumps while remaining identical when chunks are generated separately.
    fn tree_candidate(&self, x: i32, z: i32) -> Option<(u8, i32)> {
        let cell_x = x.div_euclid(7);
        let cell_z = z.div_euclid(7);
        let jitter_x = (self.hash2(cell_x, cell_z) * 3.0).round() as i32 + 3;
        let jitter_z = (self.hash2(cell_x + 913, cell_z - 271) * 3.0).round() as i32 + 3;
        let tx = cell_x * 7 + jitter_x;
        let tz = cell_z * 7 + jitter_z;
        if tx != x || tz != z { return None; }
        let info = self.column_info(x, z);
        if !matches!(info.biome, Biome::Forest | Biome::Meadow | Biome::Wetland) { return None; }
        let chance = (self.hash2(x ^ 0x1357, z ^ 0x2468) + 1.0) * 0.5;
        let threshold = match info.biome { Biome::Forest => 0.35, Biome::Wetland => 0.90, _ => 0.82 };
        if chance < threshold { return None; }
        let pine = info.height > 67 || self.hash2(x + 99, z - 51) > 0.58;
        let height = 4 + (((self.hash2(x - 44, z + 88) + 1.0) * 1.5).floor() as i32);
        Some((if pine { 1 } else { 0 }, height))
    }

    fn cave_field(&self, x: i32, y: i32, z: i32) -> f32 {
        let a = self.noise3(x as f32 * 0.045, y as f32 * 0.055, z as f32 * 0.045);
        let b = self.noise3(x as f32 * 0.09 + 31.0, y as f32 * 0.08 - 17.0, z as f32 * 0.09 + 9.0);
        a * 0.68 + b * 0.32
    }

    fn noise2(&self, x: f32, z: f32) -> f32 {
        let x0 = x.floor() as i32; let z0 = z.floor() as i32;
        let tx = smooth(x - x.floor()); let tz = smooth(z - z.floor());
        let a = self.hash2(x0, z0); let b = self.hash2(x0 + 1, z0);
        let c = self.hash2(x0, z0 + 1); let d = self.hash2(x0 + 1, z0 + 1);
        lerp(lerp(a, b, tx), lerp(c, d, tx), tz)
    }

    fn noise3(&self, x: f32, y: f32, z: f32) -> f32 {
        let x0=x.floor() as i32; let y0=y.floor() as i32; let z0=z.floor() as i32;
        let tx=smooth(x-x.floor()); let ty=smooth(y-y.floor()); let tz=smooth(z-z.floor());
        let c00=lerp(self.hash3(x0,y0,z0), self.hash3(x0+1,y0,z0), tx);
        let c10=lerp(self.hash3(x0,y0+1,z0), self.hash3(x0+1,y0+1,z0), tx);
        let c01=lerp(self.hash3(x0,y0,z0+1), self.hash3(x0+1,y0,z0+1), tx);
        let c11=lerp(self.hash3(x0,y0+1,z0+1), self.hash3(x0+1,y0+1,z0+1), tx);
        lerp(lerp(c00,c10,ty), lerp(c01,c11,ty), tz)
    }

    fn hash2(&self, x: i32, z: i32) -> f32 {
        let mut n = self.seed ^ (x as u64).wrapping_mul(0x9E3779B185EBCA87) ^ (z as u64).wrapping_mul(0xC2B2AE3D27D4EB4F);
        n = (n ^ (n >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        n = (n ^ (n >> 27)).wrapping_mul(0x94D049BB133111EB);
        n ^= n >> 31;
        ((n >> 40) as f32 / 8_388_607.5) - 1.0
    }

    fn hash3(&self, x: i32, y: i32, z: i32) -> f32 {
        let mix = (y as u64).wrapping_mul(0x165667B19E3779F9);
        let old_seed = self.seed;
        let generator = Self { seed: old_seed ^ mix };
        generator.hash2(x, z)
    }
}

#[inline] fn smooth(t: f32) -> f32 { t * t * (3.0 - 2.0 * t) }
#[inline] fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn same_seed_same_chunk() {
        let g = TerrainGenerator::new(12345);
        let a = g.generate_chunk(-2, 7); let b = g.generate_chunk(-2, 7);
        assert_eq!(a.raw_blocks(), b.raw_blocks());
    }
    #[test] fn different_seed_changes_world() {
        let a = TerrainGenerator::new(1).generate_chunk(0, 0);
        let b = TerrainGenerator::new(2).generate_chunk(0, 0);
        assert_ne!(a.raw_blocks(), b.raw_blocks());
    }
    #[test] fn tree_blocks_are_part_of_generated_world() {
        let g = TerrainGenerator::new(0x504F_434F_F600_0001);
        let mut found_tree = false;
        for cz in -4..=4 {
            for cx in -4..=4 {
                let c = g.generate_chunk(cx, cz);
                if c.raw_blocks().iter().any(|v| matches!(*v, 11 | 12 | 13 | 14)) {
                    found_tree = true;
                    break;
                }
            }
            if found_tree { break; }
        }
        assert!(found_tree, "the test region should contain at least one tree");
    }

    #[test] fn generated_chunk_has_bedrock_and_surface() {
        let c = TerrainGenerator::new(99).generate_chunk(0, 0);
        assert!(c.generated);
        for z in 0..CHUNK_SIZE { for x in 0..CHUNK_SIZE {
            assert_eq!(c.get(x, 0, z), BlockId::Bedrock);
            assert!(c.highest_solid_y(x, z).is_some());
        }}
    }
}
