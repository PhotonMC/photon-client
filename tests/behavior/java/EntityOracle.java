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
            if (row.has("dye")) {
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

    /** The pig and cow variants the game's data pack holds: their textures and model. */
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
        out.addProperty("baby_asset", baby.toString());
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
            {net.minecraft.world.entity.monster.Creeper.class, "DATA_SWELL_DIR", "creeper-swell-dir"},
            {net.minecraft.world.entity.monster.Creeper.class, "DATA_IS_IGNITED", "creeper-ignited"},
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
