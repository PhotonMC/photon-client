import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityVehicleOracle extends EntityRotationOracle {
    static net.minecraft.client.renderer.entity.state.BoatRenderState boatState(final JsonObject row) {
        var state = new net.minecraft.client.renderer.entity.state.BoatRenderState();
        state.yRot = Oracle.f32(row, "y_rot");
        state.hurtDir = row.get("hurt_dir").getAsInt();
        state.hurtTime = Oracle.f32(row, "hurt_time");
        state.damageTime = Oracle.f32(row, "damage_time");
        state.bubbleAngle = Oracle.f32(row, "bubble_angle");
        state.isUnderWater = row.get("under_water").getAsBoolean();
        state.rowingTimeLeft = Oracle.f32(row, "rowing_left");
        state.rowingTimeRight = Oracle.f32(row, "rowing_right");
        return state;
    }

    /**
     * The boat models (boats, chest boats, rafts) posed by the reference's own `setupAnim`, drawn,
     * from the time each paddle has rowed, and the water patch.
     */
    static JsonObject boatModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.Model<?> model;
            if (id.contains("water_patch")) {
                model = new net.minecraft.client.model.Model.Simple(root, t -> net.minecraft.client.renderer.rendertype.RenderTypes.waterMask());
            } else {
                var state = boatState(row);
                var boat = id.contains("/bamboo")
                    ? new net.minecraft.client.model.object.boat.RaftModel(root)
                    : new net.minecraft.client.model.object.boat.BoatModel(root);
                boat.setupAnim(state);
                model = boat;
            }
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Where the reference's own `AbstractBoatRenderer.submit` draws a boat's model (and a boat's
     * water patch): the matrix it submits each with, from the boat's turn, hit, bubbles and
     * water, from an identity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject boatPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = boatState(row);
            var renderer = UNSAFE.allocateInstance(net.minecraft.client.renderer.entity.BoatRenderer.class);
            var poses = new JsonArray();
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
                        poses.add(described);
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            ((net.minecraft.client.renderer.entity.BoatRenderer) renderer).submit(state, stack, collector, null);
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
     * Where the reference's own minecart renderer draws a minecart (and the block it carries) from
     * the state its old behaviour extracts: the pose each of its two submissions leaves.
     */
    static JsonObject minecartPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var tnt = row.has("tnt_fuse");
            var state = tnt ? new net.minecraft.client.renderer.entity.state.MinecartTntRenderState()
                : new net.minecraft.client.renderer.entity.state.MinecartRenderState();
            if (tnt) {
                ((net.minecraft.client.renderer.entity.state.MinecartTntRenderState) state).fuseRemainingInTicks =
                    Oracle.f32(row, "tnt_fuse");
            }
            state.x = Oracle.f64(row, "x");
            state.y = Oracle.f64(row, "y");
            state.z = Oracle.f64(row, "z");
            state.xRot = Oracle.f32(row, "x_rot");
            state.yRot = Oracle.f32(row, "y_rot");
            state.offsetSeed = Long.parseLong(row.get("offset_seed").getAsString());
            state.hurtDir = row.get("hurt_dir").getAsInt();
            state.hurtTime = Oracle.f32(row, "hurt_time");
            state.damageTime = Oracle.f32(row, "damage_time");
            state.displayOffset = row.get("display_offset").getAsInt();
            state.isNewRender = row.has("new_render");
            if (row.has("rail")) {
                var rail = row.getAsJsonObject("rail");
                state.posOnRail = vec3(rail.getAsJsonArray("pos"));
                state.frontPos = vec3(rail.getAsJsonArray("front"));
                state.backPos = vec3(rail.getAsJsonArray("back"));
            }
            if (row.get("block").getAsBoolean()) {
                var parts = state.displayBlockModel.setupModel(new org.joml.Matrix4f(), false);
                parts.add((net.minecraft.client.renderer.block.dispatch.BlockStateModelPart)
                    java.lang.reflect.Proxy.newProxyInstance(EntityOracle.class.getClassLoader(),
                        new Class<?>[] {net.minecraft.client.renderer.block.dispatch.BlockStateModelPart.class},
                        (proxy, method, args) -> null));
            }
            var renderer = UNSAFE.allocateInstance(tnt ? net.minecraft.client.renderer.entity.TntMinecartRenderer.class
                : net.minecraft.client.renderer.entity.MinecartRenderer.class);
            var poses = new JsonArray();
            var overlays = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("submitModel") || method.getName().equals("submitBlockModel")) {
                        var stack = (com.mojang.blaze3d.vertex.PoseStack) (method.getName().equals("submitModel") ? args[2] : args[0]);
                        var matrix = new float[16];
                        stack.last().pose().get(matrix);
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                        if (method.getName().equals("submitBlockModel")) {
                            overlays.add((int) args[5]);
                        }
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            stack.translate(Oracle.f32(row, "feet_x"), Oracle.f32(row, "feet_y"), Oracle.f32(row, "feet_z"));
            if (tnt) {
                ((net.minecraft.client.renderer.entity.TntMinecartRenderer) renderer).submit(
                    (net.minecraft.client.renderer.entity.state.MinecartTntRenderState) state, stack, collector, null);
            } else {
                ((net.minecraft.client.renderer.entity.MinecartRenderer) renderer).submit(state, stack, collector, null);
            }
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
