import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityHeldOracle extends EntityFlyerOracle {
    /**
     * A humanoid model posed with its arms' poses, and where the reference's own `ItemInHandLayer`
     * puts the item each hand holds: the quads of the posed model and the pose (a matrix) each held
     * item is submitted with, before its own display transform.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject heldItemPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String kind = row.get("kind").getAsString();
            boolean baby = row.get("baby").getAsBoolean();
            net.minecraft.client.model.HumanoidModel model;
            net.minecraft.client.renderer.entity.state.HumanoidRenderState state;
            switch (kind) {
                case "player", "player_slim" -> {
                    boolean slim = kind.equals("player_slim");
                    var mesh = net.minecraft.client.model.player.PlayerModel.createMesh(
                        net.minecraft.client.model.geom.builders.CubeDeformation.NONE, slim);
                    var root = net.minecraft.client.model.geom.builders.LayerDefinition.create(mesh, 64, 64).bakeRoot();
                    model = new net.minecraft.client.model.player.PlayerModel(root, slim);
                    state = new net.minecraft.client.renderer.entity.state.AvatarRenderState();
                }
                case "zombie" -> {
                    var root = bakedLayer(baby ? "minecraft:zombie_baby#main" : "minecraft:zombie#main");
                    model = baby
                        ? new net.minecraft.client.model.monster.zombie.BabyZombieModel<>(root)
                        : new net.minecraft.client.model.monster.zombie.ZombieModel<>(root);
                    var zombie = new net.minecraft.client.renderer.entity.state.ZombieRenderState();
                    zombie.isAggressive = row.get("aggressive").getAsBoolean();
                    state = zombie;
                }
                case "skeleton" -> {
                    model = new net.minecraft.client.model.monster.skeleton.SkeletonModel<>(
                        bakedLayer("minecraft:skeleton#main"));
                    var skeleton = new net.minecraft.client.renderer.entity.state.SkeletonRenderState();
                    skeleton.isAggressive = row.get("aggressive").getAsBoolean();
                    skeleton.isHoldingBow = row.get("holding_bow").getAsBoolean();
                    state = skeleton;
                }
                case "piglin" -> {
                    var root = bakedLayer(baby ? "minecraft:piglin_baby#main" : "minecraft:piglin#main");
                    model = baby
                        ? new net.minecraft.client.model.monster.piglin.BabyPiglinModel(root)
                        : new net.minecraft.client.model.monster.piglin.AdultPiglinModel(root);
                    state = new net.minecraft.client.renderer.entity.state.PiglinRenderState();
                }
                default -> throw new IllegalArgumentException("Unknown humanoid " + kind);
            }
            humanoidFields(state, row);
            state.isBaby = baby;
            state.isCrouching = row.get("crouching").getAsBoolean();
            state.isUsingItem = row.get("using_item").getAsBoolean();
            state.useItemHand = row.get("use_hand").getAsString().equals("off")
                ? net.minecraft.world.InteractionHand.OFF_HAND : net.minecraft.world.InteractionHand.MAIN_HAND;
            state.isPassenger = row.get("passenger").getAsBoolean();
            state.isFallFlying = row.get("fall_flying").getAsBoolean();
            state.speedValue = Oracle.f32(row, "speed_value");
            state.ticksUsingItem = Oracle.f32(row, "ticks_using_item");
            state.maxCrossbowChargeDuration = Oracle.f32(row, "max_crossbow_charge");
            state.mainArm = row.get("main_arm").getAsString().equals("left")
                ? net.minecraft.world.entity.HumanoidArm.LEFT
                : net.minecraft.world.entity.HumanoidArm.RIGHT;
            state.rightArmPose = net.minecraft.client.model.HumanoidModel.ArmPose.valueOf(
                row.get("right_pose").getAsString().toUpperCase());
            state.leftArmPose = net.minecraft.client.model.HumanoidModel.ArmPose.valueOf(
                row.get("left_pose").getAsString().toUpperCase());
            if (row.has("right_item") || row.has("left_item")) {
                // Real stacks: a spear's components pose its arm and move its item.
                for (var side : new String[] {"right", "left"}) {
                    if (!row.has(side + "_item")) {
                        continue;
                    }
                    var item = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                        net.minecraft.resources.Identifier.parse(row.get(side + "_item").getAsString()));
                    var stack = new net.minecraft.world.item.ItemStack(item);
                    if (side.equals("right")) {
                        state.rightHandItemStack = stack;
                    } else {
                        state.leftHandItemStack = stack;
                    }
                }
                state.swingAnimationType = net.minecraft.world.item.SwingAnimationType.valueOf(
                    row.get("swing").getAsString().toUpperCase(java.util.Locale.ROOT));
                state.attackArm = row.get("attack_arm").getAsString().equals("left")
                    ? net.minecraft.world.entity.HumanoidArm.LEFT
                    : net.minecraft.world.entity.HumanoidArm.RIGHT;
                state.ticksSinceKineticHitFeedback = Oracle.f32(row, "hit_feedback");
            }
            if (row.has("eye_item") && row.get("eye_item").getAsBoolean()) {
                // A spyglass in use is held to the eye.
                ((net.minecraft.client.renderer.entity.state.AvatarRenderState) state).heldOnHead.newLayer();
            }
            if (row.get("holds_right").getAsBoolean()) {
                state.rightHandItemState.newLayer();
            }
            if (row.get("holds_left").getAsBoolean()) {
                state.leftHandItemState.newLayer();
            }
            model.setupAnim(state);
            var poses = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitItem")) {
                        var stack = (com.mojang.blaze3d.vertex.PoseStack) args[0];
                        var matrix = new float[16];
                        stack.last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                    }
                    return null;
                });
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
            var layer = kind.startsWith("player")
                ? new net.minecraft.client.renderer.entity.layers.PlayerItemInHandLayer(parent)
                : new net.minecraft.client.renderer.entity.layers.ItemInHandLayer(parent);
            layer.submit(new com.mojang.blaze3d.vertex.PoseStack(), collector, 0, state, 0.0F, 0.0F);
            var out = posedQuads(row.get("label").getAsString(), model);
            out.add("poses", poses);
            if (row.has("head_scale")) {
                // What the head item is placed with, by the renderer's transforms (a vertical
                // offset, the skull's, and a horizontal scale).
                state.headItem.newLayer();
                var transforms = new net.minecraft.client.renderer.entity.layers.CustomHeadLayer.Transforms(
                    Oracle.f32(row, "head_y_offset"), Oracle.f32(row, "head_skull_y"), Oracle.f32(row, "head_scale"));
                var headPoses = new JsonArray();
                var headCollector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                    EntityOracle.class.getClassLoader(),
                    new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                    (proxy, method, args) -> {
                        if (method.getName().equals("submitItem")) {
                            var stack = (com.mojang.blaze3d.vertex.PoseStack) args[0];
                            var matrix = new float[16];
                            stack.last().pose().get(matrix);
                            var described = new JsonArray();
                            for (float value : matrix) {
                                described.add(Oracle.bits(value));
                            }
                            headPoses.add(described);
                        }
                        return null;
                    });
                new net.minecraft.client.renderer.entity.layers.CustomHeadLayer(parent, null, null, transforms)
                    .submit(new com.mojang.blaze3d.vertex.PoseStack(), headCollector, 0, state, 0.0F, 0.0F);
                out.add("head_poses", headPoses);
                if (row.has("skull")) {
                    // A skull on the head, by the real layer and the real skull model: the quads where
                    // the layer draws them.
                    state.headItem.clear();
                    state.wornHeadType = net.minecraft.world.level.block.SkullBlock.Types.valueOf(
                        row.get("skull").getAsString().toUpperCase(java.util.Locale.ROOT));
                    state.wornHeadAnimationPos = Oracle.f32(row, "skull_animation");
                    var skullQuads = new JsonArray();
                    var skullCollector = (net.minecraft.client.renderer.SubmitNodeCollector)
                        java.lang.reflect.Proxy.newProxyInstance(
                            EntityOracle.class.getClassLoader(),
                            new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                            (proxy, method, args) -> {
                                if (method.getName().equals("submitModel")
                                        && args[0] instanceof net.minecraft.client.model.object.skull.SkullModelBase skull) {
                                    skull.setupAnim((net.minecraft.client.model.object.skull.SkullModelBase.State) args[1]);
                                    var capture = new Capture();
                                    skull.root().render((com.mojang.blaze3d.vertex.PoseStack) args[2], capture, 0, 0);
                                    var quads = new java.util.ArrayList<String>();
                                    for (int i = 0; i < capture.vertices.size(); i += 4) {
                                        quads.add(String.join("", capture.vertices.subList(i, i + 4)));
                                    }
                                    java.util.Collections.sort(quads);
                                    quads.forEach(skullQuads::add);
                                }
                                return null;
                            });
                    var modelSet = new net.minecraft.client.model.geom.EntityModelSet(
                        net.minecraft.client.model.geom.LayerDefinitions.createRoots());
                    new net.minecraft.client.renderer.entity.layers.CustomHeadLayer(parent, modelSet, null, transforms)
                        .submit(new com.mojang.blaze3d.vertex.PoseStack(), skullCollector, 0, state, 0.0F, 0.0F);
                    out.add("skull_quads", skullQuads);
                }
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
