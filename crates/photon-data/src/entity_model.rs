//! The reference's model layers: for each layer `LayerDefinitions` builds,
//! the size of its texture and the tree of parts (a pose, cubes and child
//! parts) that the entity models bake and the renderers pose.
//!
//! The layers themselves are in `tables/entity_models.rs`; this holds the
//! types they are written with and the lookup.

use crate::tables::entity_models::LAYERS;

/// `Direction`, in the order of its names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Down,
    East,
    North,
    South,
    Up,
    West,
}

impl Face {
    /// `Direction.getName`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::East => "east",
            Self::North => "north",
            Self::South => "south",
            Self::Up => "up",
            Self::West => "west",
        }
    }
}

/// The faces a cube shows when its definition names no others.
pub const ALL_FACES: &[Face] = &[
    Face::Down,
    Face::East,
    Face::North,
    Face::South,
    Face::Up,
    Face::West,
];

/// `PartPose`: where a part sits and how it turns and scales.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartPose {
    pub position: [f32; 3],
    /// Rotation about x, y and z, in radians.
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl PartPose {
    pub const ZERO: Self = Self::offset_and_rotation([0.0; 3], [0.0; 3]);

    /// `PartPose.offset`.
    pub const fn offset(position: [f32; 3]) -> Self {
        Self::offset_and_rotation(position, [0.0; 3])
    }

    /// `PartPose.rotation`.
    pub const fn rotation(rotation: [f32; 3]) -> Self {
        Self::offset_and_rotation([0.0; 3], rotation)
    }

    /// `PartPose.offsetAndRotation`.
    pub const fn offset_and_rotation(position: [f32; 3], rotation: [f32; 3]) -> Self {
        Self::new(position, rotation, [1.0; 3])
    }

    /// A pose with a scale as well (`scaled` and `withScale` make these).
    pub const fn new(position: [f32; 3], rotation: [f32; 3], scale: [f32; 3]) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
}

/// `CubeDefinition`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelCube {
    /// Where the cube starts in the part, and how big it is.
    pub origin: [f32; 3],
    pub dimensions: [f32; 3],
    /// `CubeDeformation`: how far each side grows.
    pub grow: [f32; 3],
    pub mirror: bool,
    /// The texture offset of the cube's unfolded net, and its scale.
    pub uv: [f32; 2],
    pub uv_scale: [f32; 2],
    pub faces: &'static [Face],
}

impl ModelCube {
    /// A cube with the texture offset `uv`, from `origin`, `dimensions` big.
    pub const fn of(uv: [f32; 2], origin: [f32; 3], dimensions: [f32; 3]) -> Self {
        Self {
            origin,
            dimensions,
            grow: [0.0; 3],
            mirror: false,
            uv,
            uv_scale: [1.0; 2],
            faces: ALL_FACES,
        }
    }

    pub const fn grow(mut self, grow: [f32; 3]) -> Self {
        self.grow = grow;
        self
    }

    pub const fn mirror(mut self) -> Self {
        self.mirror = true;
        self
    }

    pub const fn uv_scale(mut self, scale: [f32; 2]) -> Self {
        self.uv_scale = scale;
        self
    }

    pub const fn faces(mut self, faces: &'static [Face]) -> Self {
        self.faces = faces;
        self
    }
}

/// `PartDefinition`: a pose, cubes and child parts by name.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelPart {
    pub name: &'static str,
    pub pose: PartPose,
    pub cubes: &'static [ModelCube],
    /// In order of name.
    pub children: &'static [ModelPart],
}

impl ModelPart {
    pub const fn of(
        name: &'static str,
        pose: PartPose,
        cubes: &'static [ModelCube],
        children: &'static [ModelPart],
    ) -> Self {
        Self {
            name,
            pose,
            cubes,
            children,
        }
    }
}

/// `LayerDefinition`: a mesh of parts and the size of the texture it is
/// drawn with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelLayer {
    /// `ModelLayerLocation`: the model, `#`, the layer.
    pub id: &'static str,
    pub texture_width: u32,
    pub texture_height: u32,
    pub root: ModelPart,
}

impl ModelLayer {
    pub const fn of(
        id: &'static str,
        texture_width: u32,
        texture_height: u32,
        root: ModelPart,
    ) -> Self {
        Self {
            id,
            texture_width,
            texture_height,
            root,
        }
    }
}

/// Every layer, in order of id.
pub fn layers() -> &'static [ModelLayer] {
    &LAYERS
}

/// The layer with this id (`minecraft:cow#main`).
pub fn layer(id: &str) -> Option<&'static ModelLayer> {
    LAYERS
        .binary_search_by(|layer| layer.id.cmp(id))
        .ok()
        .map(|index| &LAYERS[index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layers_are_in_order_of_id_so_a_lookup_finds_each() {
        assert!(layers().windows(2).all(|pair| pair[0].id < pair[1].id));
        for layer_ in layers() {
            assert_eq!(layer(layer_.id).map(|found| found.id), Some(layer_.id));
        }
        assert!(layer("minecraft:no_such#main").is_none());
    }

    #[test]
    fn the_player_layers_are_there() {
        let player = layer("minecraft:player#main").expect("the player's mesh");
        assert_eq!((player.texture_width, player.texture_height), (64, 64));
        assert!(player.root.children.iter().any(|part| part.name == "head"));
    }
}
