use super::remote_placement::{flipped, remote_placement};
use super::*;
use crate::equipment::arm_poses;
use crate::equipment::held_items::HandActions;
use crate::objects::displays;
use photon_render::entity::tag_rules::{NameRules, entity_name_tag, eye_height, rules_of};

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
        // A player who only watches is not drawn.
        if !entity.is_player() || entity.is_spectator() {
            continue;
        }
        let Some(mut snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        if flipped(session, entity) {
            snapshot.turn_upside_down(entity.leash_size().height);
        }
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
        let placement = remote_placement((session, entity, &snapshot), (view, rect));
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
            motions: HandActions::of_entity(spears.motions, session, entity),
        };
        hands.collect(held);
        let skull = hands.skull();
        vertices.extend(armor.skull_vertices(
            skull,
            snapshot.walk_position,
            textures,
            (placement.light, snapshot.red_overlay),
        ));
        let ghost = ghosted(entity, view, &snapshot);
        vertices.extend(shown_body(session, entity, (model, &placement), ghost));
        let wearing = wear(armor, worn, model.slim, textures, (&state, &placement));
        vertices.extend(glowing(session, entity, wearing));
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

/// The triangles of a player, which join the outline when the player glows: the colour is its
/// team's, found by its profile name.
fn glowing(
    session: &PlaySession,
    entity: &photon_world::ClientEntity,
    triangles: Vec<Vertex>,
) -> Vec<Vertex> {
    if let (true, Some(name)) = (entity.is_glowing(), session.player_name(entity.uuid)) {
        crate::mobs::outline::record_holder(name.to_owned(), triangles.clone());
    }
    triangles
}

/// Whether the body of a player is hidden: none when it shows, else the camera a spectating
/// viewer sees its ghost from (none when the viewer sees nothing).
fn ghosted(
    entity: &photon_world::ClientEntity,
    view: FrameView,
    snapshot: &photon_world::EntitySnapshot,
) -> Option<Option<[f32; 3]>> {
    entity
        .is_invisible()
        .then_some((view.spectating || snapshot.friendly_ghost).then_some(view.camera))
}

/// The body a player shows, which also joins the outline when it glows: an invisible glowing
/// player is seen by its outline alone.
fn shown_body(
    session: &PlaySession,
    entity: &photon_world::ClientEntity,
    (model, placement): (&PlayerModel, &Placement),
    ghost: Option<Option<[f32; 3]>>,
) -> Vec<Vertex> {
    if ghost.is_some() {
        glowing(session, entity, body(model, placement, None));
        return body(model, placement, ghost);
    }
    glowing(session, entity, body(model, placement, None))
}

/// `isBodyVisible`: an invisible player keeps what it wears and holds, not its body; a spectator
/// sees the body as a ghost. `invisible` is none for a visible player, else the camera a
/// spectating viewer sees the ghost from.
pub(super) fn body(
    model: &PlayerModel,
    placement: &Placement,
    invisible: Option<Option<[f32; 3]>>,
) -> Vec<Vertex> {
    match invisible {
        None => photon_render::entity::player_renderer::player_vertices(model, placement),
        Some(None) => Vec::new(),
        Some(Some(camera)) => {
            photon_render::entity::player_renderer::player_ghost_vertices(model, placement, camera)
        }
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

/// What the name tags of a frame are made with, and the triangles made so far.
struct Tags<'a> {
    text: (
        &'a photon_render::gui::font::Font,
        &'a photon_render::gui::GuiAtlas,
    ),
    camera: &'a TagCamera,
    orientation: photon_render::joml::Quaternion,
    see_through: Vec<Vertex>,
    normal: Vec<Vertex>,
}

impl Tags<'_> {
    /// `SubmitNodeCollection.submitNameTag` for a tag over an entity at `feet`, lit as at
    /// `light`.
    fn submit(
        &mut self,
        tag: &photon_render::entity::name_tag::NameTag,
        (feet, light): (Vec3, i32),
    ) {
        use photon_render::entity::name_tag::{DisplayMode, submit_vertices, submits};
        let (font, atlas) = self.text;
        let camera = self.camera;
        let placement = Vec3::new(
            feet.x - camera.position.x,
            feet.y - camera.position.y,
            feet.z - camera.position.z,
        );
        let widths = (
            font.width_of_text_runs(&tag.runs),
            tag.score
                .as_ref()
                .map_or(0, |score| font.width_of_text_runs(score)),
        );
        for submit in submits(tag, placement, widths, (light, 0.25), &self.orientation) {
            let runs = match (&tag.score, submit.score) {
                (Some(score), true) => score,
                _ => &tag.runs,
            };
            let vertices = submit_vertices(&submit, runs, font, atlas, camera.relative);
            match submit.mode {
                DisplayMode::SeeThrough => self.see_through.extend(vertices),
                DisplayMode::Normal => self.normal.extend(vertices),
            }
        }
    }
}

/// `EntityRenderer.submitNameDisplay` for the other players, then for the other entities: the
/// text seen through blocks, and the text tested against the depth, each as triangles from the
/// mesh origin. The background is the options' default for chat only, as the option that
/// changes it is not carried.
pub fn name_tags(
    session: &PlaySession,
    (lang, font, atlas): (
        &photon_game::text::Lang,
        &photon_render::gui::font::Font,
        &photon_render::gui::GuiAtlas,
    ),
    camera: &TagCamera,
    view: FrameView,
) -> (Vec<Vertex>, Vec<Vertex>) {
    use photon_render::entity::name_tag::{Viewer, camera_orientation};
    let viewer = Viewer {
        camera_position: camera.position,
        spectator: camera.spectating,
        hud_hidden: camera.hud_hidden,
    };
    let mut tags = Tags {
        text: (font, atlas),
        camera,
        orientation: camera_orientation(camera.rotation.0, camera.rotation.1),
        see_through: Vec::new(),
        normal: Vec::new(),
    };
    player_tags(session, (&mut tags, lang), viewer, view.partial as f32);
    entity_tags(session, (&mut tags, lang), viewer, view.partial as f32);
    (tags.see_through, tags.normal)
}

fn player_tags(
    session: &PlaySession,
    (tags, lang): (&mut Tags, &photon_game::text::Lang),
    viewer: photon_render::entity::name_tag::Viewer,
    partial: f32,
) {
    let world = session.world();
    for entity in world.entities().all() {
        if !entity.is_player() || entity.is_spectator() {
            continue;
        }
        // A player the server did not tell of has no name to show.
        let Some(name) = session.player_name(entity.uuid) else {
            continue;
        };
        let inputs = photon_gui::entity_name::tag_inputs(
            entity,
            Some(name),
            (lang, session.scoreboard()),
            session.local_name(),
        );
        let Some(mut tag) = photon_render::entity::name_tag::name_tag(
            entity,
            name,
            (viewer, inputs.teams),
            partial,
        ) else {
            continue;
        };
        tag.runs = inputs.runs;
        let tag = tag.with_score(entity, viewer, inputs.score);
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let (x, y, z) = snapshot.position;
        // `Entity.getEyePosition`: the pose's eye height, scaled.
        let eye_height = f64::from(
            snapshot
                .pose
                .player_dimensions()
                .scale(entity.scale(), entity.scale())
                .eye_height(),
        );
        let eye = Vec3::new(x, y + eye_height, z);
        tags.submit(&tag, (Vec3::new(x, y, z), packed_light(world, eye)));
    }
}

/// `EntityRenderer.extractNameTags` for the entities that are not players: the ones a name shows
/// over (a custom name that is visible, or under the crosshair).
fn entity_tags(
    session: &PlaySession,
    (tags, lang): (&mut Tags, &photon_game::text::Lang),
    viewer: photon_render::entity::name_tag::Viewer,
    partial: f32,
) {
    let world = session.world();
    let picked = crate::view::crosshair_entity(session);
    for entity in world.entities().drawn_with_invisible() {
        let Some(rules) = rules_of(entity) else {
            continue;
        };
        let Some(inputs) = frame_tag::inputs_of(entity, rules, (lang, session)) else {
            continue;
        };
        let picked = picked == Some(entity.id());
        let Some(tag) =
            entity_name_tag(entity, inputs.runs, (rules, picked), (viewer, inputs.teams))
        else {
            continue;
        };
        let tag = tag.with_score(entity, viewer, inputs.score);
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let mut tag = tag;
        if rules == NameRules::ItemFrame {
            tag.render_offset = frame_tag::render_offset(entity);
        }
        let (x, y, z) = snapshot.position;
        let eye = Vec3::new(x, y + eye_height(entity).unwrap_or(0.0), z);
        let light = packed_light(world, eye);
        let light = if rules == NameRules::ItemFrame && is_glow_frame(entity) {
            // `ItemFrameRenderer.getBlockLightLevel`: a glow frame is lit to 5 at least.
            (light & !0xFFFF) | (((light >> 4) & 15).max(5) << 4)
        } else {
            light
        };
        let light = if snapshot.burning {
            (light & !0xFFFF) | (15 << 4)
        } else {
            light
        };
        tags.submit(&tag, (Vec3::new(x, y, z), light));
    }
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
/// `shadowRadius` and `shadowStrength` are the renderers' (`shadow_radius`). `local` is the player's own place, when
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
    let mut casters: Vec<Caster> = Vec::new();
    for entity in world.entities().all() {
        // `EntityRenderer.extractShadow`: an invisible entity casts none, nor does a spectator.
        if casts_no_shadow(entity) {
            continue;
        }
        let (radius, strength) = if entity.display().is_some() {
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
        } else if let Some(shadow) = super::shadow_radius::of(entity) {
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

/// `EntityRenderer.getBlockLightLevel`: a burning entity is lit by full block light.
pub(super) fn burning_light(light: i32, burning: bool) -> i32 {
    if burning {
        super::light::full_block_light(light)
    } else {
        light
    }
}

/// Whether the entity is a glow item frame.
fn is_glow_frame(entity: &photon_world::ClientEntity) -> bool {
    photon_data::entity_type::name(entity.type_id) == Some("minecraft:glow_item_frame")
}
