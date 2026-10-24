import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The rotations, winding and vector helpers the model rows share. */
class ModelMath {
    /** The transform an element's three-axis rotation block produces. */
    static JsonObject euler(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        float x = Float.intBitsToFloat(Integer.parseUnsignedInt(row.get("x").getAsString(), 16));
        float y = Float.intBitsToFloat(Integer.parseUnsignedInt(row.get("y").getAsString(), 16));
        float z = Float.intBitsToFloat(Integer.parseUnsignedInt(row.get("z").getAsString(), 16));
        boolean rescale = row.get("rescale").getAsBoolean();
        var origin = new org.joml.Vector3f(0.5F, 0.5F, 0.5F);
        var value = new net.minecraft.client.resources.model.cuboid.CuboidRotation
            .EulerXYZRotation(x, y, z);
        var transform = new net.minecraft.client.resources.model.cuboid.CuboidRotation(
            origin, value, rescale).transform();
        entry.addProperty("outcome", "rotated");
        var matrix = new JsonArray();
        for (int column = 0; column < 4; column++) {
            for (int rowIndex = 0; rowIndex < 4; rowIndex++) {
                matrix.add(String.format(java.util.Locale.ROOT, "%08x",
                    Float.floatToRawIntBits(transform.get(column, rowIndex))));
            }
        }
        entry.add("transform", matrix);
        return entry;
    }

    /** Normal, chosen direction and vertex order for four given corners. */
    static JsonObject winding(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var given = row.get("corners").getAsJsonArray();
        var positions = new org.joml.Vector3fc[4];
        for (int index = 0; index < 4; index++) {
            positions[index] = vector(given.get(index).getAsJsonArray());
        }

        var normal = new org.joml.Vector3f();
        org.joml.GeometryUtils.normal(positions[0], positions[1], positions[2], normal);
        entry.addProperty("outcome", "wound");
        entry.addProperty("normal_x", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(normal.x())));
        entry.addProperty("normal_y", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(normal.y())));
        entry.addProperty("normal_z", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(normal.z())));

        try {
            var bakery = net.minecraft.client.resources.model.cuboid.FaceBakery.class;
            var findClosest = Reflect.method(bakery, "findClosestDirection", org.joml.Vector3f.class);
            var chosen = (net.minecraft.core.Direction)findClosest.invoke(null, normal);
            entry.addProperty("direction", chosen == null ? "none" : chosen.getName());

            if (chosen != null) {
                var uvs = new long[] {10L, 20L, 30L, 40L};
                var rewind = Reflect.method(bakery, "recalculateWinding",
                    org.joml.Vector3fc[].class, long[].class, net.minecraft.core.Direction.class);
                try {
                    rewind.invoke(null, positions, uvs, chosen);
                    var wound = new JsonArray();
                    for (int index = 0; index < 4; index++) {
                        var corner = new JsonObject();
                        corner.addProperty("x", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(positions[index].x())));
                        corner.addProperty("y", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(positions[index].y())));
                        corner.addProperty("z", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(positions[index].z())));
                        wound.add(corner);
                    }
                    entry.add("wound", wound);
                    var uvOrder = new JsonArray();
                    for (long value : uvs) {
                        uvOrder.add(value);
                    }
                    entry.add("uv_order", uvOrder);
                } catch (java.lang.reflect.InvocationTargetException failure) {
                    // The reference throws when no vertex matches a corner of
                    // the bounding box, which a non-axis-aligned quad hits.
                    entry.addProperty("winding", "no_matching_vertex");
                }
            }
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("face bakery internals unavailable", error);
        }
        return entry;
    }

    /** The four face corners after one element rotation. */
    static JsonObject rotation(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var from = vector(row.get("from").getAsJsonArray());
        var to = vector(row.get("to").getAsJsonArray());
        var facing = net.minecraft.core.Direction.byName(row.get("facing").getAsString());
        var origin = vector(row.get("origin").getAsJsonArray());
        // The deserializer scales the origin into block units before use.
        origin.mul(0.0625F);
        float angle = Float.intBitsToFloat(
            Integer.parseUnsignedInt(row.get("angle").getAsString(), 16));
        var axis = net.minecraft.core.Direction.Axis.byName(row.get("axis").getAsString());
        boolean rescale = row.get("rescale").getAsBoolean();
        if (facing == null || axis == null) {
            entry.addProperty("outcome", "unknown_input");
            return entry;
        }

        var value = new net.minecraft.client.resources.model.cuboid.CuboidRotation
            .SingleAxisRotation(axis, angle);
        var elementRotation = new net.minecraft.client.resources.model.cuboid.CuboidRotation(
            origin, value, rescale);
        var transform = elementRotation.transform();

        entry.addProperty("outcome", "rotated");
        var matrix = new JsonArray();
        for (int column = 0; column < 4; column++) {
            for (int rowIndex = 0; rowIndex < 4; rowIndex++) {
                matrix.add(String.format(java.util.Locale.ROOT, "%08x",
                    Float.floatToRawIntBits(transform.get(column, rowIndex))));
            }
        }
        entry.add("transform", matrix);

        var info = net.minecraft.client.renderer.FaceInfo.fromFacing(facing);
        var corners = new JsonArray();
        for (int index = 0; index < 4; index++) {
            var vertex = info.getVertexInfo(index).select(from, to).div(16.0F);
            // Java's rotateVertexBy: move to the origin, transform, move back.
            vertex.sub(origin);
            transform.transformPosition(vertex);
            vertex.add(origin);
            var corner = new JsonObject();
            corner.addProperty("x", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(vertex.x())));
            corner.addProperty("y", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(vertex.y())));
            corner.addProperty("z", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(vertex.z())));
            corners.add(corner);
        }
        entry.add("corners", corners);
        return entry;
    }

    static String bits(Class<?> owner, Object instance, String accessor) throws ReflectiveOperationException {
        var method = owner.getMethod(accessor);
        Reflect.open(method);
        float value = (Float)method.invoke(instance);
        return String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value));
    }

    static org.joml.Vector3f vector(JsonArray array) {
        return new org.joml.Vector3f(
            Float.intBitsToFloat(Integer.parseUnsignedInt(array.get(0).getAsString(), 16)),
            Float.intBitsToFloat(Integer.parseUnsignedInt(array.get(1).getAsString(), 16)),
            Float.intBitsToFloat(Integer.parseUnsignedInt(array.get(2).getAsString(), 16))
        );
    }

    static JsonArray vectorBits(org.joml.Vector3fc value) {
        var array = new JsonArray();
        array.add(String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value.x())));
        array.add(String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value.y())));
        array.add(String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value.z())));
        return array;
    }

    static JsonArray matrixBits(org.joml.Matrix4fc matrix) {
        var out = new JsonArray();
        for (int column = 0; column < 4; column++) {
            for (int rowIndex = 0; rowIndex < 4; rowIndex++) {
                out.add(String.format(java.util.Locale.ROOT, "%08x",
                    Float.floatToRawIntBits(matrix.get(column, rowIndex))));
            }
        }
        return out;
    }

    /** Which face a vector points at, as the cullface rotation asks. */
    static JsonObject nearestDirection(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "resolved");
        entry.addProperty("direction", net.minecraft.core.Direction.getApproximateNearest(
            Oracle.f32(row, "x"), Oracle.f32(row, "y"), Oracle.f32(row, "z")).getName());
        return entry;
    }

    /** Where a face's cullface direction lands once the variant is turned. */
    static JsonObject cullRotation(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var group = com.mojang.math.Quadrant.fromXYZAngles(
            com.mojang.math.Quadrant.parseJson(row.get("x").getAsInt()),
            com.mojang.math.Quadrant.parseJson(row.get("y").getAsInt()),
            com.mojang.math.Quadrant.parseJson(row.get("z").getAsInt()));
        var rotation = net.minecraft.client.renderer.block.dispatch.BlockModelRotation.get(group);
        var facing = net.minecraft.core.Direction.byName(row.get("facing").getAsString());
        entry.addProperty("outcome", "resolved");
        entry.addProperty("direction", net.minecraft.core.Direction
            .rotate(rotation.transformation().getMatrix(), facing).getName());
        return entry;
    }

    /** The per-vertex texture coordinates for one face rotation. */
    static JsonObject uvRotation(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        int degrees = row.get("degrees").getAsInt();
        com.mojang.math.Quadrant quadrant;
        try {
            quadrant = com.mojang.math.Quadrant.parseJson(degrees);
        } catch (RuntimeException error) {
            entry.addProperty("outcome", "rejected");
            return entry;
        }
        var values = row.get("uvs").getAsJsonArray();
        var uvs = new net.minecraft.client.resources.model.cuboid.CuboidFace.UVs(
            Float.intBitsToFloat(Integer.parseUnsignedInt(values.get(0).getAsString(), 16)),
            Float.intBitsToFloat(Integer.parseUnsignedInt(values.get(1).getAsString(), 16)),
            Float.intBitsToFloat(Integer.parseUnsignedInt(values.get(2).getAsString(), 16)),
            Float.intBitsToFloat(Integer.parseUnsignedInt(values.get(3).getAsString(), 16))
        );
        entry.addProperty("outcome", "rotated");
        entry.addProperty("shift", quadrant.shift);
        var pairs = new JsonArray();
        for (int vertex = 0; vertex < 4; vertex++) {
            var pair = new JsonObject();
            pair.addProperty("u", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(
                net.minecraft.client.resources.model.cuboid.CuboidFace.getU(uvs, quadrant, vertex))));
            pair.addProperty("v", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(
                net.minecraft.client.resources.model.cuboid.CuboidFace.getV(uvs, quadrant, vertex))));
            pairs.add(pair);
        }
        entry.add("pairs", pairs);
        return entry;
    }
}
