//! Grid traversal for precise voxel selection.
//!
//! The algorithm visits voxel cells in ray order (Amanatides-Woo DDA), rather
//! than sampling the ray at fixed intervals. This avoids skipped thin targets
//! and gives the adjacent empty cell needed for face-based block placement.
use crate::block::BlockId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelHit {
    /// World-space integer coordinates of the block that was hit.
    pub cell: [i32; 3],
    /// The cell immediately before the hit, on the ray's approach side.
    pub previous: [i32; 3],
    /// Outward normal of the face entered by the ray (one axis is +/-1).
    pub face_normal: [i32; 3],
    pub block: BlockId,
}

#[inline]
fn initial_axis(origin: f32, direction: f32, cell: i32) -> (i32, f32, f32) {
    if direction > 0.0 {
        let next_boundary = cell as f32 + 1.0;
        (1, (next_boundary - origin) / direction, 1.0 / direction)
    } else if direction < 0.0 {
        let next_boundary = cell as f32;
        (-1, (origin - next_boundary) / -direction, 1.0 / -direction)
    } else {
        (0, f32::INFINITY, f32::INFINITY)
    }
}

/// Finds the first non-air voxel intersected by a ray.
///
/// `block_at` must return Air for cells that are outside the currently
/// available world. The ray direction does not need to be normalized.
pub fn raycast_voxels<F>(
    origin: [f32; 3],
    direction: [f32; 3],
    reach: f32,
    mut block_at: F,
) -> Option<VoxelHit>
where
    F: FnMut(i32, i32, i32) -> BlockId,
{
    if !reach.is_finite() || reach < 0.0 || origin.iter().any(|v| !v.is_finite())
        || direction.iter().any(|v| !v.is_finite())
    {
        return None;
    }

    let length_sq = direction[0] * direction[0]
        + direction[1] * direction[1]
        + direction[2] * direction[2];
    if length_sq < 1.0e-12 { return None; }
    let inv_len = length_sq.sqrt().recip();
    let dir = [direction[0] * inv_len, direction[1] * inv_len, direction[2] * inv_len];

    let mut cell = [origin[0].floor() as i32, origin[1].floor() as i32, origin[2].floor() as i32];
    let mut previous = cell;
    let mut face_normal = [0, 0, 0];
    let (sx, mut tx, dx) = initial_axis(origin[0], dir[0], cell[0]);
    let (sy, mut ty, dy) = initial_axis(origin[1], dir[1], cell[1]);
    let (sz, mut tz, dz) = initial_axis(origin[2], dir[2], cell[2]);
    let steps = [sx, sy, sz];
    let deltas = [dx, dy, dz];
    let mut maxima = [tx, ty, tz];

    // Check the starting cell as well, which makes the function well-defined
    // for rays that begin inside a selectable block.
    let first = block_at(cell[0], cell[1], cell[2]);
    if first != BlockId::Air {
        return Some(VoxelHit { cell, previous, face_normal, block: first });
    }

    loop {
        let axis = if maxima[0] <= maxima[1] && maxima[0] <= maxima[2] { 0 }
            else if maxima[1] <= maxima[2] { 1 } else { 2 };
        let distance = maxima[axis];
        if !distance.is_finite() || distance > reach { return None; }

        previous = cell;
        cell[axis] += steps[axis];
        face_normal = [0, 0, 0];
        face_normal[axis] = -steps[axis];
        maxima[axis] += deltas[axis];

        let block = block_at(cell[0], cell[1], cell[2]);
        if block != BlockId::Air {
            return Some(VoxelHit { cell, previous, face_normal, block });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hits_nearest_voxel_and_reports_previous_cell() {
        let hit = raycast_voxels([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], 8.0,
            |x, y, z| if (x, y, z) == (3, 0, 0) { BlockId::Stone } else { BlockId::Air });
        assert_eq!(hit, Some(VoxelHit {
            cell: [3, 0, 0], previous: [2, 0, 0], face_normal: [-1, 0, 0], block: BlockId::Stone,
        }));
    }

    #[test]
    fn handles_negative_direction_and_negative_coordinates() {
        let hit = raycast_voxels([-0.5, 2.5, 0.5], [-4.0, 0.0, 0.0], 4.0,
            |x, y, z| if (x, y, z) == (-3, 2, 0) { BlockId::OakLog } else { BlockId::Air });
        assert_eq!(hit.unwrap().cell, [-3, 2, 0]);
        assert_eq!(hit.unwrap().previous, [-2, 2, 0]);
        assert_eq!(hit.unwrap().face_normal, [1, 0, 0]);
    }

    #[test]
    fn respects_reach_and_zero_direction() {
        let world = |x, y, z| if (x, y, z) == (5, 0, 0) { BlockId::Stone } else { BlockId::Air };
        assert!(raycast_voxels([0.5, 0.5, 0.5], [1.0, 0.0, 0.0], 3.0, world).is_none());
        assert!(raycast_voxels([0.5, 0.5, 0.5], [0.0, 0.0, 0.0], 10.0, world).is_none());
    }

    #[test]
    fn checks_vertical_rays() {
        let hit = raycast_voxels([1.5, 1.25, 1.5], [0.0, 2.0, 0.0], 5.0,
            |x, y, z| if (x, y, z) == (1, 4, 1) { BlockId::Dirt } else { BlockId::Air }).unwrap();
        assert_eq!(hit.cell, [1, 4, 1]);
        assert_eq!(hit.previous, [1, 3, 1]);
        assert_eq!(hit.face_normal, [0, -1, 0]);
    }
}
