import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityWildOracle extends EntityVillagerOracle {
    /** The wolf's model posed by the reference's own `WolfModel.setupAnim` (grown and baby), drawn. */
    static JsonObject wolfModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.animal.wolf.WolfModel model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.wolf.BabyWolfModel(root)
                : new net.minecraft.client.model.animal.wolf.AdultWolfModel(root);
            var state = new net.minecraft.client.renderer.entity.state.WolfRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.isAngry = row.get("angry").getAsBoolean();
            state.isSitting = row.get("sitting").getAsBoolean();
            state.tailAngle = Oracle.f32(row, "tail_angle");
            state.headRollAngle = Oracle.f32(row, "head_roll_angle");
            state.shakeAnim = Oracle.f32(row, "shake_anim");
            state.ageScale = Oracle.f32(row, "age_scale");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The cat's and ocelot's models posed by the reference's own `setupAnim` (grown and baby), drawn. */
    static JsonObject felineModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            boolean baby = id.contains("_baby");
            var state = id.contains("cat")
                ? new net.minecraft.client.renderer.entity.state.CatRenderState()
                : new net.minecraft.client.renderer.entity.state.FelineRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.isCrouching = row.get("crouching").getAsBoolean();
            state.isSprinting = row.get("sprinting").getAsBoolean();
            state.isSitting = row.get("sitting").getAsBoolean();
            state.lieDownAmount = Oracle.f32(row, "lie_down_amount");
            state.lieDownAmountTail = Oracle.f32(row, "lie_down_amount_tail");
            state.relaxStateOneAmount = Oracle.f32(row, "relax_state_one_amount");
            state.ageScale = Oracle.f32(row, "age_scale");
            net.minecraft.client.model.EntityModel<?> model;
            if (id.contains("cat")) {
                var cat = (net.minecraft.client.renderer.entity.state.CatRenderState) state;
                var catModel = baby
                    ? new net.minecraft.client.model.animal.feline.BabyCatModel(root)
                    : new net.minecraft.client.model.animal.feline.AdultCatModel(root);
                catModel.setupAnim(cat);
                model = catModel;
            } else {
                var ocelotModel = baby
                    ? new net.minecraft.client.model.animal.feline.BabyOcelotModel(root)
                    : new net.minecraft.client.model.animal.feline.AdultOcelotModel(root);
                ocelotModel.setupAnim(state);
                model = ocelotModel;
            }
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The polar bear's model posed by the reference's own `PolarBearModel.setupAnim` (grown and baby), drawn. */
    static JsonObject polarBearModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.animal.polarbear.PolarBearModel(bakedLayer(row.get("layer").getAsString()));
            var state = new net.minecraft.client.renderer.entity.state.PolarBearRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.standScale = Oracle.f32(row, "stand_scale");
            state.ageScale = Oracle.f32(row, "age_scale");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The hoglin's (and zoglin's) model posed by the reference's own `setupAnim` (grown and baby), drawn. */
    static JsonObject hoglinModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.monster.hoglin.HoglinModel model = id.contains("_baby")
                ? new net.minecraft.client.model.monster.hoglin.BabyHoglinModel(root)
                : new net.minecraft.client.model.monster.hoglin.HoglinModel(root);
            var state = new net.minecraft.client.renderer.entity.state.HoglinRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.attackAnimationRemainingTicks = row.get("attack_ticks").getAsInt();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The fox's model posed by the reference's own `FoxModel.setupAnim` (grown and baby), drawn, and
     * the matrix `FoxHeldItemLayer` submits what it holds with from an identity entity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject foxModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            boolean baby = id.contains("_baby");
            net.minecraft.client.model.animal.fox.FoxModel model = baby
                ? new net.minecraft.client.model.animal.fox.BabyFoxModel(root)
                : new net.minecraft.client.model.animal.fox.AdultFoxModel(root);
            var state = new net.minecraft.client.renderer.entity.state.FoxRenderState();
            state.isBaby = baby;
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.headRollAngle = Oracle.f32(row, "head_roll_angle");
            state.crouchAmount = Oracle.f32(row, "crouch_amount");
            state.isCrouching = row.get("crouching").getAsBoolean();
            state.isSleeping = row.get("sleeping").getAsBoolean();
            state.isSitting = row.get("sitting").getAsBoolean();
            state.isFaceplanted = row.get("faceplanted").getAsBoolean();
            state.isPouncing = row.get("pouncing").getAsBoolean();
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.ageScale = Oracle.f32(row, "age_scale");
            model.setupAnim(state);
            var out = posedQuads(row.get("label").getAsString(), model);
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
            state.heldItem.newLayer();
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
            new net.minecraft.client.renderer.entity.layers.FoxHeldItemLayer(parent)
                .submit(new com.mojang.blaze3d.vertex.PoseStack(), collector, 0, state, state.yRot, state.xRot);
            out.add("poses", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The fox's textures by variant, state and age, from the renderer's own table. */
    static JsonObject foxTextures() throws Exception {
        var field = net.minecraft.client.renderer.entity.FoxRenderer.class.getDeclaredField("TEXTURES_BY_VARIANT");
        field.setAccessible(true);
        var table = (java.util.Map<?, ?>) field.get(null);
        var rows = new JsonArray();
        for (var variant : net.minecraft.world.entity.animal.fox.Fox.Variant.values()) {
            var byState = table.get(variant);
            for (String stateName : new String[] {"idle", "sleeping"}) {
                var stateMethod = byState.getClass().getDeclaredMethod(stateName);
                stateMethod.setAccessible(true);
                var byAge = stateMethod.invoke(byState);
                for (String age : new String[] {"adult", "baby"}) {
                    var ageMethod = byAge.getClass().getDeclaredMethod(age);
                    ageMethod.setAccessible(true);
                    var out = new JsonObject();
                    out.addProperty("label", variant.getId() + "-" + stateName + "-" + age);
                    out.addProperty("texture", ageMethod.invoke(byAge).toString());
                    rows.add(out);
                }
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The panda's model posed by the reference's own `PandaModel.setupAnim` (grown and baby), drawn.
     */
    static JsonObject pandaModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.panda.BabyPandaModel(root)
                : new net.minecraft.client.model.animal.panda.PandaModel(root);
            var state = new net.minecraft.client.renderer.entity.state.PandaRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isUnhappy = row.get("unhappy").getAsBoolean();
            state.isSneezing = row.get("sneezing").getAsBoolean();
            state.sneezeTime = row.get("sneeze_time").getAsInt();
            state.isEating = row.get("eating").getAsBoolean();
            state.isScared = row.get("scared").getAsBoolean();
            state.sitAmount = Oracle.f32(row, "sit_amount");
            state.lieOnBackAmount = Oracle.f32(row, "lie_on_back_amount");
            state.rollAmount = Oracle.f32(row, "roll_amount");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The panda's textures by gene and age, from the renderer's own tables. */
    static JsonObject pandaTextures() throws Exception {
        var rows = new JsonArray();
        for (String table : new String[] {"TEXTURES", "BABY_TEXTURES"}) {
            var field = net.minecraft.client.renderer.entity.PandaRenderer.class.getDeclaredField(table);
            field.setAccessible(true);
            var map = (java.util.Map<?, ?>) field.get(null);
            for (var gene : net.minecraft.world.entity.animal.panda.Panda.Gene.values()) {
                var out = new JsonObject();
                out.addProperty("label", gene.getId() + "-" + table);
                out.addProperty("texture", map.get(gene).toString());
                rows.add(out);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The ravager's model posed by the reference's own `RavagerModel.setupAnim`, drawn. */
    static JsonObject ravagerModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.ravager.RavagerModel(bakedLayer(row.get("layer").getAsString()));
            var state = new net.minecraft.client.renderer.entity.state.RavagerRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.stunnedTicksRemaining = Oracle.f32(row, "stunned");
            state.attackTicksRemaining = Oracle.f32(row, "attack");
            state.roarAnimation = Oracle.f32(row, "roar");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The strider's models (main, baby and saddle) posed by the reference's own `setupAnim`, drawn. */
    static JsonObject striderModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.monster.strider.StriderModel model = id.contains("_baby")
                ? new net.minecraft.client.model.monster.strider.BabyStriderModel(root)
                : new net.minecraft.client.model.monster.strider.AdultStriderModel(root);
            var state = new net.minecraft.client.renderer.entity.state.StriderRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isRidden = row.get("ridden").getAsBoolean();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The strider's textures by state and age, from the renderer's own constants. */
    static JsonObject striderTextures() throws Exception {
        var rows = new JsonArray();
        for (String name : new String[] {"STRIDER_LOCATION", "STRIDER_BABY_LOCATION", "COLD_LOCATION", "COLD_BABY_LOCATION"}) {
            var field = net.minecraft.client.renderer.entity.StriderRenderer.class.getDeclaredField(name);
            field.setAccessible(true);
            var out = new JsonObject();
            out.addProperty("label", name);
            out.addProperty("texture", field.get(null).toString());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A quadruped's model from the layer table, posed by the reference's own `setupAnim`, drawn. */
    static JsonObject quadrupedModel(final JsonObject request) throws Exception {
        Oracle.vanillaRegistries();
        var layers = new java.util.HashMap<String, net.minecraft.client.model.geom.builders.LayerDefinition>();
        for (var entry : net.minecraft.client.model.geom.LayerDefinitions.createRoots().entrySet()) {
            layers.put(entry.getKey().toString(), entry.getValue());
        }
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = layers.get(id).bakeRoot();
            net.minecraft.client.model.QuadrupedModel<net.minecraft.client.renderer.entity.state.LivingEntityRenderState>
                model = id.startsWith("minecraft:cow") || id.startsWith("minecraft:warm_cow")
                    || id.startsWith("minecraft:cold_cow") || id.startsWith("minecraft:mooshroom")
                    ? new net.minecraft.client.model.animal.cow.CowModel(root)
                    : new net.minecraft.client.model.animal.pig.PigModel(root);
            var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            model.setupAnim(state);
            var capture = new Capture();
            model.root().render(new com.mojang.blaze3d.vertex.PoseStack(), capture, 0, 0);
            var quads = new java.util.ArrayList<String>();
            for (int i = 0; i < capture.vertices.size(); i += 4) {
                quads.add(String.join("", capture.vertices.subList(i, i + 4)));
            }
            java.util.Collections.sort(quads);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var array = new JsonArray();
            quads.forEach(array::add);
            out.add("quads", array);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
