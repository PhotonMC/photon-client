import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityBossOracle extends EntityBlockOracle {
    /** The beam of an end crystal, as the reference's own `submitCrystalBeams` builds it. */
    static JsonObject crystalBeam(final JsonObject request) throws Exception {
        var rows = new JsonArray();
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
            net.minecraft.client.renderer.entity.EnderDragonRenderer.submitCrystalBeams(
                Oracle.f32(row, "dx"), Oracle.f32(row, "dy"), Oracle.f32(row, "dz"), Oracle.f32(row, "time"),
                stack, collector, row.get("light").getAsInt());
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("vertices", vertices);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The rays of a dying ender dragon, as the reference's own `submitRays` builds them. */
    static JsonObject dragonRays(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var render = net.minecraft.client.renderer.entity.EnderDragonRenderer.class.getDeclaredMethod("submitRays",
            com.mojang.blaze3d.vertex.PoseStack.class, float.class,
            net.minecraft.client.renderer.SubmitNodeCollector.class,
            net.minecraft.client.renderer.rendertype.RenderType.class);
        render.setAccessible(true);
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
            render.invoke(null, stack, Oracle.f32(row, "death_time"), collector, null);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("vertices", vertices);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The shulker's model posed by the reference's own `ShulkerModel.setupAnim`, drawn. */
    static JsonObject shulkerModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.shulker.ShulkerModel(
                bakedLayer(row.get("layer").getAsString()));
            var state = new net.minecraft.client.renderer.entity.state.ShulkerRenderState();
            state.peekAmount = Oracle.f32(row, "peek_amount");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.xRot = Oracle.f32(row, "x_rot");
            state.yHeadRot = Oracle.f32(row, "y_head_rot");
            state.yBodyRot = Oracle.f32(row, "y_body_rot");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The magma cube's model posed by the reference's own `MagmaCubeModel.setupAnim`, drawn. */
    static JsonObject magmaCubeModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.slime.MagmaCubeModel(
                bakedLayer(row.get("layer").getAsString()));
            var state = new net.minecraft.client.renderer.entity.state.SlimeRenderState();
            state.squish = Oracle.f32(row, "squish");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
