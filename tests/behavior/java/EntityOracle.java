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
