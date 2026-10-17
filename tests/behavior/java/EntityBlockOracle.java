import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityBlockOracle extends EntityAquaticOracle {
    /**
     * The render type the armor's enchantment glint asks for (`RenderTypes.armorEntityGlint`): its
     * pipeline's state and texture, and the matrix that moves the texture at some times and glint
     * speeds (`TextureTransform.setupGlintTexturing`, the time and the option set as asked).
     */
    /** A chest's model (a half of a double one too), its lid at a given eased openness. */
    static JsonObject chestModel(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.object.chest.ChestModel(bakedLayer(row.get("layer").getAsString()));
            model.setupAnim(Oracle.f32(row, "open"));
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the real `ChestRenderer.submit` hands the collector: the pose, the lid's eased openness,
     * the light and the sprite, for a chest's type, material, facing and openness.
     */
    static JsonObject chestPose(final JsonObject request) throws Exception {
        Oracle.vanillaRegistries();
        var roots = net.minecraft.client.model.geom.LayerDefinitions.createRoots();
        java.util.function.Function<net.minecraft.client.model.geom.ModelLayerLocation,
            net.minecraft.client.model.object.chest.ChestModel> bake = layer ->
                new net.minecraft.client.model.object.chest.ChestModel(roots.get(layer).bakeRoot());
        var models = net.minecraft.client.renderer.blockentity.ChestRenderer.LAYERS.map(bake);
        var renderer = (net.minecraft.client.renderer.blockentity.ChestRenderer<?>) UNSAFE.allocateInstance(
            net.minecraft.client.renderer.blockentity.ChestRenderer.class);
        var modelsField = net.minecraft.client.renderer.blockentity.ChestRenderer.class.getDeclaredField("models");
        modelsField.setAccessible(true);
        modelsField.set(renderer, models);
        // The first use of each facing's matrix works its flags out (a facing that turns nothing
        // becomes the identity), as the first chest drawn does in the game.
        for (var facing : net.minecraft.core.Direction.Plane.HORIZONTAL) {
            com.mojang.math.MatrixUtil.isPureTranslation(
                net.minecraft.client.renderer.blockentity.ChestRenderer.modelTransformation(facing).getMatrix());
        }
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.blockentity.state.ChestRenderState();
            state.type = net.minecraft.world.level.block.state.properties.ChestType.valueOf(
                row.get("type").getAsString().toUpperCase());
            state.facing = net.minecraft.core.Direction.valueOf(row.get("facing").getAsString().toUpperCase());
            state.material = net.minecraft.client.renderer.blockentity.state.ChestRenderState.ChestMaterialType
                .valueOf(row.get("material").getAsString());
            state.open = Oracle.f32(row, "open");
            state.lightCoords = row.get("light").getAsInt();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
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
                        out.add("matrix", described);
                        out.addProperty("open", Oracle.bits((Float) args[1]));
                        out.addProperty("light", (Integer) args[3]);
                        out.addProperty("texture", ((net.minecraft.client.resources.model.sprite.SpriteId) args[6])
                            .texture().getPath());
                        out.addProperty("layer", args[0] == models.single() ? "single"
                            : args[0] == models.left() ? "left" : "right");
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "x"), Oracle.f32(row, "y"), Oracle.f32(row, "z"));
            renderer.submit(state, stack, collector, null);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Everything the real `BannerRenderer.submit` hands the collector for a banner block: the pole
     * and bar, the flag, and each pattern layer in the order it is drawn, each with its pose, light,
     * sprite and tint.
     */
    static JsonObject bannerPose(final JsonObject request) throws Exception {
        var registries = Oracle.vanillaRegistries();
        var patterns = registries.lookupOrThrow(net.minecraft.core.registries.Registries.BANNER_PATTERN);
        var requested = new net.minecraft.client.resources.model.sprite.SpriteId[1];
        var sprites = (net.minecraft.client.resources.model.sprite.SpriteGetter) java.lang.reflect.Proxy.newProxyInstance(
            EntityOracle.class.getClassLoader(),
            new Class<?>[] {net.minecraft.client.resources.model.sprite.SpriteGetter.class},
            (proxy, method, args) -> {
                requested[0] = (net.minecraft.client.resources.model.sprite.SpriteId) args[0];
                return null;
            });
        var renderer = new net.minecraft.client.renderer.blockentity.BannerRenderer(
            net.minecraft.client.model.geom.EntityModelSet.vanilla(), sprites);
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.blockentity.state.BannerRenderState();
            var transformations = net.minecraft.client.renderer.blockentity.BannerRenderer.TRANSFORMATIONS;
            if (row.get("wall").isJsonNull()) {
                state.transformation = transformations.freeTransformations(row.get("rotation").getAsInt());
                state.attachmentType = net.minecraft.world.level.block.BannerBlock.AttachmentType.GROUND;
            } else {
                state.transformation = transformations.wallTransformation(
                    net.minecraft.core.Direction.valueOf(row.get("wall").getAsString().toUpperCase()));
                state.attachmentType = net.minecraft.world.level.block.BannerBlock.AttachmentType.WALL;
            }
            state.baseColor = net.minecraft.world.item.DyeColor.valueOf(row.get("color").getAsString().toUpperCase());
            var layers = new java.util.ArrayList<net.minecraft.world.level.block.entity.BannerPatternLayers.Layer>();
            for (var layerElement : row.get("patterns").getAsJsonArray()) {
                var layer = layerElement.getAsJsonObject();
                var key = net.minecraft.resources.ResourceKey.create(net.minecraft.core.registries.Registries.BANNER_PATTERN,
                    net.minecraft.resources.Identifier.parse(layer.get("pattern").getAsString()));
                layers.add(new net.minecraft.world.level.block.entity.BannerPatternLayers.Layer(
                    patterns.getOrThrow(key),
                    net.minecraft.world.item.DyeColor.valueOf(layer.get("color").getAsString().toUpperCase())));
            }
            state.patterns = new net.minecraft.world.level.block.entity.BannerPatternLayers(layers);
            state.phase = Oracle.f32(row, "phase");
            state.lightCoords = row.get("light").getAsInt();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var calls = new JsonArray();
            var order = new int[] {0};
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
                        var call = new JsonObject();
                        var matrix = new float[16];
                        ((com.mojang.blaze3d.vertex.PoseStack) args[2]).last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        call.add("matrix", described);
                        call.addProperty("order", order[0]);
                        @SuppressWarnings("unchecked")
                        var model = (net.minecraft.client.model.Model<Object>) args[0];
                        if (args[3] instanceof Integer) {
                            // Plain sprite call: light, overlay, tint, sprite.
                            call.addProperty("light", (Integer) args[3]);
                            call.addProperty("tint", Integer.toHexString((Integer) args[5]));
                            call.addProperty("texture", ((net.minecraft.client.resources.model.sprite.SpriteId) args[6])
                                .texture().getPath());
                        } else {
                            // Pattern layer: render type, light, overlay, tint, sprite.
                            call.addProperty("light", (Integer) args[4]);
                            call.addProperty("tint", Integer.toHexString((Integer) args[6]));
                            call.addProperty("texture", requested[0].texture().getPath());
                        }
                        model.setupAnim(args[1]);
                        call.add("quads", posedQuads("", model).get("quads"));
                        calls.add(call);
                    }
                    return null;
                });
            renderer.submit(state, new com.mojang.blaze3d.vertex.PoseStack(), holder[0], null);
            out.add("calls", calls);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the real `SkullBlockRenderer.submit` hands the collector for a skull block: the pose its
     * wall or rotation gives, the light, the texture and the model posed for the animation.
     */
    static JsonObject skullBlockPose(final JsonObject request) throws Exception {
        Oracle.vanillaRegistries();
        var modelSet = net.minecraft.client.model.geom.EntityModelSet.vanilla();
        var renderer = (net.minecraft.client.renderer.blockentity.SkullBlockRenderer) UNSAFE.allocateInstance(
            net.minecraft.client.renderer.blockentity.SkullBlockRenderer.class);
        var modelsField = net.minecraft.client.renderer.blockentity.SkullBlockRenderer.class.getDeclaredField("modelByType");
        modelsField.setAccessible(true);
        java.util.function.Function<net.minecraft.world.level.block.SkullBlock.Type,
            net.minecraft.client.model.object.skull.SkullModelBase> models = net.minecraft.util.Util.memoize(
                type -> net.minecraft.client.renderer.blockentity.SkullBlockRenderer.createModel(modelSet, type));
        modelsField.set(renderer, models);
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.blockentity.state.SkullBlockRenderState();
            state.skullType = net.minecraft.world.level.block.SkullBlock.Types.valueOf(row.get("kind").getAsString());
            var transformations = net.minecraft.client.renderer.blockentity.SkullBlockRenderer.TRANSFORMATIONS;
            state.transformation = row.get("wall").isJsonNull()
                ? transformations.freeTransformations(row.get("rotation").getAsInt())
                : transformations.wallTransformation(
                    net.minecraft.core.Direction.valueOf(row.get("wall").getAsString().toUpperCase()));
            state.animationProgress = Oracle.f32(row, "animation");
            state.lightCoords = row.get("light").getAsInt();
            state.renderType = net.minecraft.client.renderer.blockentity.SkullBlockRenderer.getSkullRenderType(
                state.skullType, null);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
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
                        out.add("matrix", described);
                        out.addProperty("light", (Integer) args[4]);
                        var found = java.util.regex.Pattern.compile("textures/([a-z0-9_/]+)\\.png")
                            .matcher(String.valueOf(args[3]));
                        out.addProperty("texture", found.find() ? found.group(1) : String.valueOf(args[3]));
                        @SuppressWarnings("unchecked")
                        var model = (net.minecraft.client.model.Model<Object>) args[0];
                        model.setupAnim(args[1]);
                        out.add("quads", posedQuads("", model).get("quads"));
                    }
                    return null;
                });
            renderer.submit(state, new com.mojang.blaze3d.vertex.PoseStack(), collector, null);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the real `ElderGuardianParticleGroup` hands the collector for an elder guardian particle
     * of an age, the camera looking along a yaw and a pitch: the pose, the light, the colour and
     * the texture. (The model is the elder guardian's own, drawn unposed.)
     */
    static JsonObject elderGuardianParticlePose(final JsonObject request) throws Exception {
        Oracle.vanillaRegistries();
        var group = (net.minecraft.client.particle.ElderGuardianParticleGroup) UNSAFE.allocateInstance(
            net.minecraft.client.particle.ElderGuardianParticleGroup.class);
        var queueField = net.minecraft.client.particle.ParticleGroup.class.getDeclaredField("particles");
        queueField.setAccessible(true);
        var ageField = net.minecraft.client.particle.Particle.class.getDeclaredField("age");
        ageField.setAccessible(true);
        var lifetimeField = net.minecraft.client.particle.Particle.class.getDeclaredField("lifetime");
        lifetimeField.setAccessible(true);
        var typeField = net.minecraft.client.particle.ElderGuardianParticle.class.getDeclaredField("renderType");
        typeField.setAccessible(true);
        var setRotation = net.minecraft.client.Camera.class.getDeclaredMethod("setRotation", float.class, float.class);
        setRotation.setAccessible(true);
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var particle = (net.minecraft.client.particle.ElderGuardianParticle) UNSAFE.allocateInstance(
                net.minecraft.client.particle.ElderGuardianParticle.class);
            ageField.setInt(particle, row.get("age").getAsInt());
            lifetimeField.setInt(particle, row.get("lifetime").getAsInt());
            typeField.set(particle, net.minecraft.client.renderer.rendertype.RenderTypes.entityTranslucent(
                net.minecraft.client.renderer.entity.ElderGuardianRenderer.GUARDIAN_ELDER_LOCATION));
            queueField.set(group, new java.util.ArrayDeque<>(java.util.List.of(particle)));
            var camera = new net.minecraft.client.Camera();
            setRotation.invoke(camera, Oracle.f32(row, "yaw"), Oracle.f32(row, "pitch"));
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
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
                        out.add("matrix", described);
                        out.addProperty("light", (Integer) args[4]);
                        out.addProperty("color", (Integer) args[6]);
                        var found = java.util.regex.Pattern.compile("textures/([a-z0-9_/]+)\\.png")
                            .matcher(String.valueOf(args[3]));
                        out.addProperty("texture", found.find() ? found.group(1) : String.valueOf(args[3]));
                    }
                    return null;
                });
            group.extractRenderState(null, camera, Oracle.f32(row, "partial")).submit(collector, null);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the real `BellRenderer.submit` hands the collector: the model with the bell swung for the
     * ticks and the side it was struck from, the light and the sprite.
     */
    static JsonObject bellPose(final JsonObject request) throws Exception {
        Oracle.vanillaRegistries();
        var renderer = (net.minecraft.client.renderer.blockentity.BellRenderer) UNSAFE.allocateInstance(
            net.minecraft.client.renderer.blockentity.BellRenderer.class);
        var modelField = net.minecraft.client.renderer.blockentity.BellRenderer.class.getDeclaredField("model");
        modelField.setAccessible(true);
        modelField.set(renderer, new net.minecraft.client.model.object.bell.BellModel(
            bakedLayer("minecraft:bell#main")));
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.blockentity.state.BellRenderState();
            state.ticks = Oracle.f32(row, "ticks");
            state.shakeDirection = row.get("shake").isJsonNull() ? null
                : net.minecraft.core.Direction.valueOf(row.get("shake").getAsString().toUpperCase());
            state.lightCoords = row.get("light").getAsInt();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitModel")) {
                        var modelState = (net.minecraft.client.model.object.bell.BellModel.State) args[1];
                        out.addProperty("ticks", Oracle.bits(modelState.ticks()));
                        out.addProperty("light", (Integer) args[3]);
                        out.addProperty("texture", ((net.minecraft.client.resources.model.sprite.SpriteId) args[6])
                            .texture().getPath());
                        out.add("quads", posedQuads("", (net.minecraft.client.model.Model<?>) args[0]).get("quads"));
                    }
                    return null;
                });
            renderer.submit(state, new com.mojang.blaze3d.vertex.PoseStack(), collector, null);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The bits of some floats, as an array. */
    static JsonArray floatBits(final float... values) {
        var array = new JsonArray();
        for (float value : values) {
            array.add(Oracle.bits(value));
        }
        return array;
    }

    /**
     * What the real `EnchantTableRenderer` hands the collector for an enchanting table whose book
     * is at some numbers, at a partial tick: what the render state takes from the entity, the pose,
     * the model's state and the model set up with it.
     */
    static JsonObject enchantTablePose(final JsonObject request) throws Exception {
        Oracle.vanillaRegistries();
        var renderer = (net.minecraft.client.renderer.blockentity.EnchantTableRenderer) UNSAFE.allocateInstance(
            net.minecraft.client.renderer.blockentity.EnchantTableRenderer.class);
        var modelField = net.minecraft.client.renderer.blockentity.EnchantTableRenderer.class.getDeclaredField("bookModel");
        modelField.setAccessible(true);
        modelField.set(renderer, new net.minecraft.client.model.object.book.BookModel(bakedLayer("minecraft:book#main")));
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var entity = new net.minecraft.world.level.block.entity.EnchantingTableBlockEntity(
                net.minecraft.core.BlockPos.ZERO, net.minecraft.world.level.block.Blocks.ENCHANTING_TABLE.defaultBlockState());
            entity.time = row.get("time").getAsInt();
            entity.flip = Oracle.f32(row, "flip");
            entity.oFlip = Oracle.f32(row, "o_flip");
            entity.open = Oracle.f32(row, "open");
            entity.oOpen = Oracle.f32(row, "o_open");
            entity.rot = Oracle.f32(row, "rot");
            entity.oRot = Oracle.f32(row, "o_rot");
            var state = renderer.createRenderState();
            renderer.extractRenderState(entity, state, Oracle.f32(row, "partial"), net.minecraft.world.phys.Vec3.ZERO, null);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("view", floatBits(state.flip, state.open, state.time, state.yRot));
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
                        out.add("matrix", described);
                        var book = (net.minecraft.client.model.object.book.BookModel.State) args[1];
                        out.add("book", floatBits(book.openness(), book.pageFlip1(), book.pageFlip2()));
                        out.addProperty("texture", ((net.minecraft.client.resources.model.sprite.SpriteId) args[6])
                            .texture().getPath());
                        @SuppressWarnings("unchecked")
                        var model = (net.minecraft.client.model.Model<Object>) args[0];
                        model.setupAnim(book);
                        out.add("quads", posedQuads("", model).get("quads"));
                    }
                    return null;
                });
            renderer.submit(state, new com.mojang.blaze3d.vertex.PoseStack(), collector, null);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the real `ShulkerBoxRenderer.submit` hands the collector: the pose, the light, the sprite
     * and the model with its lid set for the progress, for a colour, facing and progress.
     */
    static JsonObject shulkerBoxPose(final JsonObject request) throws Exception {
        Oracle.vanillaRegistries();
        var renderer = new net.minecraft.client.renderer.blockentity.ShulkerBoxRenderer(
            net.minecraft.client.model.geom.EntityModelSet.vanilla(), null);
        // The first use of each facing's matrix works its flags out, as the first box drawn does.
        for (var facing : net.minecraft.core.Direction.values()) {
            com.mojang.math.MatrixUtil.isPureTranslation(
                net.minecraft.client.renderer.blockentity.ShulkerBoxRenderer.modelTransform(facing).getMatrix());
        }
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.blockentity.state.ShulkerBoxRenderState();
            state.direction = net.minecraft.core.Direction.valueOf(row.get("facing").getAsString().toUpperCase());
            state.color = row.get("color").isJsonNull() ? null
                : net.minecraft.world.item.DyeColor.valueOf(row.get("color").getAsString().toUpperCase());
            state.progress = Oracle.f32(row, "progress");
            state.lightCoords = row.get("light").getAsInt();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
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
                        out.add("matrix", described);
                        out.addProperty("progress", Oracle.bits((Float) args[1]));
                        out.addProperty("light", (Integer) args[3]);
                        out.addProperty("texture", ((net.minecraft.client.resources.model.sprite.SpriteId) args[6])
                            .texture().getPath());
                        out.add("quads", posedQuads("", (net.minecraft.client.model.Model<?>) args[0]).get("quads"));
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "x"), Oracle.f32(row, "y"), Oracle.f32(row, "z"));
            renderer.submit(state, stack, collector, null);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
