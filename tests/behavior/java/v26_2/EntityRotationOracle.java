import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityRotationOracle extends EntityProjectileOracle {
    /**
     * What the reference's own `setupRotations` leaves the pose stack at for a panda, a cat or a fox
     * from the state that drives it, and, for a panda, the matrix `PandaHoldsItemLayer` submits what
     * it holds with from an identity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject setupRotations(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String kind = row.get("kind").getAsString();
            Class<?> rendererClass;
            net.minecraft.client.renderer.entity.state.LivingEntityRenderState state;
            var poses = new JsonArray();
            if (kind.equals("panda")) {
                rendererClass = net.minecraft.client.renderer.entity.PandaRenderer.class;
                var panda = new net.minecraft.client.renderer.entity.state.PandaRenderState();
                panda.rollTime = Oracle.f32(row, "roll_time");
                panda.sitAmount = Oracle.f32(row, "sit_amount");
                panda.lieOnBackAmount = Oracle.f32(row, "lie_on_back_amount");
                panda.isScared = row.get("scared").getAsBoolean();
                panda.isSitting = row.get("sitting").getAsBoolean();
                panda.isEating = row.get("eating").getAsBoolean();
                state = panda;
            } else if (kind.equals("squid")) {
                rendererClass = net.minecraft.client.renderer.entity.SquidRenderer.class;
                var squid = new net.minecraft.client.renderer.entity.state.SquidRenderState();
                squid.xBodyRot = Oracle.f32(row, "x_body_rot");
                squid.zBodyRot = Oracle.f32(row, "z_body_rot");
                state = squid;
            } else if (kind.equals("shulker")) {
                rendererClass = net.minecraft.client.renderer.entity.ShulkerRenderer.class;
                var shulker = new net.minecraft.client.renderer.entity.state.ShulkerRenderState();
                shulker.attachFace = net.minecraft.core.Direction.from3DDataValue(row.get("attach_face").getAsInt());
                state = shulker;
            } else if (kind.equals("slime") || kind.equals("magma_cube") || kind.equals("sulfur_cube")) {
                rendererClass = kind.equals("slime")
                    ? net.minecraft.client.renderer.entity.SlimeRenderer.class
                    : kind.equals("sulfur_cube")
                        ? net.minecraft.client.renderer.entity.SulfurCubeRenderer.class
                        : net.minecraft.client.renderer.entity.MagmaCubeRenderer.class;
                var cube = kind.equals("sulfur_cube")
                    ? new net.minecraft.client.renderer.entity.state.SulfurCubeRenderState()
                    : new net.minecraft.client.renderer.entity.state.SlimeRenderState();
                if (cube instanceof net.minecraft.client.renderer.entity.state.SulfurCubeRenderState sulfur) {
                    sulfur.fuseRemainingTicks = Oracle.f32(row, "fuse_remaining");
                    sulfur.hasRedOverlay = row.has("red_overlay");
                    if (row.has("contained_block")) {
                        fillBlock(sulfur.containedBlock);
                    }
                }
                cube.size = row.get("size").getAsInt();
                cube.squish = Oracle.f32(row, "squish");
                state = cube;
            } else if (kind.equals("cod")) {
                rendererClass = net.minecraft.client.renderer.entity.CodRenderer.class;
                state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
            } else if (kind.equals("salmon")) {
                rendererClass = net.minecraft.client.renderer.entity.SalmonRenderer.class;
                state = new net.minecraft.client.renderer.entity.state.SalmonRenderState();
            } else if (kind.equals("tropical_fish")) {
                rendererClass = net.minecraft.client.renderer.entity.TropicalFishRenderer.class;
                state = new net.minecraft.client.renderer.entity.state.TropicalFishRenderState();
            } else if (kind.equals("pufferfish")) {
                rendererClass = net.minecraft.client.renderer.entity.PufferfishRenderer.class;
                state = new net.minecraft.client.renderer.entity.state.PufferfishRenderState();
            } else if (kind.equals("phantom")) {
                rendererClass = net.minecraft.client.renderer.entity.PhantomRenderer.class;
                var phantom = new net.minecraft.client.renderer.entity.state.PhantomRenderState();
                phantom.size = row.get("size").getAsInt();
                state = phantom;
            } else if (kind.equals("cat")) {
                rendererClass = net.minecraft.client.renderer.entity.CatRenderer.class;
                var cat = new net.minecraft.client.renderer.entity.state.CatRenderState();
                cat.lieDownAmount = Oracle.f32(row, "lie_down_amount");
                cat.isLyingOnTopOfSleepingPlayer = row.get("on_sleeping_player").getAsBoolean();
                state = cat;
            } else {
                rendererClass = net.minecraft.client.renderer.entity.FoxRenderer.class;
                var fox = new net.minecraft.client.renderer.entity.state.FoxRenderState();
                fox.isPouncing = row.get("pouncing").getAsBoolean();
                fox.isFaceplanted = row.get("faceplanted").getAsBoolean();
                state = fox;
            }
            state.isBaby = row.get("baby").getAsBoolean();
            state.xRot = Oracle.f32(row, "x_rot");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isInWater = row.has("in_water") && row.get("in_water").getAsBoolean();
            if (row.has("death_time")) {
                state.deathTime = Oracle.f32(row, "death_time");
            }
            var renderer = UNSAFE.allocateInstance(rendererClass);
            boolean cubeMob = kind.equals("slime") || kind.equals("magma_cube") || kind.equals("sulfur_cube");
            // The cube mobs' renderers take the common turns of `LivingEntityRenderer`.
            var method = Reflect.method((cubeMob ? net.minecraft.client.renderer.entity.LivingEntityRenderer.class : rendererClass), "setupRotations",
                    cubeMob ? net.minecraft.client.renderer.entity.state.LivingEntityRenderState.class : state.getClass(),
                    com.mojang.blaze3d.vertex.PoseStack.class, float.class, float.class);
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            if (row.has("feet_x")) {
                // What `submit` does before `setupRotations`: to the entity's place, then its scale.
                stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
                float scale = Oracle.f32(row, "scale");
                stack.scale(scale, scale, scale);
            }
            method.invoke(renderer, state, stack, Oracle.f32(row, "body_rot"), 1.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var matrix = new float[16];
            stack.last().pose().get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            out.add("pose", described);
            if (kind.equals("phantom") || cubeMob) {
                // What `submit` does after `setupRotations`: the flip, the renderer's own scale
                // and the model's lift. (The magma cube takes its scale from the cube mobs' base.)
                stack.scale(-1.0F, -1.0F, 1.0F);
                var hook = Reflect.method((kind.equals("magma_cube")
                        ? net.minecraft.client.renderer.entity.AbstractCubeMobRenderer.class : rendererClass), "scale", cubeMob && !kind.equals("sulfur_cube")
                        ? net.minecraft.client.renderer.entity.state.SlimeRenderState.class : state.getClass(),
                        com.mojang.blaze3d.vertex.PoseStack.class);
                hook.invoke(renderer, state, stack);
                stack.translate(0.0F, -1.501F, 0.0F);
                var full = new float[16];
                stack.last().pose().get(full);
                var fullDescribed = new JsonArray();
                for (float value : full) {
                    fullDescribed.add(Oracle.bits(value));
                }
                out.add("full_pose", fullDescribed);
            }
            var blockOverlay = new JsonArray();
            if (row.has("contained_block")) {
                // `SulfurCubeInnerLayer` draws the block it holds from the pose the renderer left.
                var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                    EntityOracle.class.getClassLoader(),
                    new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                    (proxy, method2, args) -> {
                        if (method2.getName().equals("submitBlockModel")) {
                            var layerMatrix = new float[16];
                            ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(layerMatrix);
                            var block = new JsonArray();
                            for (float value : layerMatrix) {
                                block.add(Oracle.bits(value));
                            }
                            poses.add(block);
                            blockOverlay.add((Integer) args[5]);
                        }
                        return null;
                    });
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> null;
                Oracle.vanillaRegistries();
                new net.minecraft.client.renderer.entity.layers.SulfurCubeInnerLayer(
                    parent, new net.minecraft.client.model.geom.EntityModelSet(
                        net.minecraft.client.model.geom.LayerDefinitions.createRoots()))
                    .submit(stack, collector, 0, (net.minecraft.client.renderer.entity.state.SulfurCubeRenderState) state, 0.0F, 0.0F);
            }
            if (kind.equals("panda")) {
                var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                    EntityOracle.class.getClassLoader(),
                    new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                    (proxy, method2, args) -> {
                        if (method2.getName().equals("submitItem")) {
                            var layerMatrix = new float[16];
                            ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(layerMatrix);
                            var item = new JsonArray();
                            for (float value : layerMatrix) {
                                item.add(Oracle.bits(value));
                            }
                            poses.add(item);
                        }
                        return null;
                    });
                var pandaState = (net.minecraft.client.renderer.entity.state.PandaRenderState) state;
                pandaState.heldItem.newLayer();
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> null;
                new net.minecraft.client.renderer.entity.layers.PandaHoldsItemLayer(parent)
                    .submit(new com.mojang.blaze3d.vertex.PoseStack(), collector, 0, pandaState, 0.0F, 0.0F);
            }
            out.add("poses", poses);
            if (!blockOverlay.isEmpty()) {
                out.add("block_overlay", blockOverlay);
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the reference's own `setupRotations` leaves the pose stack at for a mob that may shake: a
     * cow (frozen through is the only reason), a hoglin (converting too) and a strider (suffocating
     * too), with the dimension's `piglins_zombify` read from the registry's own dimension type.
     */
    static JsonObject shakingRotations(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var dimensionTypes = Oracle.vanillaRegistries()
            .lookupOrThrow(net.minecraft.core.registries.Registries.DIMENSION_TYPE);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String kind = row.get("kind").getAsString();
            boolean fullyFrozen = row.get("ticks_frozen").getAsInt() >= 140;
            Class<?> rendererClass;
            net.minecraft.client.renderer.entity.state.LivingEntityRenderState state;
            if (kind.equals("hoglin")) {
                rendererClass = net.minecraft.client.renderer.entity.HoglinRenderer.class;
                var hoglin = new net.minecraft.client.renderer.entity.state.HoglinRenderState();
                var type = dimensionTypes.getOrThrow(net.minecraft.resources.ResourceKey.create(
                    net.minecraft.core.registries.Registries.DIMENSION_TYPE,
                    net.minecraft.resources.Identifier.parse(row.get("dimension").getAsString()))).value();
                boolean zombify = type.attributes().applyModifier(
                    net.minecraft.world.attribute.EnvironmentAttributes.PIGLINS_ZOMBIFY, true);
                // `Hoglin.isConverting`, from the data it reads.
                hoglin.isConverting = !row.get("immune").getAsBoolean() && !row.get("no_ai").getAsBoolean()
                    && zombify;
                state = hoglin;
            } else if (kind.equals("strider")) {
                rendererClass = net.minecraft.client.renderer.entity.StriderRenderer.class;
                var strider = new net.minecraft.client.renderer.entity.state.StriderRenderState();
                strider.isSuffocating = row.get("suffocating").getAsBoolean();
                state = strider;
            } else {
                rendererClass = net.minecraft.client.renderer.entity.CowRenderer.class;
                state = new net.minecraft.client.renderer.entity.state.CowRenderState();
            }
            state.isFullyFrozen = fullyFrozen;
            state.ageInTicks = (float) row.get("ticks").getAsInt() + Oracle.f32(row, "partial");
            var renderer = UNSAFE.allocateInstance(rendererClass);
            var method = Reflect.method(net.minecraft.client.renderer.entity.LivingEntityRenderer.class, 
                "setupRotations", net.minecraft.client.renderer.entity.state.LivingEntityRenderState.class,
                com.mojang.blaze3d.vertex.PoseStack.class, float.class, float.class);
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            method.invoke(renderer, state, stack, Oracle.f32(row, "body_rot"), 1.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var matrix = new float[16];
            stack.last().pose().get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            out.add("pose", described);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
