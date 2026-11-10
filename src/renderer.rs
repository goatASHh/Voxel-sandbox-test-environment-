//! Full 3D voxel meshing. Chunk block data is authoritative; GPU meshes are
//! disposable caches built from exposed voxel faces, including underground caves.
use macroquad::prelude::*;
use std::collections::HashMap;
use crate::{
    block::BlockId,
    chunk::{Chunk, CHUNK_SIZE, WORLD_HEIGHT},
    terrain::{Biome, TerrainGenerator},
};

const SECTION_HEIGHT: usize = 16;
const MAX_MESH_VERTICES: usize = 60_000;

fn rgba_for(block: BlockId, biome: Biome) -> [u8; 4] {
    match block {
        BlockId::Grass => match biome {
            Biome::Forest => [66, 119, 66, 255],
            Biome::Wetland => [91, 132, 89, 255],
            Biome::Tundra => [133, 151, 139, 255],
            _ => [104, 158, 83, 255],
        },
        BlockId::Sand => [194, 177, 119, 255],
        BlockId::Snow => [218, 229, 235, 255],
        BlockId::Stone => [111, 115, 119, 255],
        BlockId::Dirt => [116, 82, 59, 255],
        BlockId::Sandstone => [176, 145, 98, 255],
        BlockId::Gravel => [126, 125, 119, 255],
        BlockId::Clay => [131, 112, 104, 255],
        BlockId::OakLog => [112, 76, 43, 255],
        BlockId::OakLeaves => [62, 133, 62, 255],
        BlockId::PineLog => [91, 66, 43, 255],
        BlockId::PineLeaves => [43, 105, 76, 255],
        BlockId::Water => [54, 119, 174, 150],
        BlockId::Bedrock => [55, 58, 62, 255],
        BlockId::Air => [0, 0, 0, 0],
    }
}

fn shade(c: [u8; 4], factor: f32) -> [u8; 4] {
    [
        (c[0] as f32 * factor) as u8,
        (c[1] as f32 * factor) as u8,
        (c[2] as f32 * factor) as u8,
        c[3],
    ]
}

fn vertex(position: Vec3, color: [u8; 4]) -> Vertex {
    Vertex { position, uv: vec2(0.0, 0.0), color, normal: vec4(0.0, 0.0, 0.0, 0.0) }
}

fn append_quad(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    points: [Vec3; 4],
    color: [u8; 4],
) {
    let base = vertices.len() as u16;
    for point in points {
        vertices.push(vertex(point, color));
    }
    indices.extend_from_slice(&[
        base, base + 1, base + 2,
        base, base + 2, base + 3,
    ]);
}

fn world_block(chunk_map: &HashMap<(i32, i32), &Chunk>, wx: i32, wy: i32, wz: i32) -> BlockId {
    if wy < 0 || wy >= WORLD_HEIGHT as i32 {
        return BlockId::Air;
    }
    let cx = wx.div_euclid(CHUNK_SIZE as i32);
    let cz = wz.div_euclid(CHUNK_SIZE as i32);
    let lx = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
    let lz = wz.rem_euclid(CHUNK_SIZE as i32) as usize;
    chunk_map
        .get(&(cx, cz))
        .map(|chunk| chunk.get(lx, wy as usize, lz))
        .unwrap_or(BlockId::Air)
}

#[inline]
fn face_is_visible(block: BlockId, neighbour: BlockId) -> bool {
    if block == BlockId::Air || neighbour == block {
        return false;
    }
    if neighbour == BlockId::Air || neighbour.is_transparent() {
        return true;
    }
    // A transparent block does not expose a face against an opaque neighbour:
    // that face is hidden inside the opaque block.
    false
}

fn push_visible_face(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    points: [Vec3; 4],
    color: [u8; 4],
) {
    // Keep every Mesh index safely inside u16, even for pathological checkerboard
    // voxel arrangements with many exposed faces.
    if vertices.len() + 4 > MAX_MESH_VERTICES {
        return;
    }
    append_quad(vertices, indices, points, color);
}

/// Builds a list of bounded-size meshes for one chunk. The chunk is split into
/// 16-block-high vertical sections; unusually face-dense sections can produce
/// more than one mesh to stay within Macroquad's u16 index format.
///
/// Unlike the previous height-surface renderer, this visits every voxel and
/// emits all exposed faces. Underground walls, cave ceilings/floors, trees,
/// player edits and arbitrary structures therefore share the same renderer.
pub fn build_chunk_meshes(chunk: &Chunk, chunks: &[Chunk], generator: &TerrainGenerator) -> Vec<Mesh> {
    let mut result = Vec::new();
    let chunk_map: HashMap<(i32, i32), &Chunk> =
        chunks.iter().map(|c| ((c.x, c.z), c)).collect();
    let ox = (chunk.x * CHUNK_SIZE as i32) as f32;
    let oz = (chunk.z * CHUNK_SIZE as i32) as f32;

    // Biome is a column property. Cache it once per x/z column rather than
    // recalculating terrain classification for every solid voxel.
    let mut biome_columns = [[Biome::Meadow; CHUNK_SIZE]; CHUNK_SIZE];
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let wx = chunk.x * CHUNK_SIZE as i32 + x as i32;
            let wz = chunk.z * CHUNK_SIZE as i32 + z as i32;
            biome_columns[z][x] = generator.column_info(wx, wz).biome;
        }
    }

    for section_start in (0..WORLD_HEIGHT).step_by(SECTION_HEIGHT) {
        let section_end = (section_start + SECTION_HEIGHT).min(WORLD_HEIGHT);
        let mut vertices = Vec::with_capacity(12_000);
        let mut indices = Vec::with_capacity(18_000);

        for y in section_start..section_end {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = chunk.get(x, y, z);
                    if block == BlockId::Air {
                        continue;
                    }

                    let wx = chunk.x * CHUNK_SIZE as i32 + x as i32;
                    let wy = y as i32;
                    let wz = chunk.z * CHUNK_SIZE as i32 + z as i32;
                    let biome = biome_columns[z][x];
                    let base = rgba_for(block, biome);

                    let x0 = ox + x as f32;
                    let x1 = x0 + 1.0;
                    let y0 = y as f32;
                    let y1 = y0 + 1.0;
                    let z0 = oz + z as f32;
                    let z1 = z0 + 1.0;

                    let faces = [
                        // -X
                        (
                            world_block(&chunk_map, wx - 1, wy, wz),
                            [vec3(x0,y0,z0), vec3(x0,y0,z1), vec3(x0,y1,z1), vec3(x0,y1,z0)],
                            0.78,
                        ),
                        // +X
                        (
                            world_block(&chunk_map, wx + 1, wy, wz),
                            [vec3(x1,y0,z1), vec3(x1,y0,z0), vec3(x1,y1,z0), vec3(x1,y1,z1)],
                            0.78,
                        ),
                        // -Y
                        (
                            world_block(&chunk_map, wx, wy - 1, wz),
                            [vec3(x0,y0,z1), vec3(x0,y0,z0), vec3(x1,y0,z0), vec3(x1,y0,z1)],
                            0.58,
                        ),
                        // +Y
                        (
                            world_block(&chunk_map, wx, wy + 1, wz),
                            [vec3(x0,y1,z0), vec3(x0,y1,z1), vec3(x1,y1,z1), vec3(x1,y1,z0)],
                            1.00,
                        ),
                        // -Z
                        (
                            world_block(&chunk_map, wx, wy, wz - 1),
                            [vec3(x1,y0,z0), vec3(x0,y0,z0), vec3(x0,y1,z0), vec3(x1,y1,z0)],
                            0.68,
                        ),
                        // +Z
                        (
                            world_block(&chunk_map, wx, wy, wz + 1),
                            [vec3(x0,y0,z1), vec3(x1,y0,z1), vec3(x1,y1,z1), vec3(x0,y1,z1)],
                            0.88,
                        ),
                    ];

                    for (neighbour, points, brightness) in faces {
                        if face_is_visible(block, neighbour) {
                            if vertices.len() + 4 > MAX_MESH_VERTICES {
                                result.push(Mesh {
                                    vertices: std::mem::take(&mut vertices),
                                    indices: std::mem::take(&mut indices),
                                    texture: None,
                                });
                            }
                            push_visible_face(
                                &mut vertices,
                                &mut indices,
                                points,
                                shade(base, brightness),
                            );
                        }
                    }
                }
            }
        }

        if !indices.is_empty() {
            result.push(Mesh { vertices, indices, texture: None });
        }
    }

    result
}

pub fn draw_chunk_meshes(meshes: &[Mesh]) {
    for mesh in meshes {
        draw_mesh(mesh);
    }
}
