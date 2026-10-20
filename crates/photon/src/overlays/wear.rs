use super::*;
use crate::equipment::arm_poses;
use crate::equipment::held_items::HandActions;
use crate::objects::displays;

/// `LivingEntityRenderer` for the other players: each in its default skin
/// (a profile's own would be downloaded), posed from where it is between
/// two ticks, as the first-person renderer is for the local player's own
/// model.
pub fn remote_players(
    session: &PlaySession,
    (models, armor): (&mut RemoteModels, &mut Armor),
    textures: &photon_render::entity::textures::EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.entities().drawn() {
        // A player who only watches, or who is invisible, is not drawn.
        if !entity.is_player() || entity.is_spectator() || entity.is_invisible() {
            continue;
        }
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let skin = photon_render::entity::player_renderer::default_skin(entity.uuid);
        let Some(rect) = textures.rect(skin.texture) else {
            continue;
        };
        let model = if skin.slim {
            &mut models.slim
        } else {
            &mut models.wide
        };
        let crouching = snapshot.pose == photon_world::ClientEntityPose::Crouching;
        let (right, left) = (
            entity.item_held_by_arm(false),
            entity.item_held_by_arm(true),
        );
        let (state, spears) = remote_render_state(session, (entity, &snapshot), partial);
        model.setup_anim(&state);
        let (x, y, z) = snapshot.position;
        let eye = Vec3::new(x, y + if crouching { 1.27 } else { 1.62 }, z);
        let placement = Placement {
            feet: [
                (x - f64::from(view.origin[0])) as f32,
                (y - f64::from(view.origin[1])) as f32,
                (z - f64::from(view.origin[2])) as f32,
            ],
            body_rot: snapshot.body_yaw,
            x_rot: snapshot.pitch,
            swim_amount: snapshot.swim_amount,
            in_water: false,
            visually_swimming: snapshot.pose == photon_world::ClientEntityPose::Swimming,
            crouching,
            light: packed_light(world, eye),
            skin: rect,
            death_time: snapshot.death_time,
            red_overlay: snapshot.red_overlay,
            fall_flying: arm_poses::fall_flying(entity, &snapshot, partial),
            auto_spin: entity
                .is_auto_spin_attack()
                .then(|| entity.tick_count() as f32 + partial),
        };
        let worn = Worn {
            head: entity.equipment(EquipmentSlot::Head),
            chest: entity.equipment(EquipmentSlot::Chest),
            legs: entity.equipment(EquipmentSlot::Legs),
            feet: entity.equipment(EquipmentSlot::Feet),
        };
        let hands = Hands {
            base: &photon_render::entity::player_renderer::placement_pose(&placement),
            root: &model.root,
            hand_shift: if model.slim { 0.5 } else { 0.0 },
            baby: false,
            light: placement.light,
            right,
            left,
            head: worn.head_item(armor),
            head_transforms: HEAD_TRANSFORMS,
            eye: arm_poses::eye_arm_of(session, entity, snapshot.attack_time, partial),
            motions: HandActions::new(spears.motions, arm_poses::entity_uses(entity)),
        };
        hands.collect(held);
        let skull = hands.skull();
        vertices.extend(armor.skull_vertices(
            skull,
            snapshot.walk_position,
            textures,
            (placement.light, snapshot.red_overlay),
        ));
        vertices.extend(photon_render::entity::player_renderer::player_vertices(
            model, &placement,
        ));
        vertices.extend(wear(
            armor,
            worn,
            model.slim,
            textures,
            (&state, &placement),
        ));
        vertices.extend(armor.wings(
            worn.chest,
            (false, crouching, entity.elytra_rotations(partial)),
            textures,
            &|model, rect, colour| {
                photon_render::entity::player_renderer::layer_vertices(
                    model, &placement, rect, colour,
                )
            },
        ));
    }
}

/// What a player wears, drawn over its model: the armor models of its build,
/// posed as it is.
pub(super) fn wear(
    armor: &mut Armor,
    worn: Worn,
    slim: bool,
    textures: &photon_render::entity::textures::EntityTextures,
    (state, placement): (&PlayerRenderState, &Placement),
) -> Vec<Vertex> {
    let prefix = if slim {
        "minecraft:player_slim"
    } else {
        "minecraft:player"
    };
    armor.vertices(
        worn,
        (prefix, false),
        textures,
        &|model| model.setup_humanoid(state),
        &|model, rect, colour| {
            photon_render::entity::player_renderer::layer_vertices(model, placement, rect, colour)
        },
    )
}

/// `EntityRenderer.submitNameDisplay` for the other players: the text seen
/// through blocks, and the text tested against the depth, each as triangles
/// from the mesh origin. The background is the options' default for chat
/// only, as the option that changes it is not carried.
pub fn name_tags(
    session: &PlaySession,
    (font, atlas): (
        &photon_render::gui::font::Font,
        &photon_render::gui::GuiAtlas,
    ),
    camera: &TagCamera,
    view: FrameView,
) -> (Vec<Vertex>, Vec<Vertex>) {
    use photon_render::entity::name_tag::{
        DisplayMode, Viewer, camera_orientation, name_tag, submits,
    };
    let world = session.world();
    let partial = view.partial as f32;
    let viewer = Viewer {
        camera_position: camera.position,
        spectator: camera.spectating,
        hud_hidden: camera.hud_hidden,
    };
    let orientation = camera_orientation(camera.rotation.0, camera.rotation.1);
    let (mut see_through, mut normal) = (Vec::new(), Vec::new());
    for entity in world.entities().all() {
        if !entity.is_player() || entity.is_spectator() {
            continue;
        }
        // A player the server did not tell of has no name to show.
        let Some(name) = session.player_name(entity.uuid) else {
            continue;
        };
        let Some(tag) = name_tag(entity, name, viewer, partial) else {
            continue;
        };
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let crouching = snapshot.pose == photon_world::ClientEntityPose::Crouching;
        let (x, y, z) = snapshot.position;
        let eye = Vec3::new(x, y + if crouching { 1.27 } else { 1.62 }, z);
        let placement = Vec3::new(
            x - camera.position.x,
            y - camera.position.y,
            z - camera.position.z,
        );
        for submit in submits(
            &tag,
            placement,
            font.width(&tag.text),
            packed_light(world, eye),
            &orientation,
            0.25,
        ) {
            let vertices = photon_render::entity::name_tag::submit_vertices(
                &submit,
                &tag.text,
                font,
                atlas,
                camera.relative,
            );
            match submit.mode {
                DisplayMode::SeeThrough => see_through.extend(vertices),
                DisplayMode::Normal => normal.extend(vertices),
            }
        }
    }
    (see_through, normal)
}

/// `ExperienceOrbRenderer` for the orbs the server added: a quad each, turned
/// to the camera, with the icon its value picks.
pub fn experience_orbs(
    (session, pickups): (&PlaySession, &[&photon_world::particles::ItemPickup]),
    textures: &photon_render::entity::textures::EntityTextures,
    camera: &TagCamera,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let Some(rect) = textures.rect(ORB_TEXTURE) else {
        return;
    };
    let Some(orb_type) = photon_data::entity_type::id("minecraft:experience_orb") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    let orientation =
        photon_render::entity::name_tag::camera_orientation(camera.rotation.0, camera.rotation.1);
    let orb =
        |(x, y, z): (f64, f64, f64), (value, age_in_ticks): (i32, f32), lit_at: (f64, f64, f64)| {
            // `getBlockLightLevel`: seven brighter than the block it is in.
            let light = packed_light(world, Vec3::new(lit_at.0, lit_at.1 + 0.425, lit_at.2));
            let block = ((light >> 4 & 15) + 7).min(15);
            let light = (light & !(15 << 4)) | block << 4;
            let corners = photon_render::objects::xp_orb::corners(
                photon_render::objects::xp_orb::icon(value),
                age_in_ticks,
                [
                    (x - camera.position.x) as f32,
                    (y - camera.position.y) as f32,
                    (z - camera.position.z) as f32,
                ],
                &orientation,
            );
            photon_render::objects::xp_orb::vertices(&corners, rect, camera.relative, light)
        };
    for entity in world
        .entities()
        .all()
        .filter(|entity| entity.type_id == orb_type)
    {
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let state = (
            entity.orb_value().unwrap_or(0),
            entity.tick_count() as f32 + partial,
        );
        vertices.extend(orb(snapshot.position, state, snapshot.position));
    }
    // The orbs flying to whoever took them, as they were when taken, lit where they were.
    for pickup in pickups {
        if let photon_world::particles::PickedUp::Orb {
            value,
            age_in_ticks,
        } = pickup.picked
        {
            let at = pickup.position_at(partial);
            vertices.extend(orb(at, (value, age_in_ticks), pickup.from));
        }
    }
}

/// Whether an entity casts no shadow: an invisible one, or a spectator.
fn casts_no_shadow(entity: &photon_world::ClientEntity) -> bool {
    entity.is_invisible() || entity.is_player() && entity.is_spectator()
}

/// `EntityRenderer.extractShadow` and `ShadowFeatureRenderer` for the
/// entities the frame draws: the shadow each casts on the blocks below it.
/// `shadowRadius` and `shadowStrength` are the renderers': a mob's is its
/// age's share of 0.7, a player's 0.5, a dropped item's and an orb's 0.15
/// at three quarters of the strength. `local` is the player's own place, when
/// the camera shows them.
pub fn entity_shadows(
    session: &PlaySession,
    textures: &photon_render::entity::textures::EntityTextures,
    camera: &TagCamera,
    local: Option<Vec3>,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    use photon_world::ShadowInput;
    let Some(rect) = textures.rect(SHADOW_TEXTURE) else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    let id = |name: &str| photon_data::entity_type::id(name);
    let (pig, cow, item, orb) = (
        id("minecraft:pig"),
        id("minecraft:cow"),
        id("minecraft:item"),
        id("minecraft:experience_orb"),
    );
    let mut casters: Vec<Caster> = Vec::new();
    for entity in world.entities().all() {
        // `EntityRenderer.extractShadow`: an invisible entity casts none, nor does a spectator.
        if casts_no_shadow(entity) {
            continue;
        }
        let type_id = Some(entity.type_id);
        let (radius, strength) = if entity.is_player() {
            (0.5, 1.0)
        } else if type_id == pig || type_id == cow {
            (if entity.is_baby() { 0.35 } else { 0.7 }, 1.0)
        } else if entity.is_cube_mob() {
            // `AbstractCubeMobRenderer.getShadowRadius`.
            (entity.cube_size() as f32 * 0.25, 1.0)
        } else if type_id == item || type_id == orb {
            (0.15, 0.75)
        } else if entity.display().is_some() {
            let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
                continue;
            };
            let Some(shadow) = displays::display_shadow(
                entity,
                snapshot.position,
                (camera.position.x, camera.position.y, camera.position.z),
                partial,
            ) else {
                continue;
            };
            shadow
        } else {
            continue;
        };
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        casters.push(Caster {
            drawn: snapshot.position,
            ticked: entity.position(),
            radius,
            strength,
        });
    }
    if let Some(at) = local {
        casters.push(Caster {
            drawn: (at.x, at.y, at.z),
            ticked: (at.x, at.y, at.z),
            radius: 0.5,
            strength: 1.0,
        });
    }
    for Caster {
        drawn,
        ticked,
        radius,
        strength,
    } in casters
    {
        let dx = camera.position.x - ticked.0;
        let dy = camera.position.y - ticked.1;
        let dz = camera.position.z - ticked.2;
        let radius = radius.min(32.0);
        let pieces = world.entity_shadow_pieces(ShadowInput {
            position: drawn,
            radius,
            strength,
            distance_sq: dx * dx + dy * dy + dz * dz,
            // The sky's dimming (`gameplay/sky_light_level`) is not carried.
            sky_darken: 0,
        });
        let translation = [
            (drawn.0 - camera.position.x) as f32,
            (drawn.1 - camera.position.y) as f32,
            (drawn.2 - camera.position.z) as f32,
        ];
        vertices.extend(photon_render::entity::shadow::vertices(
            &pieces,
            radius,
            translation,
            rect,
            camera.relative,
        ));
    }
}
