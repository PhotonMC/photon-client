import java.util.HexFormat;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import java.util.List;
import net.minecraft.client.resources.model.sprite.Material;
import net.minecraft.client.resources.model.ModelDebugName;
import net.minecraft.client.resources.model.sprite.TextureSlots;
import java.util.TreeMap;

/** Test glue only: drives the original texture slot resolver. */
public final class ModelOracle extends ModelMath {
    static final List<String> SLOT_PROBES = List.of(
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
                case "model_ao" -> rows.add(modelAmbientOcclusion(element.getAsJsonObject()));
                case "model_state" -> rows.add(modelState(element.getAsJsonObject()));
                case "blockstate_parse" -> rows.add(blockstate(element.getAsJsonObject()));
                case "uv_lock" -> rows.add(uvLock(element.getAsJsonObject()));
                case "nearest_direction" -> rows.add(nearestDirection(element.getAsJsonObject()));
                case "cull_rotation" -> rows.add(cullRotation(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException("Unknown model operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * One blockstate file, read through the dispatcher's codec.
     *
     * Only the parse is driven here. Turning a selector into a predicate needs
     * the block's state definition, which needs the registry.
     */
    static JsonObject blockstate(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        try {
            var json = com.google.gson.JsonParser.parseString(row.get("json").getAsString());
            var parsed = net.minecraft.client.renderer.block.dispatch.BlockStateModelDispatcher.CODEC
                .parse(com.mojang.serialization.JsonOps.INSTANCE, json)
                .getOrThrow();
            entry.addProperty("outcome", "parsed");
            if (parsed.simpleModels().isPresent()) {
                var models = parsed.simpleModels().get().models();
                var variants = new JsonObject();
                // The codec builds an unordered map, so the keys are sorted
                // here to make the comparison deterministic.
                for (String key : new java.util.TreeSet<>(models.keySet())) {
                    variants.add(key, describeUnbaked(models.get(key)));
                }
                entry.add("variants", variants);
            }
            if (parsed.multiPart().isPresent()) {
                var selectors = new JsonArray();
                for (var selector : parsed.multiPart().get().selectors()) {
                    var described = new JsonObject();
                    described.add("when", selector.condition()
                        .map(ModelOracle::describeCondition)
                        .orElse(null));
                    described.add("apply", describeUnbaked(selector.variant()));
                    selectors.add(described);
                }
                entry.add("multipart", selectors);
            }
        } catch (Exception failure) {
            entry.addProperty("outcome", "rejected");
        }
        return entry;
    }

    /** A variant or a weighted list of them, flattened to one array. */
    static JsonArray describeUnbaked(
        net.minecraft.client.renderer.block.dispatch.BlockStateModel.Unbaked unbaked) {
        var out = new JsonArray();
        if (unbaked instanceof net.minecraft.client.renderer.block.dispatch
                .SingleVariant.Unbaked single) {
            out.add(describeVariant(single.variant(), 1));
            return out;
        }
        var weighted = (net.minecraft.client.renderer.block.dispatch
            .WeightedVariants.Unbaked) unbaked;
        for (var element : weighted.entries().unwrap()) {
            var single = (net.minecraft.client.renderer.block.dispatch
                .SingleVariant.Unbaked) element.value();
            out.add(describeVariant(single.variant(), element.weight()));
        }
        return out;
    }

    static JsonObject describeVariant(
        net.minecraft.client.renderer.block.dispatch.Variant variant, int weight) {
        var described = new JsonObject();
        described.addProperty("model", variant.modelLocation().toString());
        described.addProperty("x", variant.modelState().x().name());
        described.addProperty("y", variant.modelState().y().name());
        described.addProperty("z", variant.modelState().z().name());
        described.addProperty("uvlock", variant.modelState().uvLock());
        described.addProperty("weight", weight);
        return described;
    }

    static JsonObject describeCondition(
        net.minecraft.client.renderer.block.dispatch.multipart.Condition condition) {
        var described = new JsonObject();
        if (condition instanceof net.minecraft.client.renderer.block.dispatch.multipart
                .KeyValueCondition keyValue) {
            described.addProperty("kind", "key_value");
            var tests = new JsonObject();
            for (String key : new java.util.TreeSet<>(keyValue.tests().keySet())) {
                var terms = new JsonArray();
                for (var term : keyValue.tests().get(key).entries()) {
                    var described_term = new JsonObject();
                    described_term.addProperty("value", term.value());
                    described_term.addProperty("negated", term.negated());
                    terms.add(described_term);
                }
                tests.add(key, terms);
            }
            described.add("tests", tests);
            return described;
        }
        var combined = (net.minecraft.client.renderer.block.dispatch.multipart
            .CombinedCondition) condition;
        described.addProperty("kind", "combined");
        described.addProperty("operation", combined.operation().getSerializedName());
        var terms = new JsonArray();
        for (var term : combined.terms()) {
            terms.add(describeCondition(term));
        }
        described.add("terms", terms);
        return described;
    }

    /** Corner layout and default texture mapping for one cuboid face. */
    static JsonObject geometry(JsonObject row) {
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

    /** The transform a blockstate variant's x/y rotation produces. */
    static JsonObject modelState(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var xQuadrant = com.mojang.math.Quadrant.parseJson(row.get("x").getAsInt());
        var yQuadrant = com.mojang.math.Quadrant.parseJson(row.get("y").getAsInt());
        var zQuadrant = com.mojang.math.Quadrant.parseJson(row.get("z").getAsInt());
        boolean uvLock = row.get("uvlock").getAsBoolean();
        // A variant may turn about all three axes, and the dispatcher always
        // goes through the three-axis composition even when z is zero.
        var group = com.mojang.math.Quadrant.fromXYZAngles(xQuadrant, yQuadrant, zQuadrant);
        var rotation = net.minecraft.client.renderer.block.dispatch.BlockModelRotation.get(group);
        net.minecraft.client.renderer.block.dispatch.ModelState state =
            uvLock ? rotation.withUvLock() : rotation;

        entry.addProperty("outcome", "resolved");
        entry.add("transformation", matrixBits(state.transformation().getMatrix()));
        var faces = new JsonArray();
        for (var face : net.minecraft.core.Direction.values()) {
            var described = new JsonObject();
            described.addProperty("face", face.getName());
            described.add("forward", matrixBits(state.faceTransformation(face)));
            described.add("inverse", matrixBits(state.inverseFaceTransformation(face)));
            faces.add(described);
        }
        entry.add("faces", faces);
        return entry;
    }

    /**
     * What a UV-locked face does to one texture coordinate.
     *
     * The matrix, the identity test and the transform are the reference's
     * own. The only thing written out here is the half-unit shift to the
     * face's centre and back, which is two additions read from `FaceBakery`.
     */
    static JsonObject uvLock(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var xQuadrant = com.mojang.math.Quadrant.parseJson(row.get("x").getAsInt());
        var yQuadrant = com.mojang.math.Quadrant.parseJson(row.get("y").getAsInt());
        var zQuadrant = com.mojang.math.Quadrant.parseJson(row.get("z").getAsInt());
        var group = com.mojang.math.Quadrant.fromXYZAngles(xQuadrant, yQuadrant, zQuadrant);
        var rotation = net.minecraft.client.renderer.block.dispatch.BlockModelRotation.get(group);
        net.minecraft.client.renderer.block.dispatch.ModelState state =
            row.get("uvlock").getAsBoolean() ? rotation.withUvLock() : rotation;
        var facing = net.minecraft.core.Direction.byName(row.get("facing").getAsString());
        var uvTransform = state.inverseFaceTransformation(facing);
        float rawU = Oracle.f32(row, "u");
        float rawV = Oracle.f32(row, "v");
        boolean identity = com.mojang.math.MatrixUtil.isIdentity(uvTransform);
        float transformedU;
        float transformedV;
        if (identity) {
            transformedU = rawU;
            transformedV = rawV;
        } else {
            var transformed = uvTransform.transformPosition(
                new org.joml.Vector3f(rawU - 0.5F, rawV - 0.5F, 0.0F));
            transformedU = transformed.x + 0.5F;
            transformedV = transformed.y + 0.5F;
        }
        entry.addProperty("outcome", "transformed");
        entry.addProperty("identity", identity);
        entry.addProperty("u", Oracle.bits(transformedU));
        entry.addProperty("v", Oracle.bits(transformedV));
        return entry;
    }

    /**
     * The ambient occlusion flag a model ends up with after inheritance, read
     * from a real resolved chain built by the model discovery.
     */
    static JsonObject modelAmbientOcclusion(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var models = new java.util.HashMap<net.minecraft.resources.Identifier,
            net.minecraft.client.resources.model.UnbakedModel>();
        for (var element : row.getAsJsonArray("models")) {
            var model = element.getAsJsonObject();
            String text = new String(
                HexFormat.of().parseHex(model.get("text").getAsString()),
                java.nio.charset.StandardCharsets.UTF_8);
            models.put(
                net.minecraft.resources.Identifier.parse(model.get("name").getAsString()),
                net.minecraft.client.resources.model.cuboid.CuboidModel
                    .fromStream(new java.io.StringReader(text)));
        }
        var leaf = net.minecraft.resources.Identifier.parse(row.get("leaf").getAsString());
        var discovery = new net.minecraft.client.resources.model.ModelDiscovery(
            models, net.minecraft.client.resources.model.cuboid.MissingCuboidModel.missingModel());
        discovery.addRoot(resolver -> resolver.markDependency(leaf));
        var resolved = discovery.resolve().get(leaf);
        entry.addProperty("outcome", "resolved");
        entry.addProperty("ambient_occlusion", resolved.getTopAmbientOcclusion());
        return entry;
    }

    /** A whole model file, read through the reference's own configuration. */
    static JsonObject parseModel(JsonObject row) {
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
                    var target = Reflect.method(contents.getClass(), "target");
                    described.addProperty("target", (String)target.invoke(contents));
                } else {
                    var material = Reflect.method(contents.getClass(), "material");
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

        // Each display context's transform, as the model's own `display`
        // gives it; a context it leaves out is the identity.
        var display = new JsonObject();
        if (model.transforms() != null) {
            for (var context : net.minecraft.world.item.ItemDisplayContext.values()) {
                var transform = model.transforms().getTransform(context);
                if (transform == net.minecraft.client.resources.model.cuboid.ItemTransform.NO_TRANSFORM) {
                    continue;
                }
                var described = new JsonObject();
                described.add("rotation", vectorBits(transform.rotation()));
                described.add("translation", vectorBits(transform.translation()));
                described.add("scale", vectorBits(transform.scale()));
                display.add(context.getSerializedName(), described);
            }
        }
        entry.add("display", display);
        return entry;
    }

    static JsonObject describeElement(
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

    static JsonObject resolve(JsonObject row) {
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
