import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityProjectileOracle extends EntityPoseOracle {
    /**
     * Where the reference's own thrown item renderer, and its firework rocket's, draw an item from
     * the camera it is seen by: the pose each leaves the item's single layer at.
     */
    static JsonObject thrownItemPose(final JsonObject request) throws Exception {
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

    /** The quad the reference's own dragon fireball renderer submits, seen by a camera. */
    static JsonObject dragonFireballQuad(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var camera = new net.minecraft.client.renderer.state.level.CameraRenderState();
            float yaw = Oracle.f32(row, "camera_y_rot");
            float pitch = Oracle.f32(row, "camera_x_rot");
            // `Camera.setRotation`.
            camera.orientation.rotationYXZ((float) Math.PI - yaw * (float) (Math.PI / 180.0),
                -pitch * (float) (Math.PI / 180.0), 0.0F);
            var vertices = new JsonArray();
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
            var state = new net.minecraft.client.renderer.entity.state.EntityRenderState();
            state.lightCoords = row.get("light").getAsInt();
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.DragonFireballRenderer.class);
            ((net.minecraft.client.renderer.entity.DragonFireballRenderer) renderer).submit(state, stack, collector, camera);
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
     * Where the reference's own shulker bullet, llama spit and evoker fangs renderers draw their
     * models from: the pose each submission leaves (none while fangs have not begun to bite).
     */
    static JsonObject smallProjectilePose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
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
            switch (row.get("kind").getAsString()) {
                case "shulker_bullet" -> {
                    var state = new net.minecraft.client.renderer.entity.state.ShulkerBulletRenderState();
                    state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                    var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.ShulkerBulletRenderer.class);
                    ((net.minecraft.client.renderer.entity.ShulkerBulletRenderer) renderer).submit(state, stack, collector, null);
                }
                case "leash_knot" -> {
                    var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.LeashKnotRenderer.class);
                    ((net.minecraft.client.renderer.entity.LeashKnotRenderer) renderer).submit(
                        new net.minecraft.client.renderer.entity.state.EntityRenderState(), stack, collector, null);
                }
                case "llama_spit" -> {
                    var state = new net.minecraft.client.renderer.entity.state.LlamaSpitRenderState();
                    state.yRot = Oracle.f32(row, "y_rot");
                    state.xRot = Oracle.f32(row, "x_rot");
                    var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.LlamaSpitRenderer.class);
                    ((net.minecraft.client.renderer.entity.LlamaSpitRenderer) renderer).submit(state, stack, collector, null);
                }
                default -> {
                    var state = new net.minecraft.client.renderer.entity.state.EvokerFangsRenderState();
                    state.yRot = Oracle.f32(row, "y_rot");
                    state.biteProgress = Oracle.f32(row, "bite_progress");
                    var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.EvokerFangsRenderer.class);
                    ((net.minecraft.client.renderer.entity.EvokerFangsRenderer) renderer).submit(state, stack, collector, null);
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
     * A leash's strip as the reference's own `LeashFeatureRenderer` builds it: each vertex from its
     * own `addVertexPair`, out along one edge and back along the other, from a leash state.
     */
    static JsonObject leashStrip(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var pair = net.minecraft.client.renderer.feature.LeashFeatureRenderer.class.getDeclaredMethod("addVertexPair",
            com.mojang.blaze3d.vertex.VertexConsumer.class, org.joml.Matrix4fc.class,
            float.class, float.class, float.class, float.class, float.class, float.class, int.class,
            boolean.class, net.minecraft.client.renderer.entity.state.EntityRenderState.LeashState.class);
        pair.setAccessible(true);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.entity.state.EntityRenderState.LeashState();
            state.offset = vec3(row.getAsJsonArray("offset"));
            state.start = vec3(row.getAsJsonArray("start"));
            state.end = vec3(row.getAsJsonArray("end"));
            state.startBlockLight = row.get("start_block").getAsInt();
            state.endBlockLight = row.get("end_block").getAsInt();
            state.startSkyLight = row.get("start_sky").getAsInt();
            state.endSkyLight = row.get("end_sky").getAsInt();
            state.slack = row.get("slack").getAsBoolean();
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            // `LeashFeatureRenderer.prepare`, with the vertex consumer in hand.
            var pose = new org.joml.Matrix4f(stack.last().pose());
            float dx = (float) (state.end.x - state.start.x);
            float dy = (float) (state.end.y - state.start.y);
            float dz = (float) (state.end.z - state.start.z);
            float offsetFactor = net.minecraft.util.Mth.invSqrt(dx * dx + dz * dz) * 0.05F / 2.0F;
            float dxOff = dz * offsetFactor;
            float dzOff = dx * offsetFactor;
            pose.translate((float) state.offset.x, (float) state.offset.y, (float) state.offset.z);
            var strip = new JsonArray();
            var capture = new BeamCapture(strip);
            for (int k = 0; k <= 24; k++) {
                pair.invoke(null, capture, pose, dx, dy, dz, 0.05F, dxOff, dzOff, k, false, state);
            }
            for (int k = 24; k >= 0; k--) {
                pair.invoke(null, capture, pose, dx, dy, dz, 0.0F, dxOff, dzOff, k, true, state);
            }
            capture.flush();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("strip", strip);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Where the reference's own arrow renderer draws an arrow from the turns it was sent. */
    static JsonObject arrowPose(final JsonObject request) throws Exception {
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
     * The quads the reference's screen effects (`ScreenEffectRenderer`) hand the vertex consumer: the block the eyes
     * are in, the fire and the water, from stand-in sprite corners and the player's look.
     */
    static JsonObject screenEffectQuads(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var renderer = net.minecraft.client.renderer.ScreenEffectRenderer.class;
        var spriteClass = net.minecraft.client.renderer.texture.TextureAtlasSprite.class;
        var stackClass = com.mojang.blaze3d.vertex.PoseStack.class;
        var collectorClass = net.minecraft.client.renderer.SubmitNodeCollector.class;
        var fire = renderer.getDeclaredMethod("submitFire", stackClass, collectorClass, spriteClass);
        var block = renderer.getDeclaredMethod("submitBlockSprite", spriteClass, stackClass, collectorClass, int.class);
        var quad = renderer.getDeclaredMethod("buildQuad", com.mojang.blaze3d.vertex.VertexConsumer.class,
            org.joml.Matrix4f.class, float.class, float.class, float.class, float.class, float.class, float.class,
            float.class, float.class, float.class, int.class);
        fire.setAccessible(true);
        block.setAccessible(true);
        quad.setAccessible(true);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var vertices = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {collectorClass},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitCustomGeometry")) {
                        var capture = new BeamCapture(vertices);
                        ((net.minecraft.client.renderer.SubmitNodeCollector.CustomGeometryRenderer) args[2])
                            .render(((com.mojang.blaze3d.vertex.PoseStack) args[0]).last(), capture);
                        capture.flush();
                    }
                    return null;
                });
            var sprite = (net.minecraft.client.renderer.texture.TextureAtlasSprite) UNSAFE.allocateInstance(spriteClass);
            for (var corner : new String[] {"u0", "u1", "v0", "v1"}) {
                var field = spriteClass.getDeclaredField(corner);
                field.setAccessible(true);
                if (row.has(corner)) {
                    field.setFloat(sprite, Oracle.f32(row, corner));
                }
            }
            var atlas = spriteClass.getDeclaredField("atlasLocation");
            atlas.setAccessible(true);
            atlas.set(sprite, net.minecraft.resources.Identifier.withDefaultNamespace("textures/atlas/blocks.png"));
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            switch (row.get("kind").getAsString()) {
                case "fire" -> fire.invoke(null, stack, collector, sprite);
                case "block" -> block.invoke(null, sprite, stack, collector, row.get("color").getAsInt());
                case "water" -> {
                    // `submitWater`, with the brightness and the look given.
                    float brightness = Oracle.f32(row, "brightness");
                    int color = net.minecraft.util.ARGB.colorFromFloat(0.1F, brightness, brightness, brightness);
                    float u0 = -Oracle.f32(row, "y_rot") / 64.0F;
                    float v0 = Oracle.f32(row, "x_rot") / 64.0F;
                    var capture = new BeamCapture(vertices);
                    quad.invoke(null, capture, stack.last().pose(), -1.0F, -1.0F, 1.0F, 1.0F, -0.5F,
                        u0 + 4.0F, v0 + 4.0F, u0, v0, color);
                    capture.flush();
                }
                default -> throw new IllegalArgumentException(row.get("kind").getAsString());
            }
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
     * The vertices the reference's own painting renderer hands the vertex consumer for a size, a facing and
     * the light of each block, with stand-in sprites (their own corners, to see which is used where).
     */
    static JsonObject paintingGeometry(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var render = net.minecraft.client.renderer.entity.PaintingRenderer.class.getDeclaredMethod("renderPainting",
            com.mojang.blaze3d.vertex.PoseStack.class, net.minecraft.client.renderer.SubmitNodeCollector.class,
            net.minecraft.client.renderer.rendertype.RenderType.class, int[].class, int.class, int.class,
            net.minecraft.client.renderer.texture.TextureAtlasSprite.class,
            net.minecraft.client.renderer.texture.TextureAtlasSprite.class);
        render.setAccessible(true);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var vertices = new JsonArray();
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
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            // `PaintingRenderer.submit`: turned to the way it hangs.
            stack.mulPose(com.mojang.math.Axis.YP.rotationDegrees(180 - row.get("direction_2d").getAsInt() * 90));
            var lights = row.getAsJsonArray("lights");
            var lightMap = new int[lights.size()];
            for (int i = 0; i < lightMap.length; i++) {
                lightMap[i] = lights.get(i).getAsInt();
            }
            var sprites = new net.minecraft.client.renderer.texture.TextureAtlasSprite[2];
            var names = new String[] {"front", "back"};
            for (int s = 0; s < 2; s++) {
                var sprite = (net.minecraft.client.renderer.texture.TextureAtlasSprite)
                    UNSAFE.allocateInstance(net.minecraft.client.renderer.texture.TextureAtlasSprite.class);
                for (var corner : new String[] {"u0", "u1", "v0", "v1"}) {
                    var field = net.minecraft.client.renderer.texture.TextureAtlasSprite.class.getDeclaredField(corner);
                    field.setAccessible(true);
                    field.setFloat(sprite, Oracle.f32(row, names[s] + "_" + corner));
                }
                sprites[s] = sprite;
            }
            var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.PaintingRenderer.class);
            render.invoke(renderer, stack, collector, null, lightMap, row.get("width").getAsInt(),
                row.get("height").getAsInt(), sprites[0], sprites[1]);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("vertices", vertices);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The vertices the reference's own lightning renderer hands the vertex consumer for a bolt's seed. */
    static JsonObject lightningBolt(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var vertices = new JsonArray();
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
            var state = new net.minecraft.client.renderer.entity.state.LightningBoltRenderState();
            state.seed = Long.parseLong(row.get("seed").getAsString());
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.LightningBoltRenderer.class);
            ((net.minecraft.client.renderer.entity.LightningBoltRenderer) renderer).submit(state, stack, collector, null);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("vertices", vertices);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Where the reference's own tnt renderer draws the block of a primed tnt from the fuse it has left. */
    static JsonObject primedTntPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.entity.state.TntRenderState();
            state.fuseRemainingInTicks = Oracle.f32(row, "fuse");
            var parts = state.blockState.setupModel(new org.joml.Matrix4f(), false);
            parts.add((net.minecraft.client.renderer.block.dispatch.BlockStateModelPart)
                java.lang.reflect.Proxy.newProxyInstance(EntityOracle.class.getClassLoader(),
                    new Class<?>[] {net.minecraft.client.renderer.block.dispatch.BlockStateModelPart.class},
                    (proxy, method, args) -> null));
            var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.TntRenderer.class);
            var poses = new JsonArray();
            var overlays = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitBlockModel")) {
                        var matrix = new float[16];
                        ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                        overlays.add((int) args[5]);
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            ((net.minecraft.client.renderer.entity.TntRenderer) renderer).submit(state, stack, collector, null);
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
}
