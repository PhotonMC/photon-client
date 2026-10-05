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
public final class EntityOracle {
    /** Keeps each vertex's position, texture coordinates and normal. */
    static final class Capture implements com.mojang.blaze3d.vertex.VertexConsumer {
        final java.util.List<String> vertices = new java.util.ArrayList<>();

        @Override
        public void addVertex(
            final float x, final float y, final float z, final int color, final float u, final float v,
            final int overlayCoords, final int lightCoords, final float nx, final float ny, final float nz) {
            this.vertices.add(Oracle.bits(x) + Oracle.bits(y) + Oracle.bits(z) + Oracle.bits(u) + Oracle.bits(v)
                + Oracle.bits(nx) + Oracle.bits(ny) + Oracle.bits(nz));
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer addVertex(final float x, final float y, final float z) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int r, final int g, final int b, final int a) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int color) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv(final float u, final float v) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv1(final int u, final int v) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv2(final int u, final int v) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setNormal(final float x, final float y, final float z) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setLineWidth(final float width) {
            throw new UnsupportedOperationException();
        }
    }

    /** The reference's own limb animation state, stepped and read between ticks. */
    private static JsonObject walkAnimation(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.world.entity.WalkAnimationState();
            var steps = new JsonArray();
            for (var stepElement : row.get("steps").getAsJsonArray()) {
                var step = stepElement.getAsJsonObject();
                if (step.get("stop").getAsBoolean()) {
                    state.stop();
                } else {
                    state.update(Oracle.f32(step, "target"), Oracle.f32(step, "factor"), Oracle.f32(step, "scale"));
                }
                var out = new JsonObject();
                out.addProperty("speed", Oracle.bits(state.speed()));
                out.addProperty("position", Oracle.bits(state.position()));
                var partials = new JsonArray();
                for (var partialElement : row.get("partials").getAsJsonArray()) {
                    float partial = Float.intBitsToFloat(Integer.parseUnsignedInt(partialElement.getAsString(), 16));
                    partials.add(Oracle.bits(state.position(partial)) + Oracle.bits(state.speed(partial)));
                }
                out.add("partials", partials);
                steps.add(out);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("steps", steps);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static Object field(final Object owner, final String name) throws Exception {
        for (Class<?> type = owner.getClass(); type != null; type = type.getSuperclass()) {
            try {
                var field = type.getDeclaredField(name);
                field.setAccessible(true);
                return field.get(owner);
            } catch (NoSuchFieldException ignored) {
                // look in the class above
            }
        }
        throw new NoSuchFieldException(name);
    }

    private static JsonArray floats(final float... values) {
        var array = new JsonArray();
        for (float value : values) {
            array.add(Oracle.bits(value));
        }
        return array;
    }

    /** One part of a layer's definition: its pose, its cubes and its children by name. */
    @SuppressWarnings("unchecked")
    private static JsonObject partDefinition(final String name, final Object part) throws Exception {
        var out = new JsonObject();
        out.addProperty("name", name);
        var pose = (net.minecraft.client.model.geom.PartPose) field(part, "partPose");
        out.add("pose", floats(pose.x(), pose.y(), pose.z(), pose.xRot(), pose.yRot(), pose.zRot(),
            pose.xScale(), pose.yScale(), pose.zScale()));
        var cubes = new JsonArray();
        for (var cube : (java.util.List<Object>) field(part, "cubes")) {
            var described = new JsonObject();
            var origin = (org.joml.Vector3fc) field(cube, "origin");
            var dimensions = (org.joml.Vector3fc) field(cube, "dimensions");
            var grow = field(cube, "grow");
            var uv = (net.minecraft.client.model.geom.builders.UVPair) field(cube, "texCoord");
            var scale = (net.minecraft.client.model.geom.builders.UVPair) field(cube, "texScale");
            described.add("origin", floats(origin.x(), origin.y(), origin.z()));
            described.add("dimensions", floats(dimensions.x(), dimensions.y(), dimensions.z()));
            described.add("grow", floats((float) field(grow, "growX"), (float) field(grow, "growY"),
                (float) field(grow, "growZ")));
            described.addProperty("mirror", (boolean) field(cube, "mirror"));
            described.add("uv", floats(uv.u(), uv.v()));
            described.add("uv_scale", floats(scale.u(), scale.v()));
            var faces = new java.util.TreeSet<String>();
            for (var face : (java.util.Set<net.minecraft.core.Direction>) field(cube, "visibleFaces")) {
                faces.add(face.getName());
            }
            var visible = new JsonArray();
            faces.forEach(visible::add);
            described.add("faces", visible);
            cubes.add(described);
        }
        out.add("cubes", cubes);
        var children = new java.util.TreeMap<String, Object>();
        for (var child : ((java.util.Map<String, Object>) field(part, "children")).entrySet()) {
            children.put(child.getKey(), child.getValue());
        }
        var described = new JsonArray();
        for (var child : children.entrySet()) {
            described.add(partDefinition(child.getKey(), child.getValue()));
        }
        out.add("children", described);
        return out;
    }

    private static final sun.misc.Unsafe UNSAFE = unsafe();

    private static sun.misc.Unsafe unsafe() {
        try {
            var field = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
            field.setAccessible(true);
            return (sun.misc.Unsafe) field.get(null);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(error);
        }
    }

    /**
     * A dropped item's poses as the reference's own renderer submits them: the bob and the spin,
     * then each copy of the stack with the offsets its seed gives, for a model of a given extent.
     */
    private static JsonObject itemEntityPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var renderer = (net.minecraft.client.renderer.entity.ItemEntityRenderer) UNSAFE.allocateInstance(
                net.minecraft.client.renderer.entity.ItemEntityRenderer.class);
            var randomField = net.minecraft.client.renderer.entity.ItemEntityRenderer.class.getDeclaredField("random");
            randomField.setAccessible(true);
            randomField.set(renderer, net.minecraft.util.RandomSource.create());
            var state = new net.minecraft.client.renderer.entity.state.ItemEntityRenderState();
            state.ageInTicks = Oracle.f32(row, "age");
            state.bobOffset = Oracle.f32(row, "bob");
            state.count = row.get("count").getAsInt();
            state.seed = row.get("seed").getAsInt();
            var min = new org.joml.Vector3f(Oracle.f32(row, "min_x"), Oracle.f32(row, "min_y"), Oracle.f32(row, "min_z"));
            var max = new org.joml.Vector3f(Oracle.f32(row, "max_x"), Oracle.f32(row, "max_y"), Oracle.f32(row, "max_z"));
            state.item.newLayer().setExtents(() -> new org.joml.Vector3fc[] {min, max});
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
            renderer.submit(state, new com.mojang.blaze3d.vertex.PoseStack(), collector, null);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("poses", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A humanoid model posed with its arms' poses, and where the reference's own `ItemInHandLayer`
     * puts the item each hand holds: the quads of the posed model and the pose (a matrix) each held
     * item is submitted with, before its own display transform.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject heldItemPose(final JsonObject request) throws Exception {
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
            state.mainArm = row.get("main_arm").getAsString().equals("left")
                ? net.minecraft.world.entity.HumanoidArm.LEFT
                : net.minecraft.world.entity.HumanoidArm.RIGHT;
            state.rightArmPose = net.minecraft.client.model.HumanoidModel.ArmPose.valueOf(
                row.get("right_pose").getAsString().toUpperCase());
            state.leftArmPose = net.minecraft.client.model.HumanoidModel.ArmPose.valueOf(
                row.get("left_pose").getAsString().toUpperCase());
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
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A sheep's models from the layer table (the body, the undercoat and the wool, grown and
     * baby), posed by the reference's own `setupAnim`, drawn.
     */
    private static JsonObject sheepModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.QuadrupedModel<net.minecraft.client.renderer.entity.state.SheepRenderState> model =
                id.endsWith("#main")
                    ? (id.contains("_baby")
                        ? new net.minecraft.client.model.animal.sheep.BabySheepModel(root)
                        : new net.minecraft.client.model.animal.sheep.SheepModel(root))
                    : new net.minecraft.client.model.animal.sheep.SheepFurModel(root);
            var state = new net.minecraft.client.renderer.entity.state.SheepRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.headEatPositionScale = Oracle.f32(row, "eat_position");
            state.headEatAngleScale = Oracle.f32(row, "eat_angle");
            state.ageScale = Oracle.f32(row, "age_scale");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A chicken's models from the layer table (grown, cold and baby), posed by the reference's own
     * `setupAnim` with the flap, drawn.
     */
    private static JsonObject chickenModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.animal.chicken.ChickenModel model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.chicken.BabyChickenModel(root)
                : id.startsWith("minecraft:cold")
                    ? new net.minecraft.client.model.animal.chicken.ColdChickenModel(root)
                    : new net.minecraft.client.model.animal.chicken.AdultChickenModel(root);
            var state = new net.minecraft.client.renderer.entity.state.ChickenRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.flap = Oracle.f32(row, "flap");
            state.flapSpeed = Oracle.f32(row, "flap_speed");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What a sheep's wool is multiplied by: for each dye the reference's table, and the colour of a
     * `jeb_` sheep at ticks along its cycle.
     */
    private static JsonObject sheepColors(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            if (row.has("texture_dye")) {
                out.addProperty("color", net.minecraft.world.item.DyeColor.byId(row.get("texture_dye").getAsInt())
                    .getTextureDiffuseColor());
            } else if (row.has("dye")) {
                out.addProperty("color", net.minecraft.client.color.ColorLerper.Type.SHEEP.getColor(
                    net.minecraft.world.item.DyeColor.byId(row.get("dye").getAsInt())));
            } else {
                out.addProperty("color", net.minecraft.client.color.ColorLerper.getLerpedColor(
                    net.minecraft.client.color.ColorLerper.Type.SHEEP, Oracle.f32(row, "tick")));
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * How far a sheep's head is down and how it is tipped while it eats, from the reference's own
     * `Sheep` with its eating timer and its pitch set.
     */
    private static JsonObject sheepEat(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var sheep = (net.minecraft.world.entity.animal.sheep.Sheep) UNSAFE.allocateInstance(
                net.minecraft.world.entity.animal.sheep.Sheep.class);
            var timer = net.minecraft.world.entity.animal.sheep.Sheep.class.getDeclaredField("eatAnimationTick");
            timer.setAccessible(true);
            timer.setInt(sheep, row.get("eat_tick").getAsInt());
            for (String name : new String[] {"xRot", "xRotO"}) {
                var field = net.minecraft.world.entity.Entity.class.getDeclaredField(name);
                field.setAccessible(true);
                field.setFloat(sheep, Oracle.f32(row, "x_rot"));
            }
            float partial = Oracle.f32(row, "partial");
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("position", Oracle.bits(sheep.getHeadEatPositionScale(partial)));
            out.addProperty("angle", Oracle.bits(sheep.getHeadEatAngleScale(partial)));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A spider's models from the layer table (the spider and the cave spider), posed by the
     * reference's own `SpiderModel.setupAnim`, drawn; and a creeper's by `CreeperModel`.
     */
    private static JsonObject spiderModel(final JsonObject request, final boolean creeper) throws Exception {
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
    private static JsonObject creeperRender(final JsonObject request) throws Exception {
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
    private static JsonObject equipmentLayers(final JsonObject request) throws Exception {
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
    private static JsonObject armorStandModel(final JsonObject request) throws Exception {
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

    private static net.minecraft.core.Rotations rotations(final JsonArray bits) {
        return new net.minecraft.core.Rotations(
            Float.intBitsToFloat((int) Long.parseLong(bits.get(0).getAsString(), 16)),
            Float.intBitsToFloat((int) Long.parseLong(bits.get(1).getAsString(), 16)),
            Float.intBitsToFloat((int) Long.parseLong(bits.get(2).getAsString(), 16)));
    }

    /** `.data/minecraft/assets/jar/assets/minecraft`, found from the game's jar. */
    private static java.nio.file.Path jarAssets() throws Exception {
        var jar = java.nio.file.Path.of(net.minecraft.client.Minecraft.class
            .getProtectionDomain().getCodeSource().getLocation().toURI());
        return jar.getParent().getParent().resolve("assets/jar/assets/minecraft");
    }

    /** The hat a villager texture's metadata asks for, through the reference's own codec. */
    private static net.minecraft.client.resources.metadata.animation.VillagerMetadataSection.Hat villagerHat(
        final String kind, final String name) throws Exception {
        var file = jarAssets().resolve("textures/entity/villager/" + kind + "/" + name + ".png.mcmeta");
        if (!java.nio.file.Files.exists(file)) {
            return net.minecraft.client.resources.metadata.animation.VillagerMetadataSection.Hat.NONE;
        }
        var json = com.google.gson.JsonParser.parseReader(java.nio.file.Files.newBufferedReader(file)).getAsJsonObject();
        return net.minecraft.client.resources.metadata.animation.VillagerMetadataSection.CODEC
            .parse(com.mojang.serialization.JsonOps.INSTANCE, json.get("villager")).getOrThrow().hat();
    }

    /**
     * The villager types and professions in the order the wire numbers them (the built-in
     * registries), and for each the hat its texture's metadata asks for.
     */
    private static JsonObject villagerRegistries() throws Exception {
        Oracle.vanillaRegistries();
        var rows = new JsonArray();
        var types = new java.util.TreeMap<Integer, String>();
        for (var type : net.minecraft.core.registries.BuiltInRegistries.VILLAGER_TYPE) {
            types.put(net.minecraft.core.registries.BuiltInRegistries.VILLAGER_TYPE.getId(type),
                net.minecraft.core.registries.BuiltInRegistries.VILLAGER_TYPE.getKey(type).getPath());
        }
        var professions = new java.util.TreeMap<Integer, String>();
        for (var profession : net.minecraft.core.registries.BuiltInRegistries.VILLAGER_PROFESSION) {
            professions.put(net.minecraft.core.registries.BuiltInRegistries.VILLAGER_PROFESSION.getId(profession),
                net.minecraft.core.registries.BuiltInRegistries.VILLAGER_PROFESSION.getKey(profession).getPath());
        }
        for (var entry : new Object[][] {{"type", types}, {"profession", professions}}) {
            @SuppressWarnings("unchecked")
            var names = (java.util.TreeMap<Integer, String>) entry[1];
            int index = 0;
            for (var name : names.entrySet()) {
                var out = new JsonObject();
                out.addProperty("label", entry[0] + ":" + name.getValue());
                out.addProperty("kind", (String) entry[0]);
                out.addProperty("id", name.getKey());
                out.addProperty("name", name.getValue());
                out.addProperty("hat", villagerHat((String) entry[0], name.getValue()).getSerializedName());
                rows.add(out);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What `VillagerProfessionLayer` draws for a villager: the model (the hatted one or not) and
     * the texture and colour of each pass, from the reference's own layer with the hats its textures
     * ask for.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject villagerLayers(final JsonObject request) throws Exception {
        Oracle.vanillaRegistries();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            boolean baby = row.get("baby").getAsBoolean();
            String prefix = row.get("path").getAsString();
            boolean zombie = prefix.equals("zombie_villager");
            net.minecraft.client.model.EntityModel model = zombie
                ? new net.minecraft.client.model.monster.zombie.ZombieVillagerModel(bakedLayer("minecraft:" + prefix + "#main"))
                : new net.minecraft.client.model.npc.VillagerModel(bakedLayer("minecraft:" + prefix + "#main"));
            net.minecraft.client.model.EntityModel babyModel = zombie
                ? new net.minecraft.client.model.monster.zombie.BabyZombieVillagerModel(bakedLayer("minecraft:" + prefix + "_baby#main"))
                : new net.minecraft.client.model.npc.BabyVillagerModel(bakedLayer("minecraft:" + prefix + "_baby#main"));
            net.minecraft.client.model.EntityModel noHat = zombie
                ? new net.minecraft.client.model.monster.zombie.ZombieVillagerModel(bakedLayer("minecraft:" + prefix + "_no_hat#main"))
                : new net.minecraft.client.model.npc.VillagerModel(bakedLayer("minecraft:" + prefix + "_no_hat#main"));
            net.minecraft.client.model.EntityModel noHatBaby = zombie
                ? new net.minecraft.client.model.monster.zombie.BabyZombieVillagerModel(
                    bakedLayer("minecraft:" + prefix + "_baby_no_hat#main"))
                : new net.minecraft.client.model.npc.BabyVillagerModel(bakedLayer("minecraft:" + prefix + "_baby_no_hat#main"));
            var current = baby ? babyModel : model;
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> current;
            var layer = new net.minecraft.client.renderer.entity.layers.VillagerProfessionLayer(
                parent, null, prefix, noHat, noHatBaby);
            var typeKey = net.minecraft.resources.ResourceKey.create(net.minecraft.core.registries.Registries.VILLAGER_TYPE,
                net.minecraft.resources.Identifier.withDefaultNamespace(row.get("type").getAsString()));
            var professionKey = net.minecraft.resources.ResourceKey.create(
                net.minecraft.core.registries.Registries.VILLAGER_PROFESSION,
                net.minecraft.resources.Identifier.withDefaultNamespace(row.get("profession").getAsString()));
            var typeCache = net.minecraft.client.renderer.entity.layers.VillagerProfessionLayer.class.getDeclaredField("typeHatCache");
            typeCache.setAccessible(true);
            ((java.util.Map) typeCache.get(layer)).put(typeKey, villagerHat("type", typeKey.identifier().getPath()));
            var professionCache = net.minecraft.client.renderer.entity.layers.VillagerProfessionLayer.class
                .getDeclaredField("professionHatCache");
            professionCache.setAccessible(true);
            ((java.util.Map) professionCache.get(layer)).put(professionKey,
                villagerHat("profession", professionKey.identifier().getPath()));
            var villagerData = new net.minecraft.world.entity.npc.villager.VillagerData(
                net.minecraft.core.registries.BuiltInRegistries.VILLAGER_TYPE.getOrThrow(typeKey),
                net.minecraft.core.registries.BuiltInRegistries.VILLAGER_PROFESSION.getOrThrow(professionKey),
                row.get("level").getAsInt());
            net.minecraft.client.renderer.entity.state.LivingEntityRenderState state;
            if (zombie) {
                var zombieState = new net.minecraft.client.renderer.entity.state.ZombieVillagerRenderState();
                zombieState.villagerData = villagerData;
                state = zombieState;
            } else {
                var villagerState = new net.minecraft.client.renderer.entity.state.VillagerRenderState();
                villagerState.villagerData = villagerData;
                state = villagerState;
            }
            state.isBaby = baby;
            var passes = new JsonArray();
            var holder = new net.minecraft.client.renderer.SubmitNodeCollector[1];
            holder[0] = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("order")) {
                        return holder[0];
                    }
                    if (method.getName().equals("submitModel") && args.length == 10) {
                        var pass = new JsonObject();
                        pass.addProperty("model", args[0] == model ? "main" : args[0] == babyModel ? "main_baby"
                            : args[0] == noHat ? "no_hat" : "no_hat_baby");
                        var text = String.valueOf(args[3]);
                        var found = java.util.regex.Pattern.compile("minecraft:(textures/[a-z_/]+\\.png)").matcher(text);
                        pass.addProperty("texture", found.find() ? found.group(1) : text);
                        pass.addProperty("color", (int) args[6]);
                        passes.add(pass);
                    }
                    return null;
                });
            layer.submit(new com.mojang.blaze3d.vertex.PoseStack(), holder[0], 0, state, 0.0F, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("passes", passes);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A villager's models from the layer table (the villager, the wandering trader, grown and baby,
     * hatted or not), posed by the reference's own `VillagerModel.setupAnim`, drawn.
     */
    private static JsonObject villagerModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.npc.VillagerModel model = id.contains("_baby")
                ? new net.minecraft.client.model.npc.BabyVillagerModel(root)
                : new net.minecraft.client.model.npc.VillagerModel(root);
            var state = new net.minecraft.client.renderer.entity.state.VillagerRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isUnhappy = row.get("unhappy").getAsBoolean();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * An illager's model from the layer table, posed by the reference's own
     * `IllagerModel.setupAnim` for an arm pose, a held item and a crossbow's charge, drawn.
     */
    private static JsonObject illagerModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.illager.IllagerModel<
                net.minecraft.client.renderer.entity.state.IllagerRenderState>(bakedLayer(row.get("layer").getAsString()));
            model.getHat().visible = row.get("hat").getAsBoolean();
            var state = new net.minecraft.client.renderer.entity.state.IllagerRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isRiding = row.get("riding").getAsBoolean();
            state.armPose = net.minecraft.world.entity.monster.illager.AbstractIllager.IllagerArmPose.valueOf(
                row.get("arm_pose").getAsString());
            state.mainArm = row.get("main_arm").getAsString().equals("left")
                ? net.minecraft.world.entity.HumanoidArm.LEFT : net.minecraft.world.entity.HumanoidArm.RIGHT;
            state.attackTime = Oracle.f32(row, "attack_time");
            state.attackAnim = state.attackTime;
            state.maxCrossbowChargeDuration = row.get("max_crossbow_charge").getAsInt();
            state.ticksUsingItem = Oracle.f32(row, "ticks_using_item");
            if (row.get("has_item").getAsBoolean()) {
                state.getMainHandItemState().ensureCapacity(1);
                state.getMainHandItemState().newLayer();
            }
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The witch's model posed by the reference's own `WitchModel.setupAnim`, drawn. */
    private static JsonObject witchModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.witch.WitchModel(bakedLayer("minecraft:witch#main"));
            var state = new net.minecraft.client.renderer.entity.state.WitchRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.entityId = row.get("entity_id").getAsInt();
            state.isHoldingItem = row.get("holding_item").getAsBoolean();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The iron golem's model posed by the reference's own `IronGolemModel.setupAnim`, drawn. */
    private static JsonObject ironGolemModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.animal.golem.IronGolemModel(bakedLayer("minecraft:iron_golem#main"));
            var state = new net.minecraft.client.renderer.entity.state.IronGolemRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.attackTicksRemaining = Oracle.f32(row, "attack_ticks");
            state.offerFlowerTick = row.get("offer_flower_tick").getAsInt();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The snow golem's model posed by the reference's own `SnowGolemModel.setupAnim`, drawn. */
    private static JsonObject snowGolemModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.animal.golem.SnowGolemModel(bakedLayer("minecraft:snow_golem#main"));
            var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The enderman's model posed by the reference's own `EndermanModel.setupAnim`, drawn; a
     * carried block makes the state's block non-empty through the reference's own class.
     */
    private static JsonObject endermanModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.enderman.EndermanModel<
                net.minecraft.client.renderer.entity.state.EndermanRenderState>(bakedLayer("minecraft:enderman#main"));
            var state = new net.minecraft.client.renderer.entity.state.EndermanRenderState();
            humanoidFields(state, row);
            state.isCreepy = row.get("creepy").getAsBoolean();
            if (row.get("carrying").getAsBoolean()) {
                state.carriedBlock.setupModel(new org.joml.Matrix4f(), false);
            }
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the iron golem's renderer adds to the pose while it walks (`setupRotations`), as the
     * matrix of the pose stack, and the crackiness level its health fraction gives.
     */
    private static JsonObject ironGolemRender(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var renderer = (net.minecraft.client.renderer.entity.IronGolemRenderer) UNSAFE.allocateInstance(
            net.minecraft.client.renderer.entity.IronGolemRenderer.class);
        var rotations = net.minecraft.client.renderer.entity.IronGolemRenderer.class.getDeclaredMethod("setupRotations",
            net.minecraft.client.renderer.entity.state.IronGolemRenderState.class,
            com.mojang.blaze3d.vertex.PoseStack.class, float.class, float.class);
        rotations.setAccessible(true);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.entity.state.IronGolemRenderState();
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            rotations.invoke(renderer, state, stack, Oracle.f32(row, "body_rot"), 1.0F);
            var matrix = new float[16];
            stack.last().pose().get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("pose", described);
            out.addProperty("crackiness",
                net.minecraft.world.entity.Crackiness.GOLEM.byFraction(Oracle.f32(row, "fraction")).name());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A block model part with no quads: enough for a block model state to submit something. */
    private static net.minecraft.client.renderer.block.dispatch.BlockStateModelPart emptyPart() {
        return (net.minecraft.client.renderer.block.dispatch.BlockStateModelPart) java.lang.reflect.Proxy.newProxyInstance(
            EntityOracle.class.getClassLoader(),
            new Class<?>[] {net.minecraft.client.renderer.block.dispatch.BlockStateModelPart.class},
            (proxy, method, args) -> {
                var type = method.getReturnType();
                if (type == java.util.List.class) {
                    return java.util.List.of();
                }
                if (type == boolean.class) {
                    return false;
                }
                return type == int.class ? 0 : null;
            });
    }

    /** Makes a block model state non-empty, with a part that draws nothing. */
    private static void fillBlock(final net.minecraft.client.renderer.block.BlockModelRenderState block) {
        block.setupModel(new org.joml.Matrix4f(), false).add(emptyPart());
    }

    /**
     * The pose each layer that draws a block on an entity puts the block at, from the reference's
     * own layer classes: the matrix the block is submitted with, from an identity entity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject entityBlockLayers(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var captured = new float[16];
            var seen = new boolean[1];
            var all = new JsonArray();
            var holder = new net.minecraft.client.renderer.SubmitNodeCollector[1];
            holder[0] = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("order")) {
                        return holder[0];
                    }
                    if (method.getName().equals("submitBlockModel")) {
                        ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(captured);
                        seen[0] = true;
                        var one = new JsonArray();
                        for (float value : captured) {
                            one.add(Oracle.bits(value));
                        }
                        all.add(one);
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            String kind = row.get("layer").getAsString();
            if (kind.equals("carried")) {
                var state = new net.minecraft.client.renderer.entity.state.EndermanRenderState();
                fillBlock(state.carriedBlock);
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> null;
                new net.minecraft.client.renderer.entity.layers.CarriedBlockLayer(parent)
                    .submit(stack, holder[0], 0, state, 0.0F, 0.0F);
            } else if (kind.equals("mushrooms")) {
                var model = new net.minecraft.client.model.animal.cow.CowModel(bakedLayer("minecraft:mooshroom#main"));
                var state = new net.minecraft.client.renderer.entity.state.MushroomCowRenderState();
                state.yRot = Oracle.f32(row, "head_y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                model.setupAnim(state);
                fillBlock(state.mushroomModel);
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
                new net.minecraft.client.renderer.entity.layers.MushroomCowMushroomLayer(parent)
                    .submit(stack, holder[0], 0, state, 0.0F, 0.0F);
            } else if (kind.equals("snow_head")) {
                var model = new net.minecraft.client.model.animal.golem.SnowGolemModel(bakedLayer("minecraft:snow_golem#main"));
                var state = new net.minecraft.client.renderer.entity.state.SnowGolemRenderState();
                state.yRot = Oracle.f32(row, "head_y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                model.setupAnim(state);
                fillBlock(state.headBlock);
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
                new net.minecraft.client.renderer.entity.layers.SnowGolemHeadLayer(parent)
                    .submit(stack, holder[0], 0, state, 0.0F, 0.0F);
            } else {
                var model = new net.minecraft.client.model.animal.golem.IronGolemModel(bakedLayer("minecraft:iron_golem#main"));
                var state = new net.minecraft.client.renderer.entity.state.IronGolemRenderState();
                state.yRot = Oracle.f32(row, "head_y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.attackTicksRemaining = Oracle.f32(row, "attack_ticks");
                state.offerFlowerTick = row.get("offer_flower_tick").getAsInt();
                model.setupAnim(state);
                fillBlock(state.flowerBlock);
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
                new net.minecraft.client.renderer.entity.layers.IronGolemFlowerLayer(parent)
                    .submit(stack, holder[0], 0, state, 0.0F, 0.0F);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("drawn", seen[0]);
            var described = new JsonArray();
            for (float value : captured) {
                described.add(Oracle.bits(value));
            }
            out.add("pose", described);
            out.add("poses", all);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the glowing-eyes layers submit, from the reference's own `SpiderEyesLayer` and
     * `EnderEyesLayer`: the draw order, the texture and the pipeline's state of the render type,
     * the light coordinates and the overlay.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject eyesLayers(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var order = new int[] {-1};
            var holder = new net.minecraft.client.renderer.SubmitNodeCollector[1];
            holder[0] = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("order")) {
                        order[0] = (int) args[0];
                        return holder[0];
                    }
                    if (method.getName().equals("submitModel")) {
                        var renderType = (net.minecraft.client.renderer.rendertype.RenderType) args[3];
                        out.addProperty("order", order[0]);
                        out.addProperty("light", (int) args[4]);
                        out.addProperty("overlay", (int) args[5]);
                        var found = java.util.regex.Pattern.compile("minecraft:(textures/[a-z_/]+\\.png)")
                            .matcher(String.valueOf(renderType));
                        out.addProperty("texture", found.find() ? found.group(1) : String.valueOf(renderType));
                        var stateField = net.minecraft.client.renderer.rendertype.RenderType.class.getDeclaredField("state");
                        stateField.setAccessible(true);
                        var setup = stateField.get(renderType);
                        var pipelineField = setup.getClass().getDeclaredField("pipeline");
                        pipelineField.setAccessible(true);
                        var pipeline = (com.mojang.blaze3d.pipeline.RenderPipeline) pipelineField.get(setup);
                        out.addProperty("pipeline", pipeline.getLocation().toString());
                        var flags = new java.util.TreeSet<>(pipeline.getShaderDefines().flags());
                        out.addProperty("flags", String.join(",", flags));
                        var values = new java.util.TreeMap<>(pipeline.getShaderDefines().values());
                        var text = new StringBuilder();
                        values.forEach((key, value) -> text.append(key).append('=').append(value).append(','));
                        out.addProperty("values", text.toString());
                        var blend = pipeline.getColorTargetState().blendFunction();
                        out.addProperty("blend", blend.isEmpty() ? "NONE"
                            : blend.get().equals(com.mojang.blaze3d.pipeline.BlendFunction.TRANSLUCENT) ? "TRANSLUCENT"
                            : blend.get().equals(com.mojang.blaze3d.pipeline.BlendFunction.ADDITIVE) ? "ADDITIVE" : "OTHER");
                        out.addProperty("depth_test", pipeline.getDepthStencilState().depthTest().name());
                        out.addProperty("depth_write", pipeline.getDepthStencilState().writeDepth());
                        out.addProperty("cull", pipeline.isCull());
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            var model = new net.minecraft.client.model.monster.spider.SpiderModel(bakedLayer("minecraft:spider#main"));
            if (row.get("layer").getAsString().equals("spider")) {
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
                new net.minecraft.client.renderer.entity.layers.SpiderEyesLayer(parent).submit(
                    stack, holder[0], 15728880, new net.minecraft.client.renderer.entity.state.LivingEntityRenderState(),
                    0.0F, 0.0F);
            } else {
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> null;
                new net.minecraft.client.renderer.entity.layers.EnderEyesLayer(parent).submit(
                    stack, holder[0], 15728880, new net.minecraft.client.renderer.entity.state.EndermanRenderState(),
                    0.0F, 0.0F);
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The goat's model from the layer table, posed by the reference's own `GoatModel.setupAnim`, drawn. */
    private static JsonObject goatModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.goat.BabyGoatModel(root)
                : new net.minecraft.client.model.animal.goat.GoatModel(root);
            var state = new net.minecraft.client.renderer.entity.state.GoatRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.hasLeftHorn = row.get("left_horn").getAsBoolean();
            state.hasRightHorn = row.get("right_horn").getAsBoolean();
            state.rammingXHeadRot = Oracle.f32(row, "ramming");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Where the layers that hold something on a villager, a trader or a witch put it, from the
     * reference's own layer classes: the matrix each is submitted with, from an identity entity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject villagerHeldLayers(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var poses = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitItem")) {
                        var matrix = new float[16];
                        ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            String kind = row.get("kind").getAsString();
            boolean baby = row.get("baby").getAsBoolean();
            if (kind.startsWith("witch")) {
                var model = new net.minecraft.client.model.monster.witch.WitchModel(bakedLayer("minecraft:witch#main"));
                var state = new net.minecraft.client.renderer.entity.state.WitchRenderState();
                state.yRot = Oracle.f32(row, "head_y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.entityId = row.get("entity_id").getAsInt();
                state.isHoldingItem = true;
                state.isHoldingPotion = kind.equals("witch_potion");
                model.setupAnim(state);
                state.heldItem.newLayer();
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
                new net.minecraft.client.renderer.entity.layers.WitchItemLayer(parent)
                    .submit(stack, collector, 0, state, 0.0F, 0.0F);
            } else {
                var model = baby
                    ? new net.minecraft.client.model.npc.BabyVillagerModel(bakedLayer("minecraft:villager_baby#main"))
                    : new net.minecraft.client.model.npc.VillagerModel(bakedLayer("minecraft:villager#main"));
                var state = new net.minecraft.client.renderer.entity.state.VillagerRenderState();
                state.yRot = Oracle.f32(row, "head_y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.isUnhappy = row.get("unhappy").getAsBoolean();
                model.setupAnim(state);
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
                if (kind.equals("villager_head")) {
                    state.headItem.newLayer();
                    var transforms = row.get("villager_transforms").getAsBoolean()
                        ? net.minecraft.client.renderer.entity.VillagerRenderer.CUSTOM_HEAD_TRANSFORMS
                        : net.minecraft.client.renderer.entity.layers.CustomHeadLayer.Transforms.DEFAULT;
                    new net.minecraft.client.renderer.entity.layers.CustomHeadLayer(parent, null, null, transforms)
                        .submit(stack, collector, 0, state, 0.0F, 0.0F);
                } else {
                    state.heldItem.newLayer();
                    new net.minecraft.client.renderer.entity.layers.CrossedArmsItemLayer(parent)
                        .submit(stack, collector, 0, state, 0.0F, 0.0F);
                }
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("poses", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The attributes every living entity type starts with (`DefaultAttributes`: the base value of
     * each attribute its supplier has), in entity type order, and each attribute's range.
     */
    private static JsonObject entityAttributes() throws Exception {
        Oracle.vanillaRegistries();
        var rows = new JsonArray();
        var types = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE;
        var attributes = net.minecraft.core.registries.BuiltInRegistries.ATTRIBUTE;
        for (var type : types) {
            if (!net.minecraft.world.entity.ai.attributes.DefaultAttributes.hasSupplier(
                (net.minecraft.world.entity.EntityType) type)) {
                continue;
            }
            var supplier = net.minecraft.world.entity.ai.attributes.DefaultAttributes.getSupplier(
                (net.minecraft.world.entity.EntityType) type);
            var out = new JsonObject();
            out.addProperty("label", types.getKey(type).toString());
            out.addProperty("kind", "type");
            out.addProperty("id", types.getId(type));
            var list = new JsonArray();
            var holders = new java.util.ArrayList<net.minecraft.core.Holder.Reference<
                net.minecraft.world.entity.ai.attributes.Attribute>>(attributes.listElements().toList());
            holders.sort(java.util.Comparator.comparingInt(holder -> attributes.getId(holder.value())));
            for (var holder : holders) {
                if (supplier.hasAttribute(holder)) {
                    var pair = new JsonObject();
                    pair.addProperty("name", holder.key().identifier().toString());
                    pair.addProperty("base", Oracle.bits(supplier.getBaseValue(holder)));
                    list.add(pair);
                }
            }
            out.add("attributes", list);
            rows.add(out);
        }
        for (var attribute : attributes) {
            var out = new JsonObject();
            out.addProperty("label", attributes.getKey(attribute).toString());
            out.addProperty("kind", "range");
            out.addProperty("id", attributes.getId(attribute));
            out.addProperty("default", Oracle.bits(attribute.getDefaultValue()));
            if (attribute instanceof net.minecraft.world.entity.ai.attributes.RangedAttribute ranged) {
                out.addProperty("min", Oracle.bits(ranged.getMinValue()));
                out.addProperty("max", Oracle.bits(ranged.getMaxValue()));
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static final String[] ANIMATION_CLASSES = {
        "Armadillo", "BabyArmadillo", "BabyAxolotl", "BabyRabbit", "Bat", "Breeze", "Camel", "CamelBaby",
        "CopperGolem", "Creaking", "FoxBaby", "Frog", "Nautilus", "Rabbit", "Sniffer", "Warden"};

    /** Every animation definition of the reference's `definitions` package, by `Class.FIELD`. */
    private static java.util.Map<String, net.minecraft.client.animation.AnimationDefinition> animationDefinitions()
        throws Exception {
        var found = new java.util.LinkedHashMap<String, net.minecraft.client.animation.AnimationDefinition>();
        for (String name : ANIMATION_CLASSES) {
            var type = Class.forName("net.minecraft.client.animation.definitions." + name + "Animation");
            for (var field : type.getDeclaredFields()) {
                if (field.getType() == net.minecraft.client.animation.AnimationDefinition.class
                    && java.lang.reflect.Modifier.isStatic(field.getModifiers())) {
                    found.put(name + "." + field.getName(),
                        (net.minecraft.client.animation.AnimationDefinition) field.get(null));
                }
            }
        }
        return found;
    }

    private static JsonArray vector(final org.joml.Vector3fc vector) {
        var out = new JsonArray();
        out.add(Oracle.bits(vector.x()));
        out.add(Oracle.bits(vector.y()));
        out.add(Oracle.bits(vector.z()));
        return out;
    }

    /**
     * Every animation definition: its length, whether it loops, and for each bone (in the order
     * the reference keeps them) each channel's target and keyframes.
     */
    private static JsonObject animationDefinitionRows() throws Exception {
        var rows = new JsonArray();
        for (var entry : animationDefinitions().entrySet()) {
            var definition = entry.getValue();
            var out = new JsonObject();
            out.addProperty("label", entry.getKey());
            out.addProperty("length", Oracle.bits(definition.lengthInSeconds()));
            out.addProperty("looping", definition.looping());
            var bones = new JsonArray();
            for (var bone : definition.boneAnimations().entrySet()) {
                var described = new JsonObject();
                described.addProperty("name", bone.getKey());
                var channels = new JsonArray();
                for (var channel : bone.getValue()) {
                    var one = new JsonObject();
                    one.addProperty("target", channel.target() == net.minecraft.client.animation.AnimationChannel.Targets.POSITION
                        ? "POSITION" : channel.target() == net.minecraft.client.animation.AnimationChannel.Targets.ROTATION
                        ? "ROTATION" : channel.target() == net.minecraft.client.animation.AnimationChannel.Targets.SCALE
                        ? "SCALE" : "OTHER");
                    var frames = new JsonArray();
                    for (var frame : channel.keyframes()) {
                        var two = new JsonObject();
                        two.addProperty("time", Oracle.bits(frame.timestamp()));
                        two.add("pre", vector(frame.preTarget()));
                        two.add("post", vector(frame.postTarget()));
                        two.addProperty("interpolation",
                            frame.interpolation() == net.minecraft.client.animation.AnimationChannel.Interpolations.LINEAR
                                ? "LINEAR" : frame.interpolation()
                                    == net.minecraft.client.animation.AnimationChannel.Interpolations.CATMULLROM
                                ? "CATMULLROM" : "OTHER");
                        frames.add(two);
                    }
                    one.add("keyframes", frames);
                    channels.add(one);
                }
                described.add("channels", channels);
                bones.add(described);
            }
            out.add("bones", bones);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A model's pose after the reference's own `KeyframeAnimation.apply(millis, scale)` of a definition,
     * drawn.
     */
    private static JsonObject keyframeAnimation(final JsonObject request) throws Exception {
        var definitions = animationDefinitions();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.Model<?> model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.rabbit.BabyRabbitModel(root)
                : new net.minecraft.client.model.animal.rabbit.AdultRabbitModel(root);
            var animation = definitions.get(row.get("definition").getAsString()).bake(root);
            animation.apply(row.get("millis").getAsLong(), Oracle.f32(row, "scale"));
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** `AnimationState`: what each call sequence makes of the time an animation has run. */
    private static JsonObject animationStates(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.world.entity.AnimationState();
            var millis = new JsonArray();
            for (var step : row.getAsJsonArray("steps")) {
                var call = step.getAsJsonObject();
                switch (call.get("call").getAsString()) {
                    case "start" -> state.start(call.get("tick").getAsInt());
                    case "start_if_stopped" -> state.startIfStopped(call.get("tick").getAsInt());
                    case "animate_when" -> state.animateWhen(call.get("condition").getAsBoolean(), call.get("tick").getAsInt());
                    case "stop" -> state.stop();
                    case "fast_forward" -> state.fastForward(call.get("ticks").getAsInt(), Oracle.f32(call, "scale"));
                    default -> throw new IllegalArgumentException("Unknown call " + call);
                }
                var seen = new JsonObject();
                seen.addProperty("started", state.isStarted());
                seen.addProperty("millis", state.getTimeInMillis(Oracle.f32(call, "age")));
                millis.add(seen);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("steps", millis);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The rabbit's model posed by the reference's own `RabbitModel.setupAnim` with its animation states, drawn. */
    private static JsonObject rabbitModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.animal.rabbit.RabbitModel model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.rabbit.BabyRabbitModel(root)
                : new net.minecraft.client.model.animal.rabbit.AdultRabbitModel(root);
            var state = new net.minecraft.client.renderer.entity.state.RabbitRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            if (row.has("hop_start")) {
                state.hopAnimationState.start(row.get("hop_start").getAsInt());
            }
            if (row.has("idle_start")) {
                state.idleHeadTiltAnimationState.start(row.get("idle_start").getAsInt());
            }
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Where the layers a copper golem carries things with put them, from the reference's own layer
     * classes on the model as posed: the matrix each is submitted with, from an identity entity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject copperGolemLayers(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var poses = new JsonArray();
            var holder = new net.minecraft.client.renderer.SubmitNodeCollector[1];
            holder[0] = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("order")) {
                        return holder[0];
                    }
                    if (method.getName().equals("submitItem") || method.getName().equals("submitBlockModel")) {
                        var matrix = new float[16];
                        ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                    }
                    return null;
                });
            var model = new net.minecraft.client.model.animal.golem.CopperGolemModel(
                bakedLayer("minecraft:copper_golem#main"));
            var state = new net.minecraft.client.renderer.entity.state.CopperGolemRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.copperGolemState = net.minecraft.world.entity.animal.golem.CopperGolemState.valueOf(
                row.get("action").getAsString());
            state.rightHandItemState.newLayer();
            state.leftHandItemState.newLayer();
            state.headItem.newLayer();
            fillBlock(state.blockOnAntenna);
            model.setupAnim(state);
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            String layer = row.get("layer").getAsString();
            if (layer.equals("hands")) {
                new net.minecraft.client.renderer.entity.layers.ItemInHandLayer(parent)
                    .submit(stack, holder[0], 0, state, 0.0F, 0.0F);
            } else if (layer.equals("head")) {
                new net.minecraft.client.renderer.entity.layers.CustomHeadLayer(parent, null, null,
                    net.minecraft.client.renderer.entity.layers.CustomHeadLayer.Transforms.DEFAULT)
                    .submit(stack, holder[0], 0, state, 0.0F, 0.0F);
            } else {
                new net.minecraft.client.renderer.entity.layers.BlockDecorationLayer(parent,
                    (java.util.function.Function<net.minecraft.client.renderer.entity.state.CopperGolemRenderState,
                        net.minecraft.client.renderer.block.BlockModelRenderState>) s -> s.blockOnAntenna,
                    (java.util.function.Consumer<com.mojang.blaze3d.vertex.PoseStack>) model::applyBlockOnAntennaTransform)
                    .submit(stack, holder[0], 0, state, 0.0F, 0.0F);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("poses", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The squid's models posed by the reference's own `SquidModel.setupAnim`, drawn. */
    private static JsonObject squidModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.squid.BabySquidModel(root)
                : new net.minecraft.client.model.animal.squid.SquidModel(root);
            var state = new net.minecraft.client.renderer.entity.state.SquidRenderState();
            state.tentacleAngle = Oracle.f32(row, "tentacle_angle");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The dolphin's and the turtle's models posed by the reference's own `setupAnim`, drawn: the
     * dolphin from the head's turn and pitch, the age and whether it moves; the turtle from the
     * walk, whether it is on land, laying an egg, and carrying one.
     */
    private static JsonObject seaAnimalModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.EntityModel<?> model;
            if (id.startsWith("minecraft:dolphin")) {
                var state = new net.minecraft.client.renderer.entity.state.DolphinRenderState();
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.isMoving = row.get("is_moving").getAsBoolean();
                var dolphin = id.contains("_baby")
                    ? new net.minecraft.client.model.animal.dolphin.BabyDolphinModel(root)
                    : new net.minecraft.client.model.animal.dolphin.DolphinModel(root);
                dolphin.setupAnim(state);
                model = dolphin;
            } else {
                var state = new net.minecraft.client.renderer.entity.state.TurtleRenderState();
                state.yRot = Oracle.f32(row, "head_y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.isOnLand = row.get("on_land").getAsBoolean();
                state.isLayingEgg = row.get("laying_egg").getAsBoolean();
                state.hasEgg = row.get("has_egg").getAsBoolean();
                net.minecraft.client.model.animal.turtle.TurtleModel turtle = id.contains("_baby")
                    ? new net.minecraft.client.model.animal.turtle.BabyTurtleModel(root)
                    : new net.minecraft.client.model.animal.turtle.AdultTurtleModel(root);
                turtle.setupAnim(state);
                model = turtle;
            }
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The axolotl's models posed by the reference's own `setupAnim`, drawn: an adult from its blend
     * factors, a baby from the animation states it runs (each started at the tick the row names).
     */
    private static JsonObject axolotlModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var state = new net.minecraft.client.renderer.entity.state.AxolotlRenderState();
            state.yRot = Oracle.f32(row, "y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.playingDeadFactor = Oracle.f32(row, "playing_dead");
            state.movingFactor = Oracle.f32(row, "moving");
            state.inWaterFactor = Oracle.f32(row, "in_water");
            state.onGroundFactor = Oracle.f32(row, "on_ground");
            var animations = new Object[][] {
                {"swim", state.swimAnimation}, {"walk", state.walkAnimationState},
                {"idle_under_water", state.idleUnderWaterAnimationState},
                {"idle_under_water_on_ground", state.idleUnderWaterOnGroundAnimationState},
                {"idle_on_ground", state.idleOnGroundAnimationState}, {"play_dead", state.playDeadAnimationState},
            };
            for (var animation : animations) {
                if (row.has((String) animation[0])) {
                    ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                }
            }
            net.minecraft.client.model.EntityModel<net.minecraft.client.renderer.entity.state.AxolotlRenderState> model =
                id.contains("_baby")
                    ? new net.minecraft.client.model.animal.axolotl.BabyAxolotlModel(root)
                    : new net.minecraft.client.model.animal.axolotl.AdultAxolotlModel(root);
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The frog's and the tadpole's models posed by the reference's own `setupAnim`, drawn: a frog
     * from the animation states it runs (each started at the tick the row names) and whether it
     * swims, a tadpole from its age and whether it is in water.
     */
    private static JsonObject frogModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            if (id.contains("tadpole")) {
                var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.isInWater = row.get("in_water").getAsBoolean();
                var model = new net.minecraft.client.model.animal.frog.TadpoleModel(root);
                model.setupAnim(state);
                rows.add(posedQuads(row.get("label").getAsString(), model));
                continue;
            }
            var state = new net.minecraft.client.renderer.entity.state.FrogRenderState();
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isSwimming = row.get("swimming").getAsBoolean();
            var animations = new Object[][] {
                {"jump", state.jumpAnimationState}, {"croak", state.croakAnimationState},
                {"tongue", state.tongueAnimationState}, {"swim_idle", state.swimIdleAnimationState},
            };
            for (var animation : animations) {
                if (row.has((String) animation[0])) {
                    ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                }
            }
            var model = new net.minecraft.client.model.animal.frog.FrogModel(root);
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The blaze, ghast and phantom models posed by the reference's own `setupAnim`, drawn, from the
     * age (and the look, for a blaze; the flap time, for a phantom).
     */
    private static JsonObject airborneModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.Model<?> model;
            if (id.contains("blaze")) {
                var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                var blaze = new net.minecraft.client.model.monster.blaze.BlazeModel(root);
                blaze.setupAnim(state);
                model = blaze;
            } else if (id.contains("happy_ghast")) {
                var state = new net.minecraft.client.renderer.entity.state.HappyGhastRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.isRidden = row.get("ridden").getAsBoolean();
                if (row.get("body_item").getAsBoolean()) {
                    state.bodyItem = new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.HARNESS.white());
                }
                if (id.contains("harness")) {
                    var harness = new net.minecraft.client.model.animal.ghast.HappyGhastHarnessModel(root);
                    harness.setupAnim(state);
                    model = harness;
                } else {
                    var happy = new net.minecraft.client.model.animal.ghast.HappyGhastModel(root);
                    happy.setupAnim(state);
                    model = happy;
                }
            } else if (id.contains("breeze")) {
                var state = new net.minecraft.client.renderer.entity.state.BreezeRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                var animations = new Object[][] {
                    {"idle", state.idle}, {"shoot", state.shoot}, {"slide", state.slide},
                    {"slide_back", state.slideBack}, {"inhale", state.inhale}, {"long_jump", state.longJump},
                };
                for (var animation : animations) {
                    if (row.has((String) animation[0])) {
                        ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                    }
                }
                var breeze = new net.minecraft.client.model.monster.breeze.BreezeModel(root);
                breeze.setupAnim(state);
                model = breeze;
            } else if (id.contains("trident")) {
                var trident = new net.minecraft.client.model.object.projectile.TridentModel(root);
                trident.setupAnim(net.minecraft.util.Unit.INSTANCE);
                model = trident;
            } else if (id.contains("arrow")) {
                var state = new net.minecraft.client.renderer.entity.state.ArrowRenderState();
                state.shake = Oracle.f32(row, "shake");
                var arrow = new net.minecraft.client.model.object.projectile.ArrowModel(root);
                arrow.setupAnim(state);
                model = arrow;
            } else if (id.contains("end_crystal")) {
                var state = new net.minecraft.client.renderer.entity.state.EndCrystalRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.showsBottom = row.get("shows_bottom").getAsBoolean();
                var crystal = new net.minecraft.client.model.object.crystal.EndCrystalModel(root);
                crystal.setupAnim(state);
                model = crystal;
            } else if (id.contains("ender_dragon")) {
                var state = new net.minecraft.client.renderer.entity.state.EnderDragonRenderState();
                state.flapTime = Oracle.f32(row, "flap_time");
                state.partialTicks = Oracle.f32(row, "partial_ticks");
                state.isLandingOrTakingOff = row.get("landing_or_taking_off").getAsBoolean();
                state.isSitting = row.get("sitting").getAsBoolean();
                state.distanceToEgg = Oracle.f64(row, "distance_to_egg");
                for (var sample : row.getAsJsonArray("history")) {
                    var pair = sample.getAsJsonObject();
                    state.flightHistory.record(Oracle.f64(pair, "y"), Oracle.f32(pair, "y_rot"));
                }
                var dragon = new net.minecraft.client.model.monster.dragon.EnderDragonModel(root);
                dragon.setupAnim(state);
                model = dragon;
            } else if (id.contains("wither")) {
                var state = new net.minecraft.client.renderer.entity.state.WitherRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.bodyRot = Oracle.f32(row, "body_rot");
                state.xHeadRots[0] = Oracle.f32(row, "x_head_0");
                state.xHeadRots[1] = Oracle.f32(row, "x_head_1");
                state.yHeadRots[0] = Oracle.f32(row, "y_head_0");
                state.yHeadRots[1] = Oracle.f32(row, "y_head_1");
                var wither = new net.minecraft.client.model.monster.wither.WitherBossModel(root);
                wither.setupAnim(state);
                model = wither;
            } else if (id.contains("sniffer")) {
                var state = new net.minecraft.client.renderer.entity.state.SnifferRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.isSearching = row.get("searching").getAsBoolean();
                var animations = new Object[][] {
                    {"digging", state.diggingAnimationState}, {"sniffing", state.sniffingAnimationState},
                    {"rising", state.risingAnimationState}, {"happy", state.feelingHappyAnimationState},
                    {"scenting", state.scentingAnimationState},
                };
                for (var animation : animations) {
                    if (row.has((String) animation[0])) {
                        ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                    }
                }
                // The renderer makes the baby's model from the same class as the adult's.
                var sniffer = new net.minecraft.client.model.animal.sniffer.SnifferModel(root);
                sniffer.setupAnim(state);
                model = sniffer;
            } else if (id.contains("armadillo")) {
                var state = new net.minecraft.client.renderer.entity.state.ArmadilloRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.isHidingInShell = row.get("hiding").getAsBoolean();
                var animations = new Object[][] {
                    {"roll_out", state.rollOutAnimationState}, {"roll_up", state.rollUpAnimationState},
                    {"peek", state.peekAnimationState},
                };
                for (var animation : animations) {
                    if (row.has((String) animation[0])) {
                        ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                    }
                }
                net.minecraft.client.model.animal.armadillo.ArmadilloModel armadillo = id.contains("baby")
                    ? new net.minecraft.client.model.animal.armadillo.BabyArmadilloModel(root)
                    : new net.minecraft.client.model.animal.armadillo.AdultArmadilloModel(root);
                armadillo.setupAnim(state);
                model = armadillo;
            } else if (id.contains("creaking")) {
                var state = new net.minecraft.client.renderer.entity.state.CreakingRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.canMove = row.get("can_move").getAsBoolean();
                var animations = new Object[][] {
                    {"attack", state.attackAnimationState}, {"invulnerable", state.invulnerabilityAnimationState},
                    {"death", state.deathAnimationState},
                };
                for (var animation : animations) {
                    if (row.has((String) animation[0])) {
                        ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                    }
                }
                var creaking = new net.minecraft.client.model.monster.creaking.CreakingModel(root);
                creaking.setupAnim(state);
                model = creaking;
            } else if (id.contains("warden")) {
                var state = new net.minecraft.client.renderer.entity.state.WardenRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.tendrilAnimation = Oracle.f32(row, "tendril");
                var animations = new Object[][] {
                    {"roar", state.roarAnimationState}, {"sniff", state.sniffAnimationState},
                    {"emerge", state.emergeAnimationState}, {"digging", state.diggingAnimationState},
                    {"attack", state.attackAnimationState}, {"sonic_boom", state.sonicBoomAnimationState},
                };
                for (var animation : animations) {
                    if (row.has((String) animation[0])) {
                        ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                    }
                }
                var warden = new net.minecraft.client.model.monster.warden.WardenModel(root);
                warden.setupAnim(state);
                model = warden;
            } else if (id.contains("parrot")) {
                var state = new net.minecraft.client.renderer.entity.state.ParrotRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.flapAngle = Oracle.f32(row, "flap_angle");
                state.pose = net.minecraft.client.model.animal.parrot.ParrotModel.Pose.valueOf(row.get("pose").getAsString());
                var parrot = new net.minecraft.client.model.animal.parrot.ParrotModel(root);
                parrot.setupAnim(state);
                model = parrot;
            } else if (id.contains("bat")) {
                var state = new net.minecraft.client.renderer.entity.state.BatRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.isResting = row.get("resting").getAsBoolean();
                if (row.has("fly")) {
                    state.flyAnimationState.start(row.get("fly").getAsInt());
                }
                if (row.has("rest")) {
                    state.restAnimationState.start(row.get("rest").getAsInt());
                }
                var bat = new net.minecraft.client.model.ambient.BatModel(root);
                bat.setupAnim(state);
                model = bat;
            } else if (id.contains("vex")) {
                var state = new net.minecraft.client.renderer.entity.state.VexRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.isCharging = row.get("charging").getAsBoolean();
                if (row.get("right_item").getAsBoolean()) {
                    state.rightHandItemState.newLayer();
                }
                if (row.get("left_item").getAsBoolean()) {
                    state.leftHandItemState.newLayer();
                }
                var vex = new net.minecraft.client.model.monster.vex.VexModel(root);
                vex.setupAnim(state);
                model = vex;
            } else if (id.contains("allay")) {
                var state = new net.minecraft.client.renderer.entity.state.AllayRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.isDancing = row.get("dancing").getAsBoolean();
                state.isSpinning = row.get("spinning").getAsBoolean();
                state.spinningProgress = Oracle.f32(row, "spinning_progress");
                state.holdingAnimationProgress = Oracle.f32(row, "holding_progress");
                var allay = new net.minecraft.client.model.animal.allay.AllayModel(root);
                allay.setupAnim(state);
                model = allay;
            } else if (id.contains("ghast")) {
                var state = new net.minecraft.client.renderer.entity.state.GhastRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                var ghast = new net.minecraft.client.model.monster.ghast.GhastModel(root);
                ghast.setupAnim(state);
                model = ghast;
            } else {
                var state = new net.minecraft.client.renderer.entity.state.PhantomRenderState();
                state.flapTime = Oracle.f32(row, "flap_time");
                var phantom = new net.minecraft.client.model.monster.phantom.PhantomModel(root);
                phantom.setupAnim(state);
                model = phantom;
            }
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A vex's or an allay's model posed by the reference's own `setupAnim`, and where its own
     * `ItemInHandLayer` puts what each hand holds: the pose (a matrix) each held item is
     * submitted with, before its own display transform, the right hand's first.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject flierItemPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            boolean vex = row.get("kind").getAsString().equals("vex");
            net.minecraft.client.renderer.entity.state.ArmedEntityRenderState state;
            net.minecraft.client.model.EntityModel model;
            if (vex) {
                var vexState = new net.minecraft.client.renderer.entity.state.VexRenderState();
                vexState.isCharging = row.get("charging").getAsBoolean();
                state = vexState;
            } else {
                var allayState = new net.minecraft.client.renderer.entity.state.AllayRenderState();
                allayState.holdingAnimationProgress = Oracle.f32(row, "holding_progress");
                state = allayState;
            }
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.mainArm = row.get("main_arm").getAsString().equals("left")
                ? net.minecraft.world.entity.HumanoidArm.LEFT
                : net.minecraft.world.entity.HumanoidArm.RIGHT;
            if (row.get("holds_right").getAsBoolean()) {
                state.rightHandItemState.newLayer();
            }
            if (row.get("holds_left").getAsBoolean()) {
                state.leftHandItemState.newLayer();
            }
            var root = bakedLayer(vex ? "minecraft:vex#main" : "minecraft:allay#main");
            if (vex) {
                var vexModel = new net.minecraft.client.model.monster.vex.VexModel(root);
                vexModel.setupAnim((net.minecraft.client.renderer.entity.state.VexRenderState) state);
                model = vexModel;
            } else {
                var allayModel = new net.minecraft.client.model.animal.allay.AllayModel(root);
                allayModel.setupAnim((net.minecraft.client.renderer.entity.state.AllayRenderState) state);
                model = allayModel;
            }
            var poses = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitItem")) {
                        var matrix = new float[16];
                        ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                    }
                    return null;
                });
            var finalModel = model;
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> finalModel;
            new net.minecraft.client.renderer.entity.layers.ItemInHandLayer(parent)
                .submit(new com.mojang.blaze3d.vertex.PoseStack(), collector, 0, state, 0.0F, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("poses", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static net.minecraft.client.renderer.entity.state.BoatRenderState boatState(final JsonObject row) {
        var state = new net.minecraft.client.renderer.entity.state.BoatRenderState();
        state.yRot = Oracle.f32(row, "y_rot");
        state.hurtDir = row.get("hurt_dir").getAsInt();
        state.hurtTime = Oracle.f32(row, "hurt_time");
        state.damageTime = Oracle.f32(row, "damage_time");
        state.bubbleAngle = Oracle.f32(row, "bubble_angle");
        state.isUnderWater = row.get("under_water").getAsBoolean();
        state.rowingTimeLeft = Oracle.f32(row, "rowing_left");
        state.rowingTimeRight = Oracle.f32(row, "rowing_right");
        return state;
    }

    /**
     * The boat models (boats, chest boats, rafts) posed by the reference's own `setupAnim`, drawn,
     * from the time each paddle has rowed, and the water patch.
     */
    private static JsonObject boatModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.Model<?> model;
            if (id.contains("water_patch")) {
                model = new net.minecraft.client.model.Model.Simple(root, t -> net.minecraft.client.renderer.rendertype.RenderTypes.waterMask());
            } else {
                var state = boatState(row);
                var boat = id.contains("/bamboo")
                    ? new net.minecraft.client.model.object.boat.RaftModel(root)
                    : new net.minecraft.client.model.object.boat.BoatModel(root);
                boat.setupAnim(state);
                model = boat;
            }
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Where the reference's own `AbstractBoatRenderer.submit` draws a boat's model (and a boat's
     * water patch): the matrix it submits each with, from the boat's turn, hit, bubbles and
     * water, from an identity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject boatPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = boatState(row);
            var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.BoatRenderer.class);
            var poses = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitModel")) {
                        var matrix = new float[16];
                        ((com.mojang.blaze3d.vertex.PoseStack) args[2]).last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            ((net.minecraft.client.renderer.entity.BoatRenderer) renderer).submit(state, stack, collector, null);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("poses", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Where the reference's own thrown item renderer, and its firework rocket's, draw an item from
     * the camera it is seen by: the pose each leaves the item's single layer at.
     */
    private static JsonObject thrownItemPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var camera = new net.minecraft.client.renderer.state.level.CameraRenderState();
            float yaw = Oracle.f32(row, "camera_y_rot");
            float pitch = Oracle.f32(row, "camera_x_rot");
            // `Camera.setRotation`.
            camera.orientation.rotationYXZ((float) Math.PI - yaw * (float) (Math.PI / 180.0),
                -pitch * (float) (Math.PI / 180.0), 0.0F);
            var poses = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitItem")) {
                        var matrix = new float[16];
                        ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            if (row.get("firework").getAsBoolean()) {
                var state = new net.minecraft.client.renderer.entity.state.FireworkRocketRenderState();
                state.isShotAtAngle = row.get("shot_at_angle").getAsBoolean();
                state.item.newLayer();
                var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.FireworkEntityRenderer.class);
                ((net.minecraft.client.renderer.entity.FireworkEntityRenderer) renderer).submit(state, stack, collector, camera);
            } else {
                var state = new net.minecraft.client.renderer.entity.state.ThrownItemRenderState();
                state.item.newLayer();
                var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.ThrownItemRenderer.class);
                var scale = net.minecraft.client.renderer.entity.ThrownItemRenderer.class.getDeclaredField("scale");
                scale.setAccessible(true);
                scale.setFloat(renderer, Oracle.f32(row, "scale"));
                ((net.minecraft.client.renderer.entity.ThrownItemRenderer) renderer).submit(state, stack, collector, camera);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("poses", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Where the reference's own arrow renderer draws an arrow from the turns it was sent. */
    private static JsonObject arrowPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.entity.state.ArrowRenderState();
            state.xRot = Oracle.f32(row, "x_rot");
            state.yRot = Oracle.f32(row, "y_rot");
            state.shake = Oracle.f32(row, "shake");
            boolean trident = row.has("trident");
            var tridentState = new net.minecraft.client.renderer.entity.state.ThrownTridentRenderState();
            tridentState.xRot = state.xRot;
            tridentState.yRot = state.yRot;
            var renderer = UNSAFE.allocateInstance(trident ? net.minecraft.client.renderer.entity.ThrownTridentRenderer.class
                : net.minecraft.client.renderer.entity.SpectralArrowRenderer.class);
            var poses = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("order")) {
                        return proxy;
                    }
                    if (method.getName().equals("submitModel")) {
                        var matrix = new float[16];
                        ((com.mojang.blaze3d.vertex.PoseStack) args[2]).last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            if (trident) {
                ((net.minecraft.client.renderer.entity.ThrownTridentRenderer) renderer).submit(tridentState, stack, collector, null);
            } else {
                ((net.minecraft.client.renderer.entity.SpectralArrowRenderer) renderer).submit(state, stack, collector, null);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("poses", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Where the reference's own minecart renderer draws a minecart (and the block it carries) from
     * the state its old behaviour extracts: the pose each of its two submissions leaves.
     */
    private static JsonObject minecartPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var tnt = row.has("tnt_fuse");
            var state = tnt ? new net.minecraft.client.renderer.entity.state.MinecartTntRenderState()
                : new net.minecraft.client.renderer.entity.state.MinecartRenderState();
            if (tnt) {
                ((net.minecraft.client.renderer.entity.state.MinecartTntRenderState) state).fuseRemainingInTicks =
                    Oracle.f32(row, "tnt_fuse");
            }
            state.x = Oracle.f64(row, "x");
            state.y = Oracle.f64(row, "y");
            state.z = Oracle.f64(row, "z");
            state.xRot = Oracle.f32(row, "x_rot");
            state.yRot = Oracle.f32(row, "y_rot");
            state.offsetSeed = Long.parseLong(row.get("offset_seed").getAsString());
            state.hurtDir = row.get("hurt_dir").getAsInt();
            state.hurtTime = Oracle.f32(row, "hurt_time");
            state.damageTime = Oracle.f32(row, "damage_time");
            state.displayOffset = row.get("display_offset").getAsInt();
            state.isNewRender = false;
            if (row.has("rail")) {
                var rail = row.getAsJsonObject("rail");
                state.posOnRail = vec3(rail.getAsJsonArray("pos"));
                state.frontPos = vec3(rail.getAsJsonArray("front"));
                state.backPos = vec3(rail.getAsJsonArray("back"));
            }
            if (row.get("block").getAsBoolean()) {
                var parts = state.displayBlockModel.setupModel(new org.joml.Matrix4f(), false);
                parts.add((net.minecraft.client.renderer.block.dispatch.BlockStateModelPart)
                    java.lang.reflect.Proxy.newProxyInstance(EntityOracle.class.getClassLoader(),
                        new Class<?>[] {net.minecraft.client.renderer.block.dispatch.BlockStateModelPart.class},
                        (proxy, method, args) -> null));
            }
            var renderer = UNSAFE.allocateInstance(tnt ? net.minecraft.client.renderer.entity.TntMinecartRenderer.class
                : net.minecraft.client.renderer.entity.MinecartRenderer.class);
            var poses = new JsonArray();
            var overlays = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitModel") || method.getName().equals("submitBlockModel")) {
                        var stack = (com.mojang.blaze3d.vertex.PoseStack) (method.getName().equals("submitModel") ? args[2] : args[0]);
                        var matrix = new float[16];
                        stack.last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                        if (method.getName().equals("submitBlockModel")) {
                            overlays.add((int) args[5]);
                        }
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            if (tnt) {
                ((net.minecraft.client.renderer.entity.TntMinecartRenderer) renderer).submit(
                    (net.minecraft.client.renderer.entity.state.MinecartTntRenderState) state, stack, collector, null);
            } else {
                ((net.minecraft.client.renderer.entity.MinecartRenderer) renderer).submit(state, stack, collector, null);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("poses", poses);
            out.add("overlays", overlays);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The texture each parrot variant wears (`ParrotRenderer.getVariantTexture`), by id. */
    private static JsonObject parrotTextures(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("texture", net.minecraft.client.renderer.entity.ParrotRenderer.getVariantTexture(
                net.minecraft.world.entity.animal.parrot.Parrot.Variant.byId(row.get("id").getAsInt())).toString());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The guardian models (grown, elder) posed by the reference's own `setupAnim`, drawn, from the
     * look, the spikes' and the tail's animation and (optionally) what the eye follows.
     */
    private static JsonObject guardianModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var root = bakedLayer(row.get("layer").getAsString());
            var state = new net.minecraft.client.renderer.entity.state.GuardianRenderState();
            state.yRot = Oracle.f32(row, "y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.spikesAnimation = Oracle.f32(row, "spikes");
            state.tailAnimation = Oracle.f32(row, "tail");
            if (row.has("eye")) {
                var eye = row.getAsJsonObject("eye");
                state.eyePosition = vec3(eye.getAsJsonArray("eye_position"));
                state.lookDirection = vec3(eye.getAsJsonArray("look_direction"));
                state.lookAtPosition = vec3(eye.getAsJsonArray("look_at_position"));
            }
            var model = new net.minecraft.client.model.monster.guardian.GuardianModel(root);
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A guardian's beam as the reference's own `renderBeam` submits it from the pose the entity's
     * eye leaves: every vertex it hands the vertex consumer (placed, with its texture coordinate,
     * colour, light and normal).
     */
    private static JsonObject guardianBeam(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var render = net.minecraft.client.renderer.entity.GuardianRenderer.class.getDeclaredMethod("renderBeam",
            com.mojang.blaze3d.vertex.PoseStack.class, net.minecraft.client.renderer.SubmitNodeCollector.class,
            net.minecraft.world.phys.Vec3.class, float.class, float.class, float.class);
        render.setAccessible(true);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var vertices = new JsonArray();
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "x"), Oracle.f32(row, "y"), Oracle.f32(row, "z"));
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitCustomGeometry")) {
                        var capture = new BeamCapture(vertices);
                        ((net.minecraft.client.renderer.SubmitNodeCollector.CustomGeometryRenderer) args[2])
                            .render(((com.mojang.blaze3d.vertex.PoseStack) args[0]).last(), capture);
                        capture.flush();
                    }
                    return null;
                });
            render.invoke(null, stack, collector, vec3(row.getAsJsonArray("vector")),
                Oracle.f32(row, "time"), Oracle.f32(row, "scale"), Oracle.f32(row, "tex_v_off"));
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("vertices", vertices);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The beam of an end crystal, as the reference's own `submitCrystalBeams` builds it. */
    private static JsonObject crystalBeam(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var vertices = new JsonArray();
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "x"), Oracle.f32(row, "y"), Oracle.f32(row, "z"));
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitCustomGeometry")) {
                        var capture = new BeamCapture(vertices);
                        ((net.minecraft.client.renderer.SubmitNodeCollector.CustomGeometryRenderer) args[2])
                            .render(((com.mojang.blaze3d.vertex.PoseStack) args[0]).last(), capture);
                        capture.flush();
                    }
                    return null;
                });
            net.minecraft.client.renderer.entity.EnderDragonRenderer.submitCrystalBeams(
                Oracle.f32(row, "dx"), Oracle.f32(row, "dy"), Oracle.f32(row, "dz"), Oracle.f32(row, "time"),
                stack, collector, row.get("light").getAsInt());
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("vertices", vertices);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The rays of a dying ender dragon, as the reference's own `submitRays` builds them. */
    private static JsonObject dragonRays(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var render = net.minecraft.client.renderer.entity.EnderDragonRenderer.class.getDeclaredMethod("submitRays",
            com.mojang.blaze3d.vertex.PoseStack.class, float.class,
            net.minecraft.client.renderer.SubmitNodeCollector.class,
            net.minecraft.client.renderer.rendertype.RenderType.class);
        render.setAccessible(true);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var vertices = new JsonArray();
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "x"), Oracle.f32(row, "y"), Oracle.f32(row, "z"));
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitCustomGeometry")) {
                        var capture = new BeamCapture(vertices);
                        ((net.minecraft.client.renderer.SubmitNodeCollector.CustomGeometryRenderer) args[2])
                            .render(((com.mojang.blaze3d.vertex.PoseStack) args[0]).last(), capture);
                        capture.flush();
                    }
                    return null;
                });
            render.invoke(null, stack, Oracle.f32(row, "death_time"), collector, null);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("vertices", vertices);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Takes each vertex a beam's geometry builds, one chained call at a time. */
    static final class BeamCapture implements com.mojang.blaze3d.vertex.VertexConsumer {
        private final JsonArray out;
        private float[] position;
        private float u;
        private float v;
        private int color;
        private int light;
        private float[] normal;

        BeamCapture(final JsonArray out) {
            this.out = out;
        }

        void flush() {
            if (this.position == null) {
                return;
            }
            this.out.add(Oracle.bits(this.position[0]) + Oracle.bits(this.position[1]) + Oracle.bits(this.position[2])
                + Oracle.bits(this.u) + Oracle.bits(this.v) + String.format("%08x%08x", this.color, this.light)
                + (this.normal == null ? "000000000000000000000000"
                    : Oracle.bits(this.normal[0]) + Oracle.bits(this.normal[1]) + Oracle.bits(this.normal[2])));
            this.position = null;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer addVertex(final float x, final float y, final float z) {
            this.flush();
            this.position = new float[] {x, y, z};
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int r, final int g, final int b, final int a) {
            this.color = (a << 24) | (r << 16) | (g << 8) | b;
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int color) {
            this.color = color;
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv(final float u, final float v) {
            this.u = u;
            this.v = v;
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv1(final int u, final int v) {
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv2(final int u, final int v) {
            this.light = (v << 16) | (u & 0xFFFF);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setNormal(final float x, final float y, final float z) {
            this.normal = new float[] {x, y, z};
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setLineWidth(final float width) {
            return this;
        }

        @Override
        public void addVertex(
            final float x, final float y, final float z, final int color, final float u, final float v,
            final int overlayCoords, final int lightCoords, final float nx, final float ny, final float nz) {
            throw new UnsupportedOperationException();
        }
    }

    private static net.minecraft.world.phys.Vec3 vec3(final JsonArray bits) {
        return new net.minecraft.world.phys.Vec3(
            Double.longBitsToDouble(Long.parseUnsignedLong(bits.get(0).getAsString(), 16)),
            Double.longBitsToDouble(Long.parseUnsignedLong(bits.get(1).getAsString(), 16)),
            Double.longBitsToDouble(Long.parseUnsignedLong(bits.get(2).getAsString(), 16)));
    }

    /**
     * The nautilus models (grown, baby, zombie, zombie with corals) posed by the reference's own
     * `setupAnim`, drawn, from the look, the stride, the age and whether body armor is worn.
     */
    private static JsonObject nautilusModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var state = new net.minecraft.client.renderer.entity.state.NautilusRenderState();
            state.yRot = Oracle.f32(row, "y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            if (row.get("armored").getAsBoolean()) {
                state.bodyArmorItem = new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.IRON_HORSE_ARMOR);
            }
            net.minecraft.client.model.animal.nautilus.NautilusModel model = id.contains("coral")
                ? new net.minecraft.client.model.monster.nautilus.ZombieNautilusCoralModel(root)
                : id.contains("saddle")
                    ? new net.minecraft.client.model.animal.nautilus.NautilusSaddleModel(root)
                    : id.contains("_armor")
                        ? new net.minecraft.client.model.animal.nautilus.NautilusArmorModel(root)
                        : new net.minecraft.client.model.animal.nautilus.NautilusModel(root);
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Where `DolphinCarryingItemLayer` submits what a dolphin carries, from an identity pose and
     * the dolphin's pitch: the matrix of the item.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject dolphinItemPose(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var poses = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitItem")) {
                        var matrix = new float[16];
                        ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(matrix);
                        for (float value : matrix) {
                            poses.add(Oracle.bits(value));
                        }
                    }
                    return null;
                });
            var state = new net.minecraft.client.renderer.entity.state.DolphinRenderState();
            state.xRot = Oracle.f32(row, "x_rot");
            state.heldItem.newLayer();
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> null;
            new net.minecraft.client.renderer.entity.layers.DolphinCarryingItemLayer(parent)
                .submit(new com.mojang.blaze3d.vertex.PoseStack(), collector, 0, state, 0.0F, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("pose", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The fish's models posed by the reference's own `setupAnim`, drawn: each layer by its class,
     * with the age and whether it is in water.
     */
    private static JsonObject fishModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            float age = Oracle.f32(row, "age_in_ticks");
            boolean inWater = row.get("in_water").getAsBoolean();
            net.minecraft.client.model.EntityModel<?> model;
            if (id.startsWith("minecraft:cod#")) {
                var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
                state.ageInTicks = age;
                state.isInWater = inWater;
                var cod = new net.minecraft.client.model.animal.fish.CodModel(root);
                cod.setupAnim(state);
                model = cod;
            } else if (id.startsWith("minecraft:salmon")) {
                var state = new net.minecraft.client.renderer.entity.state.SalmonRenderState();
                state.ageInTicks = age;
                state.isInWater = inWater;
                var salmon = new net.minecraft.client.model.animal.fish.SalmonModel(root);
                salmon.setupAnim(state);
                model = salmon;
            } else if (id.startsWith("minecraft:tropical_fish")) {
                var state = new net.minecraft.client.renderer.entity.state.TropicalFishRenderState();
                state.ageInTicks = age;
                state.isInWater = inWater;
                if (id.startsWith("minecraft:tropical_fish_large")) {
                    var fish = new net.minecraft.client.model.animal.fish.TropicalFishLargeModel(root);
                    fish.setupAnim(state);
                    model = fish;
                } else {
                    var fish = new net.minecraft.client.model.animal.fish.TropicalFishSmallModel(root);
                    fish.setupAnim(state);
                    model = fish;
                }
            } else {
                var state = new net.minecraft.client.renderer.entity.state.EntityRenderState();
                state.ageInTicks = age;
                if (id.startsWith("minecraft:pufferfish_small")) {
                    var fish = new net.minecraft.client.model.animal.fish.PufferfishSmallModel(root);
                    fish.setupAnim(state);
                    model = fish;
                } else if (id.startsWith("minecraft:pufferfish_medium")) {
                    var fish = new net.minecraft.client.model.animal.fish.PufferfishMidModel(root);
                    fish.setupAnim(state);
                    model = fish;
                } else {
                    var fish = new net.minecraft.client.model.animal.fish.PufferfishBigModel(root);
                    fish.setupAnim(state);
                    model = fish;
                }
            }
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Which textures the fish wear: the cod's, the salmon's and the pufferfish's, and for each
     * tropical fish pattern its packed id, the texture of its base and the texture of its pattern,
     * from the renderer's and the layer's own constants.
     */
    private static JsonObject fishTextures(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            if (!row.has("packed_id")) {
                var described = switch (row.get("label").getAsString()) {
                    case "cod" -> new Object[] {net.minecraft.client.renderer.entity.CodRenderer.class, "COD_LOCATION"};
                    case "salmon" -> new Object[] {net.minecraft.client.renderer.entity.SalmonRenderer.class, "SALMON_LOCATION"};
                    default -> new Object[] {net.minecraft.client.renderer.entity.PufferfishRenderer.class, "PUFFER_LOCATION"};
                };
                var field = ((Class<?>) described[0]).getDeclaredField((String) described[1]);
                field.setAccessible(true);
                out.addProperty("texture", field.get(null).toString());
                rows.add(out);
                continue;
            }
            int packed = row.get("packed_id").getAsInt();
            var pattern = net.minecraft.world.entity.animal.fish.TropicalFish.getPattern(packed);
            boolean large = pattern.base() == net.minecraft.world.entity.animal.fish.TropicalFish.Base.LARGE;
            var base = net.minecraft.client.renderer.entity.TropicalFishRenderer.class
                .getDeclaredField(large ? "LARGE_TEXTURE" : "SMALL_TEXTURE");
            base.setAccessible(true);
            var layer = net.minecraft.client.renderer.entity.layers.TropicalFishPatternLayer.class
                .getDeclaredField(pattern.name() + "_TEXTURE");
            layer.setAccessible(true);
            out.addProperty("packed_id", packed);
            out.addProperty("texture", base.get(null).toString());
            out.addProperty("pattern_texture", layer.get(null).toString());
            out.addProperty("base_dye", net.minecraft.world.entity.animal.fish.TropicalFish.getBaseColor(packed).getId());
            out.addProperty("pattern_dye", net.minecraft.world.entity.animal.fish.TropicalFish.getPatternColor(packed).getId());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The shulker's model posed by the reference's own `ShulkerModel.setupAnim`, drawn. */
    private static JsonObject shulkerModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.shulker.ShulkerModel(
                bakedLayer(row.get("layer").getAsString()));
            var state = new net.minecraft.client.renderer.entity.state.ShulkerRenderState();
            state.peekAmount = Oracle.f32(row, "peek_amount");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.xRot = Oracle.f32(row, "x_rot");
            state.yHeadRot = Oracle.f32(row, "y_head_rot");
            state.yBodyRot = Oracle.f32(row, "y_body_rot");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The magma cube's model posed by the reference's own `MagmaCubeModel.setupAnim`, drawn. */
    private static JsonObject magmaCubeModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.slime.MagmaCubeModel(
                bakedLayer(row.get("layer").getAsString()));
            var state = new net.minecraft.client.renderer.entity.state.SlimeRenderState();
            state.squish = Oracle.f32(row, "squish");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What `GlowSquidRenderer.getBlockLightLevel` makes of the dark ticks and the light around: the
     * glow comes from the reference's own `Mth.clampedLerp`, cut to a whole level.
     */
    private static JsonObject glowSquidLight(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            int glow = (int) net.minecraft.util.Mth.clampedLerp(1.0F - row.get("dark_ticks").getAsInt() / 10.0F, 0.0F, 15.0F);
            int light = glow == 15 ? 15 : Math.max(glow, row.get("block_light").getAsInt());
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("light", light);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The copper golem's model posed by the reference's own `CopperGolemModel.setupAnim`, drawn. */
    private static JsonObject copperGolemModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.animal.golem.CopperGolemModel(
                bakedLayer("minecraft:copper_golem#main"));
            var state = new net.minecraft.client.renderer.entity.state.CopperGolemRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            if (row.get("holds_item").getAsBoolean()) {
                state.rightHandItemState.newLayer();
            }
            var animations = new Object[][] {
                {"idle", state.idleAnimationState},
                {"get_item", state.interactionGetItem},
                {"get_no_item", state.interactionGetNoItem},
                {"drop_item", state.interactionDropItem},
                {"drop_no_item", state.interactionDropNoItem},
            };
            for (var animation : animations) {
                if (row.has((String) animation[0])) {
                    ((net.minecraft.world.entity.AnimationState) animation[1]).start(
                        row.get((String) animation[0]).getAsInt());
                }
            }
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The silverfish's and the endermite's models, posed by the reference's own `setupAnim`, drawn. */
    private static JsonObject segmentedModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.EntityModel<net.minecraft.client.renderer.entity.state.EntityRenderState> model =
                id.startsWith("minecraft:silverfish")
                    ? new net.minecraft.client.model.monster.silverfish.SilverfishModel(root)
                    : new net.minecraft.client.model.monster.endermite.EndermiteModel(root);
            var state = new net.minecraft.client.renderer.entity.state.EntityRenderState();
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The bee's models, posed by the reference's own `BeeModel.setupAnim`, drawn. */
    private static JsonObject beeModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.animal.bee.BeeModel model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.bee.BabyBeeModel(root)
                : new net.minecraft.client.model.animal.bee.AdultBeeModel(root);
            var state = new net.minecraft.client.renderer.entity.state.BeeRenderState();
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.rollAmount = Oracle.f32(row, "roll_amount");
            state.hasStinger = row.get("has_stinger").getAsBoolean();
            state.isOnGround = row.get("on_ground").getAsBoolean();
            state.isAngry = row.get("angry").getAsBoolean();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the charged creeper's swirl layer submits, from the reference's own `CreeperPowerLayer`:
     * nothing unless it is charged; then the draw order, the texture, the pipeline's state, the
     * texture offset, the colour it is multiplied by, the light and the overlay.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject swirlLayers(final JsonObject request) throws Exception {
        var roots = net.minecraft.client.model.geom.LayerDefinitions.createRoots();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var order = new int[] {-1};
            var holder = new net.minecraft.client.renderer.SubmitNodeCollector[1];
            holder[0] = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("order")) {
                        order[0] = (int) args[0];
                        return holder[0];
                    }
                    if (method.getName().equals("submitModel")) {
                        var renderType = (net.minecraft.client.renderer.rendertype.RenderType) args[3];
                        out.addProperty("order", order[0]);
                        out.addProperty("light", (int) args[4]);
                        out.addProperty("overlay", (int) args[5]);
                        out.addProperty("colour", (int) args[6]);
                        var found = java.util.regex.Pattern.compile("minecraft:(textures/[a-z_/]+\\.png)")
                            .matcher(String.valueOf(renderType));
                        out.addProperty("texture", found.find() ? found.group(1) : String.valueOf(renderType));
                        var stateField = net.minecraft.client.renderer.rendertype.RenderType.class.getDeclaredField("state");
                        stateField.setAccessible(true);
                        var setup = stateField.get(renderType);
                        var pipelineField = setup.getClass().getDeclaredField("pipeline");
                        pipelineField.setAccessible(true);
                        var pipeline = (com.mojang.blaze3d.pipeline.RenderPipeline) pipelineField.get(setup);
                        out.addProperty("pipeline", pipeline.getLocation().toString());
                        out.addProperty("flags", String.join(",", new java.util.TreeSet<>(pipeline.getShaderDefines().flags())));
                        var values = new java.util.TreeMap<>(pipeline.getShaderDefines().values());
                        var text = new StringBuilder();
                        values.forEach((key, value) -> text.append(key).append('=').append(value).append(','));
                        out.addProperty("values", text.toString());
                        var blend = pipeline.getColorTargetState().blendFunction();
                        out.addProperty("blend", blend.isEmpty() ? "NONE"
                            : blend.get().equals(com.mojang.blaze3d.pipeline.BlendFunction.TRANSLUCENT) ? "TRANSLUCENT"
                            : blend.get().equals(com.mojang.blaze3d.pipeline.BlendFunction.ADDITIVE) ? "ADDITIVE" : "OTHER");
                        out.addProperty("depth_test", pipeline.getDepthStencilState().depthTest().name());
                        out.addProperty("depth_write", pipeline.getDepthStencilState().writeDepth());
                        out.addProperty("cull", pipeline.isCull());
                        var transformField = setup.getClass().getDeclaredField("textureTransform");
                        transformField.setAccessible(true);
                        var matrix = ((net.minecraft.client.renderer.rendertype.TextureTransform) transformField.get(setup))
                            .createMatrix();
                        out.addProperty("u_offset", Oracle.bits(matrix.m30()));
                        out.addProperty("v_offset", Oracle.bits(matrix.m31()));
                    }
                    return null;
                });
            var state = new net.minecraft.client.renderer.entity.state.CreeperRenderState();
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isPowered = row.get("powered").getAsBoolean();
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> null;
            new net.minecraft.client.renderer.entity.layers.CreeperPowerLayer(
                parent, new net.minecraft.client.model.geom.EntityModelSet(roots))
                .submit(new com.mojang.blaze3d.vertex.PoseStack(), holder[0], 15728880, state, 0.0F, 0.0F);
            out.addProperty("submitted", out.has("order"));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A horse, donkey or mule model from the layer table, posed by the reference's own `setupAnim`, drawn. */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject equineModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            boolean baby = id.contains("_baby");
            boolean donkey = id.startsWith("minecraft:donkey") || id.startsWith("minecraft:mule");
            net.minecraft.client.model.animal.equine.AbstractEquineModel model;
            net.minecraft.client.renderer.entity.state.EquineRenderState state;
            if (donkey) {
                model = baby
                    ? new net.minecraft.client.model.animal.equine.BabyDonkeyModel(root)
                    : new net.minecraft.client.model.animal.equine.DonkeyModel(root);
                var donkeyState = new net.minecraft.client.renderer.entity.state.DonkeyRenderState();
                donkeyState.hasChest = row.get("has_chest").getAsBoolean();
                state = donkeyState;
            } else {
                model = baby
                    ? new net.minecraft.client.model.animal.equine.BabyHorseModel(root)
                    : new net.minecraft.client.model.animal.equine.HorseModel(root);
                state = new net.minecraft.client.renderer.entity.state.EquineRenderState();
            }
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.eatAnimation = Oracle.f32(row, "eat");
            state.standAnimation = Oracle.f32(row, "stand");
            state.feedingAnimation = Oracle.f32(row, "feeding");
            state.animateTail = row.get("animate_tail").getAsBoolean();
            state.isInWater = row.get("in_water").getAsBoolean();
            state.ageScale = baby ? 0.5F : 1.0F;
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The camel's model posed by the reference's own `CamelModel.setupAnim` with its animation states, drawn. */
    private static JsonObject camelModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.animal.camel.CamelModel model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.camel.BabyCamelModel(root)
                : new net.minecraft.client.model.animal.camel.AdultCamelModel(root);
            var state = new net.minecraft.client.renderer.entity.state.CamelRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.jumpCooldown = Oracle.f32(row, "jump_cooldown");
            var states = new net.minecraft.world.entity.AnimationState[] {
                state.sitAnimationState, state.sitPoseAnimationState, state.sitUpAnimationState,
                state.idleAnimationState, state.dashAnimationState};
            String[] names = {"sit", "sit_pose", "sit_up", "idle", "dash"};
            for (int i = 0; i < names.length; i++) {
                if (row.has(names[i])) {
                    states[i].start(row.get(names[i]).getAsInt());
                }
            }
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The llama's model from the layer table, posed by the reference's own `LlamaModel.setupAnim`, drawn. */
    private static JsonObject llamaModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.llama.BabyLlamaModel(root)
                : new net.minecraft.client.model.animal.llama.LlamaModel(root);
            var state = new net.minecraft.client.renderer.entity.state.LlamaRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.hasChest = row.get("has_chest").getAsBoolean();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The wolf's model posed by the reference's own `WolfModel.setupAnim` (grown and baby), drawn. */
    private static JsonObject wolfModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.animal.wolf.WolfModel model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.wolf.BabyWolfModel(root)
                : new net.minecraft.client.model.animal.wolf.AdultWolfModel(root);
            var state = new net.minecraft.client.renderer.entity.state.WolfRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.isAngry = row.get("angry").getAsBoolean();
            state.isSitting = row.get("sitting").getAsBoolean();
            state.tailAngle = Oracle.f32(row, "tail_angle");
            state.headRollAngle = Oracle.f32(row, "head_roll_angle");
            state.shakeAnim = Oracle.f32(row, "shake_anim");
            state.ageScale = Oracle.f32(row, "age_scale");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The cat's and ocelot's models posed by the reference's own `setupAnim` (grown and baby), drawn. */
    private static JsonObject felineModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            boolean baby = id.contains("_baby");
            var state = id.contains("cat")
                ? new net.minecraft.client.renderer.entity.state.CatRenderState()
                : new net.minecraft.client.renderer.entity.state.FelineRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.isCrouching = row.get("crouching").getAsBoolean();
            state.isSprinting = row.get("sprinting").getAsBoolean();
            state.isSitting = row.get("sitting").getAsBoolean();
            state.lieDownAmount = Oracle.f32(row, "lie_down_amount");
            state.lieDownAmountTail = Oracle.f32(row, "lie_down_amount_tail");
            state.relaxStateOneAmount = Oracle.f32(row, "relax_state_one_amount");
            state.ageScale = Oracle.f32(row, "age_scale");
            net.minecraft.client.model.EntityModel<?> model;
            if (id.contains("cat")) {
                var cat = (net.minecraft.client.renderer.entity.state.CatRenderState) state;
                var catModel = baby
                    ? new net.minecraft.client.model.animal.feline.BabyCatModel(root)
                    : new net.minecraft.client.model.animal.feline.AdultCatModel(root);
                catModel.setupAnim(cat);
                model = catModel;
            } else {
                var ocelotModel = baby
                    ? new net.minecraft.client.model.animal.feline.BabyOcelotModel(root)
                    : new net.minecraft.client.model.animal.feline.AdultOcelotModel(root);
                ocelotModel.setupAnim(state);
                model = ocelotModel;
            }
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The polar bear's model posed by the reference's own `PolarBearModel.setupAnim` (grown and baby), drawn. */
    private static JsonObject polarBearModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.animal.polarbear.PolarBearModel(bakedLayer(row.get("layer").getAsString()));
            var state = new net.minecraft.client.renderer.entity.state.PolarBearRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.standScale = Oracle.f32(row, "stand_scale");
            state.ageScale = Oracle.f32(row, "age_scale");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The hoglin's (and zoglin's) model posed by the reference's own `setupAnim` (grown and baby), drawn. */
    private static JsonObject hoglinModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.monster.hoglin.HoglinModel model = id.contains("_baby")
                ? new net.minecraft.client.model.monster.hoglin.BabyHoglinModel(root)
                : new net.minecraft.client.model.monster.hoglin.HoglinModel(root);
            var state = new net.minecraft.client.renderer.entity.state.HoglinRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.attackAnimationRemainingTicks = row.get("attack_ticks").getAsInt();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The fox's model posed by the reference's own `FoxModel.setupAnim` (grown and baby), drawn, and
     * the matrix `FoxHeldItemLayer` submits what it holds with from an identity entity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject foxModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            boolean baby = id.contains("_baby");
            net.minecraft.client.model.animal.fox.FoxModel model = baby
                ? new net.minecraft.client.model.animal.fox.BabyFoxModel(root)
                : new net.minecraft.client.model.animal.fox.AdultFoxModel(root);
            var state = new net.minecraft.client.renderer.entity.state.FoxRenderState();
            state.isBaby = baby;
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.headRollAngle = Oracle.f32(row, "head_roll_angle");
            state.crouchAmount = Oracle.f32(row, "crouch_amount");
            state.isCrouching = row.get("crouching").getAsBoolean();
            state.isSleeping = row.get("sleeping").getAsBoolean();
            state.isSitting = row.get("sitting").getAsBoolean();
            state.isFaceplanted = row.get("faceplanted").getAsBoolean();
            state.isPouncing = row.get("pouncing").getAsBoolean();
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.ageScale = Oracle.f32(row, "age_scale");
            model.setupAnim(state);
            var out = posedQuads(row.get("label").getAsString(), model);
            var poses = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitItem")) {
                        var matrix = new float[16];
                        ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                    }
                    return null;
                });
            state.heldItem.newLayer();
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
            new net.minecraft.client.renderer.entity.layers.FoxHeldItemLayer(parent)
                .submit(new com.mojang.blaze3d.vertex.PoseStack(), collector, 0, state, state.yRot, state.xRot);
            out.add("poses", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The fox's textures by variant, state and age, from the renderer's own table. */
    private static JsonObject foxTextures() throws Exception {
        var field = net.minecraft.client.renderer.entity.FoxRenderer.class.getDeclaredField("TEXTURES_BY_VARIANT");
        field.setAccessible(true);
        var table = (java.util.Map<?, ?>) field.get(null);
        var rows = new JsonArray();
        for (var variant : net.minecraft.world.entity.animal.fox.Fox.Variant.values()) {
            var byState = table.get(variant);
            for (String stateName : new String[] {"idle", "sleeping"}) {
                var stateMethod = byState.getClass().getDeclaredMethod(stateName);
                stateMethod.setAccessible(true);
                var byAge = stateMethod.invoke(byState);
                for (String age : new String[] {"adult", "baby"}) {
                    var ageMethod = byAge.getClass().getDeclaredMethod(age);
                    ageMethod.setAccessible(true);
                    var out = new JsonObject();
                    out.addProperty("label", variant.getId() + "-" + stateName + "-" + age);
                    out.addProperty("texture", ageMethod.invoke(byAge).toString());
                    rows.add(out);
                }
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The panda's model posed by the reference's own `PandaModel.setupAnim` (grown and baby), drawn.
     */
    private static JsonObject pandaModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.panda.BabyPandaModel(root)
                : new net.minecraft.client.model.animal.panda.PandaModel(root);
            var state = new net.minecraft.client.renderer.entity.state.PandaRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isUnhappy = row.get("unhappy").getAsBoolean();
            state.isSneezing = row.get("sneezing").getAsBoolean();
            state.sneezeTime = row.get("sneeze_time").getAsInt();
            state.isEating = row.get("eating").getAsBoolean();
            state.isScared = row.get("scared").getAsBoolean();
            state.sitAmount = Oracle.f32(row, "sit_amount");
            state.lieOnBackAmount = Oracle.f32(row, "lie_on_back_amount");
            state.rollAmount = Oracle.f32(row, "roll_amount");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The panda's textures by gene and age, from the renderer's own tables. */
    private static JsonObject pandaTextures() throws Exception {
        var rows = new JsonArray();
        for (String table : new String[] {"TEXTURES", "BABY_TEXTURES"}) {
            var field = net.minecraft.client.renderer.entity.PandaRenderer.class.getDeclaredField(table);
            field.setAccessible(true);
            var map = (java.util.Map<?, ?>) field.get(null);
            for (var gene : net.minecraft.world.entity.animal.panda.Panda.Gene.values()) {
                var out = new JsonObject();
                out.addProperty("label", gene.getId() + "-" + table);
                out.addProperty("texture", map.get(gene).toString());
                rows.add(out);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the reference's own `setupRotations` leaves the pose stack at for a panda, a cat or a fox
     * from the state that drives it, and, for a panda, the matrix `PandaHoldsItemLayer` submits what
     * it holds with from an identity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static JsonObject setupRotations(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String kind = row.get("kind").getAsString();
            Class<?> rendererClass;
            net.minecraft.client.renderer.entity.state.LivingEntityRenderState state;
            var poses = new JsonArray();
            if (kind.equals("panda")) {
                rendererClass = net.minecraft.client.renderer.entity.PandaRenderer.class;
                var panda = new net.minecraft.client.renderer.entity.state.PandaRenderState();
                panda.rollTime = Oracle.f32(row, "roll_time");
                panda.sitAmount = Oracle.f32(row, "sit_amount");
                panda.lieOnBackAmount = Oracle.f32(row, "lie_on_back_amount");
                panda.isScared = row.get("scared").getAsBoolean();
                panda.isSitting = row.get("sitting").getAsBoolean();
                panda.isEating = row.get("eating").getAsBoolean();
                state = panda;
            } else if (kind.equals("squid")) {
                rendererClass = net.minecraft.client.renderer.entity.SquidRenderer.class;
                var squid = new net.minecraft.client.renderer.entity.state.SquidRenderState();
                squid.xBodyRot = Oracle.f32(row, "x_body_rot");
                squid.zBodyRot = Oracle.f32(row, "z_body_rot");
                state = squid;
            } else if (kind.equals("shulker")) {
                rendererClass = net.minecraft.client.renderer.entity.ShulkerRenderer.class;
                var shulker = new net.minecraft.client.renderer.entity.state.ShulkerRenderState();
                shulker.attachFace = net.minecraft.core.Direction.from3DDataValue(row.get("attach_face").getAsInt());
                state = shulker;
            } else if (kind.equals("slime") || kind.equals("magma_cube") || kind.equals("sulfur_cube")) {
                rendererClass = kind.equals("slime")
                    ? net.minecraft.client.renderer.entity.SlimeRenderer.class
                    : kind.equals("sulfur_cube")
                        ? net.minecraft.client.renderer.entity.SulfurCubeRenderer.class
                        : net.minecraft.client.renderer.entity.MagmaCubeRenderer.class;
                var cube = kind.equals("sulfur_cube")
                    ? new net.minecraft.client.renderer.entity.state.SulfurCubeRenderState()
                    : new net.minecraft.client.renderer.entity.state.SlimeRenderState();
                if (cube instanceof net.minecraft.client.renderer.entity.state.SulfurCubeRenderState sulfur) {
                    sulfur.fuseRemainingTicks = Oracle.f32(row, "fuse_remaining");
                    sulfur.hasRedOverlay = row.has("red_overlay");
                    if (row.has("contained_block")) {
                        fillBlock(sulfur.containedBlock);
                    }
                }
                cube.size = row.get("size").getAsInt();
                cube.squish = Oracle.f32(row, "squish");
                state = cube;
            } else if (kind.equals("cod")) {
                rendererClass = net.minecraft.client.renderer.entity.CodRenderer.class;
                state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
            } else if (kind.equals("salmon")) {
                rendererClass = net.minecraft.client.renderer.entity.SalmonRenderer.class;
                state = new net.minecraft.client.renderer.entity.state.SalmonRenderState();
            } else if (kind.equals("tropical_fish")) {
                rendererClass = net.minecraft.client.renderer.entity.TropicalFishRenderer.class;
                state = new net.minecraft.client.renderer.entity.state.TropicalFishRenderState();
            } else if (kind.equals("pufferfish")) {
                rendererClass = net.minecraft.client.renderer.entity.PufferfishRenderer.class;
                state = new net.minecraft.client.renderer.entity.state.PufferfishRenderState();
            } else if (kind.equals("phantom")) {
                rendererClass = net.minecraft.client.renderer.entity.PhantomRenderer.class;
                var phantom = new net.minecraft.client.renderer.entity.state.PhantomRenderState();
                phantom.size = row.get("size").getAsInt();
                state = phantom;
            } else if (kind.equals("cat")) {
                rendererClass = net.minecraft.client.renderer.entity.CatRenderer.class;
                var cat = new net.minecraft.client.renderer.entity.state.CatRenderState();
                cat.lieDownAmount = Oracle.f32(row, "lie_down_amount");
                cat.isLyingOnTopOfSleepingPlayer = row.get("on_sleeping_player").getAsBoolean();
                state = cat;
            } else {
                rendererClass = net.minecraft.client.renderer.entity.FoxRenderer.class;
                var fox = new net.minecraft.client.renderer.entity.state.FoxRenderState();
                fox.isPouncing = row.get("pouncing").getAsBoolean();
                fox.isFaceplanted = row.get("faceplanted").getAsBoolean();
                state = fox;
            }
            state.isBaby = row.get("baby").getAsBoolean();
            state.xRot = Oracle.f32(row, "x_rot");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isInWater = row.has("in_water") && row.get("in_water").getAsBoolean();
            if (row.has("death_time")) {
                state.deathTime = Oracle.f32(row, "death_time");
            }
            var renderer = UNSAFE.allocateInstance(rendererClass);
            boolean cubeMob = kind.equals("slime") || kind.equals("magma_cube") || kind.equals("sulfur_cube");
            // The cube mobs' renderers take the common turns of `LivingEntityRenderer`.
            var method = (cubeMob ? net.minecraft.client.renderer.entity.LivingEntityRenderer.class : rendererClass)
                .getDeclaredMethod("setupRotations",
                    cubeMob ? net.minecraft.client.renderer.entity.state.LivingEntityRenderState.class : state.getClass(),
                    com.mojang.blaze3d.vertex.PoseStack.class, float.class, float.class);
            method.setAccessible(true);
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            if (row.has("feet_x")) {
                // What `submit` does before `setupRotations`: to the entity's place, then its scale.
                stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
                float scale = Oracle.f32(row, "scale");
                stack.scale(scale, scale, scale);
            }
            method.invoke(renderer, state, stack, Oracle.f32(row, "body_rot"), 1.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var matrix = new float[16];
            stack.last().pose().get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            out.add("pose", described);
            if (kind.equals("phantom") || cubeMob) {
                // What `submit` does after `setupRotations`: the flip, the renderer's own scale
                // and the model's lift. (The magma cube takes its scale from the cube mobs' base.)
                stack.scale(-1.0F, -1.0F, 1.0F);
                var hook = (kind.equals("magma_cube")
                        ? net.minecraft.client.renderer.entity.AbstractCubeMobRenderer.class : rendererClass)
                    .getDeclaredMethod("scale", cubeMob && !kind.equals("sulfur_cube")
                        ? net.minecraft.client.renderer.entity.state.SlimeRenderState.class : state.getClass(),
                        com.mojang.blaze3d.vertex.PoseStack.class);
                hook.setAccessible(true);
                hook.invoke(renderer, state, stack);
                stack.translate(0.0F, -1.501F, 0.0F);
                var full = new float[16];
                stack.last().pose().get(full);
                var fullDescribed = new JsonArray();
                for (float value : full) {
                    fullDescribed.add(Oracle.bits(value));
                }
                out.add("full_pose", fullDescribed);
            }
            var blockOverlay = new JsonArray();
            if (row.has("contained_block")) {
                // `SulfurCubeInnerLayer` draws the block it holds from the pose the renderer left.
                var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                    EntityOracle.class.getClassLoader(),
                    new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                    (proxy, method2, args) -> {
                        if (method2.getName().equals("submitBlockModel")) {
                            var layerMatrix = new float[16];
                            ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(layerMatrix);
                            var block = new JsonArray();
                            for (float value : layerMatrix) {
                                block.add(Oracle.bits(value));
                            }
                            poses.add(block);
                            blockOverlay.add((Integer) args[5]);
                        }
                        return null;
                    });
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> null;
                Oracle.vanillaRegistries();
                new net.minecraft.client.renderer.entity.layers.SulfurCubeInnerLayer(
                    parent, new net.minecraft.client.model.geom.EntityModelSet(
                        net.minecraft.client.model.geom.LayerDefinitions.createRoots()))
                    .submit(stack, collector, 0, (net.minecraft.client.renderer.entity.state.SulfurCubeRenderState) state, 0.0F, 0.0F);
            }
            if (kind.equals("panda")) {
                var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                    EntityOracle.class.getClassLoader(),
                    new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                    (proxy, method2, args) -> {
                        if (method2.getName().equals("submitItem")) {
                            var layerMatrix = new float[16];
                            ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(layerMatrix);
                            var item = new JsonArray();
                            for (float value : layerMatrix) {
                                item.add(Oracle.bits(value));
                            }
                            poses.add(item);
                        }
                        return null;
                    });
                var pandaState = (net.minecraft.client.renderer.entity.state.PandaRenderState) state;
                pandaState.heldItem.newLayer();
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> null;
                new net.minecraft.client.renderer.entity.layers.PandaHoldsItemLayer(parent)
                    .submit(new com.mojang.blaze3d.vertex.PoseStack(), collector, 0, pandaState, 0.0F, 0.0F);
            }
            out.add("poses", poses);
            if (!blockOverlay.isEmpty()) {
                out.add("block_overlay", blockOverlay);
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the reference's own `setupRotations` leaves the pose stack at for a mob that may shake: a
     * cow (frozen through is the only reason), a hoglin (converting too) and a strider (suffocating
     * too), with the dimension's `piglins_zombify` read from the registry's own dimension type.
     */
    private static JsonObject shakingRotations(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var dimensionTypes = Oracle.vanillaRegistries()
            .lookupOrThrow(net.minecraft.core.registries.Registries.DIMENSION_TYPE);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String kind = row.get("kind").getAsString();
            boolean fullyFrozen = row.get("ticks_frozen").getAsInt() >= 140;
            Class<?> rendererClass;
            net.minecraft.client.renderer.entity.state.LivingEntityRenderState state;
            if (kind.equals("hoglin")) {
                rendererClass = net.minecraft.client.renderer.entity.HoglinRenderer.class;
                var hoglin = new net.minecraft.client.renderer.entity.state.HoglinRenderState();
                var type = dimensionTypes.getOrThrow(net.minecraft.resources.ResourceKey.create(
                    net.minecraft.core.registries.Registries.DIMENSION_TYPE,
                    net.minecraft.resources.Identifier.parse(row.get("dimension").getAsString()))).value();
                boolean zombify = type.attributes().applyModifier(
                    net.minecraft.world.attribute.EnvironmentAttributes.PIGLINS_ZOMBIFY, true);
                // `Hoglin.isConverting`, from the data it reads.
                hoglin.isConverting = !row.get("immune").getAsBoolean() && !row.get("no_ai").getAsBoolean()
                    && zombify;
                state = hoglin;
            } else if (kind.equals("strider")) {
                rendererClass = net.minecraft.client.renderer.entity.StriderRenderer.class;
                var strider = new net.minecraft.client.renderer.entity.state.StriderRenderState();
                strider.isSuffocating = row.get("suffocating").getAsBoolean();
                state = strider;
            } else {
                rendererClass = net.minecraft.client.renderer.entity.CowRenderer.class;
                state = new net.minecraft.client.renderer.entity.state.CowRenderState();
            }
            state.isFullyFrozen = fullyFrozen;
            state.ageInTicks = (float) row.get("ticks").getAsInt() + Oracle.f32(row, "partial");
            var renderer = UNSAFE.allocateInstance(rendererClass);
            var method = net.minecraft.client.renderer.entity.LivingEntityRenderer.class.getDeclaredMethod(
                "setupRotations", net.minecraft.client.renderer.entity.state.LivingEntityRenderState.class,
                com.mojang.blaze3d.vertex.PoseStack.class, float.class, float.class);
            method.setAccessible(true);
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            method.invoke(renderer, state, stack, Oracle.f32(row, "body_rot"), 1.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var matrix = new float[16];
            stack.last().pose().get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            out.add("pose", described);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The ravager's model posed by the reference's own `RavagerModel.setupAnim`, drawn. */
    private static JsonObject ravagerModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.ravager.RavagerModel(bakedLayer(row.get("layer").getAsString()));
            var state = new net.minecraft.client.renderer.entity.state.RavagerRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.stunnedTicksRemaining = Oracle.f32(row, "stunned");
            state.attackTicksRemaining = Oracle.f32(row, "attack");
            state.roarAnimation = Oracle.f32(row, "roar");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The strider's models (main, baby and saddle) posed by the reference's own `setupAnim`, drawn. */
    private static JsonObject striderModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.monster.strider.StriderModel model = id.contains("_baby")
                ? new net.minecraft.client.model.monster.strider.BabyStriderModel(root)
                : new net.minecraft.client.model.monster.strider.AdultStriderModel(root);
            var state = new net.minecraft.client.renderer.entity.state.StriderRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isRidden = row.get("ridden").getAsBoolean();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The strider's textures by state and age, from the renderer's own constants. */
    private static JsonObject striderTextures() throws Exception {
        var rows = new JsonArray();
        for (String name : new String[] {"STRIDER_LOCATION", "STRIDER_BABY_LOCATION", "COLD_LOCATION", "COLD_BABY_LOCATION"}) {
            var field = net.minecraft.client.renderer.entity.StriderRenderer.class.getDeclaredField(name);
            field.setAccessible(true);
            var out = new JsonObject();
            out.addProperty("label", name);
            out.addProperty("texture", field.get(null).toString());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Stops `LivingEntityRenderer.submit` once the pose it builds is in hand. */
    private static final class PoseCaptured extends RuntimeException {
        PoseCaptured() {
            super(null, null, false, false);
        }
    }

    /**
     * A living entity's renderer as far as its own pose goes: the real
     * `submit` runs up to the point where it hands the pose to `scale`, which
     * this renderer's does not return from.
     */
    private static final class PoseProbe extends net.minecraft.client.renderer.entity.LivingEntityRenderer<
        net.minecraft.world.entity.LivingEntity,
        net.minecraft.client.renderer.entity.state.LivingEntityRenderState,
        net.minecraft.client.model.EntityModel<net.minecraft.client.renderer.entity.state.LivingEntityRenderState>> {
        static final float[] CAPTURED = new float[16];
        static float flip = 90.0F;

        private PoseProbe() {
            super(null, null, 0.0F);
        }

        @Override
        public net.minecraft.resources.Identifier getTextureLocation(
            final net.minecraft.client.renderer.entity.state.LivingEntityRenderState state) {
            return null;
        }

        @Override
        public net.minecraft.client.renderer.entity.state.LivingEntityRenderState createRenderState() {
            return new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
        }

        @Override
        protected float getFlipDegrees() {
            return flip;
        }

        @Override
        protected void scale(
            final net.minecraft.client.renderer.entity.state.LivingEntityRenderState state,
            final com.mojang.blaze3d.vertex.PoseStack poseStack) {
            // What `submit` does next, before it asks for the render type.
            poseStack.translate(0.0F, -1.501F, 0.0F);
            poseStack.last().pose().get(CAPTURED);
            throw new PoseCaptured();
        }
    }

    /** The pose a living entity is drawn from, by `LivingEntityRenderer.submit`. */
    private static JsonObject livingPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var renderer = (PoseProbe) UNSAFE.allocateInstance(PoseProbe.class);
            var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
            state.scale = Oracle.f32(row, "scale");
            state.bodyRot = Oracle.f32(row, "body_rot");
            state.deathTime = Oracle.f32(row, "death_time");
            PoseProbe.flip = row.has("flip_degrees") ? Oracle.f32(row, "flip_degrees") : 90.0F;
            try {
                renderer.submit(state, new com.mojang.blaze3d.vertex.PoseStack(), null, null);
            } catch (PoseCaptured expected) {
                // The pose is in hand.
            }
            var described = new JsonArray();
            for (float value : PoseProbe.CAPTURED) {
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

    /** A layer of the table baked, as the game bakes it. */
    private static net.minecraft.client.model.geom.ModelPart bakedLayer(final String id) {
        Oracle.vanillaRegistries();
        for (var entry : net.minecraft.client.model.geom.LayerDefinitions.createRoots().entrySet()) {
            if (entry.getKey().toString().equals(id)) {
                return entry.getValue().bakeRoot();
            }
        }
        throw new IllegalArgumentException("No layer " + id);
    }

    /** Every quad of a posed model, sorted, as the part renderer writes them. */
    private static JsonObject posedQuads(final String label, final net.minecraft.client.model.Model<?> model) {
        var capture = new Capture();
        model.root().render(new com.mojang.blaze3d.vertex.PoseStack(), capture, 0, 0);
        var quads = new java.util.ArrayList<String>();
        for (int i = 0; i < capture.vertices.size(); i += 4) {
            quads.add(String.join("", capture.vertices.subList(i, i + 4)));
        }
        java.util.Collections.sort(quads);
        var out = new JsonObject();
        out.addProperty("label", label);
        var array = new JsonArray();
        quads.forEach(array::add);
        out.add("quads", array);
        return out;
    }

    private static void humanoidFields(final net.minecraft.client.renderer.entity.state.HumanoidRenderState state, final JsonObject row) {
        state.yRot = Oracle.f32(row, "head_y_rot");
        state.xRot = Oracle.f32(row, "x_rot");
        state.walkAnimationPos = Oracle.f32(row, "walk_position");
        state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
        state.ageInTicks = Oracle.f32(row, "age_in_ticks");
        state.attackTime = Oracle.f32(row, "attack_time");
        state.swimAmount = Oracle.f32(row, "swim_amount");
    }

    /** A skeleton family model posed by `SkeletonModel.setupAnim`, drawn. */
    private static JsonObject skeletonModel(final JsonObject request) throws Exception {
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
    private static JsonObject piglinModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            boolean baby = id.contains("_baby");
            boolean dancing = row.get("dancing").getAsBoolean();
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
                if (dancing) {
                    state.armPose = net.minecraft.world.entity.monster.piglin.PiglinArmPose.DANCING;
                }
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
    private static JsonObject zombieModel(final JsonObject request) throws Exception {
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

    /** A quadruped's model from the layer table, posed by the reference's own `setupAnim`, drawn. */
    private static JsonObject quadrupedModel(final JsonObject request) throws Exception {
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
            net.minecraft.client.model.QuadrupedModel<net.minecraft.client.renderer.entity.state.LivingEntityRenderState>
                model = id.startsWith("minecraft:cow") || id.startsWith("minecraft:warm_cow")
                    || id.startsWith("minecraft:cold_cow") || id.startsWith("minecraft:mooshroom")
                    ? new net.minecraft.client.model.animal.cow.CowModel(root)
                    : new net.minecraft.client.model.animal.pig.PigModel(root);
            var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
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

    /** The entity type registry: each type's id and name, in id order. */
    private static JsonObject entityTypes() {
        Oracle.vanillaRegistries();
        var registry = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE;
        var rows = new JsonArray();
        for (var type : registry) {
            var out = new JsonObject();
            out.addProperty("label", registry.getKey(type).toString());
            out.addProperty("id", registry.getId(type));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Records the corners `ExperienceOrbRenderer` writes, one vertex at a time. */
    private static final class OrbCapture implements com.mojang.blaze3d.vertex.VertexConsumer {
        final JsonArray vertices = new JsonArray();
        private JsonObject current;

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer addVertex(final float x, final float y, final float z) {
            this.current = new JsonObject();
            this.current.addProperty("x", Oracle.bits(x));
            this.current.addProperty("y", Oracle.bits(y));
            this.current.addProperty("z", Oracle.bits(z));
            this.vertices.add(this.current);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int r, final int g, final int b, final int a) {
            this.current.addProperty("color", r + "," + g + "," + b + "," + a);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int color) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv(final float u, final float v) {
            this.current.addProperty("u", Oracle.bits(u));
            this.current.addProperty("v", Oracle.bits(v));
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv1(final int u, final int v) {
            this.current.addProperty("overlay", u | v << 16);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv2(final int u, final int v) {
            this.current.addProperty("light", u | v << 16);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setNormal(final float x, final float y, final float z) {
            this.current.addProperty("normal", Oracle.bits(x) + Oracle.bits(y) + Oracle.bits(z));
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setLineWidth(final float width) {
            throw new UnsupportedOperationException();
        }
    }

    /** `ExperienceOrbRenderer.submit`: the quad it writes, from the camera's side. */
    private static JsonObject experienceOrb(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var renderer = (net.minecraft.client.renderer.entity.ExperienceOrbRenderer) UNSAFE.allocateInstance(
                net.minecraft.client.renderer.entity.ExperienceOrbRenderer.class);
            var state = new net.minecraft.client.renderer.entity.state.ExperienceOrbRenderState();
            state.icon = row.get("icon").getAsInt();
            state.ageInTicks = Oracle.f32(row, "age");
            state.lightCoords = row.get("light").getAsInt();
            var camera = new net.minecraft.client.renderer.state.level.CameraRenderState();
            // `Camera.setRotation`.
            camera.orientation = new org.joml.Quaternionf().rotationYXZ(
                (float) Math.PI - Oracle.f32(row, "yaw") * (float) (Math.PI / 180.0),
                -Oracle.f32(row, "pitch") * (float) (Math.PI / 180.0), 0.0F);
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "px"), Oracle.f32(row, "py"), Oracle.f32(row, "pz"));
            var capture = new OrbCapture();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitCustomGeometry")) {
                        ((net.minecraft.client.renderer.SubmitNodeCollector.CustomGeometryRenderer) args[2])
                            .render(((com.mojang.blaze3d.vertex.PoseStack) args[0]).last(), capture);
                    }
                    return null;
                });
            renderer.submit(state, stack, collector, camera);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("vertices", capture.vertices);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The cat, chicken, cow, frog, pig and zombie nautilus variants the game's data pack holds: their textures and model. */
    private static JsonObject entityVariants() {
        var registries = Oracle.vanillaRegistries();
        var rows = new java.util.TreeMap<String, JsonObject>();
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.PIG_VARIANT).listElements().forEach(holder -> {
            var variant = holder.value();
            variants(rows, "pig", holder.key().identifier().toString(), variant.modelAndTexture().asset().id(),
                variant.babyTexture().id(), variant.modelAndTexture().model().name());
        });
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.COW_VARIANT).listElements().forEach(holder -> {
            var variant = holder.value();
            variants(rows, "cow", holder.key().identifier().toString(), variant.modelAndTexture().asset().id(),
                variant.babyTexture().id(), variant.modelAndTexture().model().name());
        });
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.CHICKEN_VARIANT).listElements().forEach(holder -> {
            var variant = holder.value();
            variants(rows, "chicken", holder.key().identifier().toString(), variant.modelAndTexture().asset().id(),
                variant.babyTexture().id(), variant.modelAndTexture().model().name());
        });
        // A cat's variant has no model family of its own: the model is the cat's.
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.CAT_VARIANT).listElements().forEach(holder -> {
            var variant = holder.value();
            variants(rows, "cat", holder.key().identifier().toString(), variant.adultAssetInfo().id(),
                variant.babyAssetInfo().id(), "normal");
        });
        // A frog's variant has one texture and no baby (a frog is never one).
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.FROG_VARIANT).listElements().forEach(holder -> {
            variants(rows, "frog", holder.key().identifier().toString(), holder.value().assetInfo().id(), null, "normal");
        });
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.ZOMBIE_NAUTILUS_VARIANT).listElements().forEach(holder -> {
            var variant = holder.value();
            variants(rows, "zombie_nautilus", holder.key().identifier().toString(),
                variant.modelAndTexture().asset().id(), null, variant.modelAndTexture().model().getSerializedName());
        });
        var list = new JsonArray();
        rows.values().forEach(list::add);
        var result = new JsonObject();
        result.add("rows", list);
        return result;
    }

    /** The wolf variants of the game's data pack: the six textures each wears (grown and baby; wild, tame, angry). */
    private static JsonObject wolfVariants() {
        var rows = new java.util.TreeMap<String, JsonObject>();
        Oracle.vanillaRegistries().lookupOrThrow(net.minecraft.core.registries.Registries.WOLF_VARIANT)
            .listElements().forEach(holder -> {
                var variant = holder.value();
                var out = new JsonObject();
                String name = holder.key().identifier().toString();
                out.addProperty("label", name);
                out.addProperty("name", name);
                var adult = variant.adultInfo();
                var baby = variant.babyInfo();
                out.addProperty("wild", adult.wild().id().toString());
                out.addProperty("tame", adult.tame().id().toString());
                out.addProperty("angry", adult.angry().id().toString());
                out.addProperty("baby_wild", baby.wild().id().toString());
                out.addProperty("baby_tame", baby.tame().id().toString());
                out.addProperty("baby_angry", baby.angry().id().toString());
                rows.put(name, out);
            });
        var list = new JsonArray();
        rows.values().forEach(list::add);
        var result = new JsonObject();
        result.add("rows", list);
        return result;
    }

    private static void variants(
        final java.util.Map<String, JsonObject> rows, final String kind, final String name,
        final net.minecraft.resources.Identifier asset, final net.minecraft.resources.Identifier baby,
        final String model) {
        var out = new JsonObject();
        out.addProperty("label", kind + ":" + name);
        out.addProperty("kind", kind);
        out.addProperty("name", name);
        out.addProperty("asset", asset.toString());
        out.addProperty("baby_asset", baby == null ? "" : baby.toString());
        out.addProperty("model", model.toLowerCase());
        rows.put(kind + ":" + name, out);
    }

    /**
     * Where the variant sits among a pig's and a cow's data (the accessor
     * their classes define last but for the sound), and which serializer
     * carries it.
     */
    private static JsonObject entityVariantFields() throws Exception {
        Oracle.vanillaRegistries();
        var rows = new JsonArray();
        Object[][] fields = {
            {net.minecraft.world.entity.animal.pig.Pig.class, "DATA_VARIANT_ID", "pig"},
            {net.minecraft.world.entity.animal.cow.Cow.class, "DATA_VARIANT_ID", "cow"},
            {net.minecraft.world.entity.ExperienceOrb.class, "DATA_VALUE", "experienceorb"},
            {net.minecraft.world.entity.monster.piglin.Piglin.class, "DATA_BABY_ID", "piglin-baby"},
            {net.minecraft.world.entity.monster.piglin.Piglin.class, "DATA_IS_DANCING", "piglin-dancing"},
            {net.minecraft.world.entity.Avatar.class, "DATA_PLAYER_MAIN_HAND", "avatar-main-hand"},
            {net.minecraft.world.entity.animal.sheep.Sheep.class, "DATA_WOOL_ID", "sheep-wool"},
            {net.minecraft.world.entity.animal.chicken.Chicken.class, "DATA_VARIANT_ID", "chicken"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_CLIENT_FLAGS", "armor-stand-flags"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_HEAD_POSE", "armor-stand-head"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_BODY_POSE", "armor-stand-body"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_LEFT_ARM_POSE", "armor-stand-left-arm"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_RIGHT_ARM_POSE", "armor-stand-right-arm"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_LEFT_LEG_POSE", "armor-stand-left-leg"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_RIGHT_LEG_POSE", "armor-stand-right-leg"},
            {net.minecraft.world.entity.npc.villager.Villager.class, "DATA_VILLAGER_DATA", "villager-data"},
            {net.minecraft.world.entity.npc.villager.AbstractVillager.class, "DATA_UNHAPPY_COUNTER", "villager-unhappy"},
            {net.minecraft.world.entity.animal.equine.AbstractHorse.class, "DATA_ID_FLAGS", "horse-flags"},
            {net.minecraft.world.entity.animal.equine.Horse.class, "DATA_ID_TYPE_VARIANT", "horse-variant"},
            {net.minecraft.world.entity.animal.equine.AbstractChestedHorse.class, "DATA_ID_CHEST", "horse-chest"},
            {net.minecraft.world.entity.animal.equine.Llama.class, "DATA_VARIANT_ID", "llama-variant"},
            {net.minecraft.world.entity.animal.camel.Camel.class, "DASH", "camel-dash"},
            {net.minecraft.world.entity.animal.camel.Camel.class, "LAST_POSE_CHANGE_TICK", "camel-last-pose"},
            {net.minecraft.world.entity.TamableAnimal.class, "DATA_FLAGS_ID", "tamable-flags"},
            {net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_INTERESTED_ID", "wolf-interested"},
            {net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_COLLAR_COLOR", "wolf-collar"},
            {net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_ANGER_END_TIME", "wolf-anger-end"},
            {net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_VARIANT_ID", "wolf-variant"},
            {net.minecraft.world.entity.animal.polarbear.PolarBear.class, "DATA_STANDING_ID", "polar-bear-standing"},
            {net.minecraft.world.entity.monster.Strider.class, "DATA_SUFFOCATING", "strider-suffocating"},
            {net.minecraft.world.entity.monster.hoglin.Hoglin.class, "DATA_IMMUNE_TO_ZOMBIFICATION", "hoglin-immune"},
            {net.minecraft.world.entity.animal.golem.CopperGolem.class, "DATA_WEATHER_STATE", "copper-golem-weather"},
            {net.minecraft.world.entity.animal.golem.CopperGolem.class, "COPPER_GOLEM_STATE", "copper-golem-state"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "UNHAPPY_COUNTER", "panda-unhappy"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "SNEEZE_COUNTER", "panda-sneeze"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "EAT_COUNTER", "panda-eat"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "MAIN_GENE_ID", "panda-main-gene"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "HIDDEN_GENE_ID", "panda-hidden-gene"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "DATA_ID_FLAGS", "panda-flags"},
            {net.minecraft.world.entity.animal.fox.Fox.class, "DATA_TYPE_ID", "fox-variant"},
            {net.minecraft.world.entity.animal.fox.Fox.class, "DATA_FLAGS_ID", "fox-flags"},
            {net.minecraft.world.entity.animal.feline.Cat.class, "DATA_VARIANT_ID", "cat-variant"},
            {net.minecraft.world.entity.animal.feline.Cat.class, "IS_LYING", "cat-lying"},
            {net.minecraft.world.entity.animal.feline.Cat.class, "RELAX_STATE_ONE", "cat-relax"},
            {net.minecraft.world.entity.animal.feline.Cat.class, "DATA_COLLAR_COLOR", "cat-collar"},
            {net.minecraft.world.entity.animal.bee.Bee.class, "DATA_FLAGS_ID", "bee-flags"},
            {net.minecraft.world.entity.animal.bee.Bee.class, "DATA_ANGER_END_TIME", "bee-anger-end"},
            {net.minecraft.world.entity.animal.rabbit.Rabbit.class, "DATA_TYPE_ID", "rabbit-type"},
            {net.minecraft.world.entity.animal.goat.Goat.class, "DATA_HAS_LEFT_HORN", "goat-left-horn"},
            {net.minecraft.world.entity.animal.goat.Goat.class, "DATA_HAS_RIGHT_HORN", "goat-right-horn"},
            {net.minecraft.world.entity.animal.cow.MushroomCow.class, "DATA_TYPE", "mooshroom-type"},
            {net.minecraft.world.entity.animal.golem.SnowGolem.class, "DATA_PUMPKIN_ID", "snow-golem-pumpkin"},
            {net.minecraft.world.entity.monster.EnderMan.class, "DATA_CARRY_STATE", "enderman-carry"},
            {net.minecraft.world.entity.monster.EnderMan.class, "DATA_CREEPY", "enderman-creepy"},
            {net.minecraft.world.entity.monster.zombie.ZombieVillager.class, "DATA_VILLAGER_DATA", "zombie-villager-data"},
            {net.minecraft.world.entity.raid.Raider.class, "IS_CELEBRATING", "raider-celebrating"},
            {net.minecraft.world.entity.monster.illager.SpellcasterIllager.class, "DATA_SPELL_CASTING_ID", "spellcaster-casting"},
            {net.minecraft.world.entity.monster.Creeper.class, "DATA_IS_POWERED", "creeper-powered"},
            {net.minecraft.world.entity.monster.Creeper.class, "DATA_SWELL_DIR", "creeper-swell-dir"},
            {net.minecraft.world.entity.monster.Creeper.class, "DATA_IS_IGNITED", "creeper-ignited"},
            {net.minecraft.world.entity.animal.frog.Frog.class, "DATA_VARIANT_ID", "frog-variant"},
            {net.minecraft.world.entity.animal.nautilus.ZombieNautilus.class, "DATA_VARIANT_ID", "zombie-nautilus-variant"},
            {net.minecraft.world.entity.vehicle.VehicleEntity.class, "DATA_ID_HURT", "boat-hurt"},
            {net.minecraft.world.entity.vehicle.VehicleEntity.class, "DATA_ID_HURTDIR", "boat-hurt-dir"},
            {net.minecraft.world.entity.vehicle.VehicleEntity.class, "DATA_ID_DAMAGE", "boat-damage"},
            {net.minecraft.world.entity.vehicle.boat.AbstractBoat.class, "DATA_ID_PADDLE_LEFT", "boat-paddle-left"},
            {net.minecraft.world.entity.vehicle.boat.AbstractBoat.class, "DATA_ID_PADDLE_RIGHT", "boat-paddle-right"},
            {net.minecraft.world.entity.vehicle.boat.AbstractBoat.class, "DATA_ID_BUBBLE_TIME", "boat-bubble-time"},
            {net.minecraft.world.entity.animal.parrot.Parrot.class, "DATA_VARIANT_ID", "parrot-variant"},
            {net.minecraft.world.entity.ambient.Bat.class, "DATA_ID_FLAGS", "bat-flags"},
            {net.minecraft.world.entity.monster.Vex.class, "DATA_FLAGS_ID", "vex-flags"},
            {net.minecraft.world.entity.animal.allay.Allay.class, "DATA_DANCING", "allay-dancing"},
            {net.minecraft.world.entity.monster.Ghast.class, "DATA_IS_CHARGING", "ghast-charging"},
            {net.minecraft.world.entity.monster.Phantom.class, "ID_SIZE", "phantom-size"},
            {net.minecraft.world.entity.monster.Guardian.class, "DATA_ID_MOVING", "guardian-moving"},
            {net.minecraft.world.entity.monster.Guardian.class, "DATA_ID_ATTACK_TARGET", "guardian-attack-target"},
            {net.minecraft.world.entity.animal.axolotl.Axolotl.class, "DATA_VARIANT", "axolotl-variant"},
            {net.minecraft.world.entity.animal.axolotl.Axolotl.class, "DATA_PLAYING_DEAD", "axolotl-playing-dead"},
            {net.minecraft.world.entity.animal.turtle.Turtle.class, "HAS_EGG", "turtle-has-egg"},
            {net.minecraft.world.entity.animal.turtle.Turtle.class, "LAYING_EGG", "turtle-laying-egg"},
            {net.minecraft.world.entity.animal.fish.Salmon.class, "DATA_TYPE", "salmon-type"},
            {net.minecraft.world.entity.animal.fish.TropicalFish.class, "DATA_ID_TYPE_VARIANT", "tropical-fish-variant"},
            {net.minecraft.world.entity.animal.fish.Pufferfish.class, "PUFF_STATE", "pufferfish-puff-state"},
        };
        for (var described : fields) {
            var field = ((Class<?>) described[0]).getDeclaredField((String) described[1]);
            field.setAccessible(true);
            var accessor = (net.minecraft.network.syncher.EntityDataAccessor<?>) field.get(null);
            var out = new JsonObject();
            out.addProperty("label", (String) described[2]);
            out.addProperty("data_id", accessor.id());
            out.addProperty("serializer",
                net.minecraft.network.syncher.EntityDataSerializers.getSerializedId(accessor.serializer()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Every entity data serializer by the id the wire carries, named by the
     * field that holds it in `EntityDataSerializers`.
     */
    private static JsonObject entityDataSerializers() throws Exception {
        Oracle.vanillaRegistries();
        var byId = new java.util.TreeMap<Integer, String>();
        for (var field : net.minecraft.network.syncher.EntityDataSerializers.class.getDeclaredFields()) {
            if (!java.lang.reflect.Modifier.isStatic(field.getModifiers())
                || !net.minecraft.network.syncher.EntityDataSerializer.class.isAssignableFrom(field.getType())) {
                continue;
            }
            field.setAccessible(true);
            var serializer = (net.minecraft.network.syncher.EntityDataSerializer<?>) field.get(null);
            int id = net.minecraft.network.syncher.EntityDataSerializers.getSerializedId(serializer);
            if (id >= 0) {
                byId.put(id, field.getName());
            }
        }
        var rows = new JsonArray();
        for (var entry : byId.entrySet()) {
            var out = new JsonObject();
            out.addProperty("label", entry.getValue());
            out.addProperty("id", entry.getKey());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Every model layer the game builds: its texture size and the tree of its parts. */
    private static JsonObject modelLayers() throws Exception {
        // The layers are made from registries (decorated pot patterns): the game's bootstrap first.
        Oracle.vanillaRegistries();
        var layers = new java.util.TreeMap<String, Object>();
        for (var entry : net.minecraft.client.model.geom.LayerDefinitions.createRoots().entrySet()) {
            layers.put(entry.getKey().toString(), entry.getValue());
        }
        var rows = new JsonArray();
        for (var entry : layers.entrySet()) {
            var layer = entry.getValue();
            var mesh = field(layer, "mesh");
            var material = field(layer, "material");
            var out = new JsonObject();
            out.addProperty("label", entry.getKey());
            out.addProperty("texture_width", (int) field(material, "xTexSize"));
            out.addProperty("texture_height", (int) field(material, "yTexSize"));
            var root = ((net.minecraft.client.model.geom.builders.MeshDefinition) mesh).getRoot();
            out.add("root", partDefinition("root", root));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

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
        if (operation.equals("arrow_pose")) {
            return arrowPose(request);
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
