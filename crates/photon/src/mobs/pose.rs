use super::*;
use crate::equipment::arm_poses;
use crate::mobs::humanoids::villagers;

/// `LivingEntityRenderer` for the humanoid mobs the server added (zombies,
/// skeletons, piglins), each posed by its model's `setupAnim` from where it is
/// between two ticks, with what it holds in its hands. The attack swing and
/// the arm poses the held items ask for (a piglin's weapon or crossbow, a
/// spear) are not read yet.
pub(super) fn remote_humanoids(
    session: &PlaySession,
    (models, armor): (&mut MobModels, &mut Armor),
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.drawn_mobs() {
        let Some(kind) = models.humanoids.get(&entity.type_id).copied() else {
            continue;
        };
        let baby = entity.is_baby();
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let (humanoid, holding_bow, arm_pose) =
            humanoid_state(session, entity, &snapshot, (kind, partial));
        let aggressive = entity.is_aggressive();
        let placement = placement(world, &snapshot, view, kind.scale * entity.scale());
        // What it wears, in the armor models of its build, posed as it is.
        let worn = Worn {
            head: entity.equipment(EquipmentSlot::Head),
            chest: entity.equipment(EquipmentSlot::Chest),
            legs: entity.equipment(EquipmentSlot::Legs),
            feet: entity.equipment(EquipmentSlot::Feet),
        };
        let mut first = true;
        let mut skull = None;
        for (layer, texture) in model_layers(kind, baby).into_iter().flatten() {
            let (Some(rect), Some(model)) = (textures.rect(texture), models.model(layer)) else {
                continue;
            };
            pose_humanoid(
                model,
                kind,
                (humanoid, aggressive, holding_bow),
                (entity, arm_pose),
            );
            if first {
                first = false;
                let hands = Hands {
                    base: &placement_pose(&placement),
                    root: model.root(),
                    hand_shift: if matches!(kind.family, Family::Skeleton) {
                        1.0
                    } else {
                        0.0
                    },
                    baby,
                    light: placement.light,
                    right: entity.item_held_by_arm(false),
                    left: entity.item_held_by_arm(true),
                    head: worn.head_item(armor),
                    // `PiglinRenderer.PIGLIN_CUSTOM_HEAD_TRANSFORMS`.
                    head_transforms: if matches!(kind.family, Family::Piglin { .. }) {
                        (0.0, 1.0019531)
                    } else {
                        HEAD_TRANSFORMS
                    },
                    eye: None,
                    motions: arm_poses::entity_actions(
                        entity,
                        (humanoid.attack_time, humanoid.attack_arm),
                        (humanoid.right_arm_pose, humanoid.left_arm_pose),
                        (world.game_time(), partial),
                    ),
                };
                hands.collect(held);
                skull = hands.skull();
            }
            vertices.extend(visible_body(entity, model, (&placement, rect)));
        }
        vertices.extend(armor.skull_vertices(
            skull,
            snapshot.walk_position,
            textures,
            (placement.light, snapshot.red_overlay),
        ));
        vertices.extend(worn_layers(
            armor,
            (worn, textures),
            (entity, kind, baby),
            (humanoid, aggressive, holding_bow, arm_pose),
            (&placement, partial),
        ));
        profession_layers(
            (models, textures),
            kind,
            (entity, baby),
            (humanoid, aggressive, holding_bow, arm_pose),
            &placement,
            vertices,
        );
    }
}

/// What a humanoid mob's model is posed from: its walk, arms and use, whether it holds a bow, and a
/// piglin's arm pose.
pub(super) fn humanoid_state(
    session: &PlaySession,
    entity: &photon_world::ClientEntity,
    snapshot: &photon_world::EntitySnapshot,
    (kind, partial): (&HumanoidKind, f32),
) -> (PlayerRenderState, bool, PiglinArmPose) {
    let (mut humanoid, holding_bow) = humanoid_pose(entity, snapshot, kind, partial);
    humanoid.ticks_using_item = entity.ticks_using_item(partial);
    humanoid.max_crossbow_charge_duration = arm_poses::charge_duration_of_use(session, entity);
    (
        humanoid,
        holding_bow,
        piglin_arm_pose(session, entity, kind),
    )
}

/// What a humanoid mob wears over its model: its armor and its wings, posed as it is.
pub(super) fn worn_layers(
    armor: &mut Armor,
    (worn, textures): (Worn, &EntityTextures),
    (entity, kind, baby): (&photon_world::ClientEntity, &HumanoidKind, bool),
    (humanoid, aggressive, holding_bow, arm_pose): (PlayerRenderState, bool, bool, PiglinArmPose),
    (placement, partial): (&MobPlacement, f32),
) -> Vec<Vertex> {
    let layer = if baby {
        kind.baby_layer
    } else {
        kind.adult_layer
    };
    let prefix = layer.strip_suffix("#main").unwrap_or(layer);
    let mut vertices = armor.vertices(
        worn,
        (prefix, baby),
        textures,
        &|model| {
            pose_humanoid(
                model,
                kind,
                (humanoid, aggressive, holding_bow),
                (entity, arm_pose),
            );
        },
        &|model, rect, colour| mob_vertices_coloured(model, placement, rect, colour),
    );
    // `WingsLayer`: an elytra on the chest, grown or baby.
    vertices.extend(armor.wings(
        worn.chest,
        (
            baby,
            entity.is_crouching(),
            entity.elytra_rotations(partial),
        ),
        textures,
        &|model, rect, colour| mob_vertices_coloured(model, placement, rect, colour),
    ));
    super::outline::record_worn(entity, vertices)
}

/// A villager's profession and level layers over its model, posed as it is.
pub(super) fn profession_layers(
    (models, textures): (&mut MobModels, &EntityTextures),
    kind: &HumanoidKind,
    (entity, baby): (&photon_world::ClientEntity, bool),
    (humanoid, aggressive, holding_bow, arm_pose): (PlayerRenderState, bool, bool, PiglinArmPose),
    placement: &MobPlacement,
    vertices: &mut Vec<Vertex>,
) {
    // `VillagerProfessionLayer` draws nothing over an invisible body.
    let Some(folder) = kind.profession_folder.filter(|_| !entity.is_invisible()) else {
        return;
    };
    for pass in villagers::profession_layer(entity, folder, baby) {
        let id = villagers::layer(folder, pass.hatless, baby);
        let (Some(rect), Some(model)) = (textures.rect(&pass.texture), models.model(&id)) else {
            continue;
        };
        pose_humanoid(
            model,
            kind,
            (humanoid, aggressive, holding_bow),
            (entity, arm_pose),
        );
        vertices.extend(mob_vertices(model, placement, rect));
    }
}

/// What a humanoid mob's model is posed from, and whether its main hand holds
/// a bow: the humanoid pose from where it is between two ticks, and the
/// arms' poses its held items ask for (`AbstractSkeletonRenderer.getArmPose`:
/// a bow drawn by the main arm of an aggressive skeleton).
pub(super) fn humanoid_pose(
    entity: &photon_world::ClientEntity,
    snapshot: &photon_world::EntitySnapshot,
    kind: &HumanoidKind,
    partial: f32,
) -> (PlayerRenderState, bool) {
    let main_left = entity.main_arm_is_left();
    let holding_bow = entity
        .equipment(EquipmentSlot::MainHand)
        .is_some_and(|stack| is_item(stack, "minecraft:bow"));
    // `getArmPose` of each renderer: a skeleton's drawn bow, else a spear's pose (a zombie's
    // also for the arm beside a spear).
    let arm_pose = |left: bool| {
        if matches!(kind.family, Family::Skeleton)
            && left == main_left
            && entity.is_aggressive()
            && holding_bow
        {
            ArmPose::BowAndArrow
        } else {
            arm_poses::mob_spear_pose(entity, left, matches!(kind.family, Family::Zombie { .. }))
        }
    };
    let attack_arm = if snapshot.attack_arm_left {
        Arm::Left
    } else {
        Arm::Right
    };
    let (right_arm_pose, left_arm_pose) = (arm_pose(false), arm_pose(true));
    let spears = arm_poses::entity_spear_view(
        entity,
        (snapshot.attack_time, attack_arm),
        (right_arm_pose, left_arm_pose),
        (0, partial),
    );
    let humanoid = PlayerRenderState {
        head_y_rot: photon_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
        x_rot: snapshot.pitch,
        walk_position: snapshot.walk_position,
        walk_speed: snapshot.walk_speed,
        age_in_ticks: entity.tick_count() as f32 + partial,
        swim_amount: snapshot.swim_amount,
        attack_time: snapshot.attack_time,
        attack_arm,
        right_arm_pose,
        left_arm_pose,
        main_arm: if main_left { Arm::Left } else { Arm::Right },
        is_baby: entity.is_baby(),
        main_hand_empty: entity.item_held_by_arm(main_left).is_none(),
        is_using_item: entity.is_using_item(),
        use_item_main_hand: !entity.used_item_is_off_hand(),
        swing_animation: spears.swing_animation,
        right_spear: spears.right,
        left_spear: spears.left,
        ..Default::default()
    };
    (humanoid, holding_bow)
}

/// The model layers and textures a humanoid is drawn in: its own, and the
/// outer one (a stray's clothes) when it has one.
pub(super) fn model_layers(
    kind: &HumanoidKind,
    baby: bool,
) -> [Option<(&'static str, &'static str)>; 2] {
    [
        Some(if baby {
            (kind.baby_layer, kind.baby_texture)
        } else {
            (kind.adult_layer, kind.adult_texture)
        }),
        kind.outer.map(|outer| {
            if baby {
                (outer[1], outer[3])
            } else {
                (outer[0], outer[2])
            }
        }),
    ]
}

/// `Piglin.getArmPose` and `PiglinBrute.getArmPose`: what the arms do, by what is held and what
/// the piglin is doing.
pub(super) fn piglin_arm_pose(
    session: &PlaySession,
    entity: &photon_world::ClientEntity,
    kind: &HumanoidKind,
) -> PiglinArmPose {
    let Family::Piglin { zombified: false } = kind.family else {
        return PiglinArmPose::Default;
    };
    let main = entity.equipment(EquipmentSlot::MainHand);
    let off = entity.equipment(EquipmentSlot::OffHand);
    let melee = entity.is_aggressive()
        && main.is_some_and(|stack| {
            photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
                .has(photon_data::item::component_types::TOOL)
        });
    if photon_data::entity_type::name(entity.type_id) == Some("minecraft:piglin_brute") {
        return if melee {
            PiglinArmPose::AttackingWithMeleeWeapon
        } else {
            PiglinArmPose::Default
        };
    }
    if entity.is_dancing() {
        PiglinArmPose::Dancing
    } else if off.is_some_and(|stack| {
        session
            .world()
            .item_in_tag(stack.item_registry_id().0, "minecraft:piglin_loved")
    }) {
        PiglinArmPose::AdmiringItem
    } else if melee {
        PiglinArmPose::AttackingWithMeleeWeapon
    } else if entity.is_charging_crossbow() {
        PiglinArmPose::CrossbowCharge
    } else if [main, off]
        .into_iter()
        .flatten()
        .any(|stack| is_item(stack, "minecraft:crossbow"))
        && main.is_some_and(arm_poses::is_charged_crossbow)
    {
        PiglinArmPose::CrossbowHold
    } else {
        PiglinArmPose::Default
    }
}

/// The model's `setupAnim` of the family.
pub(super) fn pose_humanoid(
    model: &mut MobModel,
    kind: &HumanoidKind,
    (humanoid, aggressive, holding_bow): (PlayerRenderState, bool, bool),
    (entity, arm_pose): (&photon_world::ClientEntity, PiglinArmPose),
) {
    match kind.family {
        Family::Zombie { drowned } => model.setup_zombie(&ZombieState {
            humanoid,
            aggressive,
            drowned,
        }),
        Family::Skeleton => model.setup_skeleton(&SkeletonState {
            humanoid,
            aggressive,
            holding_bow,
        }),
        Family::Piglin { zombified } => model.setup_piglin(&PiglinState {
            humanoid,
            aggressive,
            zombified,
            baby: entity.is_baby(),
            arm_pose,
        }),
    }
}

/// Whether a stack is of the item with this name.
pub(crate) fn is_item(stack: &photon_world::ClientItemStackState, name: &str) -> bool {
    photon_data::item::registry::index(name)
        .is_some_and(|index| usize::try_from(stack.item_registry_id().0) == Ok(index))
}
