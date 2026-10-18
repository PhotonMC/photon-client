import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/**
 * Test glue only: poses the original player model and records what it draws.
 *
 * <p>The model is baked from the reference's own player mesh, posed by the
 * reference's own {@code setupAnim} from a render state the scenario fills
 * in, and drawn through the reference's own part renderer into a consumer
 * that keeps each vertex. Parts are kept in a hash map, so the order they
 * draw in is not part of the contract: the quads are sorted before they are
 * written.
 */
public final class EntityOracle extends EntityWildOracle {
    public static JsonObject run(final JsonObject request) throws Exception {
        String operation = request.get("operation").getAsString();
        if (operation.equals("walk_animation")) {
            return walkAnimation(request);
        }
        if (operation.equals("model_layers")) {
            return modelLayers();
        }
        if (operation.equals("entity_types")) {
            return entityTypes();
        }
        if (operation.equals("experience_orb")) {
            return experienceOrb(request);
        }
        if (operation.equals("map_data")) {
            return mapData();
        }
        if (operation.equals("painting_variants")) {
            return paintingVariants();
        }
        if (operation.equals("wolf_variants")) {
            return wolfVariants();
        }
        if (operation.equals("entity_variants")) {
            return entityVariants();
        }
        if (operation.equals("entity_variant_fields")) {
            return entityVariantFields();
        }
        if (operation.equals("entity_data_serializers")) {
            return entityDataSerializers();
        }
        if (operation.equals("skeleton_model")) {
            return skeletonModel(request);
        }
        if (operation.equals("piglin_model")) {
            return piglinModel(request);
        }
        if (operation.equals("zombie_model")) {
            return zombieModel(request);
        }
        if (operation.equals("quadruped_model")) {
            return quadrupedModel(request);
        }
        if (operation.equals("villager_registries")) {
            return villagerRegistries();
        }
        if (operation.equals("villager_layers")) {
            return villagerLayers(request);
        }
        if (operation.equals("iron_golem_model")) {
            return ironGolemModel(request);
        }
        if (operation.equals("snow_golem_model")) {
            return snowGolemModel(request);
        }
        if (operation.equals("enderman_model")) {
            return endermanModel(request);
        }
        if (operation.equals("iron_golem_render")) {
            return ironGolemRender(request);
        }
        if (operation.equals("wolf_model")) {
            return wolfModel(request);
        }
        if (operation.equals("strider_model")) {
            return striderModel(request);
        }
        if (operation.equals("strider_textures")) {
            return striderTextures();
        }
        if (operation.equals("ravager_model")) {
            return ravagerModel(request);
        }
        if (operation.equals("panda_model")) {
            return pandaModel(request);
        }
        if (operation.equals("panda_textures")) {
            return pandaTextures();
        }
        if (operation.equals("setup_rotations")) {
            return setupRotations(request);
        }
        if (operation.equals("shaking_rotations")) {
            return shakingRotations(request);
        }
        if (operation.equals("fox_model")) {
            return foxModel(request);
        }
        if (operation.equals("fox_textures")) {
            return foxTextures();
        }
        if (operation.equals("hoglin_model")) {
            return hoglinModel(request);
        }
        if (operation.equals("polar_bear_model")) {
            return polarBearModel(request);
        }
        if (operation.equals("feline_model")) {
            return felineModel(request);
        }
        if (operation.equals("llama_model")) {
            return llamaModel(request);
        }
        if (operation.equals("camel_model")) {
            return camelModel(request);
        }
        if (operation.equals("equine_model")) {
            return equineModel(request);
        }
        if (operation.equals("swirl_layers")) {
            return swirlLayers(request);
        }
        if (operation.equals("segmented_model")) {
            return segmentedModel(request);
        }
        if (operation.equals("bee_model")) {
            return beeModel(request);
        }
        if (operation.equals("squid_model")) {
            return squidModel(request);
        }
        if (operation.equals("shulker_model")) {
            return shulkerModel(request);
        }
        if (operation.equals("magma_cube_model")) {
            return magmaCubeModel(request);
        }
        if (operation.equals("sea_animal_model")) {
            return seaAnimalModel(request);
        }
        if (operation.equals("axolotl_model")) {
            return axolotlModel(request);
        }
        if (operation.equals("boat_model")) {
            return boatModel(request);
        }
        if (operation.equals("boat_pose")) {
            return boatPose(request);
        }
        if (operation.equals("minecart_pose")) {
            return minecartPose(request);
        }
        if (operation.equals("display_light")) {
            return displayLight(request);
        }
        if (operation.equals("item_equippable")) {
            return itemEquippable(request);
        }
        if (operation.equals("text_display_submit")) {
            return textDisplaySubmit(request);
        }
        if (operation.equals("item_foil")) {
            return itemFoil(request);
        }
        if (operation.equals("glint_layers")) {
            return glintLayers(request);
        }
        if (operation.equals("end_flash_pose")) {
            return endFlashPose(request);
        }
        if (operation.equals("end_flash_script")) {
            return endFlashScript(request);
        }
        if (operation.equals("tick_rate_script")) {
            return tickRateScript(request);
        }
        if (operation.equals("end_sky_buffer")) {
            return endSkyBuffer(request);
        }
        if (operation.equals("item_activation_pose")) {
            return itemActivationPose(request);
        }
        if (operation.equals("camera_spin")) {
            return cameraSpin(request);
        }
        if (operation.equals("camera_hurt")) {
            return cameraHurt(request);
        }
        if (operation.equals("chest_model")) {
            return chestModel(request);
        }
        if (operation.equals("chest_pose")) {
            return chestPose(request);
        }
        if (operation.equals("banner_pose")) {
            return bannerPose(request);
        }
        if (operation.equals("elder_guardian_particle_pose")) {
            return elderGuardianParticlePose(request);
        }
        if (operation.equals("skull_block_pose")) {
            return skullBlockPose(request);
        }
        if (operation.equals("bell_pose")) {
            return bellPose(request);
        }
        if (operation.equals("beacon_beam")) {
            return beaconBeam(request);
        }
        if (operation.equals("brushable_pose")) {
            return brushablePose(request);
        }
        if (operation.equals("mob_effect_blend")) {
            return mobEffectBlend(request);
        }
        if (operation.equals("view_scales")) {
            return viewScales(request);
        }
        if (operation.equals("end_portal_cube")) {
            return endPortalCube(request);
        }
        if (operation.equals("copper_statue_pose")) {
            return copperStatuePose(request);
        }
        if (operation.equals("vault_pose")) {
            return vaultPose(request);
        }
        if (operation.equals("campfire_pose")) {
            return campfirePose(request);
        }
        if (operation.equals("conduit_pose")) {
            return conduitPose(request);
        }
        if (operation.equals("shelf_pose")) {
            return shelfPose(request);
        }
        if (operation.equals("gateway_beam")) {
            return gatewayBeam(request);
        }
        if (operation.equals("decorated_pot_pose")) {
            return decoratedPotPose(request);
        }
        if (operation.equals("lectern_pose")) {
            return lecternPose(request);
        }
        if (operation.equals("enchant_table_pose")) {
            return enchantTablePose(request);
        }
        if (operation.equals("shulker_box_pose")) {
            return shulkerBoxPose(request);
        }
        if (operation.equals("map_render")) {
            return mapRender(request);
        }
        if (operation.equals("display_pose")) {
            return displayPose(request);
        }
        if (operation.equals("item_frame_pose")) {
            return itemFramePose(request);
        }
        if (operation.equals("arrow_pose")) {
            return arrowPose(request);
        }
        if (operation.equals("small_projectile_pose")) {
            return smallProjectilePose(request);
        }
        if (operation.equals("screen_effect_quads")) {
            return screenEffectQuads(request);
        }
        if (operation.equals("painting_geometry")) {
            return paintingGeometry(request);
        }
        if (operation.equals("lightning_bolt")) {
            return lightningBolt(request);
        }
        if (operation.equals("primed_tnt_pose")) {
            return primedTntPose(request);
        }
        if (operation.equals("fishing_hook")) {
            return fishingHook(request);
        }
        if (operation.equals("leash_strip")) {
            return leashStrip(request);
        }
        if (operation.equals("dragon_fireball_quad")) {
            return dragonFireballQuad(request);
        }
        if (operation.equals("thrown_item_pose")) {
            return thrownItemPose(request);
        }
        if (operation.equals("flier_item_pose")) {
            return flierItemPose(request);
        }
        if (operation.equals("parrot_textures")) {
            return parrotTextures(request);
        }
        if (operation.equals("avatar_pose")) {
            return avatarPose(request);
        }
        if (operation.equals("elytra_model")) {
            return elytraModel(request);
        }
        if (operation.equals("airborne_model")) {
            return airborneModel(request);
        }
        if (operation.equals("guardian_beam")) {
            return guardianBeam(request);
        }
        if (operation.equals("dragon_rays")) {
            return dragonRays(request);
        }
        if (operation.equals("crystal_beam")) {
            return crystalBeam(request);
        }
        if (operation.equals("guardian_model")) {
            return guardianModel(request);
        }
        if (operation.equals("nautilus_model")) {
            return nautilusModel(request);
        }
        if (operation.equals("frog_model")) {
            return frogModel(request);
        }
        if (operation.equals("dolphin_item_pose")) {
            return dolphinItemPose(request);
        }
        if (operation.equals("fish_model")) {
            return fishModel(request);
        }
        if (operation.equals("fish_textures")) {
            return fishTextures(request);
        }
        if (operation.equals("glow_squid_light")) {
            return glowSquidLight(request);
        }
        if (operation.equals("copper_golem_layers")) {
            return copperGolemLayers(request);
        }
        if (operation.equals("copper_golem_model")) {
            return copperGolemModel(request);
        }
        if (operation.equals("rabbit_model")) {
            return rabbitModel(request);
        }
        if (operation.equals("animation_definitions")) {
            return animationDefinitionRows();
        }
        if (operation.equals("keyframe_animation")) {
            return keyframeAnimation(request);
        }
        if (operation.equals("animation_states")) {
            return animationStates(request);
        }
        if (operation.equals("entity_attributes")) {
            return entityAttributes();
        }
        if (operation.equals("villager_held_layers")) {
            return villagerHeldLayers(request);
        }
        if (operation.equals("goat_model")) {
            return goatModel(request);
        }
        if (operation.equals("eyes_layers")) {
            return eyesLayers(request);
        }
        if (operation.equals("entity_block_layers")) {
            return entityBlockLayers(request);
        }
        if (operation.equals("illager_model")) {
            return illagerModel(request);
        }
        if (operation.equals("witch_model")) {
            return witchModel(request);
        }
        if (operation.equals("villager_model")) {
            return villagerModel(request);
        }
        if (operation.equals("armor_stand_model")) {
            return armorStandModel(request);
        }
        if (operation.equals("equipment_layers")) {
            return equipmentLayers(request);
        }
        if (operation.equals("spider_model")) {
            return spiderModel(request, false);
        }
        if (operation.equals("creeper_model")) {
            return spiderModel(request, true);
        }
        if (operation.equals("creeper_render")) {
            return creeperRender(request);
        }
        if (operation.equals("sheep_model")) {
            return sheepModel(request);
        }
        if (operation.equals("chicken_model")) {
            return chickenModel(request);
        }
        if (operation.equals("sheep_colors")) {
            return sheepColors(request);
        }
        if (operation.equals("sheep_eat")) {
            return sheepEat(request);
        }
        if (operation.equals("held_item_pose")) {
            return heldItemPose(request);
        }
        if (operation.equals("spear_table")) {
            return spearTable();
        }
        if (operation.equals("item_entity_pose")) {
            return itemEntityPose(request);
        }
        if (operation.equals("living_pose")) {
            return livingPose(request);
        }
        if (!operation.equals("player_model")) {
            throw new IllegalArgumentException("Unknown entity operation: " + operation);
        }
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            boolean slim = row.get("slim").getAsBoolean();
            var mesh = net.minecraft.client.model.player.PlayerModel.createMesh(
                net.minecraft.client.model.geom.builders.CubeDeformation.NONE, slim);
            var root = net.minecraft.client.model.geom.builders.LayerDefinition.create(mesh, 64, 64).bakeRoot();
            var model = new net.minecraft.client.model.player.PlayerModel(root, slim);
            var state = new net.minecraft.client.renderer.entity.state.AvatarRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.attackTime = Oracle.f32(row, "attack_time");
            state.attackArm = row.get("attack_arm").getAsString().equals("left")
                ? net.minecraft.world.entity.HumanoidArm.LEFT
                : net.minecraft.world.entity.HumanoidArm.RIGHT;
            state.swimAmount = Oracle.f32(row, "swim_amount");
            state.isCrouching = row.get("crouching").getAsBoolean();
            state.isSpectator = row.get("spectator").getAsBoolean();
            model.setupAnim(state);
            var capture = new Capture();
            model.root().render(new com.mojang.blaze3d.vertex.PoseStack(), capture, 0, 0);
            var quads = new java.util.ArrayList<String>();
            for (int i = 0; i < capture.vertices.size(); i += 4) {
                quads.add(String.join("", capture.vertices.subList(i, i + 4)));
            }
            java.util.Collections.sort(quads);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var array = new JsonArray();
            quads.forEach(array::add);
            out.add("quads", array);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
