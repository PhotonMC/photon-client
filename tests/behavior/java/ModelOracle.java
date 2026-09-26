import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import java.util.ArrayList;
import java.util.List;
import java.util.HexFormat;
import java.util.TreeMap;
import net.minecraft.client.resources.model.ModelDebugName;
import net.minecraft.client.resources.model.sprite.Material;
import net.minecraft.client.resources.model.sprite.TextureSlots;

/** Test glue only: drives the original texture slot resolver. */
public final class ModelOracle {
    private static final List<String> SLOT_PROBES = List.of(
        "all", "top", "bottom", "side", "north", "particle", "missing", "#all", "#top"
    );

    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            switch (operation) {
                case "texture_slots" -> rows.add(resolve(element.getAsJsonObject()));
                case "face_geometry" -> rows.add(geometry(element.getAsJsonObject()));
                case "element_rotation" -> rows.add(rotation(element.getAsJsonObject()));
                case "quad_winding" -> rows.add(winding(element.getAsJsonObject()));
                case "euler_rotation" -> rows.add(euler(element.getAsJsonObject()));
                case "face_uv_rotation" -> rows.add(uvRotation(element.getAsJsonObject()));
                case "parse_model" -> rows.add(parseModel(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException("Unknown model operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Corner layout and default texture mapping for one cuboid face. */
    private static JsonObject geometry(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var from = vector(row.get("from").getAsJsonArray());
        var to = vector(row.get("to").getAsJsonArray());
        var facing = net.minecraft.core.Direction.byName(row.get("facing").getAsString());
        if (facing == null) {
            entry.addProperty("outcome", "unknown_facing");
            return entry;
        }
        entry.addProperty("outcome", "baked");

        try {
            var method = net.minecraft.client.resources.model.cuboid.FaceBakery.class
                .getDeclaredMethod("defaultFaceUV", org.joml.Vector3fc.class,
                                   org.joml.Vector3fc.class, net.minecraft.core.Direction.class);
            method.setAccessible(true);
            var uvs = method.invoke(null, from, to, facing);
            var uvClass = uvs.getClass();
            entry.addProperty("min_u", bits(uvClass, uvs, "minU"));
            entry.addProperty("min_v", bits(uvClass, uvs, "minV"));
            entry.addProperty("max_u", bits(uvClass, uvs, "maxU"));
            entry.addProperty("max_v", bits(uvClass, uvs, "maxV"));
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("default face UV unavailable", error);
        }

        var info = net.minecraft.client.renderer.FaceInfo.fromFacing(facing);
        var corners = new JsonArray();
        for (int index = 0; index < 4; index++) {
            // The bakery divides by 16 right after selecting the corner; the
            // division is kept here so the comparison covers it.
            var vertex = info.getVertexInfo(index).select(from, to).div(16.0F);
            var corner = new JsonObject();
            corner.addProperty("x", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(vertex.x())));
            corner.addProperty("y", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(vertex.y())));
            corner.addProperty("z", String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(vertex.z())));
            corners.add(corner);
        }
        entry.add("corners", corners);
        return entry;
    }

    /** A whole model file, read through the reference's own configuration. */
    private static JsonObject parseModel(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        String text = new String(
            HexFormat.of().parseHex(row.get("text").getAsString()),
            java.nio.charset.StandardCharsets.UTF_8);
        net.minecraft.client.resources.model.cuboid.CuboidModel model;
        try {
            model = net.minecraft.client.resources.model.cuboid.CuboidModel
                .fromStream(new java.io.StringReader(text));
        } catch (RuntimeException error) {
            entry.addProperty("outcome", "rejected");
            return entry;
        }
        entry.addProperty("outcome", "parsed");
        entry.addProperty("parent", model.parent() == null ? "none" : model.parent().toString());
        entry.addProperty("ambient_occlusion",
            model.ambientOcclusion() == null ? "unset" : model.ambientOcclusion().toString());
        entry.addProperty("gui_light", model.guiLight() == null ? "unset" : model.guiLight().name());

        var slots = new JsonArray();
        var slotValues = new java.util.TreeMap<>(model.textureSlots().values());
        for (var slot : slotValues.entrySet()) {
            var described = new JsonObject();
            described.addProperty("slot", slot.getKey());
            Object contents = slot.getValue();
            String kind = contents.getClass().getSimpleName();
            described.addProperty("kind", kind);
            try {
                if (kind.equals("Reference")) {
                    var target = contents.getClass().getDeclaredMethod("target");
                    target.setAccessible(true);
                    described.addProperty("target", (String)target.invoke(contents));
                } else {
                    var material = contents.getClass().getDeclaredMethod("material");
                    material.setAccessible(true);
                    var value = (net.minecraft.client.resources.model.sprite.Material)
                        material.invoke(contents);
                    described.addProperty("sprite", value.sprite().toString());
                    described.addProperty("force_translucent", value.forceTranslucent());
                }
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("texture slot internals unavailable", error);
            }
            slots.add(described);
        }
        entry.add("texture_slots", slots);

        var elements = new JsonArray();
        if (model.geometry() instanceof net.minecraft.client.resources.model.cuboid
                .UnbakedCuboidGeometry geometry) {
            for (var element : geometry.elements()) {
                elements.add(describeElement(element));
            }
            entry.addProperty("has_geometry", true);
        } else {
            entry.addProperty("has_geometry", false);
        }
        entry.add("elements", elements);
        return entry;
    }

    private static JsonObject describeElement(
            net.minecraft.client.resources.model.cuboid.CuboidModelElement element) {
        var described = new JsonObject();
        described.add("from", vectorBits(element.from()));
        described.add("to", vectorBits(element.to()));
        described.addProperty("shade", element.shade());
        described.addProperty("light_emission", element.lightEmission());
        var rotation = element.rotation();
        if (rotation == null) {
            described.addProperty("rotation", "none");
        } else {
            var described_rotation = new JsonObject();
            described_rotation.add("origin", vectorBits(rotation.origin()));
            described_rotation.addProperty("rescale", rotation.rescale());
            var matrix = new JsonArray();
            for (int column = 0; column < 4; column++) {
                for (int rowIndex = 0; rowIndex < 4; rowIndex++) {
                    matrix.add(String.format(java.util.Locale.ROOT, "%08x",
                        Float.floatToRawIntBits(rotation.transform().get(column, rowIndex))));
                }
            }
            described_rotation.add("transform", matrix);
            described.add("rotation", described_rotation);
        }
        var faces = new JsonArray();
        var ordered = new java.util.TreeMap<String, net.minecraft.client.resources.model.cuboid.CuboidFace>();
        element.faces().forEach((direction, face) -> ordered.put(direction.getName(), face));
        for (var face : ordered.entrySet()) {
            var describedFace = new JsonObject();
            describedFace.addProperty("direction", face.getKey());
            var value = face.getValue();
            describedFace.addProperty("cullface",
                value.cullForDirection() == null ? "none" : value.cullForDirection().getName());
            describedFace.addProperty("tint_index", value.tintIndex());
            describedFace.addProperty("texture", value.texture());
            describedFace.addProperty("uv_rotation", value.rotation().shift);
            if (value.uvs() == null) {
                describedFace.addProperty("uvs", "none");
            } else {
                var uvs = new JsonArray();
                uvs.add(String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value.uvs().minU())));
                uvs.add(String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value.uvs().minV())));
                uvs.add(String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value.uvs().maxU())));
                uvs.add(String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value.uvs().maxV())));
                describedFace.add("uvs", uvs);
            }
            faces.add(describedFace);
        }
        described.add("faces", faces);
        return described;
    }

    private static JsonArray vectorBits(org.joml.Vector3fc value) {
        var array = new JsonArray();
        array.add(String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value.x())));
        array.add(String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value.y())));
        array.add(String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value.z())));
        return array;
    }

    /** The per-vertex texture coordinates for one face rotation. */
    private static JsonObject uvRotation(JsonObject row) {
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

    /** The transform an element's three-axis rotation block produces. */
    private static JsonObject euler(JsonObject row) {
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
    private static JsonObject winding(JsonObject row) {
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
            var findClosest = bakery.getDeclaredMethod("findClosestDirection", org.joml.Vector3f.class);
            findClosest.setAccessible(true);
            var chosen = (net.minecraft.core.Direction)findClosest.invoke(null, normal);
            entry.addProperty("direction", chosen == null ? "none" : chosen.getName());

            if (chosen != null) {
                var uvs = new long[] {10L, 20L, 30L, 40L};
                var rewind = bakery.getDeclaredMethod("recalculateWinding",
                    org.joml.Vector3fc[].class, long[].class, net.minecraft.core.Direction.class);
                rewind.setAccessible(true);
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
    private static JsonObject rotation(JsonObject row) {
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

    private static String bits(Class<?> owner, Object instance, String accessor) throws ReflectiveOperationException {
        var method = owner.getMethod(accessor);
        method.setAccessible(true);
        float value = (Float)method.invoke(instance);
        return String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(value));
    }

    private static org.joml.Vector3f vector(JsonArray array) {
        return new org.joml.Vector3f(
            Float.intBitsToFloat(Integer.parseUnsignedInt(array.get(0).getAsString(), 16)),
            Float.intBitsToFloat(Integer.parseUnsignedInt(array.get(1).getAsString(), 16)),
            Float.intBitsToFloat(Integer.parseUnsignedInt(array.get(2).getAsString(), 16))
        );
    }

    private static JsonObject resolve(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var resolver = new TextureSlots.Resolver();
        // The model chain is listed child first; the resolver is fed in that
        // same order, which is how ResolvedModel walks up to the root.
        for (var levelElement : row.get("chain").getAsJsonArray()) {
            var textures = new JsonObject();
            for (var slotElement : levelElement.getAsJsonArray()) {
                var slot = slotElement.getAsJsonObject();
                String name = slot.get("slot").getAsString();
                if (slot.has("raw")) {
                    textures.add(name, new JsonPrimitive(slot.get("raw").getAsString()));
                } else {
                    var material = new JsonObject();
                    material.addProperty("sprite", slot.get("sprite").getAsString());
                    material.addProperty("force_translucent", slot.get("force_translucent").getAsBoolean());
                    textures.add(name, material);
                }
            }
            resolver.addLast(TextureSlots.parseTextureMap(textures));
        }

        TextureSlots slots;
        try {
            slots = resolver.resolve((ModelDebugName)() -> "parity");
        } catch (RuntimeException error) {
            entry.addProperty("outcome", "resolve_failed");
            return entry;
        }

        entry.addProperty("outcome", "resolved");
        var lookups = new JsonArray();
        var ordered = new TreeMap<String, Material>();
        for (String probe : SLOT_PROBES) {
            Material material = slots.getMaterial(probe);
            if (material != null) {
                ordered.put(probe, material);
            }
        }
        for (var found : ordered.entrySet()) {
            var lookup = new JsonObject();
            lookup.addProperty("probe", found.getKey());
            lookup.addProperty("sprite", found.getValue().sprite().toString());
            lookup.addProperty("force_translucent", found.getValue().forceTranslucent());
            lookups.add(lookup);
        }
        entry.add("lookups", lookups);
        return entry;
    }
}
