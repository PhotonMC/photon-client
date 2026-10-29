use super::*;
use photon_world::EntityView;

/// The paths (under `textures/`) of the textures the monsters wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    MONSTERS
        .iter()
        .flat_map(|monster| {
            [
                Some(monster.texture),
                photon_render::render_types::eyes_texture(monster.type_name),
            ]
        })
        .flatten()
        .chain([POWER_TEXTURE, GHAST_SHOOTING_TEXTURE, VEX_CHARGING_TEXTURE])
        .chain(photon_world::mobs::parrot::VARIANT_TEXTURES)
        .chain([BREEZE_WIND.1, BREEZE_EYES.1])
        .chain([CREAKING_EYES.1, WITHER_ARMOR.1, WITHER_INVULNERABLE_TEXTURE])
        .chain([
            WARDEN_BIOLUMINESCENT.1,
            WARDEN_SPOTS_1,
            WARDEN_SPOTS_2,
            WARDEN_HEART.1,
        ])
}

/// The texture a monster wears now: a ghast's mouth open to charge, a vex charging,
/// a parrot's variant, else its own.
pub(super) fn texture_of(monster: &Monster, entity: EntityView<'_>, partial: f32) -> &'static str {
    match monster.pose {
        Pose::Ghast if entity.is_ghast_charging() => GHAST_SHOOTING_TEXTURE,
        Pose::Vex if entity.is_vex_charging() => VEX_CHARGING_TEXTURE,
        Pose::Wither if wither_is_invulnerable_texture(entity, partial) => {
            WITHER_INVULNERABLE_TEXTURE
        }
        Pose::Parrot => photon_world::mobs::parrot::variant_texture(entity.parrot_variant()),
        _ => monster.texture,
    }
}

/// `ItemInHandLayer` for a vex or an allay: each hand's stack, the right hand's
/// first, with the pose its model's `translateToHand` leaves (a left-handed mob's
/// main hand is its left).
pub(super) fn flier_hand_items(
    root: &Part,
    (monster, entity): (&Monster, EntityView<'_>),
    placed: &MobPlacement,
    held: &mut Vec<HeldItem>,
) {
    fn child<'a>(part: &'a Part, name: &str) -> Option<&'a Part> {
        part.children().iter().find(|child| child.name() == name)
    }
    let base = placement_pose(placed);
    let Some(inner) = child(root, "root") else {
        return;
    };
    let Some(body) = child(inner, "body") else {
        return;
    };
    let (main, off) = (
        entity.equipment(EquipmentSlot::MainHand),
        entity.equipment(EquipmentSlot::OffHand),
    );
    let (right, left) = if entity.main_arm_is_left() {
        (off, main)
    } else {
        (main, off)
    };
    for (side, place, name, stack) in [
        (Arm::Right, Place::Right, "right_arm", right),
        (Arm::Left, Place::Left, "left_arm", left),
    ] {
        let (Some(stack), Some(arm)) = (stack, child(body, name)) else {
            continue;
        };
        let pose = if monster.pose == Pose::Vex {
            vex_hand_pose(&base, (inner, body, arm), side)
        } else {
            // The allay tilts the item as its right arm is, whichever hand holds it.
            let Some(right_arm) = child(body, "right_arm") else {
                continue;
            };
            allay_hand_pose(&base, (inner, body, right_arm), side)
        };
        held.push(HeldItem::stack(stack, pose, place, placed.light));
    }
}

/// `BreezeRenderer.extractRenderState`: the animation states the breeze runs.
pub(super) fn breeze_state(entity: EntityView<'_>, partial: f32) -> BreezeRenderState {
    let own = entity.breeze().copied().unwrap_or_default();
    BreezeRenderState {
        age_in_ticks: entity.tick_count() as f32 + partial,
        idle: own.idle,
        shoot: own.shoot,
        slide: own.slide,
        slide_back: own.slide_back,
        inhale: own.inhale,
        long_jump: own.long_jump,
    }
}

/// `WardenRenderer.extractRenderState`: the look, the walk, the pulses and the
/// animation states the warden runs.
pub(super) fn warden_state(
    entity: EntityView<'_>,
    look: QuadrupedState,
    partial: f32,
) -> WardenRenderState {
    let own = entity.warden().copied().unwrap_or_default();
    WardenRenderState {
        quadruped: look,
        age_in_ticks: entity.tick_count() as f32 + partial,
        tendril_animation: own.tendril_animation(partial),
        roar: own.roar,
        sniff: own.sniff,
        emerge: own.emerge,
        digging: own.digging,
        attack: own.attack,
        sonic_boom: own.sonic_boom,
    }
}

/// A warden's glowing layers over its body, in the order the renderer adds
/// them (`LivingEntityEmissiveLayer`): the bioluminescent parts, the two
/// pulsating spots that take turns, the tendrils and the heart, each faded by
/// its own alpha and posed as the body is.
pub(super) fn warden_layers(
    models: &mut MobModels,
    textures: &EntityTextures,
    (entity, state, partial): (EntityView<'_>, &WardenRenderState, f32),
    placed: &MobPlacement,
    vertices: &mut Vec<Vertex>,
) {
    let own = entity.warden().copied().unwrap_or_default();
    let age = state.age_in_ticks;
    let pi = std::f64::consts::PI as f32;
    let spots = |phase: f32| {
        (photon_core::math::mth_cos_f64(f64::from(age * 0.045 + phase)) * 0.25).max(0.0)
    };
    let layers = [
        (WARDEN_BIOLUMINESCENT.0, WARDEN_BIOLUMINESCENT.1, 1.0),
        (WARDEN_SPOTS, WARDEN_SPOTS_1, spots(0.0)),
        (WARDEN_SPOTS, WARDEN_SPOTS_2, spots(pi)),
        (
            WARDEN_TENDRILS.0,
            WARDEN_TENDRILS.1,
            state.tendril_animation,
        ),
        (WARDEN_HEART.0, WARDEN_HEART.1, own.heart_animation(partial)),
    ];
    for (layer, texture, alpha) in layers {
        // `LivingEntityEmissiveLayer.submit`: nothing below this alpha.
        if alpha <= 1.0e-5 {
            continue;
        }
        let (Some(rect), Some(model)) = (textures.rect(texture), models.model(layer)) else {
            continue;
        };
        model.setup_warden(state);
        vertices.extend(mob_vertices_emissive_faded(model, placed, rect, alpha));
    }
}

/// `WitherBossRenderer.extractRenderState`: what is left of the invulnerable
/// time, less the partial tick, or zero.
pub(super) fn wither_invulnerable_ticks(entity: EntityView<'_>, partial: f32) -> f32 {
    let ticks = entity.wither().map_or(0, |own| own.invulnerable_ticks());
    if ticks > 0 {
        ticks as f32 - partial
    } else {
        0.0
    }
}

/// `WitherBossRenderer.getTextureLocation`: the pale texture while it is
/// invulnerable, flashing the other for a tick in five over the last 80.
pub(super) fn wither_is_invulnerable_texture(entity: EntityView<'_>, partial: f32) -> bool {
    let ticks = wither_invulnerable_ticks(entity, partial).floor() as i32;
    ticks > 0 && (ticks > 80 || ticks / 5 % 2 != 1)
}

/// `WitherBoss.isPowered`: half its health or less.
pub(super) fn wither_is_powered(entity: EntityView<'_>) -> bool {
    match (entity.health(), entity.max_health()) {
        (Some(health), Some(max)) => health <= max / 2.0,
        _ => false,
    }
}

/// `CreakingRenderer.extractRenderState`: the look, the walk, and the animation
/// states the creaking runs; what a torn-down one does to the renderer's own
/// state is in [`creaking_tear_down`].
pub(super) fn creaking_state(
    entity: EntityView<'_>,
    look: QuadrupedState,
    partial: f32,
) -> CreakingRenderState {
    let own = entity.creaking();
    CreakingRenderState {
        quadruped: look,
        age_in_ticks: entity.tick_count() as f32 + partial,
        can_move: own.is_none_or(|own| own.can_move()),
        attack: own.map(|own| own.attack).unwrap_or_default(),
        invulnerable: own.map(|own| own.invulnerable).unwrap_or_default(),
        death: own.map(|own| own.death).unwrap_or_default(),
    }
}

/// A torn-down creaking does not fall over or flash red.
pub(super) fn creaking_tear_down(entity: EntityView<'_>, placed: &mut MobPlacement) {
    if entity
        .creaking()
        .is_some_and(CreakingState::is_tearing_down)
    {
        placed.death_time = 0.0;
        placed.red_overlay = false;
    }
}

/// Whether a creaking's eyes are lit: the blink of a torn-down one
/// (`hasGlowingEyes`), else while it is active.
pub(super) fn creaking_eyes_glow(entity: EntityView<'_>) -> bool {
    entity.creaking().is_some_and(|own| {
        if own.is_tearing_down() {
            own.has_glowing_eyes()
        } else {
            own.is_active()
        }
    })
}

/// The layers some monsters draw over their body: a breeze's wind and eyes, a
/// warden's glowing parts.
pub(super) fn glowing_layers(
    models: &mut MobModels,
    textures: &EntityTextures,
    (pose, entity, look, partial): (Pose, EntityView<'_>, QuadrupedState, f32),
    (placed, glow, swirl): (&MobPlacement, &mut Vec<Vertex>, &mut Vec<Vertex>),
    vertices: &mut Vec<Vertex>,
) {
    match pose {
        Pose::Wither => {
            // `WitherArmorLayer`: the armor's swirl while it is hurt, posed as the body is.
            if wither_is_powered(entity)
                && let (Some(rect), Some(armor)) =
                    (textures.rect(WITHER_ARMOR.1), models.model(WITHER_ARMOR.0))
            {
                let age = entity.tick_count() as f32 + partial;
                let own = entity.wither().copied().unwrap_or_default();
                armor.setup_wither(&WitherRenderState {
                    age_in_ticks: age,
                    y_rot: look.head_y_rot,
                    x_rot: look.x_rot,
                    body_rot: placed.body_rot,
                    x_head_rots: own.x_rot_heads(),
                    y_head_rots: own.y_rot_heads(),
                });
                // `WitherArmorLayer.xOffset`.
                let x = photon_core::math::mth_cos_f64(f64::from(age * 0.02)) * 3.0 % 1.0;
                swirl.extend(mob_vertices_swirl(
                    armor,
                    placed,
                    rect,
                    [x, age * 0.01 % 1.0],
                    POWER_COLOUR as u32,
                ));
            }
        }
        Pose::Creaking => {
            // The eyes: the same posed model, drawn unlit when they glow.
            let state = creaking_state(entity, look, partial);
            if let (true, Some(rect), Some(model)) = (
                creaking_eyes_glow(entity),
                textures.rect(CREAKING_EYES.1),
                models.model(CREAKING_EYES.0),
            ) {
                model.setup_creaking(&state);
                glow.extend(mob_vertices_glow(model, placed, rect));
            }
        }
        Pose::Breeze => breeze_layers(models, textures, (entity, partial), placed, vertices),
        Pose::Warden => {
            let state = warden_state(entity, look, partial);
            warden_layers(
                models,
                textures,
                (entity, &state, partial),
                placed,
                vertices,
            );
        }
        _ => {}
    }
}

/// A breeze's wind and eyes over its body: the wind scrolled along its texture
/// and lit by the world, the eyes lit by faces only; both posed as the body is.
pub(super) fn breeze_layers(
    models: &mut MobModels,
    textures: &EntityTextures,
    (entity, partial): (EntityView<'_>, f32),
    placed: &MobPlacement,
    vertices: &mut Vec<Vertex>,
) {
    let state = breeze_state(entity, partial);
    // `BreezeWindLayer.xOffset`: a fiftieth of the age, within one texture.
    let scroll = (state.age_in_ticks * 0.02) % 1.0;
    for (layer, texture) in [BREEZE_WIND, BREEZE_EYES] {
        let (Some(rect), Some(model)) = (textures.rect(texture), models.model(layer)) else {
            continue;
        };
        model.setup_breeze(&state);
        vertices.extend(if layer == BREEZE_WIND.0 {
            mob_vertices_scrolled(model, placed, rect, scroll)
        } else {
            mob_vertices_emissive(model, placed, rect)
        });
    }
}

/// `ParrotModel.setupAnim`, posed by what the parrot is doing.
pub(super) fn pose_parrot(
    model: &mut MobModel,
    entity: EntityView<'_>,
    look: QuadrupedState,
    partial: f32,
) {
    let own = entity.parrot().copied();
    // `ParrotModel.getPose`: dancing, else sitting, else flying in the air.
    let pose = if own.is_some_and(|own| own.is_party()) {
        ParrotPose::Party
    } else if entity.is_sitting() {
        ParrotPose::Sitting
    } else if !entity.on_ground() {
        ParrotPose::Flying
    } else {
        ParrotPose::Standing
    };
    model.setup_parrot(&ParrotRenderState {
        quadruped: look,
        age_in_ticks: entity.tick_count() as f32 + partial,
        flap_angle: own.map_or(0.0, |own| own.flap_angle(partial)),
        pose,
    });
}

/// `WitherBossModel.setupAnim`, and what `WitherBossRenderer` adds to the pose: twice the
/// size (less while it is invulnerable), and a block light of fifteen.
pub(super) fn pose_wither(
    model: &mut MobModel,
    entity: EntityView<'_>,
    (look, snapshot, partial): (QuadrupedState, &EntitySnapshot, f32),
    placed: &mut MobPlacement,
) {
    let invulnerable = wither_invulnerable_ticks(entity, partial);
    let own = entity.wither().copied().unwrap_or_default();
    model.setup_wither(&WitherRenderState {
        age_in_ticks: entity.tick_count() as f32 + partial,
        y_rot: look.head_y_rot,
        x_rot: look.x_rot,
        body_rot: snapshot.body_yaw,
        x_head_rots: own.x_rot_heads(),
        y_head_rots: own.y_rot_heads(),
    });
    // `WitherBossRenderer.scale` and `getBlockLightLevel`.
    let scale = 2.0
        - if invulnerable > 0.0 {
            invulnerable / 220.0 * 0.5
        } else {
            0.0
        };
    placed.hook_scale = Some([scale; 3]);
    placed.light = (placed.light & !0xFFFF) | (15 << 4);
}
