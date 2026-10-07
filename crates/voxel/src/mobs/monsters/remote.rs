use super::*;

/// The model's `setupAnim` of the monster, and what its renderer adds to the
/// pose (a swelling creeper's scale).
pub(super) fn pose_monster(
    model: &mut MobModel,
    monster: &Monster,
    entity: &ClientEntity,
    (look, snapshot, partial): (QuadrupedState, &EntitySnapshot, f32),
    placed: &mut MobPlacement,
) {
    match monster.pose {
        Pose::Spider => model.setup_spider(&look),
        Pose::Segmented(silverfish) => {
            model.setup_segmented(entity.tick_count() as f32 + partial, silverfish);
        }
        Pose::Enderman => model.setup_enderman(&EndermanState {
            humanoid: PlayerRenderState {
                head_y_rot: look.head_y_rot,
                x_rot: look.x_rot,
                walk_position: look.walk_position,
                walk_speed: look.walk_speed,
                age_in_ticks: entity.tick_count() as f32 + partial,
                swim_amount: snapshot.swim_amount,
                attack_time: snapshot.attack_time,
                ..Default::default()
            },
            carrying: entity.is_carrying_block(),
            creepy: entity.is_creepy(),
        }),
        Pose::Blaze => {
            model.setup_blaze(
                entity.tick_count() as f32 + partial,
                look.head_y_rot,
                look.x_rot,
            );
            // `BlazeRenderer.getBlockLightLevel`: fifteen, whatever the blocks say.
            placed.light = (placed.light & !0xFFFF) | (15 << 4);
        }
        Pose::Ghast => model.setup_ghast(entity.tick_count() as f32 + partial),
        Pose::Bat => {
            let own = entity.bat().copied().unwrap_or_default();
            model.setup_bat(&BatRenderState {
                head_y_rot: look.head_y_rot,
                age_in_ticks: entity.tick_count() as f32 + partial,
                is_resting: entity.is_bat_resting(),
                fly: own.fly,
                rest: own.rest,
            });
        }
        Pose::Vex => {
            // `ArmedEntityRenderState`: the right hand is the main one for a
            // right-handed mob.
            let (main, off) = (
                entity.equipment(EquipmentSlot::MainHand).is_some(),
                entity.equipment(EquipmentSlot::OffHand).is_some(),
            );
            let left_handed = entity.main_arm_is_left();
            model.setup_vex(&VexRenderState {
                head_y_rot: look.head_y_rot,
                x_rot: look.x_rot,
                age_in_ticks: entity.tick_count() as f32 + partial,
                is_charging: entity.is_vex_charging(),
                right_hand_item: if left_handed { off } else { main },
                left_hand_item: if left_handed { main } else { off },
            });
            // `VexRenderer.getBlockLightLevel`.
            placed.light = (placed.light & !0xFFFF) | (15 << 4);
        }
        Pose::Breeze => model.setup_breeze(&breeze_state(entity, partial)),
        Pose::Warden => model.setup_warden(&warden_state(entity, look, partial)),
        Pose::Wither => pose_wither(model, entity, (look, snapshot, partial), placed),
        Pose::Creaking => {
            model.setup_creaking(&creaking_state(entity, look, partial));
            creaking_tear_down(entity, placed);
        }
        Pose::Parrot => pose_parrot(model, entity, look, partial),
        Pose::Allay => {
            let own = entity.allay().copied().unwrap_or_default();
            model.setup_allay(&AllayRenderState {
                quadruped: look,
                age_in_ticks: entity.tick_count() as f32 + partial,
                is_dancing: entity.is_allay_dancing(),
                is_spinning: own.is_spinning(),
                spinning_progress: own.spinning_progress(partial),
                holding_animation_progress: own.holding_item_progress(partial),
            });
            // `AllayRenderer.getBlockLightLevel`.
            placed.light = (placed.light & !0xFFFF) | (15 << 4);
        }
        Pose::Phantom => {
            let age_in_ticks = entity.tick_count() as f32 + partial;
            model.setup_phantom((entity.id() * 3) as f32 + age_in_ticks);
            // `PhantomRenderer.scale` and `setupRotations`: it grows with its size,
            // is lifted, and tips with its pitch.
            let scale = 1.0 + 0.15 * entity.phantom_size() as f32;
            placed.hook_scale = Some([scale; 3]);
            placed.hook_translate = Some([0.0, 1.3125, 0.1875]);
            placed.pitch = Some(snapshot.pitch);
        }
        Pose::Creeper => {
            let swelling = entity.creeper_swelling(partial);
            model.setup_creeper(&look);
            placed.hook_scale = Some(creeper_scale(swelling));
            placed.white_overlay = creeper_white_overlay(swelling);
        }
    }
}

/// `LivingEntityRenderer` for the spiders, creepers and endermen the server
/// added, with the block an enderman carries. The spiders' and the enderman's
/// glowing eyes and the charged creeper's swirl are not drawn yet.
pub fn remote_monsters(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let ids: Vec<(i32, &Monster)> = MONSTERS
        .iter()
        .filter_map(|monster| Some((voxel_data::entity_type::id(monster.type_name)?, monster)))
        .collect();
    let mut vertices = Vec::new();
    let mut glow = Vec::new();
    let mut swirl = Vec::new();
    for entity in world.entities().all() {
        let Some(monster) = ids
            .iter()
            .find(|(id, _)| *id == entity.type_id)
            .map(|(_, monster)| monster)
        else {
            continue;
        };
        // `EndermanRenderer.getRenderOffset`: a creepy enderman shakes, by a
        // little in x and in z (two draws of the renderer's random source).
        let shake = (monster.pose == Pose::Enderman && entity.is_creepy()).then(|| {
            let spread = 0.02;
            [
                models.shake.next_gaussian() * spread,
                models.shake.next_gaussian() * spread,
            ]
        });
        let texture = texture_of(monster, entity, partial);
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
            models.model(monster.layer),
        ) else {
            continue;
        };
        let look = QuadrupedState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        };
        let mut placed = placement(world, &snapshot, view, entity.scale());
        placed.flip_degrees = monster.flip_degrees;
        if let Some([x, z]) = shake {
            placed.feet[0] += x as f32;
            placed.feet[2] += z as f32;
        }
        pose_monster(
            model,
            monster,
            entity,
            (look, &snapshot, partial),
            &mut placed,
        );
        vertices.extend(mob_vertices(model, &placed, rect));
        if matches!(monster.pose, Pose::Vex | Pose::Allay) {
            flier_hand_items(model.root(), (monster, entity), &placed, held);
        }
        // The eyes, over the model as it is posed, after the other layers.
        if let Some(eyes) = voxel_render::render_types::eyes_texture(monster.type_name)
            .and_then(|eyes| textures.rect(eyes))
        {
            glow.extend(mob_vertices_glow(model, &placed, eyes));
        }
        vertices.extend(glowing_layers(
            models,
            textures,
            (monster.pose, entity, look, partial),
            (&placed, &mut glow, &mut swirl),
        ));
        // A charged creeper's swirl: the armor model, posed as the creeper is.
        if monster.pose == Pose::Creeper
            && entity.is_powered()
            && let (Some(rect), Some(armor)) = (
                textures.rect(POWER_TEXTURE),
                models.model("minecraft:creeper#armor"),
            )
        {
            armor.setup_creeper(&look);
            swirl.extend(mob_vertices_swirl(
                armor,
                &placed,
                rect,
                swirl_offset(entity.tick_count() as f32 + partial),
                POWER_COLOUR as u32,
            ));
        }
        if let Some(state) = entity
            .carried_block_state()
            .filter(|_| monster.pose == Pose::Enderman)
        {
            held.push(HeldItem::block(
                state,
                carried_block_pose(&placement_pose(&placed)),
                placed.light,
                0.0,
            ));
        }
    }
    models.glow.extend(glow);
    models.swirl.extend(swirl);
    vertices
}
