//! Where the glint of an enchanted armor piece sits among the piece's layers
//! (`EquipmentLayerRenderer.renderLayers`).
//!
//! The reference draws the first layer, then the glint, then the other layers and the trim over
//! it, so an opaque texel of a later layer hides the glint. In 26.3 a piece with a trim is the
//! exception: its glint is a pass of its own after the trim and covers everything
//! (`trimmed_armor_glint_follows_trim`).

use super::{EntityTextures, MobModel, Place, Vertex, push_glint};

/// One worn piece, as far as the order of its draws is concerned.
pub(super) struct Piece {
    /// The texture path and colour of each layer, in order.
    pub layers: Vec<(String, u32)>,
    /// The texture path of the trim, none for a piece without one (and for a baby's).
    pub trim: Option<String>,
    /// Whether the piece has the enchantment glint.
    pub foil: bool,
}

/// Where the triangles go: the entity draw, the glint pass and the draw after the glints.
pub(super) struct Sinks<'a> {
    pub vertices: &'a mut Vec<Vertex>,
    pub glint: &'a mut Vec<Vertex>,
    pub late: &'a mut Vec<Vertex>,
}

/// Whether the piece's glint is drawn over all its layers (see the module).
fn glint_covers_all(piece: &Piece, glint_follows_trim: bool) -> bool {
    glint_follows_trim && piece.trim.is_some()
}

/// Whether triangles drawn after the first layer go over the glint (the entity draw after the
/// glints) rather than into the entity draw.
fn goes_over_glint(piece: &Piece, covers_all: bool) -> bool {
    piece.foil && !covers_all
}

/// The layers in order: the first one's triangles also give the glint, and those after it go
/// where [`goes_over_glint`] says. Returns the triangles the glint is made of.
fn place_layers(
    piece: &Piece,
    (model, textures, place): (&MobModel, &EntityTextures, Place),
    (covers_all, sinks): (bool, &mut Sinks),
) -> Vec<(Vec<Vertex>, [f32; 4])> {
    let mut glints = Vec::new();
    let mut after_glint = false;
    for (texture, colour) in &piece.layers {
        // A layer coloured to nothing is not drawn; white is no colour.
        let (true, Some(rect)) = (*colour != 0, textures.rect(texture)) else {
            continue;
        };
        let rgb = colour & 0x00FF_FFFF;
        let triangles = place(model, rect, (*colour != 0xFFFF_FFFF).then_some(rgb));
        if piece.foil && !after_glint {
            glints.push((triangles.clone(), rect));
        }
        if after_glint && goes_over_glint(piece, covers_all) {
            sinks.late.extend(triangles);
        } else {
            sinks.vertices.extend(triangles);
        }
        after_glint = true;
    }
    glints
}

/// Put the layers, the trim and the glint of `piece` where the played version draws them.
pub(super) fn place_piece(
    piece: Piece,
    (model, textures, place): (&MobModel, &EntityTextures, Place),
    glint_matrix: &[f32; 16],
    sinks: &mut Sinks,
) {
    let follows_trim = photon_core::versions::rules()
        .render
        .trimmed_armor_glint_follows_trim;
    let covers_all = glint_covers_all(&piece, follows_trim);
    let glints = place_layers(&piece, (model, textures, place), (covers_all, sinks));
    // The trim, over the layers, undyed.
    if let Some(rect) = piece.trim.as_deref().and_then(|path| textures.rect(path)) {
        let triangles = place(model, rect, None);
        if goes_over_glint(&piece, covers_all) {
            sinks.late.extend(triangles);
        } else {
            sinks.vertices.extend(triangles);
        }
    }
    let first = sinks.glint.len();
    for (triangles, rect) in glints {
        push_glint(sinks.glint, (&triangles, rect), (glint_matrix, textures));
    }
    if covers_all {
        // A pass of its own: the fog fades it as one in every version.
        for vertex in &mut sinks.glint[first..] {
            vertex.shade = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece(trim: bool) -> Piece {
        Piece {
            layers: vec![("a".into(), 0xFFFF_FFFF), ("b".into(), 0xFFFF_FFFF)],
            trim: trim.then(|| "t".into()),
            foil: true,
        }
    }

    #[test]
    fn only_a_trimmed_piece_of_a_version_that_says_so_has_its_glint_over_all() {
        assert!(glint_covers_all(&piece(true), true));
        assert!(!glint_covers_all(&piece(true), false));
        assert!(!glint_covers_all(&piece(false), true));
    }
}
