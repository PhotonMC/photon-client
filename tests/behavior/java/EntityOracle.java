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
            net.minecraft.client.model.HumanoidModel<net.minecraft.client.renderer.entity.state.ZombieRenderState> model;
            if (id.startsWith("minecraft:drowned_baby")) {
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
            var state = new net.minecraft.client.renderer.entity.state.ZombieRenderState();
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
                    || id.startsWith("minecraft:cold_cow")
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
