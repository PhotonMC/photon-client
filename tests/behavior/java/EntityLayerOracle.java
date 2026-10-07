import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityLayerOracle extends EntityHumanoidOracle {
    /**
     * The pose each layer that draws a block on an entity puts the block at, from the reference's
     * own layer classes: the matrix the block is submitted with, from an identity entity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject entityBlockLayers(final JsonObject request) throws Exception {
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
    static JsonObject eyesLayers(final JsonObject request) throws Exception {
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

    /**
     * What the charged creeper's swirl layer submits, from the reference's own `CreeperPowerLayer`:
     * nothing unless it is charged; then the draw order, the texture, the pipeline's state, the
     * texture offset, the colour it is multiplied by, the light and the overlay.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject swirlLayers(final JsonObject request) throws Exception {
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
}
