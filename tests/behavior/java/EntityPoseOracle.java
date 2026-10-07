import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityPoseOracle extends EntityLayerOracle {
    /** The reference's own limb animation state, stepped and read between ticks. */
    static JsonObject walkAnimation(final JsonObject request) {
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

    /**
     * A dropped item's poses as the reference's own renderer submits them: the bob and the spin,
     * then each copy of the stack with the offsets its seed gives, for a model of a given extent.
     */
    static JsonObject itemEntityPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var renderer = (net.minecraft.client.renderer.entity.ItemEntityRenderer) UNSAFE.allocateInstance(
                net.minecraft.client.renderer.entity.ItemEntityRenderer.class);
            var randomField = net.minecraft.client.renderer.entity.ItemEntityRenderer.class.getDeclaredField("random");
            randomField.setAccessible(true);
            randomField.set(renderer, net.minecraft.util.RandomSource.create());
            var state = new net.minecraft.client.renderer.entity.state.ItemEntityRenderState();
            state.ageInTicks = Oracle.f32(row, "age");
            state.bobOffset = Oracle.f32(row, "bob");
            state.count = row.get("count").getAsInt();
            state.seed = row.get("seed").getAsInt();
            var min = new org.joml.Vector3f(Oracle.f32(row, "min_x"), Oracle.f32(row, "min_y"), Oracle.f32(row, "min_z"));
            var max = new org.joml.Vector3f(Oracle.f32(row, "max_x"), Oracle.f32(row, "max_y"), Oracle.f32(row, "max_z"));
            state.item.newLayer().setExtents(() -> new org.joml.Vector3fc[] {min, max});
            var poses = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitItem")) {
                        var stack = (com.mojang.blaze3d.vertex.PoseStack) args[0];
                        var matrix = new float[16];
                        stack.last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                    }
                    return null;
                });
            renderer.submit(state, new com.mojang.blaze3d.vertex.PoseStack(), collector, null);
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
     * Every item that is a kinetic weapon or does not swing as the default does: the registry's own
     * numbers a spear is posed and timed by (its swing, and its kinetic weapon's delay and conditions).
     */
    static JsonObject spearTable() {
        var rows = new JsonArray();
        for (var item : net.minecraft.core.registries.BuiltInRegistries.ITEM) {
            var stack = new net.minecraft.world.item.ItemStack(item);
            var kinetic = stack.get(net.minecraft.core.component.DataComponents.KINETIC_WEAPON);
            var swing = stack.getSwingAnimation();
            if (kinetic == null && swing.type() == net.minecraft.world.item.SwingAnimationType.WHACK) {
                continue;
            }
            var row = new JsonObject();
            row.addProperty("name", net.minecraft.core.registries.BuiltInRegistries.ITEM.getKey(item).toString());
            row.addProperty("swing", swing.type().name());
            row.addProperty("swing_ticks", swing.duration());
            if (kinetic != null) {
                row.addProperty("delay_ticks", kinetic.delayTicks());
                row.addProperty("dismount_ticks", kinetic.dismountConditions().map(c -> c.maxDurationTicks()).orElse(0));
                row.addProperty("knockback_ticks", kinetic.knockbackConditions().map(c -> c.maxDurationTicks()).orElse(0));
                row.addProperty("damage_ticks", kinetic.damageConditions().map(c -> c.maxDurationTicks()).orElse(0));
                row.addProperty("forward_movement", Oracle.bits(kinetic.forwardMovement()));
            }
            rows.add(row);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    static final String[] ANIMATION_CLASSES = {
        "Armadillo", "BabyArmadillo", "BabyAxolotl", "BabyRabbit", "Bat", "Breeze", "Camel", "CamelBaby",
        "CopperGolem", "Creaking", "FoxBaby", "Frog", "Nautilus", "Rabbit", "Sniffer", "Warden"};

    /** Every animation definition of the reference's `definitions` package, by `Class.FIELD`. */
    static java.util.Map<String, net.minecraft.client.animation.AnimationDefinition> animationDefinitions()
        throws Exception {
        var found = new java.util.LinkedHashMap<String, net.minecraft.client.animation.AnimationDefinition>();
        for (String name : ANIMATION_CLASSES) {
            var type = Class.forName("net.minecraft.client.animation.definitions." + name + "Animation");
            for (var field : type.getDeclaredFields()) {
                if (field.getType() == net.minecraft.client.animation.AnimationDefinition.class
                    && java.lang.reflect.Modifier.isStatic(field.getModifiers())) {
                    found.put(name + "." + field.getName(),
                        (net.minecraft.client.animation.AnimationDefinition) field.get(null));
                }
            }
        }
        return found;
    }

    static JsonArray vector(final org.joml.Vector3fc vector) {
        var out = new JsonArray();
        out.add(Oracle.bits(vector.x()));
        out.add(Oracle.bits(vector.y()));
        out.add(Oracle.bits(vector.z()));
        return out;
    }

    /**
     * Every animation definition: its length, whether it loops, and for each bone (in the order
     * the reference keeps them) each channel's target and keyframes.
     */
    static JsonObject animationDefinitionRows() throws Exception {
        var rows = new JsonArray();
        for (var entry : animationDefinitions().entrySet()) {
            var definition = entry.getValue();
            var out = new JsonObject();
            out.addProperty("label", entry.getKey());
            out.addProperty("length", Oracle.bits(definition.lengthInSeconds()));
            out.addProperty("looping", definition.looping());
            var bones = new JsonArray();
            for (var bone : definition.boneAnimations().entrySet()) {
                var described = new JsonObject();
                described.addProperty("name", bone.getKey());
                var channels = new JsonArray();
                for (var channel : bone.getValue()) {
                    var one = new JsonObject();
                    one.addProperty("target", channel.target() == net.minecraft.client.animation.AnimationChannel.Targets.POSITION
                        ? "POSITION" : channel.target() == net.minecraft.client.animation.AnimationChannel.Targets.ROTATION
                        ? "ROTATION" : channel.target() == net.minecraft.client.animation.AnimationChannel.Targets.SCALE
                        ? "SCALE" : "OTHER");
                    var frames = new JsonArray();
                    for (var frame : channel.keyframes()) {
                        var two = new JsonObject();
                        two.addProperty("time", Oracle.bits(frame.timestamp()));
                        two.add("pre", vector(frame.preTarget()));
                        two.add("post", vector(frame.postTarget()));
                        two.addProperty("interpolation",
                            frame.interpolation() == net.minecraft.client.animation.AnimationChannel.Interpolations.LINEAR
                                ? "LINEAR" : frame.interpolation()
                                    == net.minecraft.client.animation.AnimationChannel.Interpolations.CATMULLROM
                                ? "CATMULLROM" : "OTHER");
                        frames.add(two);
                    }
                    one.add("keyframes", frames);
                    channels.add(one);
                }
                described.add("channels", channels);
                bones.add(described);
            }
            out.add("bones", bones);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A model's pose after the reference's own `KeyframeAnimation.apply(millis, scale)` of a definition,
     * drawn.
     */
    static JsonObject keyframeAnimation(final JsonObject request) throws Exception {
        var definitions = animationDefinitions();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.Model<?> model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.rabbit.BabyRabbitModel(root)
                : new net.minecraft.client.model.animal.rabbit.AdultRabbitModel(root);
            var animation = definitions.get(row.get("definition").getAsString()).bake(root);
            animation.apply(row.get("millis").getAsLong(), Oracle.f32(row, "scale"));
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** `AnimationState`: what each call sequence makes of the time an animation has run. */
    static JsonObject animationStates(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.world.entity.AnimationState();
            var millis = new JsonArray();
            for (var step : row.getAsJsonArray("steps")) {
                var call = step.getAsJsonObject();
                switch (call.get("call").getAsString()) {
                    case "start" -> state.start(call.get("tick").getAsInt());
                    case "start_if_stopped" -> state.startIfStopped(call.get("tick").getAsInt());
                    case "animate_when" -> state.animateWhen(call.get("condition").getAsBoolean(), call.get("tick").getAsInt());
                    case "stop" -> state.stop();
                    case "fast_forward" -> state.fastForward(call.get("ticks").getAsInt(), Oracle.f32(call, "scale"));
                    default -> throw new IllegalArgumentException("Unknown call " + call);
                }
                var seen = new JsonObject();
                seen.addProperty("started", state.isStarted());
                seen.addProperty("millis", state.getTimeInMillis(Oracle.f32(call, "age")));
                millis.add(seen);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("steps", millis);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Stops `LivingEntityRenderer.submit` once the pose it builds is in hand. */
    static final class PoseCaptured extends RuntimeException {
        PoseCaptured() {
            super(null, null, false, false);
        }
    }

    /**
     * A living entity's renderer as far as its own pose goes: the real
     * `submit` runs up to the point where it hands the pose to `scale`, which
     * this renderer's does not return from.
     */
    static final class PoseProbe extends net.minecraft.client.renderer.entity.LivingEntityRenderer<
        net.minecraft.world.entity.LivingEntity,
        net.minecraft.client.renderer.entity.state.LivingEntityRenderState,
        net.minecraft.client.model.EntityModel<net.minecraft.client.renderer.entity.state.LivingEntityRenderState>> {
        static final float[] CAPTURED = new float[16];
        static float flip = 90.0F;

        PoseProbe() {
            super(null, null, 0.0F);
        }

        @Override
        public net.minecraft.resources.Identifier getTextureLocation(
            final net.minecraft.client.renderer.entity.state.LivingEntityRenderState state) {
            return null;
        }

        @Override
        public net.minecraft.client.renderer.entity.state.LivingEntityRenderState createRenderState() {
            return new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
        }

        @Override
        protected float getFlipDegrees() {
            return flip;
        }

        @Override
        protected void scale(
            final net.minecraft.client.renderer.entity.state.LivingEntityRenderState state,
            final com.mojang.blaze3d.vertex.PoseStack poseStack) {
            // What `submit` does next, before it asks for the render type.
            poseStack.translate(0.0F, -1.501F, 0.0F);
            poseStack.last().pose().get(CAPTURED);
            throw new PoseCaptured();
        }
    }

    /** The pose a living entity is drawn from, by `LivingEntityRenderer.submit`. */
    static JsonObject livingPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var renderer = (PoseProbe) UNSAFE.allocateInstance(PoseProbe.class);
            var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
            state.scale = Oracle.f32(row, "scale");
            state.bodyRot = Oracle.f32(row, "body_rot");
            state.deathTime = Oracle.f32(row, "death_time");
            PoseProbe.flip = row.has("flip_degrees") ? Oracle.f32(row, "flip_degrees") : 90.0F;
            try {
                renderer.submit(state, new com.mojang.blaze3d.vertex.PoseStack(), null, null);
            } catch (PoseCaptured expected) {
                // The pose is in hand.
            }
            var described = new JsonArray();
            for (float value : PoseProbe.CAPTURED) {
                described.add(Oracle.bits(value));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("pose", described);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Records the corners `ExperienceOrbRenderer` writes, one vertex at a time. */
    static final class OrbCapture implements com.mojang.blaze3d.vertex.VertexConsumer {
        final JsonArray vertices = new JsonArray();
        JsonObject current;

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer addVertex(final float x, final float y, final float z) {
            this.current = new JsonObject();
            this.current.addProperty("x", Oracle.bits(x));
            this.current.addProperty("y", Oracle.bits(y));
            this.current.addProperty("z", Oracle.bits(z));
            this.vertices.add(this.current);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int r, final int g, final int b, final int a) {
            this.current.addProperty("color", r + "," + g + "," + b + "," + a);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int color) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv(final float u, final float v) {
            this.current.addProperty("u", Oracle.bits(u));
            this.current.addProperty("v", Oracle.bits(v));
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv1(final int u, final int v) {
            this.current.addProperty("overlay", u | v << 16);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv2(final int u, final int v) {
            this.current.addProperty("light", u | v << 16);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setNormal(final float x, final float y, final float z) {
            this.current.addProperty("normal", Oracle.bits(x) + Oracle.bits(y) + Oracle.bits(z));
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setLineWidth(final float width) {
            throw new UnsupportedOperationException();
        }
    }

    /** `ExperienceOrbRenderer.submit`: the quad it writes, from the camera's side. */
    static JsonObject experienceOrb(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var renderer = (net.minecraft.client.renderer.entity.ExperienceOrbRenderer) UNSAFE.allocateInstance(
                net.minecraft.client.renderer.entity.ExperienceOrbRenderer.class);
            var state = new net.minecraft.client.renderer.entity.state.ExperienceOrbRenderState();
            state.icon = row.get("icon").getAsInt();
            state.ageInTicks = Oracle.f32(row, "age");
            state.lightCoords = row.get("light").getAsInt();
            var camera = new net.minecraft.client.renderer.state.level.CameraRenderState();
            // `Camera.setRotation`.
            camera.orientation = new org.joml.Quaternionf().rotationYXZ(
                (float) Math.PI - Oracle.f32(row, "yaw") * (float) (Math.PI / 180.0),
                -Oracle.f32(row, "pitch") * (float) (Math.PI / 180.0), 0.0F);
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "px"), Oracle.f32(row, "py"), Oracle.f32(row, "pz"));
            var capture = new OrbCapture();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitCustomGeometry")) {
                        ((net.minecraft.client.renderer.SubmitNodeCollector.CustomGeometryRenderer) args[2])
                            .render(((com.mojang.blaze3d.vertex.PoseStack) args[0]).last(), capture);
                    }
                    return null;
                });
            renderer.submit(state, stack, collector, camera);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("vertices", capture.vertices);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
