//! Original voxel material catalogue. IDs are stable save-file identifiers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum BlockId {
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Sand = 4,
    Sandstone = 5,
    Water = 6,
    Bedrock = 7,
    Snow = 8,
    Gravel = 9,
    Clay = 10,
    OakLog = 11,
    OakLeaves = 12,
    PineLog = 13,
    PineLeaves = 14,
}

impl BlockId {
    pub fn from_raw(value: u16) -> Self {
        match value {
            1 => Self::Grass, 2 => Self::Dirt, 3 => Self::Stone,
            4 => Self::Sand, 5 => Self::Sandstone, 6 => Self::Water,
            7 => Self::Bedrock, 8 => Self::Snow, 9 => Self::Gravel,
            10 => Self::Clay, 11 => Self::OakLog, 12 => Self::OakLeaves,
            13 => Self::PineLog, 14 => Self::PineLeaves, _ => Self::Air,
        }
    }

    pub const fn is_solid(self) -> bool {
        !matches!(self, Self::Air | Self::Water)
    }

    pub const fn is_transparent(self) -> bool {
        matches!(self, Self::Air | Self::Water | Self::OakLeaves | Self::PineLeaves)
    }

    pub const fn is_replaceable(self) -> bool {
        matches!(self, Self::Air | Self::Water)
    }
}

pub const BLOCK_CATALOGUE: &[(&str, BlockId)] = &[
    ("Air", BlockId::Air), ("Meadow Grass", BlockId::Grass),
    ("Loam", BlockId::Dirt), ("Basalt Stone", BlockId::Stone),
    ("Pale Sand", BlockId::Sand), ("Layered Sandstone", BlockId::Sandstone),
    ("Water", BlockId::Water), ("Bedrock", BlockId::Bedrock),
    ("Snow", BlockId::Snow), ("Gravel", BlockId::Gravel),
    ("Clay", BlockId::Clay), ("Oak Log", BlockId::OakLog),
    ("Oak Leaves", BlockId::OakLeaves), ("Pine Log", BlockId::PineLog),
    ("Pine Needles", BlockId::PineLeaves),
];
