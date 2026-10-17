use super::*;
use crate::block_entity::conduit::Piece;

/// The pieces of a conduit's shell as meshes in the block atlas, each in its own model space.
pub struct ConduitMeshes {
    pieces: Vec<(Piece, ItemLayer)>,
}

impl ConduitMeshes {
    /// The mesh of a piece.
    pub fn get(&self, piece: Piece) -> Option<&ItemLayer> {
        self.pieces
            .iter()
            .find(|(held, _)| *held == piece)
            .map(|(_, layer)| layer)
    }
}

const PIECES: [Piece; 6] = [
    Piece::Shell,
    Piece::Cage,
    Piece::Wind { vertical: false },
    Piece::Wind { vertical: true },
    Piece::Eye { open: false },
    Piece::Eye { open: true },
];

impl BlockRenderer {
    /// What `ConduitRenderer` draws: each model layer in each sprite it is drawn in, none if the
    /// atlas lacks one.
    pub fn conduit_meshes(&self) -> Option<ConduitMeshes> {
        let mut pieces = Vec::new();
        for piece in &PIECES {
            let layer = photon_data::entity_model::layer(piece.layer())?;
            let model = crate::entity::mob_model::MobModel::bake(layer);
            let quads = model.quads(&crate::joml::Pose::default());
            let sprite = piece.sprite();
            let rect = self.atlas.sprites.get(&format!("minecraft:{sprite}"))?;
            let drawn = crate::entity::special_models::ModelLayer {
                quads,
                sprite: sprite.to_owned(),
                tint: 0xFFFF_FFFF,
            };
            pieces.push((
                *piece,
                ItemLayer {
                    mesh: model_mesh(&drawn, rect),
                    side_lit: false,
                    transform: None,
                },
            ));
        }
        Some(ConduitMeshes { pieces })
    }
}
