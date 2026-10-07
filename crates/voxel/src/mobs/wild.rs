use super::*;

/// The model layer a variant's model family asks for (`PigRenderer.models`,
/// `CowRenderer.models`): a pig's baby has one model whatever the family.
pub(super) fn layer_name(kind: &MobKind, model: &str, baby: bool) -> String {
    match (kind.variant_kind, model, baby) {
        ("chicken", _, true) => "minecraft:chicken_baby#main".to_owned(),
        ("chicken", "cold", false) => "minecraft:cold_chicken#main".to_owned(),
        ("chicken", _, false) => "minecraft:chicken#main".to_owned(),
        ("pig", _, true) => "minecraft:pig_baby#main".to_owned(),
        ("pig", "cold", false) => "minecraft:cold_pig#main".to_owned(),
        ("pig", _, false) => "minecraft:pig#main".to_owned(),
        (cow, model, baby) => {
            let family = match model {
                "normal" => String::new(),
                other => format!("{other}_"),
            };
            let baby = if baby { "_baby" } else { "" };
            format!("minecraft:{family}{cow}{baby}#main")
        }
    }
}

/// An asset id of the data pack as the path under `textures/`.
pub(super) fn texture_path(asset: &'static str) -> &'static str {
    asset.strip_prefix("minecraft:").unwrap_or(asset)
}

/// The paths (under `textures/`) of the textures the mobs wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    let variants = KINDS
        .iter()
        .flat_map(|kind| voxel_data::entity_type::variants(kind.variant_kind))
        .flat_map(|variant| {
            [
                texture_path(variant.asset),
                texture_path(variant.baby_asset),
            ]
        });
    let zombies = HUMANOIDS.iter().flat_map(|kind| {
        [kind.adult_texture, kind.baby_texture].into_iter().chain(
            kind.outer
                .into_iter()
                .flat_map(|outer| [outer[2], outer[3]]),
        )
    });
    variants
        .chain(zombies)
        .chain(crate::sheep::textures())
        .chain(crate::monsters::textures())
        .chain(crate::illagers::textures())
        .chain(crate::golems::textures())
        .chain(crate::farm_animals::textures())
        .chain(crate::wolves::textures())
        .chain(crate::felines::textures())
        .chain(crate::bears::textures())
        .chain(crate::armadillos::textures())
        .chain(crate::sniffers::textures())
        .chain(crate::ender_dragons::textures())
        .chain(crate::end_crystals::textures())
        .chain(crate::block_entities::textures())
        .chain(crate::minecarts::textures())
        .chain(crate::arrows::textures())
        .chain(crate::charges::textures())
        .chain(crate::bullets::textures())
        .chain(crate::fishing_hooks::textures())
        .chain(crate::paintings::textures())
        .chain(crate::item_frames::textures())
        .chain(crate::hoglins::textures())
        .chain(crate::foxes::textures())
        .chain(crate::pandas::textures())
        .chain(crate::ravagers::textures())
        .chain(crate::striders::textures())
        .chain(crate::squids::textures())
        .chain(crate::cube_mobs::textures())
        .chain(crate::shulkers::textures())
        .chain(crate::fish::textures())
        .chain(crate::dolphins::textures())
        .chain(crate::turtles::textures())
        .chain(crate::axolotls::textures())
        .chain(crate::frogs::textures())
        .chain(crate::guardians::textures())
        .chain(crate::nautilus::textures())
        .chain(crate::happy_ghasts::textures())
        .chain(crate::boats::textures())
        .chain(crate::copper_golems::textures())
        .chain([crate::armor_stands::TEXTURE])
}

/// Hand what the glowing layers, the swirls, the enchantment glints (the armor's), the dissolving
/// bodies and the rays drew this frame to the renderer, which draws them after the mobs.
pub fn hand_layers(
    models: &mut MobModels,
    backend: &mut voxel_render::VulkanBackend,
    armor: &mut crate::armor::Armor,
) -> Result<(), voxel_render::VulkanError> {
    backend.set_draw(
        voxel_render::vulkan::DrawKind::EntityGlint,
        &std::mem::take(&mut armor.glint),
    )?;
    backend.set_draw(
        voxel_render::vulkan::DrawKind::EntityGlow,
        &std::mem::take(&mut models.glow),
    )?;
    backend.set_draw(
        voxel_render::vulkan::DrawKind::EntitySwirl,
        &std::mem::take(&mut models.swirl),
    )?;
    backend.set_entity_dissolve(
        &std::mem::take(&mut models.dissolve),
        models.dissolve_mask_offset,
    )?;
    backend.set_draw(
        voxel_render::vulkan::DrawKind::DragonRays,
        &std::mem::take(&mut models.rays),
    )
}

/// Where a mob is drawn and how it is lit, from where it is between two ticks.
pub(crate) fn placement(
    world: &voxel_world::ClientWorld,
    snapshot: &voxel_world::EntitySnapshot,
    view: FrameView,
    scale: f32,
) -> MobPlacement {
    let (x, y, z) = snapshot.position;
    MobPlacement {
        feet: [
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        ],
        body_rot: snapshot.body_yaw,
        scale,
        light: packed_light(world, Vec3::new(x, y + 0.5, z)),
        death_time: snapshot.death_time,
        flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
        red_overlay: snapshot.red_overlay,
        shaking: snapshot.shaking.then_some(snapshot.age_in_ticks),
        ..MobPlacement::default()
    }
}

/// `LivingEntityRenderer` for the mobs the server added: each posed from
/// where it is between two ticks.
pub fn remote_mobs(
    session: &PlaySession,
    (models, armor): (&mut MobModels, &mut Armor),
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    models.glow.clear();
    models.swirl.clear();
    models.dissolve.clear();
    models.rays.clear();
    for entity in world.entities() {
        let Some(kind) = models.kinds.get(&entity.type_id).copied() else {
            continue;
        };
        // The variant the server named by its place in the registry, or the
        // default while it has said nothing; one the registry lacks is not drawn.
        let name = match entity.variant() {
            None => Some(DEFAULT_VARIANT),
            Some(id) => usize::try_from(id)
                .ok()
                .and_then(|id| session.registry_names(kind.registry).get(id))
                .map(String::as_str),
        };
        let Some(variant) =
            name.and_then(|name| voxel_data::entity_type::variant(kind.variant_kind, name))
        else {
            continue;
        };
        let layer = layer_name(kind, variant.model, entity.is_baby());
        let texture = texture_path(if entity.is_baby() {
            variant.baby_asset
        } else {
            variant.asset
        });
        let (Some(snapshot), Some(rect)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
        ) else {
            continue;
        };
        let Some(model) = models.model(&layer) else {
            continue;
        };
        let look = QuadrupedState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        };
        match kind.pose {
            Pose::Quadruped => model.setup_quadruped(&look),
            Pose::Chicken => {
                let (flap, flap_speed) = entity.chicken_flap(partial);
                model.setup_chicken(&ChickenState {
                    head_y_rot: look.head_y_rot,
                    x_rot: look.x_rot,
                    walk_position: look.walk_position,
                    walk_speed: look.walk_speed,
                    flap,
                    flap_speed,
                });
            }
        }
        vertices.extend(mob_vertices(
            model,
            &placement(world, &snapshot, view, entity.scale()),
            rect,
        ));
    }
    vertices.extend(remote_families(
        session,
        (models, armor),
        textures,
        (view, held),
    ));
    vertices
}

/// The felines, bears, armadillos, hoglins, sniffers, ender dragons and end crystals the server added.
pub(super) fn remote_wild_animals(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let mut vertices = crate::felines::remote_felines(session, models, textures, view);
    vertices.extend(crate::bears::remote_bears(session, models, textures, view));
    vertices.extend(crate::armadillos::remote_armadillos(
        session, models, textures, view,
    ));
    vertices.extend(crate::hoglins::remote_hoglins(
        session, models, textures, view,
    ));
    vertices.extend(crate::sniffers::remote_sniffers(
        session, models, textures, view,
    ));
    vertices.extend(crate::ender_dragons::remote_ender_dragons(
        session, models, textures, view,
    ));
    vertices.extend(crate::end_crystals::remote_end_crystals(
        session, models, textures, view,
    ));
    vertices.extend(crate::block_entities::block_entities(
        session, models, textures, view,
    ));
    vertices
}

/// The arrows, wind charges, paintings and minecarts the server added.
pub(super) fn remote_projectiles_and_vehicles(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let mut vertices = crate::arrows::remote_arrows(session, models, textures, view);
    vertices.extend(crate::charges::remote_charges(
        session, models, textures, view,
    ));
    vertices.extend(crate::bullets::remote_bullets(
        session, models, textures, view,
    ));
    vertices.extend(crate::paintings::remote_paintings(session, textures, view));
    crate::minecarts::remote_primed_tnt(session, (view, &mut *held));
    crate::lightning_bolts::remote_lightning(session, models, view);
    vertices.extend(crate::minecarts::remote_minecarts(
        session,
        models,
        textures,
        (view, held),
    ));
    vertices
}

/// The renderers of the families that have a module of their own, one after
/// the other: humanoids, sheep, monsters, villagers, farm animals, golems,
/// witches, illagers and armor stands.
pub(super) fn remote_families(
    session: &PlaySession,
    (models, armor): (&mut MobModels, &mut Armor),
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    vertices.extend(remote_humanoids(
        session,
        (models, armor),
        textures,
        (view, held),
    ));
    vertices.extend(crate::sheep::remote_sheep(session, models, textures, view));
    vertices.extend(remote_hostiles(
        session,
        models,
        textures,
        (view, &mut *held),
    ));
    vertices.extend(crate::villagers::remote_villagers(
        session,
        models,
        textures,
        (view, &mut *held),
    ));
    vertices.extend(crate::wolves::remote_wolves(
        session, models, textures, view,
    ));
    vertices.extend(remote_wild_animals(session, models, textures, view));
    vertices.extend(crate::foxes::remote_foxes(
        session,
        models,
        textures,
        (view, &mut *held),
    ));
    vertices.extend(crate::pandas::remote_pandas(
        session,
        models,
        textures,
        (view, &mut *held),
    ));
    vertices.extend(crate::ravagers::remote_ravagers(
        session, models, textures, view,
    ));
    vertices.extend(remote_projectiles_and_vehicles(
        session,
        models,
        textures,
        (view, &mut *held),
    ));
    vertices.extend(crate::farm_animals::remote_farm_animals(
        session,
        models,
        textures,
        (view, &mut *held),
    ));
    vertices.extend(crate::golems::remote_golems(
        session,
        models,
        textures,
        (view, &mut *held),
    ));
    vertices.extend(crate::villagers::remote_witches(
        session,
        models,
        textures,
        (view, &mut *held),
    ));
    vertices.extend(crate::illagers::remote_illagers(
        session,
        models,
        textures,
        (view, held),
    ));
    vertices.extend(crate::striders::remote_striders(
        session,
        (models, &mut *armor),
        textures,
        view,
    ));
    vertices.extend(crate::happy_ghasts::remote_happy_ghasts(
        session,
        (&mut *models, &mut *armor),
        textures,
        view,
    ));
    vertices.extend(crate::fish::remote_water_animals(
        session,
        (&mut *models, &mut *armor),
        textures,
        (view, &mut *held),
    ));
    vertices.extend(crate::copper_golems::remote_copper_golems(
        session,
        models,
        textures,
        (view, &mut *held),
    ));
    vertices.extend(crate::armor_stands::remote_armor_stands(
        session,
        (models, armor),
        textures,
        (view, held),
    ));
    vertices
}

/// The hostile mobs that are not humanoids: the spiders, creepers and endermen,
/// then the slimes, the magma cubes and the shulkers.
pub(super) fn remote_hostiles(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let mut vertices =
        crate::monsters::remote_monsters(session, models, textures, (view, &mut *held));
    vertices.extend(crate::cube_mobs::remote_cube_mobs(
        session, models, textures, view,
    ));
    vertices.extend(crate::cube_mobs::remote_sulfur_cubes(
        session,
        models,
        textures,
        (view, held),
    ));
    vertices.extend(crate::shulkers::remote_shulkers(
        session, models, textures, view,
    ));
    vertices
}
