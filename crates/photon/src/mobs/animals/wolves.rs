//! The wolf (`WolfRenderer`, `WolfArmorLayer`, `WolfCollarLayer`): the texture of its variant
//! and state, its pose, the armor of a grown one and the collar of a tame one.
//!
//! The darkening of a wet wolf is not drawn: whether a mob is in water is not tracked.

use photon_game::menus::container::{damage, is_damageable, max_damage};
use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{
    Crackiness, MobModel, QuadrupedState, WolfState, wolf_armor_crackiness,
};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;

use crate::equipment::armor::Armor;
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;
use photon_world::EquipmentSlot;

const ARMOR_MODEL: &str = "minecraft:wolf_armor#main";
/// The textures of the armor's cracks, low to high.
const CRACKS: [&str; 3] = [
    "entity/wolf/wolf_armor_crackiness_low",
    "entity/wolf/wolf_armor_crackiness_medium",
    "entity/wolf/wolf_armor_crackiness_high",
];
const REGISTRY: &str = "minecraft:wolf_variant";
/// The variant of a wolf the server has said nothing of (`WolfVariants.DEFAULT`).
const DEFAULT_VARIANT: &str = "minecraft:pale";
/// The collar's textures, grown and baby (`WolfCollarLayer`).
const COLLARS: [&str; 2] = ["entity/wolf/wolf_collar", "entity/wolf/wolf_collar_baby"];

fn texture_path(asset: &'static str) -> &'static str {
    asset.strip_prefix("minecraft:").unwrap_or(asset)
}

/// The paths (under `textures/`) of the textures a wolf wears.
pub fn textures() -> impl Iterator<Item = &'static str> {
    photon_data::entity_type::wolf_variants()
        .flat_map(|variant| variant.assets.into_iter().map(texture_path))
        .chain(COLLARS)
        .chain(CRACKS)
}

/// `WolfArmorLayer.ARMOR_CRACK_LOCATIONS`.
fn crack_texture(level: Crackiness) -> Option<&'static str> {
    match level {
        Crackiness::None => None,
        Crackiness::Low => Some(CRACKS[0]),
        Crackiness::Medium => Some(CRACKS[1]),
        Crackiness::High => Some(CRACKS[2]),
    }
}

/// The variant a wolf wears, from the registry the server sent.
fn variant_of(
    session: &PlaySession,
    entity: &photon_world::ClientEntity,
) -> Option<&'static photon_data::entity_type::WolfVariant> {
    let name = match entity.variant() {
        None => Some(DEFAULT_VARIANT),
        Some(id) => usize::try_from(id)
            .ok()
            .and_then(|id| session.registry_names(REGISTRY).get(id))
            .map(String::as_str),
    };
    name.and_then(photon_data::entity_type::wolf_variant)
}

/// What the layers over a posed wolf share.
struct Layers<'a> {
    textures: &'a EntityTextures,
    armor: &'a mut Armor,
    placed: &'a photon_render::entity::mob_model::MobPlacement,
    state: &'a WolfState,
}

impl Layers<'_> {
    /// `WolfArmorLayer`: the body armor of a grown wolf, on the armor model posed as it is.
    fn armor(&mut self, entity: &photon_world::ClientEntity, baby: bool, out: &mut Vec<Vertex>) {
        let (false, Some(stack)) = (baby, entity.equipment(EquipmentSlot::Body)) else {
            return;
        };
        let (state, placed) = (self.state, self.placed);
        let pose = |model: &mut MobModel| model.setup_wolf(state);
        let place = |model: &MobModel, rect, colour| {
            crate::mobs::outline::coloured(entity, model, placed, (rect, colour))
        };
        let worn = self.armor.simple_layer(
            stack,
            ("body", "wolf_body", ARMOR_MODEL),
            self.textures,
            &pose,
            &place,
        );
        if worn.is_empty() {
            return;
        }
        out.extend(worn);
        // `Crackiness.byDamage(ItemStack)`: only an item that can wear down cracks.
        let level = if is_damageable(stack) {
            wolf_armor_crackiness(damage(stack), max_damage(stack))
        } else {
            Crackiness::None
        };
        if let Some(texture) = crack_texture(level) {
            out.extend(self.armor.texture_layer(
                (ARMOR_MODEL, texture),
                self.textures,
                &pose,
                &place,
            ));
        }
    }

    /// The collar of a tame wolf, over the model as it is posed.
    fn collar(
        &self,
        entity: &photon_world::ClientEntity,
        (baby, tame): (bool, bool),
        model: &photon_render::entity::mob_model::MobModel,
        out: &mut Vec<Vertex>,
    ) {
        if tame
            && !entity.is_invisible()
            && let Some(rect) = self.textures.rect(COLLARS[usize::from(baby)])
            && let Some(colour) = photon_data::entity_type::dye_texture_colour(
                usize::try_from(entity.collar_dye()).unwrap_or(14),
            )
        {
            out.extend(crate::mobs::outline::coloured(
                entity,
                model,
                self.placed,
                (rect, Some(colour & 0x00FF_FFFF)),
            ));
        }
    }
}

/// The state the wolf renders in: its walk, its mood and its tail, head roll and shake.
fn wolf_state(
    entity: &photon_world::ClientEntity,
    snapshot: &photon_world::EntitySnapshot,
    (partial, game_time): (f32, u64),
) -> WolfState {
    let own = entity.wolf();
    let (baby, angry) = (entity.is_baby(), entity.is_angry(game_time));
    WolfState {
        quadruped: QuadrupedState {
            head_y_rot: photon_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        },
        angry,
        sitting: entity.is_sitting(),
        baby,
        tail_angle: entity.wolf_tail_angle(game_time),
        head_roll_angle: own.head_roll_angle(partial),
        shake_anim: own.shake_anim(partial),
        age_scale: if baby { 0.5 } else { 1.0 },
    }
}

/// `WolfRenderer` and `WolfCollarLayer` for the wolves the server added.
pub fn remote_wolves(
    session: &PlaySession,
    (models, armor): (&mut MobModels, &mut Armor),
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let Some(wolf) = photon_data::entity_type::id("minecraft:wolf") else {
        return;
    };
    let world = session.world();
    let (partial, game_time) = (view.partial as f32, world.game_time());
    for entity in world.drawn_mobs().filter(|entity| entity.type_id == wolf) {
        let Some(variant) = variant_of(session, entity) else {
            continue;
        };
        let baby = entity.is_baby();
        let tame = entity.is_tame();
        let angry = entity.is_angry(game_time);
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let (Some(rect), Some(model)) = (
            textures.rect(texture_path(variant.texture(baby, tame, angry))),
            models.model(if baby {
                "minecraft:wolf_baby#main"
            } else {
                "minecraft:wolf#main"
            }),
        ) else {
            continue;
        };
        let state = wolf_state(entity, &snapshot, (partial, game_time));
        model.setup_wolf(&state);
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
        let mut layers = Layers {
            textures,
            armor,
            placed: &placed,
            state: &state,
        };
        layers.armor(entity, baby, vertices);
        layers.collar(entity, (baby, tame), model, vertices);
    }
}
