import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityDisplayOracle extends EntityDataOracle {
    /**
     * Where the reference's own item frame renderer draws a frame's model and the item in it, from
     * the state its extraction makes (no map data here: the map's drawing is not asked).
     */
    static JsonObject itemFramePose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.entity.state.ItemFrameRenderState();
            state.direction = net.minecraft.core.Direction.byName(row.get("direction").getAsString());
            state.isInvisible = row.get("invisible").getAsBoolean();
            state.rotation = row.get("rotation").getAsInt();
            state.isGlowFrame = row.get("glow").getAsBoolean();
            if (!state.isInvisible) {
                var parts = state.frameModel.setupModel(new org.joml.Matrix4f(), false);
                parts.add((net.minecraft.client.renderer.block.dispatch.BlockStateModelPart)
                    java.lang.reflect.Proxy.newProxyInstance(EntityOracle.class.getClassLoader(),
                        new Class<?>[] {net.minecraft.client.renderer.block.dispatch.BlockStateModelPart.class},
                        (proxy, method, args) -> null));
            }
            if (row.get("item").getAsBoolean()) {
                state.item.newLayer();
            }
            state.lightCoords = row.get("light").getAsInt();
            if (row.get("map").getAsBoolean()) {
                state.mapId = new net.minecraft.world.level.saveddata.maps.MapId(1);
                state.mapRenderState.texture = net.minecraft.resources.Identifier.withDefaultNamespace("map/1");
            }
            var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.ItemFrameRenderer.class);
            // The map renderer only draws what the state holds.
            var mapRendererField = net.minecraft.client.renderer.entity.ItemFrameRenderer.class.getDeclaredField("mapRenderer");
            mapRendererField.setAccessible(true);
            mapRendererField.set(renderer, UNSAFE.allocateInstance(net.minecraft.client.renderer.MapRenderer.class));
            var poses = new JsonArray();
            var maps = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitBlockModel") || method.getName().equals("submitItem")) {
                        var stack = (com.mojang.blaze3d.vertex.PoseStack) args[0];
                        var matrix = new float[16];
                        stack.last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                    }
                    if (method.getName().equals("submitCustomGeometry")) {
                        describeGeometry(((com.mojang.blaze3d.vertex.PoseStack) args[0]).last(),
                            (net.minecraft.client.renderer.SubmitNodeCollector.CustomGeometryRenderer) args[2]).forEach(maps::add);
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            ((net.minecraft.client.renderer.entity.ItemFrameRenderer<?>) renderer).submit(state, stack, collector, null);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("poses", poses);
            out.add("map_vertices", maps);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the reference's own map renderer draws: the quad of the map, then each decoration (those a
     * frame shows when it is a frame's), as the vertices it adds through the pose.
     */
    static JsonObject mapRender(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.state.MapRenderState();
            state.texture = net.minecraft.resources.Identifier.withDefaultNamespace("map/1");
            var spriteFields = net.minecraft.client.renderer.texture.TextureAtlasSprite.class;
            for (var entry : row.getAsJsonArray("decorations")) {
                var decoration = entry.getAsJsonObject();
                var sprite = (net.minecraft.client.renderer.texture.TextureAtlasSprite) UNSAFE.allocateInstance(spriteFields);
                var names = new String[] {"u0", "v0", "u1", "v1"};
                for (int index = 0; index < 4; index++) {
                    var field = spriteFields.getDeclaredField(names[index]);
                    field.setAccessible(true);
                    field.set(sprite, Oracle.f32(decoration.getAsJsonObject("sprite"), names[index]));
                }
                var atlas = spriteFields.getDeclaredField("atlasLocation");
                atlas.setAccessible(true);
                atlas.set(sprite, net.minecraft.resources.Identifier.withDefaultNamespace("textures/atlas/map_decorations.png"));
                var drawn = new net.minecraft.client.renderer.state.MapRenderState.MapDecorationRenderState();
                drawn.atlasSprite = sprite;
                drawn.x = (byte) decoration.get("x").getAsInt();
                drawn.y = (byte) decoration.get("y").getAsInt();
                drawn.rot = (byte) decoration.get("rot").getAsInt();
                drawn.renderOnFrame = decoration.get("on_frame").getAsBoolean();
                state.decorations.add(drawn);
            }
            var vertices = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitCustomGeometry")) {
                        var texture = String.valueOf(args[1]).contains("map_decorations") ? "decorations" : "map";
                        for (var vertex : describeGeometry(((com.mojang.blaze3d.vertex.PoseStack) args[0]).last(),
                                (net.minecraft.client.renderer.SubmitNodeCollector.CustomGeometryRenderer) args[2])) {
                            vertices.add(texture + "," + vertex);
                        }
                    }
                    return null;
                });
            var renderer = (net.minecraft.client.renderer.MapRenderer) UNSAFE.allocateInstance(
                net.minecraft.client.renderer.MapRenderer.class);
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            renderer.render(state, stack, collector, row.get("only_frame").getAsBoolean(), row.get("light").getAsInt());
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("vertices", vertices);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Where the reference's own display renderers draw a block or an item display, from the render
     * state the entity makes: the pose (and its normal matrix) at the submit of the model or item.
     */
    static JsonObject displayPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            boolean item = row.get("kind").getAsString().equals("item");
            var parts = (java.util.function.BiFunction<String, Integer, float[]>) (key, count) -> {
                var values = new float[count];
                for (int index = 0; index < count; index++) {
                    values[index] = Float.intBitsToFloat((int) Long.parseLong(
                        row.getAsJsonArray(key).get(index).getAsString(), 16));
                }
                return values;
            };
            var translation = parts.apply("translation", 3);
            var scale = parts.apply("scale", 3);
            var left = parts.apply("left_rotation", 4);
            var right = parts.apply("right_rotation", 4);
            var transformation = new com.mojang.math.Transformation(
                new org.joml.Vector3f(translation[0], translation[1], translation[2]),
                new org.joml.Quaternionf(left[0], left[1], left[2], left[3]),
                new org.joml.Vector3f(scale[0], scale[1], scale[2]),
                new org.joml.Quaternionf(right[0], right[1], right[2], right[3]));
            var renderState = new net.minecraft.world.entity.Display.RenderState(
                net.minecraft.world.entity.Display.GenericInterpolator.constant(transformation),
                net.minecraft.world.entity.Display.BillboardConstraints.values()[row.get("billboard").getAsInt()],
                -1, net.minecraft.world.entity.Display.FloatInterpolator.constant(0.0F),
                net.minecraft.world.entity.Display.FloatInterpolator.constant(1.0F), -1);
            net.minecraft.client.renderer.entity.state.DisplayEntityRenderState state;
            if (item) {
                var itemState = new net.minecraft.client.renderer.entity.state.ItemDisplayEntityRenderState();
                itemState.item.newLayer();
                state = itemState;
            } else {
                var blockState = new net.minecraft.client.renderer.entity.state.BlockDisplayEntityRenderState();
                blockState.blockModel.setupModel(new org.joml.Matrix4f(), false).add(
                    (net.minecraft.client.renderer.block.dispatch.BlockStateModelPart)
                        java.lang.reflect.Proxy.newProxyInstance(EntityOracle.class.getClassLoader(),
                            new Class<?>[] {net.minecraft.client.renderer.block.dispatch.BlockStateModelPart.class},
                            (proxy, method, args) -> null));
                state = blockState;
            }
            state.renderState = renderState;
            state.interpolationProgress = 1.0F;
            state.entityYRot = Oracle.f32(row, "entity_y_rot");
            state.entityXRot = Oracle.f32(row, "entity_x_rot");
            state.cameraYRot = Oracle.f32(row, "camera_y_rot");
            state.cameraXRot = Oracle.f32(row, "camera_x_rot");
            var renderer = UNSAFE.allocateInstance(item
                ? net.minecraft.client.renderer.entity.DisplayRenderer.ItemDisplayRenderer.class
                : net.minecraft.client.renderer.entity.DisplayRenderer.BlockDisplayRenderer.class);
            var poses = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitBlockModel") || method.getName().equals("submitItem")) {
                        var stack = (com.mojang.blaze3d.vertex.PoseStack) args[0];
                        var matrix = new float[16];
                        stack.last().pose().get(matrix);
                        var normal = new float[9];
                        stack.last().normal().get(normal);
                        var described = new StringBuilder();
                        for (float value : matrix) {
                            described.append(Oracle.bits(value));
                        }
                        for (float value : normal) {
                            described.append(Oracle.bits(value));
                        }
                        poses.add(described.toString());
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            if (item) {
                ((net.minecraft.client.renderer.entity.DisplayRenderer.ItemDisplayRenderer) renderer).submit(
                    (net.minecraft.client.renderer.entity.state.ItemDisplayEntityRenderState) state, stack, collector, null);
            } else {
                ((net.minecraft.client.renderer.entity.DisplayRenderer.BlockDisplayRenderer) renderer).submit(
                    (net.minecraft.client.renderer.entity.state.BlockDisplayEntityRenderState) state, stack, collector, null);
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
     * The light the reference's display renderer reads for a display whose brightness the server
     * set: its block and sky levels, packed again. (What it leaves to the world is not asked.)
     */
    static JsonObject displayLight(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var renderStateField = net.minecraft.world.entity.Display.class.getDeclaredField("renderState");
        renderStateField.setAccessible(true);
        var base = net.minecraft.client.renderer.entity.DisplayRenderer.class;
        var blockLevel = base.getDeclaredMethod("getBlockLightLevel", net.minecraft.world.entity.Display.class,
            net.minecraft.core.BlockPos.class);
        var skyLevel = base.getDeclaredMethod("getSkyLightLevel", net.minecraft.world.entity.Display.class,
            net.minecraft.core.BlockPos.class);
        blockLevel.setAccessible(true);
        skyLevel.setAccessible(true);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var entity = (net.minecraft.world.entity.Display) UNSAFE.allocateInstance(
                net.minecraft.world.entity.Display.BlockDisplay.class);
            renderStateField.set(entity, new net.minecraft.world.entity.Display.RenderState(
                net.minecraft.world.entity.Display.GenericInterpolator.constant(com.mojang.math.Transformation.IDENTITY),
                net.minecraft.world.entity.Display.BillboardConstraints.FIXED, row.get("brightness").getAsInt(),
                net.minecraft.world.entity.Display.FloatInterpolator.constant(0.0F),
                net.minecraft.world.entity.Display.FloatInterpolator.constant(1.0F), -1));
            var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.DisplayRenderer.BlockDisplayRenderer.class);
            int block = (int) blockLevel.invoke(renderer, entity, net.minecraft.core.BlockPos.ZERO);
            int sky = (int) skyLevel.invoke(renderer, entity, net.minecraft.core.BlockPos.ZERO);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("light", net.minecraft.util.LightCoordsUtil.pack(block, sky));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * `ItemStack.hasFoil` of real stacks (every item's default, then stacks with a component patch
     * of enchantments and glint overrides set or removed over the item's own).
     */
    static JsonObject itemFoil(final JsonObject request) throws Exception {
        var registries = Oracle.vanillaRegistries();
        var items = net.minecraft.core.registries.BuiltInRegistries.ITEM;
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            if (row.has("all")) {
                var found = new JsonArray();
                for (var item : items) {
                    found.add(item.getDefaultInstance().hasFoil());
                }
                out.add("foil", found);
                rows.add(out);
                continue;
            }
            var item = items.getValue(net.minecraft.resources.Identifier.parse(row.get("item").getAsString()));
            var stack = item.getDefaultInstance();
            // The patch as the wire carries it, applied over the item's own components.
            var patch = row.get("patch").getAsString();
            var bytes = new byte[patch.length() / 2];
            for (int index = 0; index < bytes.length; index++) {
                bytes[index] = (byte) Integer.parseInt(patch.substring(index * 2, index * 2 + 2), 16);
            }
            var buffer = new net.minecraft.network.RegistryFriendlyByteBuf(
                io.netty.buffer.Unpooled.wrappedBuffer(bytes), registries);
            stack.applyComponents(net.minecraft.core.component.DataComponentPatch.STREAM_CODEC.decode(buffer));
            out.addProperty("foil", stack.hasFoil());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A `Minecraft` whose renderer holds a game render state of its options (the glint's speed, the text background). */
    static net.minecraft.client.renderer.state.GameRenderState installGameState() throws Exception {
        var minecraft = (net.minecraft.client.Minecraft) UNSAFE.allocateInstance(net.minecraft.client.Minecraft.class);
        var renderer = (net.minecraft.client.renderer.GameRenderer) UNSAFE.allocateInstance(
            net.minecraft.client.renderer.GameRenderer.class);
        var gameState = new net.minecraft.client.renderer.state.GameRenderState();
        var stateField = net.minecraft.client.renderer.GameRenderer.class.getDeclaredField("gameRenderState");
        stateField.setAccessible(true);
        stateField.set(renderer, gameState);
        var rendererField = net.minecraft.client.Minecraft.class.getDeclaredField("gameRenderer");
        rendererField.setAccessible(true);
        rendererField.set(minecraft, renderer);
        var instanceField = net.minecraft.client.Minecraft.class.getDeclaredField("instance");
        instanceField.setAccessible(true);
        instanceField.set(null, minecraft);
        return gameState;
    }

    /**
     * The vertices a custom geometry renderer adds, each as its position through the pose (bits),
     * its texture coordinate if it has one (bits), its colour and its light.
     */
    static java.util.List<String> describeGeometry(
            final com.mojang.blaze3d.vertex.PoseStack.Pose pose,
            final net.minecraft.client.renderer.SubmitNodeCollector.CustomGeometryRenderer renderer) {
        var described = new java.util.ArrayList<String>();
        var vertex = new float[3];
        var uv = new float[2];
        var hasUv = new boolean[1];
        var color = new int[1];
        java.lang.reflect.InvocationHandler handler = (proxy, call, callArgs) -> {
            if (call.isDefault()) {
                return java.lang.reflect.InvocationHandler.invokeDefault(proxy, call, callArgs);
            }
            switch (call.getName()) {
                case "addVertex" -> {
                    vertex[0] = (float) callArgs[0];
                    vertex[1] = (float) callArgs[1];
                    vertex[2] = (float) callArgs[2];
                    hasUv[0] = false;
                }
                case "setColor" -> {
                    if (callArgs.length == 1) {
                        color[0] = (int) callArgs[0];
                    }
                }
                case "setUv" -> {
                    uv[0] = (float) callArgs[0];
                    uv[1] = (float) callArgs[1];
                    hasUv[0] = true;
                }
                case "setUv2" -> {
                    int light = (int) callArgs[0] | ((int) callArgs[1] << 16);
                    described.add(Oracle.bits(vertex[0]) + Oracle.bits(vertex[1]) + Oracle.bits(vertex[2])
                        + (hasUv[0] ? "," + Oracle.bits(uv[0]) + Oracle.bits(uv[1]) : "") + "," + color[0] + "," + light);
                }
                default -> { }
            }
            return call.getReturnType() == com.mojang.blaze3d.vertex.VertexConsumer.class ? proxy : null;
        };
        var buffer = (com.mojang.blaze3d.vertex.VertexConsumer) java.lang.reflect.Proxy.newProxyInstance(
            EntityOracle.class.getClassLoader(), new Class<?>[] {com.mojang.blaze3d.vertex.VertexConsumer.class}, handler);
        renderer.render(pose, buffer);
        return described;
    }

    /**
     * What the reference's own text display renderer submits for lines of the given widths: the
     * pose (and the background's vertices through it) and each line's place and manner.
     */
    static JsonObject textDisplaySubmit(final JsonObject request) throws Exception {
        var gameState = installGameState();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            gameState.optionsRenderState.backgroundForChatOnly = true;
            var lines = new java.util.ArrayList<net.minecraft.world.entity.Display.TextDisplay.CachedLine>();
            int widest = 0;
            for (var width : row.getAsJsonArray("widths")) {
                lines.add(new net.minecraft.world.entity.Display.TextDisplay.CachedLine(
                    net.minecraft.util.FormattedCharSequence.forward("x", net.minecraft.network.chat.Style.EMPTY),
                    width.getAsInt()));
                widest = Math.max(widest, width.getAsInt());
            }
            var state = new net.minecraft.client.renderer.entity.state.TextDisplayEntityRenderState();
            state.renderState = new net.minecraft.world.entity.Display.RenderState(
                // What the entity's own data makes (the parts composed), not the identity constant.
                net.minecraft.world.entity.Display.GenericInterpolator.constant(new com.mojang.math.Transformation(
                    new org.joml.Vector3f(), new org.joml.Quaternionf(), new org.joml.Vector3f(1.0F, 1.0F, 1.0F),
                    new org.joml.Quaternionf())),
                net.minecraft.world.entity.Display.BillboardConstraints.FIXED, -1,
                net.minecraft.world.entity.Display.FloatInterpolator.constant(0.0F),
                net.minecraft.world.entity.Display.FloatInterpolator.constant(1.0F), -1);
            state.textRenderState = new net.minecraft.world.entity.Display.TextDisplay.TextRenderState(
                net.minecraft.network.chat.Component.empty(), 200,
                net.minecraft.world.entity.Display.IntInterpolator.constant(row.get("opacity").getAsInt()),
                net.minecraft.world.entity.Display.IntInterpolator.constant(row.get("background").getAsInt()),
                (byte) row.get("flags").getAsInt());
            state.cachedInfo = new net.minecraft.world.entity.Display.TextDisplay.CachedInfo(lines, widest);
            state.interpolationProgress = 1.0F;
            state.lightCoords = row.get("light").getAsInt();
            var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.DisplayRenderer.TextDisplayRenderer.class);
            // The line height is what the font says (a font of no glyphs: only its height is read).
            var font = UNSAFE.allocateInstance(net.minecraft.client.gui.Font.class);
            var lineHeight = net.minecraft.client.gui.Font.class.getDeclaredField("lineHeight");
            lineHeight.setAccessible(true);
            lineHeight.set(font, 9);
            var fontField = net.minecraft.client.renderer.entity.DisplayRenderer.TextDisplayRenderer.class.getDeclaredField("font");
            fontField.setAccessible(true);
            fontField.set(renderer, font);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var texts = new JsonArray();
            var background = new java.util.ArrayList<String>();
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
                    if (method.getName().equals("submitCustomGeometry")) {
                        var stack = (com.mojang.blaze3d.vertex.PoseStack) args[0];
                        var pose = new StringBuilder();
                        var matrix = new float[16];
                        stack.last().pose().get(matrix);
                        for (float value : matrix) {
                            pose.append(Oracle.bits(value));
                        }
                        out.addProperty("pose", pose.toString());
                        out.addProperty("see_through", String.valueOf(args[1]).contains("see_through"));
                        background.addAll(describeGeometry(stack.last(), (net.minecraft.client.renderer.SubmitNodeCollector.CustomGeometryRenderer) args[2]));
                    }
                    if (method.getName().equals("submitText")) {
                        var stack = (com.mojang.blaze3d.vertex.PoseStack) args[0];
                        if (!out.has("pose")) {
                            var pose = new StringBuilder();
                            var matrix = new float[16];
                            stack.last().pose().get(matrix);
                            for (float value : matrix) {
                                pose.append(Oracle.bits(value));
                            }
                            out.addProperty("pose", pose.toString());
                        }
                        out.addProperty("text_order", order[0]);
                        texts.add(Oracle.bits((float) args[1]) + "," + Oracle.bits((float) args[2]) + "," + args[4] + ","
                            + args[5] + "," + args[6] + "," + args[7] + "," + args[8] + "," + args[9]);
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            ((net.minecraft.client.renderer.entity.DisplayRenderer.TextDisplayRenderer) renderer).submit(state, stack, holder[0], null);
            var backgroundArray = new JsonArray();
            background.forEach(backgroundArray::add);
            out.add("background", backgroundArray);
            out.add("texts", texts);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** `GameRenderer.bobHurt`: the matrix the view is turned by when the camera's entity flinches or dies. */
    static JsonObject cameraHurt(final JsonObject request) throws Exception {
        var gameState = installGameState();
        var minecraft = net.minecraft.client.Minecraft.class.getDeclaredField("instance");
        minecraft.setAccessible(true);
        var renderer = (net.minecraft.client.renderer.GameRenderer)
            ((net.minecraft.client.Minecraft) minecraft.get(null)).gameRenderer;
        var bob = net.minecraft.client.renderer.GameRenderer.class.getDeclaredMethod("bobHurt",
            net.minecraft.client.renderer.state.level.CameraRenderState.class, com.mojang.blaze3d.vertex.PoseStack.class);
        bob.setAccessible(true);
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            gameState.optionsRenderState.damageTiltStrength = Oracle.f64(row, "tilt");
            var camera = new net.minecraft.client.renderer.state.level.CameraRenderState();
            var entity = camera.entityRenderState;
            entity.isLiving = true;
            entity.hurtTime = Oracle.f32(row, "hurt_time");
            entity.hurtDuration = row.get("hurt_duration").getAsInt();
            entity.hurtDir = Oracle.f32(row, "hurt_dir");
            entity.isDeadOrDying = row.get("dying").getAsBoolean();
            entity.deathTime = Oracle.f32(row, "death_time");
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            bob.invoke(renderer, camera, stack);
            var matrix = new float[16];
            stack.last().pose().get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("matrix", described);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * `GameRenderer.renderLevel`'s spinning effect: the lines that turn the projection, run on JOML's
     * own matrix. The tick that sums time and speed is the one `GameRenderer.tick` runs.
     */
    static JsonObject cameraSpin(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            float time = 0.0F;
            float speed = 0.0F;
            for (var tick : row.get("ticks").getAsJsonArray()) {
                float portalIntensity = Oracle.f32(tick.getAsJsonObject(), "portal");
                float nauseaIntensity = Oracle.f32(tick.getAsJsonObject(), "nausea");
                if (!(portalIntensity > 0.0F) && !(nauseaIntensity > 0.0F)) {
                    speed = 0.0F;
                } else {
                    speed = (portalIntensity * 20.0F + nauseaIntensity * 7.0F) / (portalIntensity + nauseaIntensity);
                    time = time + speed;
                }
            }
            float portalIntensity = Oracle.f32(row, "portal");
            float nauseaIntensity = Oracle.f32(row, "nausea");
            float screenEffectScale = Oracle.f32(row, "scale");
            float worldPartialTicks = Oracle.f32(row, "partial");
            float spinningEffectIntensity = Math.max(portalIntensity, nauseaIntensity) * (screenEffectScale * screenEffectScale);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var described = new JsonArray();
            var projectionMatrix = new org.joml.Matrix4f();
            if (spinningEffectIntensity > 0.0F) {
                float skew = 5.0F / (spinningEffectIntensity * spinningEffectIntensity + 5.0F) - spinningEffectIntensity * 0.04F;
                skew *= skew;
                var axis = new org.joml.Vector3f(0.0F, net.minecraft.util.Mth.SQRT_OF_TWO / 2.0F, net.minecraft.util.Mth.SQRT_OF_TWO / 2.0F);
                float angle = (time + worldPartialTicks * speed) * (float) (Math.PI / 180.0);
                projectionMatrix.rotate(angle, axis);
                projectionMatrix.scale(1.0F / skew, 1.0F, 1.0F);
                projectionMatrix.rotate(-angle, axis);
                var matrix = new float[16];
                projectionMatrix.get(matrix);
                for (float value : matrix) {
                    described.add(Oracle.bits(value));
                }
            }
            out.add("matrix", described);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * `ScreenEffectRenderer.renderItemActivationAnimation`'s pose: its lines on a real `PoseStack`
     * and `Axis`. The method itself needs the game's renderers, so the lines are transcribed.
     */
    static JsonObject itemActivationPose(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            int itemActivationTicks = row.get("ticks").getAsInt();
            float partialTicks = Oracle.f32(row, "partial");
            float aspectRatio = Oracle.f32(row, "aspect");
            float itemActivationOffX = Oracle.f32(row, "off_x");
            float itemActivationOffY = Oracle.f32(row, "off_y");
            var poseStack = new com.mojang.blaze3d.vertex.PoseStack();
            int tick = 40 - itemActivationTicks;
            float scale = (tick + partialTicks) / 40.0F;
            float ts = scale * scale;
            float tc = scale * ts;
            float smoothScale = 10.25F * tc * ts - 24.95F * ts * ts + 25.5F * tc - 13.8F * ts + 4.0F * scale;
            float piScale = smoothScale * (float) Math.PI;
            float offX = itemActivationOffX * 0.3F * aspectRatio;
            float offY = itemActivationOffY * 0.3F;
            poseStack.pushPose();
            poseStack.translate(offX * net.minecraft.util.Mth.abs(net.minecraft.util.Mth.sin(piScale * 2.0F)), offY * net.minecraft.util.Mth.abs(net.minecraft.util.Mth.sin(piScale * 2.0F)), -10.0F + 9.0F * net.minecraft.util.Mth.sin(piScale));
            poseStack.scale(0.8F, 0.8F, 0.8F);
            poseStack.mulPose(com.mojang.math.Axis.YP.rotationDegrees(900.0F * net.minecraft.util.Mth.abs(net.minecraft.util.Mth.sin(piScale))));
            poseStack.mulPose(com.mojang.math.Axis.XP.rotationDegrees(6.0F * net.minecraft.util.Mth.cos(scale * 8.0F)));
            poseStack.mulPose(com.mojang.math.Axis.ZP.rotationDegrees(6.0F * net.minecraft.util.Mth.cos(scale * 8.0F)));
            var matrix = new float[16];
            poseStack.last().pose().get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("matrix", described);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * `SkyRenderer.buildEndSky`: its loop and `switch` on real JOML matrices and the real
     * `BufferBuilder`, whose bytes are read back (the method itself ends in a GPU buffer).
     */
    static JsonObject endSkyBuffer(final JsonObject request) throws Exception {
        var format = com.mojang.blaze3d.vertex.DefaultVertexFormat.POSITION_TEX_COLOR;
        var rows = new JsonArray();
        try (var byteBufferBuilder = com.mojang.blaze3d.vertex.ByteBufferBuilder.exactlySized(24 * format.getVertexSize())) {
            var bufferBuilder = new com.mojang.blaze3d.vertex.BufferBuilder(
                byteBufferBuilder, com.mojang.blaze3d.PrimitiveTopology.QUADS, format);
            for (int i = 0; i < 6; i++) {
                var pose = new org.joml.Matrix4f();
                switch (i) {
                    case 1:
                        pose.rotationX((float) (Math.PI / 2));
                        break;
                    case 2:
                        pose.rotationX((float) (-Math.PI / 2));
                        break;
                    case 3:
                        pose.rotationX((float) Math.PI);
                        break;
                    case 4:
                        pose.rotationZ((float) (Math.PI / 2));
                        break;
                    case 5:
                        pose.rotationZ((float) (-Math.PI / 2));
                }
                bufferBuilder.addVertex(pose, -100.0F, -100.0F, -100.0F).setUv(0.0F, 0.0F).setColor(-14145496);
                bufferBuilder.addVertex(pose, -100.0F, -100.0F, 100.0F).setUv(0.0F, 16.0F).setColor(-14145496);
                bufferBuilder.addVertex(pose, 100.0F, -100.0F, 100.0F).setUv(16.0F, 16.0F).setColor(-14145496);
                bufferBuilder.addVertex(pose, 100.0F, -100.0F, -100.0F).setUv(16.0F, 0.0F).setColor(-14145496);
            }
            try (var meshData = bufferBuilder.buildOrThrow()) {
                var bytes = meshData.vertexBuffer().order(java.nio.ByteOrder.nativeOrder());
                int size = format.getVertexSize();
                for (int vertex = 0; vertex < 24; vertex++) {
                    int at = bytes.position() + vertex * size;
                    var row = new JsonObject();
                    var position = new JsonArray();
                    for (int k = 0; k < 3; k++) {
                        position.add(Oracle.bits(bytes.getFloat(at + 4 * k)));
                    }
                    row.add("position", position);
                    var uv = new JsonArray();
                    for (int k = 0; k < 2; k++) {
                        uv.add(Oracle.bits(bytes.getFloat(at + 12 + 4 * k)));
                    }
                    row.add("uv", uv);
                    var colour = new JsonArray();
                    for (int k = 0; k < 4; k++) {
                        colour.add(bytes.get(at + 20 + k) & 255);
                    }
                    row.add("colour", colour);
                    rows.add(row);
                }
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The real `TickRateManager` run through a script of calls, its state described after each. */
    static JsonObject tickRateScript(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var manager = new net.minecraft.world.TickRateManager();
            var states = new JsonArray();
            for (var step : row.get("steps").getAsJsonArray()) {
                var call = step.getAsJsonObject();
                switch (call.get("call").getAsString()) {
                    case "rate" -> manager.setTickRate(Oracle.f32(call, "value"));
                    case "frozen" -> manager.setFrozen(call.get("value").getAsInt() != 0);
                    case "steps" -> manager.setFrozenTicksToRun(call.get("value").getAsInt());
                    case "tick" -> manager.tick();
                    default -> throw new IllegalArgumentException(call.get("call").getAsString());
                }
                var state = new JsonObject();
                state.addProperty("runs", manager.runsNormally());
                state.addProperty("frozen", manager.isFrozen());
                state.addProperty("steps", manager.frozenTicksToRun());
                state.addProperty("rate", Oracle.bits(manager.tickrate()));
                states.add(state);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("states", states);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The real `EndFlashState` ticked through a script of clock times, described after each tick. */
    static JsonObject endFlashScript(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.EndFlashState();
            float partial = Oracle.f32(row, "partial");
            var states = new JsonArray();
            for (var clock : row.get("clocks").getAsJsonArray()) {
                state.tick(clock.getAsLong());
                var out = new JsonObject();
                out.addProperty("intensity", Oracle.bits(state.getIntensity(partial)));
                out.addProperty("intensity_now", Oracle.bits(state.getIntensity(1.0F)));
                out.addProperty("intensity_old", Oracle.bits(state.getIntensity(0.0F)));
                out.addProperty("x", Oracle.bits(state.getXAngle()));
                out.addProperty("y", Oracle.bits(state.getYAngle()));
                out.addProperty("started", state.flashStartedThisTick());
                states.add(out);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("states", states);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * `SkyRenderer.renderEndFlash`'s matrix: the pose stack's two turns, then the model view
     * stack's translate and scale, which the GPU multiplies the unit quad by (the method itself
     * draws, so its lines are transcribed).
     */
    static JsonObject endFlashPose(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            float xAngle = Oracle.f32(row, "x");
            float yAngle = Oracle.f32(row, "y");
            var poseStack = new com.mojang.blaze3d.vertex.PoseStack();
            poseStack.mulPose(com.mojang.math.Axis.YP.rotationDegrees(180.0F - yAngle));
            poseStack.mulPose(com.mojang.math.Axis.XP.rotationDegrees(-90.0F - xAngle));
            var modelViewStack = new org.joml.Matrix4fStack(4);
            modelViewStack.pushMatrix();
            modelViewStack.mul(poseStack.last().pose());
            modelViewStack.translate(0.0F, 100.0F, 0.0F);
            modelViewStack.scale(60.0F, 1.0F, 60.0F);
            var matrix = new float[16];
            new org.joml.Matrix4f(modelViewStack).get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("matrix", described);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    static JsonObject glintLayers(final JsonObject request) throws Exception {
        var gameState = installGameState();
        var renderType = request.has("render_type") && request.get("render_type").getAsString().equals("item")
            ? net.minecraft.client.renderer.rendertype.RenderTypes.glint()
            : net.minecraft.client.renderer.rendertype.RenderTypes.armorEntityGlint();
        var renderTypeState = net.minecraft.client.renderer.rendertype.RenderType.class.getDeclaredField("state");
        renderTypeState.setAccessible(true);
        var setup = renderTypeState.get(renderType);
        var pipelineField = setup.getClass().getDeclaredField("pipeline");
        pipelineField.setAccessible(true);
        var pipeline = (com.mojang.blaze3d.pipeline.RenderPipeline) pipelineField.get(setup);
        var transformField = setup.getClass().getDeclaredField("textureTransform");
        transformField.setAccessible(true);
        var transform = (net.minecraft.client.renderer.rendertype.TextureTransform) transformField.get(setup);
        var result = new JsonObject();
        result.addProperty("pipeline", pipeline.getLocation().toString());
        result.addProperty("flags", String.join(",", new java.util.TreeSet<>(pipeline.getShaderDefines().flags())));
        var text = new StringBuilder();
        new java.util.TreeMap<>(pipeline.getShaderDefines().values())
            .forEach((key, value) -> text.append(key).append('=').append(value).append(','));
        result.addProperty("values", text.toString());
        var blend = pipeline.getColorTargetState().blendFunction();
        result.addProperty("blend", blend.isEmpty() ? "NONE"
            : blend.get().equals(com.mojang.blaze3d.pipeline.BlendFunction.GLINT) ? "GLINT" : "OTHER");
        result.addProperty("depth_test", pipeline.getDepthStencilState().depthTest().name());
        result.addProperty("depth_write", pipeline.getDepthStencilState().writeDepth());
        result.addProperty("cull", pipeline.isCull());
        var found = java.util.regex.Pattern.compile("minecraft:(textures/[a-z_/]+\\.png)").matcher(String.valueOf(renderType));
        result.addProperty("texture", found.find() ? found.group(1) : String.valueOf(renderType));
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            long millis = row.get("millis").getAsLong();
            net.minecraft.util.Util.setTimeSource(() -> millis * 1000000L);
            gameState.optionsRenderState.glintSpeed = row.get("speed").getAsDouble();
            var matrix = new float[16];
            transform.createMatrix().get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("matrix", described);
            rows.add(out);
        }
        net.minecraft.util.Util.setTimeSource(System::nanoTime);
        result.add("rows", rows);
        return result;
    }

    /**
     * The `equippable` component of real stacks: the stack's patch (as the wire carries it) over the
     * item's own, as the slot's name and the equipment asset it names.
     */
    static JsonObject itemEquippable(final JsonObject request) throws Exception {
        var registries = Oracle.vanillaRegistries();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var item = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                net.minecraft.resources.Identifier.parse(row.get("item").getAsString()));
            var stack = item.getDefaultInstance();
            var patch = row.get("patch").getAsString();
            var bytes = new byte[patch.length() / 2];
            for (int index = 0; index < bytes.length; index++) {
                bytes[index] = (byte) Integer.parseInt(patch.substring(index * 2, index * 2 + 2), 16);
            }
            var buffer = new net.minecraft.network.RegistryFriendlyByteBuf(
                io.netty.buffer.Unpooled.wrappedBuffer(bytes), registries);
            stack.applyComponents(net.minecraft.core.component.DataComponentPatch.STREAM_CODEC.decode(buffer));
            var equippable = stack.get(net.minecraft.core.component.DataComponents.EQUIPPABLE);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("slot", equippable == null ? "none" : equippable.slot().getSerializedName());
            out.addProperty("asset", equippable == null || equippable.assetId().isEmpty() ? "none"
                : equippable.assetId().get().identifier().toString());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
