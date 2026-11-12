//! A banner's triangles (`BannerRenderer`): the pole and the flag, and apart from them the layers
//! of the flag, each tinted by its dye and blended over the one before.

use photon_render::block_entity::banner;
use photon_render::entity::mob_model::{mob_vertices_posed, mob_vertices_tinted};
use photon_render::entity::textures::EntityTextures;
use photon_render::joml::Pose;
use photon_render::scene::Vertex;
use photon_world::{BannerView, DYES};

use crate::mobs::MobModels;

/// The two halves of a banner: what the destroy overlay is drawn over (the pole and the flag in
/// the base sprite) and the layers of the flag, which no version cracks.
pub(crate) struct BannerParts {
    pub solid: Vec<Vertex>,
    pub layers: Vec<Vertex>,
}

impl BannerParts {
    /// What a destroy overlay is drawn over: the pole and the flag in the base sprite, never the
    /// layers.
    pub(crate) fn cracked(self) -> Vec<Vertex> {
        self.solid
    }

    /// Hand the halves over for drawing: the pole and flag with `drawn`, and the layers with it
    /// too unless the banner is being broken in a version that draws the overlay with the opaque
    /// models (`over_cracks`), which draws them after the overlay.
    pub(crate) fn route(
        self,
        (cracked, after_overlay): (bool, bool),
        (drawn, over_cracks): (&mut Vec<Vertex>, &mut Vec<Vertex>),
    ) {
        drawn.extend(self.solid);
        if cracked && after_overlay {
            over_cracks.extend(self.layers);
        } else {
            drawn.extend(self.layers);
        }
    }
}

/// One banner as its two halves.
pub(crate) fn banner_parts(
    models: &mut MobModels,
    textures: &EntityTextures,
    (base, light, phase): (&Pose, i32, f32),
    banner: &BannerView,
) -> BannerParts {
    let mut parts = BannerParts {
        solid: Vec::new(),
        layers: Vec::new(),
    };
    let (main, flag) = banner::layers(banner.wall.is_some());
    let transformation = match banner.wall {
        Some(facing) => banner::wall_transformation(facing),
        None => banner::ground_transformation(banner.rotation),
    };
    let pose = photon_render::block_entity::skull::block_pose(base, &transformation);
    if let (Some(rect), Some(pole)) = (textures.rect(banner::BASE_TEXTURE), models.model(main)) {
        pole.setup_banner_flag(phase);
        parts
            .solid
            .extend(mob_vertices_posed(pole, &pose, rect, (light, false)));
    }
    let Some(cloth) = models.model(flag) else {
        return parts;
    };
    cloth.setup_banner_flag(phase);
    if let Some(rect) = textures.rect(banner::BASE_TEXTURE) {
        parts
            .solid
            .extend(mob_vertices_posed(cloth, &pose, rect, (light, false)));
    }
    let dye = |name: &str| {
        DYES.iter()
            .position(|candidate| *candidate == name)
            .and_then(photon_data::entity_type::dye_texture_colour)
            .map(|colour| colour & 0x00FF_FFFF)
    };
    let layers = std::iter::once((banner::PATTERN_BASE_TEXTURE.to_owned(), dye(banner.color)))
        .chain(
            banner
                .patterns
                .iter()
                .take(banner::MAX_PATTERNS)
                .map(|(pattern, color)| (banner::pattern_texture(pattern), dye(color))),
        );
    for (texture, colour) in layers {
        let (Some(rect), Some(colour)) = (textures.rect(&texture), colour) else {
            continue;
        };
        parts
            .layers
            .extend(mob_vertices_tinted(cloth, &pose, rect, light, colour));
    }
    parts
}

/// One banner whole: the pole and flag, then the layers.
pub(crate) fn banner_vertices(
    models: &mut MobModels,
    textures: &EntityTextures,
    placed: (&Pose, i32, f32),
    banner: &BannerView,
    vertices: &mut Vec<Vertex>,
) {
    let parts = banner_parts(models, textures, placed, banner);
    vertices.extend(parts.solid);
    vertices.extend(parts.layers);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts() -> BannerParts {
        let at = |x: f32| Vertex {
            position: [x, 0.0, 0.0],
            uv: [0.0; 2],
            shade: 1.0,
            tint: [1.0; 4],
            light: [0.0; 2],
        };
        BannerParts {
            solid: vec![at(1.0)],
            layers: vec![at(2.0), at(3.0)],
        }
    }

    fn xs(vertices: &[Vertex]) -> Vec<f32> {
        vertices.iter().map(|vertex| vertex.position[0]).collect()
    }

    #[test]
    fn a_destroy_overlay_covers_no_layer() {
        assert_eq!(xs(&parts().cracked()), [1.0]);
    }

    #[test]
    fn a_broken_banner_draws_its_layers_after_the_overlay_where_the_version_says() {
        let (mut drawn, mut over) = (Vec::new(), Vec::new());
        parts().route((true, true), (&mut drawn, &mut over));
        assert_eq!((xs(&drawn), xs(&over)), (vec![1.0], vec![2.0, 3.0]));
    }

    #[test]
    fn a_banner_draws_its_layers_with_the_rest_otherwise() {
        for flags in [(true, false), (false, true), (false, false)] {
            let (mut drawn, mut over) = (Vec::new(), Vec::new());
            parts().route(flags, (&mut drawn, &mut over));
            assert_eq!((xs(&drawn), xs(&over)), (vec![1.0, 2.0, 3.0], vec![]));
        }
    }
}
