import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityAquaticOracle extends EntitySupport {
    /** The squid's models posed by the reference's own `SquidModel.setupAnim`, drawn. */
    static JsonObject squidModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.squid.BabySquidModel(root)
                : new net.minecraft.client.model.animal.squid.SquidModel(root);
            var state = new net.minecraft.client.renderer.entity.state.SquidRenderState();
            state.tentacleAngle = Oracle.f32(row, "tentacle_angle");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The dolphin's and the turtle's models posed by the reference's own `setupAnim`, drawn: the
     * dolphin from the head's turn and pitch, the age and whether it moves; the turtle from the
     * walk, whether it is on land, laying an egg, and carrying one.
     */
    static JsonObject seaAnimalModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.EntityModel<?> model;
            if (id.startsWith("minecraft:dolphin")) {
                var state = new net.minecraft.client.renderer.entity.state.DolphinRenderState();
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.isMoving = row.get("is_moving").getAsBoolean();
                var dolphin = id.contains("_baby")
                    ? new net.minecraft.client.model.animal.dolphin.BabyDolphinModel(root)
                    : new net.minecraft.client.model.animal.dolphin.DolphinModel(root);
                dolphin.setupAnim(state);
                model = dolphin;
            } else {
                var state = new net.minecraft.client.renderer.entity.state.TurtleRenderState();
                state.yRot = Oracle.f32(row, "head_y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.isOnLand = row.get("on_land").getAsBoolean();
                state.isLayingEgg = row.get("laying_egg").getAsBoolean();
                state.hasEgg = row.get("has_egg").getAsBoolean();
                net.minecraft.client.model.animal.turtle.TurtleModel turtle = id.contains("_baby")
                    ? new net.minecraft.client.model.animal.turtle.BabyTurtleModel(root)
                    : new net.minecraft.client.model.animal.turtle.AdultTurtleModel(root);
                turtle.setupAnim(state);
                model = turtle;
            }
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The axolotl's models posed by the reference's own `setupAnim`, drawn: an adult from its blend
     * factors, a baby from the animation states it runs (each started at the tick the row names).
     */
    static JsonObject axolotlModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var state = new net.minecraft.client.renderer.entity.state.AxolotlRenderState();
            state.yRot = Oracle.f32(row, "y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.playingDeadFactor = Oracle.f32(row, "playing_dead");
            state.movingFactor = Oracle.f32(row, "moving");
            state.inWaterFactor = Oracle.f32(row, "in_water");
            state.onGroundFactor = Oracle.f32(row, "on_ground");
            var animations = new Object[][] {
                {"swim", state.swimAnimation}, {"walk", state.walkAnimationState},
                {"idle_under_water", state.idleUnderWaterAnimationState},
                {"idle_under_water_on_ground", state.idleUnderWaterOnGroundAnimationState},
                {"idle_on_ground", state.idleOnGroundAnimationState}, {"play_dead", state.playDeadAnimationState},
            };
            for (var animation : animations) {
                if (row.has((String) animation[0])) {
                    ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                }
            }
            net.minecraft.client.model.EntityModel<net.minecraft.client.renderer.entity.state.AxolotlRenderState> model =
                id.contains("_baby")
                    ? new net.minecraft.client.model.animal.axolotl.BabyAxolotlModel(root)
                    : new net.minecraft.client.model.animal.axolotl.AdultAxolotlModel(root);
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The frog's and the tadpole's models posed by the reference's own `setupAnim`, drawn: a frog
     * from the animation states it runs (each started at the tick the row names) and whether it
     * swims, a tadpole from its age and whether it is in water.
     */
    static JsonObject frogModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            if (id.contains("tadpole")) {
                var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.isInWater = row.get("in_water").getAsBoolean();
                var model = new net.minecraft.client.model.animal.frog.TadpoleModel(root);
                model.setupAnim(state);
                rows.add(posedQuads(row.get("label").getAsString(), model));
                continue;
            }
            var state = new net.minecraft.client.renderer.entity.state.FrogRenderState();
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isSwimming = row.get("swimming").getAsBoolean();
            var animations = new Object[][] {
                {"jump", state.jumpAnimationState}, {"croak", state.croakAnimationState},
                {"tongue", state.tongueAnimationState}, {"swim_idle", state.swimIdleAnimationState},
            };
            for (var animation : animations) {
                if (row.has((String) animation[0])) {
                    ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                }
            }
            var model = new net.minecraft.client.model.animal.frog.FrogModel(root);
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A fishing bobber as the reference's own renderer submits it: the quad (the first custom geometry
     * its `submit` hands over, before it asks the running game for the line width) and the line's
     * vertices from its own `stringVertex`, through the line's sixteen steps.
     */
    static JsonObject fishingHook(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var stringVertex = Reflect.method(net.minecraft.client.renderer.entity.FishingHookRenderer.class, "stringVertex",
            float.class, float.class, float.class, com.mojang.blaze3d.vertex.VertexConsumer.class,
            com.mojang.blaze3d.vertex.PoseStack.Pose.class, float.class, float.class, float.class);
        var fraction = Reflect.method(net.minecraft.client.renderer.entity.FishingHookRenderer.class, "fraction", int.class, int.class);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var camera = new net.minecraft.client.renderer.state.level.CameraRenderState();
            float yaw = Oracle.f32(row, "camera_y_rot");
            float pitch = Oracle.f32(row, "camera_x_rot");
            // `Camera.setRotation`.
            camera.orientation.rotationYXZ((float) Math.PI - yaw * (float) (Math.PI / 180.0),
                -pitch * (float) (Math.PI / 180.0), 0.0F);
            var quad = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitCustomGeometry") && quad.isEmpty()) {
                        var capture = new BeamCapture(quad);
                        ((net.minecraft.client.renderer.SubmitNodeCollector.CustomGeometryRenderer) args[2])
                            .render(((com.mojang.blaze3d.vertex.PoseStack) args[0]).last(), capture);
                        capture.flush();
                    }
                    return null;
                });
            var state = new net.minecraft.client.renderer.entity.state.FishingHookRenderState();
            state.lightCoords = row.get("light").getAsInt();
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.FishingHookRenderer.class);
            try {
                ((net.minecraft.client.renderer.entity.FishingHookRenderer) renderer).submit(state, stack, collector, camera);
            } catch (NullPointerException expected) {
                // The line width is the running game's: the quad has been submitted by then.
            }
            var string = new JsonArray();
            var capture = new BeamCapture(string);
            float xa = Oracle.f32(row, "offset_x");
            float ya = Oracle.f32(row, "offset_y");
            float za = Oracle.f32(row, "offset_z");
            var linePose = new com.mojang.blaze3d.vertex.PoseStack();
            linePose.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            for (int i = 0; i < 16; i++) {
                float a0 = (float) fraction.invoke(null, i, 16);
                float a1 = (float) fraction.invoke(null, i + 1, 16);
                stringVertex.invoke(null, xa, ya, za, capture, linePose.last(), a0, a1, 1.0F);
                stringVertex.invoke(null, xa, ya, za, capture, linePose.last(), a1, a0, 1.0F);
            }
            capture.flush();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("quad", quad);
            out.add("string", string);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The guardian models (grown, elder) posed by the reference's own `setupAnim`, drawn, from the
     * look, the spikes' and the tail's animation and (optionally) what the eye follows.
     */
    static JsonObject guardianModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var root = bakedLayer(row.get("layer").getAsString());
            var state = new net.minecraft.client.renderer.entity.state.GuardianRenderState();
            state.yRot = Oracle.f32(row, "y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.spikesAnimation = Oracle.f32(row, "spikes");
            state.tailAnimation = Oracle.f32(row, "tail");
            if (row.has("eye")) {
                var eye = row.getAsJsonObject("eye");
                state.eyePosition = vec3(eye.getAsJsonArray("eye_position"));
                state.lookDirection = vec3(eye.getAsJsonArray("look_direction"));
                state.lookAtPosition = vec3(eye.getAsJsonArray("look_at_position"));
            }
            var model = new net.minecraft.client.model.monster.guardian.GuardianModel(root);
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A guardian's beam as the reference's own `renderBeam` submits it from the pose the entity's
     * eye leaves: every vertex it hands the vertex consumer (placed, with its texture coordinate,
     * colour, light and normal).
     */
    static JsonObject guardianBeam(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var render = Reflect.method(net.minecraft.client.renderer.entity.GuardianRenderer.class, "renderBeam",
            com.mojang.blaze3d.vertex.PoseStack.class, net.minecraft.client.renderer.SubmitNodeCollector.class,
            net.minecraft.world.phys.Vec3.class, float.class, float.class, float.class);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var vertices = new JsonArray();
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "x"), Oracle.f32(row, "y"), Oracle.f32(row, "z"));
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
            render.invoke(null, stack, collector, vec3(row.getAsJsonArray("vector")),
                Oracle.f32(row, "time"), Oracle.f32(row, "scale"), Oracle.f32(row, "tex_v_off"));
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
     * The nautilus models (grown, baby, zombie, zombie with corals) posed by the reference's own
     * `setupAnim`, drawn, from the look, the stride, the age and whether body armor is worn.
     */
    static JsonObject nautilusModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var state = new net.minecraft.client.renderer.entity.state.NautilusRenderState();
            state.yRot = Oracle.f32(row, "y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            if (row.get("armored").getAsBoolean()) {
                state.bodyArmorItem = new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.IRON_HORSE_ARMOR);
            }
            net.minecraft.client.model.animal.nautilus.NautilusModel model = id.contains("coral")
                ? new net.minecraft.client.model.monster.nautilus.ZombieNautilusCoralModel(root)
                : id.contains("saddle")
                    ? new net.minecraft.client.model.animal.nautilus.NautilusSaddleModel(root)
                    : id.contains("_armor")
                        ? new net.minecraft.client.model.animal.nautilus.NautilusArmorModel(root)
                        : new net.minecraft.client.model.animal.nautilus.NautilusModel(root);
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Where `DolphinCarryingItemLayer` submits what a dolphin carries, from an identity pose and
     * the dolphin's pitch: the matrix of the item.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject dolphinItemPose(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var poses = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitItem")) {
                        var matrix = new float[16];
                        ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(matrix);
                        for (float value : matrix) {
                            poses.add(Oracle.bits(value));
                        }
                    }
                    return null;
                });
            var state = new net.minecraft.client.renderer.entity.state.DolphinRenderState();
            state.xRot = Oracle.f32(row, "x_rot");
            state.heldItem.newLayer();
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> null;
            new net.minecraft.client.renderer.entity.layers.DolphinCarryingItemLayer(parent)
                .submit(new com.mojang.blaze3d.vertex.PoseStack(), collector, 0, state, 0.0F, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("pose", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The fish's models posed by the reference's own `setupAnim`, drawn: each layer by its class,
     * with the age and whether it is in water.
     */
    static JsonObject fishModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            float age = Oracle.f32(row, "age_in_ticks");
            boolean inWater = row.get("in_water").getAsBoolean();
            net.minecraft.client.model.EntityModel<?> model;
            if (id.startsWith("minecraft:cod#")) {
                var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
                state.ageInTicks = age;
                state.isInWater = inWater;
                var cod = new net.minecraft.client.model.animal.fish.CodModel(root);
                cod.setupAnim(state);
                model = cod;
            } else if (id.startsWith("minecraft:salmon")) {
                var state = new net.minecraft.client.renderer.entity.state.SalmonRenderState();
                state.ageInTicks = age;
                state.isInWater = inWater;
                var salmon = new net.minecraft.client.model.animal.fish.SalmonModel(root);
                salmon.setupAnim(state);
                model = salmon;
            } else if (id.startsWith("minecraft:tropical_fish")) {
                var state = new net.minecraft.client.renderer.entity.state.TropicalFishRenderState();
                state.ageInTicks = age;
                state.isInWater = inWater;
                if (id.startsWith("minecraft:tropical_fish_large")) {
                    var fish = new net.minecraft.client.model.animal.fish.TropicalFishLargeModel(root);
                    fish.setupAnim(state);
                    model = fish;
                } else {
                    var fish = new net.minecraft.client.model.animal.fish.TropicalFishSmallModel(root);
                    fish.setupAnim(state);
                    model = fish;
                }
            } else {
                var state = new net.minecraft.client.renderer.entity.state.EntityRenderState();
                state.ageInTicks = age;
                if (id.startsWith("minecraft:pufferfish_small")) {
                    var fish = new net.minecraft.client.model.animal.fish.PufferfishSmallModel(root);
                    fish.setupAnim(state);
                    model = fish;
                } else if (id.startsWith("minecraft:pufferfish_medium")) {
                    var fish = new net.minecraft.client.model.animal.fish.PufferfishMidModel(root);
                    fish.setupAnim(state);
                    model = fish;
                } else {
                    var fish = new net.minecraft.client.model.animal.fish.PufferfishBigModel(root);
                    fish.setupAnim(state);
                    model = fish;
                }
            }
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Which textures the fish wear: the cod's, the salmon's and the pufferfish's, and for each
     * tropical fish pattern its packed id, the texture of its base and the texture of its pattern,
     * from the renderer's and the layer's own constants.
     */
    static JsonObject fishTextures(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            if (!row.has("packed_id")) {
                var described = switch (row.get("label").getAsString()) {
                    case "cod" -> new Object[] {net.minecraft.client.renderer.entity.CodRenderer.class, "COD_LOCATION"};
                    case "salmon" -> new Object[] {net.minecraft.client.renderer.entity.SalmonRenderer.class, "SALMON_LOCATION"};
                    default -> new Object[] {net.minecraft.client.renderer.entity.PufferfishRenderer.class, "PUFFER_LOCATION"};
                };
                var field = ((Class<?>) described[0]).getDeclaredField((String) described[1]);
                field.setAccessible(true);
                out.addProperty("texture", field.get(null).toString());
                rows.add(out);
                continue;
            }
            int packed = row.get("packed_id").getAsInt();
            var pattern = net.minecraft.world.entity.animal.fish.TropicalFish.getPattern(packed);
            boolean large = pattern.base() == net.minecraft.world.entity.animal.fish.TropicalFish.Base.LARGE;
            var base = net.minecraft.client.renderer.entity.TropicalFishRenderer.class
                .getDeclaredField(large ? "LARGE_TEXTURE" : "SMALL_TEXTURE");
            base.setAccessible(true);
            var layer = net.minecraft.client.renderer.entity.layers.TropicalFishPatternLayer.class
                .getDeclaredField(pattern.name() + "_TEXTURE");
            layer.setAccessible(true);
            out.addProperty("packed_id", packed);
            out.addProperty("texture", base.get(null).toString());
            out.addProperty("pattern_texture", layer.get(null).toString());
            out.addProperty("base_dye", net.minecraft.world.entity.animal.fish.TropicalFish.getBaseColor(packed).getId());
            out.addProperty("pattern_dye", net.minecraft.world.entity.animal.fish.TropicalFish.getPatternColor(packed).getId());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What `GlowSquidRenderer.getBlockLightLevel` makes of the dark ticks and the light around: the
     * glow comes from the reference's own `Mth.clampedLerp`, cut to a whole level.
     */
    static JsonObject glowSquidLight(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            int glow = (int) net.minecraft.util.Mth.clampedLerp(1.0F - row.get("dark_ticks").getAsInt() / 10.0F, 0.0F, 15.0F);
            int light = glow == 15 ? 15 : Math.max(glow, row.get("block_light").getAsInt());
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("light", light);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
