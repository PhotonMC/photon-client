//! The order the scene is drawn in, and what each run of vertices is drawn with.

use super::*;

/// One part of the frame, in the order [`PASSES`] draws them: later ones draw over earlier ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Pass {
    /// The sky over the cleared fog colour, under everything.
    Sky,
    /// The opaque terrain.
    Terrain,
    /// Dropped items, entities and the flat shapes after them ([`ENTITY_RUNS`]).
    Entities,
    /// The aimed block's outline over the solid world.
    Outline,
    /// The cracks over the blocks being broken.
    Cracks,
    /// The translucent terrain, the farthest section first.
    Translucent,
    /// The particles over the water, before the clouds.
    Particles,
    /// The clouds after the terrain, tested against and writing depth.
    Clouds,
    /// Rain and snow over everything, leaving the depth alone.
    Weather,
    /// The first-person hand and what it holds, over a cleared depth.
    Hand,
    /// The title screen's cube map, under the GUI.
    Panorama,
    /// The GUI over everything, run by run in its blends.
    Gui,
}

/// The passes in draw order.
pub(super) const PASSES: [Pass; 12] = [
    Pass::Sky,
    Pass::Terrain,
    Pass::Entities,
    Pass::Outline,
    Pass::Cracks,
    Pass::Translucent,
    Pass::Particles,
    Pass::Clouds,
    Pass::Weather,
    Pass::Hand,
    Pass::Panorama,
    Pass::Gui,
];

impl Pass {
    /// The timestamp written once this pass is recorded, if it ends a measured stage.
    pub(super) fn stamp_after(self) -> Option<u32> {
        match self {
            Self::Sky => Some(1),
            Self::Terrain => Some(2),
            Self::Entities => Some(3),
            Self::Cracks => Some(4),
            Self::Translucent => Some(5),
            Self::Weather => Some(6),
            Self::Hand => Some(7),
            Self::Outline | Self::Particles | Self::Clouds | Self::Panorama | Self::Gui => None,
        }
    }
}

/// A run of vertices of one kind: its vertex list, its pipeline, the set it samples and the push
/// constants it takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Run {
    /// Dropped items and the blocks on entities, in the block atlas.
    Ground,
    /// The dropped items' quads of translucent sprites, blended over what is drawn.
    GroundTranslucent,
    Entity,
    /// The boats' water patches: the depth only, over what the entities wrote.
    Mask,
    /// Blended quads of the shadow texture over the blocks' tops.
    Shadow,
    /// Dissolving entities (a dying dragon's body): the push constants carry where the masks are.
    Dissolve,
    /// The glowing layers (the eyes of spiders and endermen), drawn with the shadow's pipeline.
    Glow,
    /// The energy swirls (a charged creeper's).
    Swirl,
    /// The enchantments' glints, over the depth the entities wrote.
    Glint,
    /// A dying dragon's rays.
    Rays,
    /// Experience orbs: blended quads over the entity textures.
    Orb,
    /// Particles: quads over the entity textures, cut out by their alpha.
    Particle,
    /// The flakes of blocks: quads over the block atlas.
    BlockParticle,
    /// The maps in item frames, drawn as text is.
    Map,
    MapDecoration,
    /// Name tags seen through blocks.
    NameTagSeeThrough,
    /// Name tags tested against the depth.
    NameTag,
    /// The aimed block's outline: lines, with the window's size in the push constants.
    Outline,
    /// The black backing of the high contrast outline, which writes no depth.
    OutlineBacking,
    /// The F3 3D crosshair: lines like the outline, in the hand's view.
    Gizmo,
    Cracks,
    /// The first-person arm.
    Hand,
    /// The items the hand holds, from the block atlas, after the arm.
    Held,
    /// The held items' quads of translucent sprites, blended, after the opaque ones.
    HeldTranslucent,
    /// The enchantment glint of what the hand holds, over the depth the items wrote.
    HandGlint,
}

/// What the entities pass draws, in order. The ground items and the entities come first, then the
/// flat shapes: shadows before what they lie under, the glowing layers, the additive ones, orbs,
/// maps, and the name tags last.
pub(super) const ENTITY_RUNS: [Run; 15] = [
    Run::Ground,
    Run::Entity,
    Run::Mask,
    Run::GroundTranslucent,
    Run::Shadow,
    Run::Dissolve,
    Run::Glow,
    Run::Swirl,
    Run::Glint,
    Run::Rays,
    Run::Orb,
    Run::Map,
    Run::MapDecoration,
    Run::NameTagSeeThrough,
    Run::NameTag,
];

/// What the particles pass draws, in order.
pub(super) const PARTICLE_RUNS: [Run; 2] = [Run::BlockParticle, Run::Particle];

/// Which of the scene's descriptor sets a run samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Sampled {
    Atlas,
    Entity,
    Gui,
    Map,
    Crack,
    /// The pipeline samples nothing; no set is bound.
    Unbound,
}

impl Sampled {
    /// The set, once it has been loaded. `Unbound` has none.
    pub(super) fn set(self, scene: &SceneResources) -> Option<vk::DescriptorSet> {
        match self {
            Self::Atlas => Some(scene.descriptor_set),
            Self::Entity => scene.entity_set,
            Self::Gui => scene.gui_set,
            Self::Map => scene.map_set,
            Self::Crack => scene.crack_set,
            Self::Unbound => None,
        }
    }
}

impl Run {
    /// The pipeline the run is drawn with.
    pub(super) fn pipeline(self, scene: &SceneResources) -> &Pipeline {
        match self {
            Self::Ground
            | Self::Held
            | Self::BlockParticle
            | Self::GroundTranslucent
            | Self::HeldTranslucent => {
                &scene.held_pipelines[usize::from(self.is_item_translucent())]
            }
            Self::Entity | Self::Hand | Self::Particle => &scene.entity_pipelines[0],
            Self::Mask => &scene.mask_pipeline,
            Self::Shadow | Self::Glow => &scene.shadow_pipeline,
            Self::Dissolve => &scene.dissolve_pipeline,
            Self::Swirl => &scene.swirl_pipeline,
            Self::Glint | Self::HandGlint => &scene.glint_pipeline,
            Self::Rays => &scene.rays_pipeline,
            Self::Orb
            | Self::MapDecoration
            | Self::Map
            | Self::NameTag
            | Self::NameTagSeeThrough => scene.text_pipeline_of(self),
            Self::Outline | Self::OutlineBacking | Self::Gizmo => scene.lines_pipeline(self),
            Self::Cracks => &scene.crumbling_pipeline,
        }
    }

    /// Whether the run is the quads of translucent sprites among the items.
    fn is_item_translucent(self) -> bool {
        matches!(self, Self::GroundTranslucent | Self::HeldTranslucent)
    }

    /// The set the run samples.
    pub(super) fn sampled(self) -> Sampled {
        match self {
            Self::Ground
            | Self::GroundTranslucent
            | Self::Held
            | Self::HeldTranslucent
            | Self::BlockParticle => Sampled::Atlas,
            Self::Map => Sampled::Map,
            Self::NameTagSeeThrough | Self::NameTag => Sampled::Gui,
            Self::Outline | Self::OutlineBacking | Self::Gizmo => Sampled::Unbound,
            Self::Cracks => Sampled::Crack,
            Self::Entity
            | Self::Mask
            | Self::Shadow
            | Self::Dissolve
            | Self::Glow
            | Self::Swirl
            | Self::Glint
            | Self::Rays
            | Self::Orb
            | Self::Particle
            | Self::MapDecoration
            | Self::Hand
            | Self::HandGlint => Sampled::Entity,
        }
    }
}

impl SceneResources {
    /// The pipeline of a run drawn with the text shaders.
    fn text_pipeline_of(&self, run: Run) -> &Pipeline {
        match run {
            Run::Orb | Run::MapDecoration => &self.text_translucent_pipeline,
            Run::NameTagSeeThrough => &self.text_see_through_pipeline,
            _ => &self.text_pipeline,
        }
    }

    /// The pipeline of a line run: the backing of the outline writes no depth.
    fn lines_pipeline(&self, run: Run) -> &Pipeline {
        &self.lines_pipelines[usize::from(run == Run::OutlineBacking)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scene_is_drawn_sky_first_and_the_gui_last() {
        assert_eq!(PASSES.first(), Some(&Pass::Sky));
        assert_eq!(PASSES.last(), Some(&Pass::Gui));
        let position = |pass| PASSES.iter().position(|&own| own == pass).unwrap();
        // The terrain is under the entities, the water over them, the hand over the weather.
        assert!(position(Pass::Terrain) < position(Pass::Entities));
        assert!(position(Pass::Entities) < position(Pass::Translucent));
        assert!(position(Pass::Translucent) < position(Pass::Particles));
        assert!(position(Pass::Particles) < position(Pass::Clouds));
        assert!(position(Pass::Clouds) < position(Pass::Weather));
        assert!(position(Pass::Weather) < position(Pass::Hand));
        assert!(position(Pass::Hand) < position(Pass::Gui));
        // The outline and the cracks come after the solid world and before the water.
        assert!(position(Pass::Entities) < position(Pass::Outline));
        assert!(position(Pass::Cracks) < position(Pass::Translucent));
    }

    #[test]
    fn the_timestamps_count_up_through_the_passes() {
        let stamps: Vec<u32> = PASSES
            .iter()
            .filter_map(|pass| pass.stamp_after())
            .collect();
        assert_eq!(stamps, [1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn the_entities_pass_draws_items_and_entities_before_the_shapes_over_them() {
        assert_eq!(
            ENTITY_RUNS[..4],
            [Run::Ground, Run::Entity, Run::Mask, Run::GroundTranslucent]
        );
        let position = |run| ENTITY_RUNS.iter().position(|&own| own == run).unwrap();
        // Shadows lie under the entities' flat shapes; name tags are the last of all.
        assert!(position(Run::Shadow) < position(Run::Glow));
        assert!(position(Run::Glow) < position(Run::Swirl));
        assert!(position(Run::Rays) < position(Run::Orb));
        assert_eq!(ENTITY_RUNS.last(), Some(&Run::NameTag));
        assert!(position(Run::NameTagSeeThrough) < position(Run::NameTag));
    }
}
