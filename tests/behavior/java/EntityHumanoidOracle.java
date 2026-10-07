import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityHumanoidOracle extends EntityHeldOracle {
    /**
     * A spider's models from the layer table (the spider and the cave spider), posed by the
     * reference's own `SpiderModel.setupAnim`, drawn; and a creeper's by `CreeperModel`.
     */
    static JsonObject spiderModel(final JsonObject request, final boolean creeper) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var root = bakedLayer(row.get("layer").getAsString());
            net.minecraft.client.renderer.entity.state.LivingEntityRenderState state = creeper
                ? new net.minecraft.client.renderer.entity.state.CreeperRenderState()
                : new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            if (creeper) {
                var model = new net.minecraft.client.model.monster.creeper.CreeperModel(root);
                model.setupAnim((net.minecraft.client.renderer.entity.state.CreeperRenderState) state);
                rows.add(posedQuads(row.get("label").getAsString(), model));
            } else {
                var model = new net.minecraft.client.model.monster.spider.SpiderModel(root);
                model.setupAnim(state);
                rows.add(posedQuads(row.get("label").getAsString(), model));
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What a creeper's renderer does to a swelling creeper: the scale its `scale` hook applies to
     * the pose, and how far it flashes white, from the reference's own `CreeperRenderer`.
     */
    static JsonObject creeperRender(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var renderer = (net.minecraft.client.renderer.entity.CreeperRenderer) UNSAFE.allocateInstance(
            net.minecraft.client.renderer.entity.CreeperRenderer.class);
        var scale = net.minecraft.client.renderer.entity.CreeperRenderer.class.getDeclaredMethod("scale",
            net.minecraft.client.renderer.entity.state.CreeperRenderState.class,
            com.mojang.blaze3d.vertex.PoseStack.class);
        scale.setAccessible(true);
        var overlay = net.minecraft.client.renderer.entity.CreeperRenderer.class.getDeclaredMethod(
            "getWhiteOverlayProgress", net.minecraft.client.renderer.entity.state.CreeperRenderState.class);
        overlay.setAccessible(true);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.entity.state.CreeperRenderState();
            state.swelling = Oracle.f32(row, "swelling");
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            scale.invoke(renderer, state, stack);
            var matrix = new float[16];
            stack.last().pose().get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("scale", described);
            out.addProperty("overlay", Oracle.bits((float) overlay.invoke(renderer, state)));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Every equipment asset of the game's resources through the reference's own codec: the layers of
     * each for the humanoid layer types, as the texture they name and what each is multiplied by
     * undyed and dyed (`EquipmentLayerRenderer.getColorForLayer`).
     */
    static JsonObject equipmentLayers(final JsonObject request) throws Exception {
        var jar = java.nio.file.Path.of(net.minecraft.client.Minecraft.class
            .getProtectionDomain().getCodeSource().getLocation().toURI());
        var folder = jar.getParent().getParent().resolve("assets/jar/assets/minecraft/equipment");
        var color = net.minecraft.client.renderer.entity.layers.EquipmentLayerRenderer.class.getDeclaredMethod(
            "getColorForLayer", net.minecraft.client.resources.model.EquipmentClientInfo.Layer.class, int.class);
        color.setAccessible(true);
        var names = new java.util.TreeSet<String>();
        try (var files = java.nio.file.Files.list(folder)) {
            files.forEach(file -> names.add(file.getFileName().toString()));
        }
        var rows = new JsonArray();
        int dyed = 0xFF336699;
        for (String file : names) {
            var json = com.google.gson.JsonParser.parseReader(java.nio.file.Files.newBufferedReader(folder.resolve(file)));
            var info = net.minecraft.client.resources.model.EquipmentClientInfo.CODEC
                .parse(com.mojang.serialization.JsonOps.INSTANCE, json).getOrThrow();
            var out = new JsonObject();
            out.addProperty("label", file);
            var types = new JsonObject();
            for (var type : new net.minecraft.client.resources.model.EquipmentClientInfo.LayerType[] {
                net.minecraft.client.resources.model.EquipmentClientInfo.LayerType.HUMANOID,
                net.minecraft.client.resources.model.EquipmentClientInfo.LayerType.HUMANOID_LEGGINGS,
                net.minecraft.client.resources.model.EquipmentClientInfo.LayerType.HUMANOID_BABY}) {
                var layers = new JsonArray();
                for (var layer : info.getLayers(type)) {
                    var described = new JsonObject();
                    described.addProperty("texture", layer.getTextureLocation(type).toString());
                    described.addProperty("undyed", (int) color.invoke(null, layer, 0));
                    described.addProperty("dyed", (int) color.invoke(null, layer, dyed));
                    described.addProperty("player_texture", layer.usePlayerTexture());
                    layers.add(described);
                }
                types.add(type.getSerializedName(), layers);
            }
            out.add("types", types);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * An armor stand's model, and the models of the armor it wears, from the layer table, posed by
     * the reference's own `ArmorStandModel`/`ArmorStandArmorModel.setupAnim`, drawn.
     */
    static JsonObject armorStandModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var state = new net.minecraft.client.renderer.entity.state.ArmorStandRenderState();
            state.yRot = Oracle.f32(row, "y_rot");
            state.showArms = row.get("show_arms").getAsBoolean();
            state.showBasePlate = row.get("show_base_plate").getAsBoolean();
            state.headPose = rotations(row.getAsJsonArray("head_pose"));
            state.bodyPose = rotations(row.getAsJsonArray("body_pose"));
            state.leftArmPose = rotations(row.getAsJsonArray("left_arm_pose"));
            state.rightArmPose = rotations(row.getAsJsonArray("right_arm_pose"));
            state.leftLegPose = rotations(row.getAsJsonArray("left_leg_pose"));
            state.rightLegPose = rotations(row.getAsJsonArray("right_leg_pose"));
            net.minecraft.client.model.object.armorstand.ArmorStandArmorModel model = id.endsWith("#main")
                ? new net.minecraft.client.model.object.armorstand.ArmorStandModel(root)
                : new net.minecraft.client.model.object.armorstand.ArmorStandArmorModel(root);
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Where the real `AvatarRenderer` puts a player: the feet, the body's turn (death, gliding, swimming
     * included) from its own `setupRotations`, then the flip and the scale `LivingEntityRenderer.submit`
     * ends with; the matrix the model is drawn from.
     */
    static JsonObject avatarPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.player.AvatarRenderer.class);
        var rotations = net.minecraft.client.renderer.entity.player.AvatarRenderer.class.getDeclaredMethod(
            "setupRotations", net.minecraft.client.renderer.entity.state.AvatarRenderState.class,
            com.mojang.blaze3d.vertex.PoseStack.class, float.class, float.class);
        rotations.setAccessible(true);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.entity.state.AvatarRenderState();
            float bodyRot = Oracle.f32(row, "body_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.swimAmount = Oracle.f32(row, "swim_amount");
            state.isInWater = row.get("in_water").getAsBoolean();
            state.isVisuallySwimming = row.get("visually_swimming").getAsBoolean();
            state.isCrouching = row.get("crouching").getAsBoolean();
            state.deathTime = Oracle.f32(row, "death_time");
            state.isFallFlying = row.get("fall_flying").getAsBoolean();
            state.isAutoSpinAttack = row.get("spin_attack").getAsBoolean();
            state.fallFlyingTimeInTicks = Oracle.f32(row, "fall_flying_ticks");
            state.shouldApplyFlyingYRot = row.get("apply_flying_y_rot").getAsBoolean();
            state.flyingYRot = Oracle.f32(row, "flying_y_rot");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            // `EntityRenderDispatcher.submit`: where the feet are, a crouching player a little lower.
            stack.translate(Oracle.f32(row, "feet_x"),
                Oracle.f32(row, "feet_y") + (state.isCrouching ? -2.0F / 16.0F : 0.0F), Oracle.f32(row, "feet_z"));
            stack.pushPose();
            stack.scale(1.0F, 1.0F, 1.0F);
            rotations.invoke(renderer, state, stack, bodyRot, 1.0F);
            stack.scale(-1.0F, -1.0F, 1.0F);
            stack.scale(0.9375F, 0.9375F, 0.9375F);
            stack.translate(0.0F, -1.501F, 0.0F);
            var matrix = new float[16];
            stack.last().pose().get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("pose", described);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The elytra models posed by the real `ElytraModel.setupAnim`, drawn. */
    static JsonObject elytraModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.object.equipment.ElytraModel(
                bakedLayer(row.get("layer").getAsString()));
            var state = new net.minecraft.client.renderer.entity.state.HumanoidRenderState();
            state.isCrouching = row.get("crouching").getAsBoolean();
            state.elytraRotX = Oracle.f32(row, "rot_x");
            state.elytraRotY = Oracle.f32(row, "rot_y");
            state.elytraRotZ = Oracle.f32(row, "rot_z");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A skeleton family model posed by `SkeletonModel.setupAnim`, drawn. */
    static JsonObject skeletonModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var state = new net.minecraft.client.renderer.entity.state.SkeletonRenderState();
            humanoidFields(state, row);
            state.isAggressive = row.get("aggressive").getAsBoolean();
            state.isHoldingBow = row.get("holding_bow").getAsBoolean();
            net.minecraft.client.model.monster.skeleton.SkeletonModel<
                net.minecraft.client.renderer.entity.state.SkeletonRenderState> model =
                new net.minecraft.client.model.monster.skeleton.SkeletonModel<>(root);
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A piglin family model posed by `PiglinModel`/`ZombifiedPiglinModel.setupAnim`, drawn. */
    static JsonObject piglinModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            boolean baby = id.contains("_baby");
            net.minecraft.client.model.Model<?> model;
            if (id.startsWith("minecraft:zombified_piglin")) {
                var state = new net.minecraft.client.renderer.entity.state.ZombifiedPiglinRenderState();
                humanoidFields(state, row);
                state.isAggressive = row.get("aggressive").getAsBoolean();
                state.isBaby = baby;
                var zombified = baby
                    ? new net.minecraft.client.model.monster.piglin.BabyZombifiedPiglinModel(root)
                    : new net.minecraft.client.model.monster.piglin.AdultZombifiedPiglinModel(root);
                zombified.setupAnim(state);
                model = zombified;
            } else {
                var state = new net.minecraft.client.renderer.entity.state.PiglinRenderState();
                humanoidFields(state, row);
                state.isBaby = baby;
                state.armPose = net.minecraft.world.entity.monster.piglin.PiglinArmPose.valueOf(
                    row.get("arm_pose").getAsString().toUpperCase());
                state.mainArm = row.get("main_arm").getAsString().equals("left")
                    ? net.minecraft.world.entity.HumanoidArm.LEFT : net.minecraft.world.entity.HumanoidArm.RIGHT;
                state.ticksUsingItem = Oracle.f32(row, "ticks_using_item");
                state.maxCrossbowChageDuration = Oracle.f32(row, "max_crossbow_charge");
                var piglin = baby
                    ? new net.minecraft.client.model.monster.piglin.BabyPiglinModel(root)
                    : new net.minecraft.client.model.monster.piglin.AdultPiglinModel(root);
                piglin.setupAnim(state);
                model = piglin;
            }
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A zombie family model from the layer table, posed by the reference's own
     * `setupAnim` (humanoid pose, held-out arms, the drowned's swim), drawn.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject zombieModel(final JsonObject request) throws Exception {
        Oracle.vanillaRegistries();
        var layers = new java.util.HashMap<String, net.minecraft.client.model.geom.builders.LayerDefinition>();
        for (var entry : net.minecraft.client.model.geom.LayerDefinitions.createRoots().entrySet()) {
            layers.put(entry.getKey().toString(), entry.getValue());
        }
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = layers.get(id).bakeRoot();
            net.minecraft.client.model.HumanoidModel model;
            boolean villager = id.startsWith("minecraft:zombie_villager");
            if (villager && id.contains("_baby")) {
                model = new net.minecraft.client.model.monster.zombie.BabyZombieVillagerModel(root);
            } else if (villager) {
                model = new net.minecraft.client.model.monster.zombie.ZombieVillagerModel(root);
            } else if (id.startsWith("minecraft:drowned_baby")) {
                model = new net.minecraft.client.model.monster.zombie.BabyDrownedModel(root);
            } else if (id.startsWith("minecraft:drowned")) {
                model = new net.minecraft.client.model.monster.zombie.DrownedModel(root);
            } else if (id.startsWith("minecraft:giant")) {
                model = new net.minecraft.client.model.monster.zombie.GiantZombieModel(root);
            } else if (id.contains("_baby")) {
                model = new net.minecraft.client.model.monster.zombie.BabyZombieModel<>(root);
            } else {
                model = new net.minecraft.client.model.monster.zombie.ZombieModel<>(root);
            }
            net.minecraft.client.renderer.entity.state.ZombieRenderState state = villager
                ? new net.minecraft.client.renderer.entity.state.ZombieVillagerRenderState()
                : new net.minecraft.client.renderer.entity.state.ZombieRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.attackTime = Oracle.f32(row, "attack_time");
            state.swimAmount = Oracle.f32(row, "swim_amount");
            state.isAggressive = row.get("aggressive").getAsBoolean();
            state.isBaby = id.contains("_baby");
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
