import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityFarmOracle extends EntityDisplayOracle {
    /**
     * A sheep's models from the layer table (the body, the undercoat and the wool, grown and
     * baby), posed by the reference's own `setupAnim`, drawn.
     */
    static JsonObject sheepModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.QuadrupedModel<net.minecraft.client.renderer.entity.state.SheepRenderState> model =
                id.endsWith("#main")
                    ? (id.contains("_baby")
                        ? new net.minecraft.client.model.animal.sheep.BabySheepModel(root)
                        : new net.minecraft.client.model.animal.sheep.SheepModel(root))
                    : new net.minecraft.client.model.animal.sheep.SheepFurModel(root);
            var state = new net.minecraft.client.renderer.entity.state.SheepRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.headEatPositionScale = Oracle.f32(row, "eat_position");
            state.headEatAngleScale = Oracle.f32(row, "eat_angle");
            state.ageScale = Oracle.f32(row, "age_scale");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A chicken's models from the layer table (grown, cold and baby), posed by the reference's own
     * `setupAnim` with the flap, drawn.
     */
    static JsonObject chickenModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.animal.chicken.ChickenModel model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.chicken.BabyChickenModel(root)
                : id.startsWith("minecraft:cold")
                    ? new net.minecraft.client.model.animal.chicken.ColdChickenModel(root)
                    : new net.minecraft.client.model.animal.chicken.AdultChickenModel(root);
            var state = new net.minecraft.client.renderer.entity.state.ChickenRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.flap = Oracle.f32(row, "flap");
            state.flapSpeed = Oracle.f32(row, "flap_speed");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What a sheep's wool is multiplied by: for each dye the reference's table, and the colour of a
     * `jeb_` sheep at ticks along its cycle.
     */
    static JsonObject sheepColors(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            if (row.has("texture_dye")) {
                out.addProperty("color", net.minecraft.world.item.DyeColor.byId(row.get("texture_dye").getAsInt())
                    .getTextureDiffuseColor());
            } else if (row.has("dye")) {
                out.addProperty("color", net.minecraft.client.color.ColorLerper.Type.SHEEP.getColor(
                    net.minecraft.world.item.DyeColor.byId(row.get("dye").getAsInt())));
            } else {
                out.addProperty("color", net.minecraft.client.color.ColorLerper.getLerpedColor(
                    net.minecraft.client.color.ColorLerper.Type.SHEEP, Oracle.f32(row, "tick")));
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * How far a sheep's head is down and how it is tipped while it eats, from the reference's own
     * `Sheep` with its eating timer and its pitch set.
     */
    static JsonObject sheepEat(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var sheep = (net.minecraft.world.entity.animal.sheep.Sheep) UNSAFE.allocateInstance(
                net.minecraft.world.entity.animal.sheep.Sheep.class);
            var timer = Reflect.field(net.minecraft.world.entity.animal.sheep.Sheep.class, "eatAnimationTick");
            timer.setInt(sheep, row.get("eat_tick").getAsInt());
            for (String name : new String[] {"xRot", "xRotO"}) {
                var field = Reflect.field(net.minecraft.world.entity.Entity.class, name);
                field.setFloat(sheep, Oracle.f32(row, "x_rot"));
            }
            float partial = Oracle.f32(row, "partial");
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("position", Oracle.bits(sheep.getHeadEatPositionScale(partial)));
            out.addProperty("angle", Oracle.bits(sheep.getHeadEatAngleScale(partial)));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The goat's model from the layer table, posed by the reference's own `GoatModel.setupAnim`, drawn. */
    static JsonObject goatModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.goat.BabyGoatModel(root)
                : new net.minecraft.client.model.animal.goat.GoatModel(root);
            var state = new net.minecraft.client.renderer.entity.state.GoatRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.hasLeftHorn = row.get("left_horn").getAsBoolean();
            state.hasRightHorn = row.get("right_horn").getAsBoolean();
            state.rammingXHeadRot = Oracle.f32(row, "ramming");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The rabbit's model posed by the reference's own `RabbitModel.setupAnim` with its animation states, drawn. */
    static JsonObject rabbitModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.animal.rabbit.RabbitModel model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.rabbit.BabyRabbitModel(root)
                : new net.minecraft.client.model.animal.rabbit.AdultRabbitModel(root);
            var state = new net.minecraft.client.renderer.entity.state.RabbitRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            if (row.has("hop_start")) {
                state.hopAnimationState.start(row.get("hop_start").getAsInt());
            }
            if (row.has("idle_start")) {
                state.idleHeadTiltAnimationState.start(row.get("idle_start").getAsInt());
            }
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The silverfish's and the endermite's models, posed by the reference's own `setupAnim`, drawn. */
    static JsonObject segmentedModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.EntityModel<net.minecraft.client.renderer.entity.state.EntityRenderState> model =
                id.startsWith("minecraft:silverfish")
                    ? new net.minecraft.client.model.monster.silverfish.SilverfishModel(root)
                    : new net.minecraft.client.model.monster.endermite.EndermiteModel(root);
            var state = new net.minecraft.client.renderer.entity.state.EntityRenderState();
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The bee's models, posed by the reference's own `BeeModel.setupAnim`, drawn. */
    static JsonObject beeModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.animal.bee.BeeModel model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.bee.BabyBeeModel(root)
                : new net.minecraft.client.model.animal.bee.AdultBeeModel(root);
            var state = new net.minecraft.client.renderer.entity.state.BeeRenderState();
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.rollAmount = Oracle.f32(row, "roll_amount");
            state.hasStinger = row.get("has_stinger").getAsBoolean();
            state.isOnGround = row.get("on_ground").getAsBoolean();
            state.isAngry = row.get("angry").getAsBoolean();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A horse, donkey or mule model from the layer table, posed by the reference's own `setupAnim`, drawn. */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject equineModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            boolean baby = id.contains("_baby");
            boolean donkey = id.startsWith("minecraft:donkey") || id.startsWith("minecraft:mule");
            net.minecraft.client.model.animal.equine.AbstractEquineModel model;
            net.minecraft.client.renderer.entity.state.EquineRenderState state;
            if (donkey) {
                model = baby
                    ? new net.minecraft.client.model.animal.equine.BabyDonkeyModel(root)
                    : new net.minecraft.client.model.animal.equine.DonkeyModel(root);
                var donkeyState = new net.minecraft.client.renderer.entity.state.DonkeyRenderState();
                donkeyState.hasChest = row.get("has_chest").getAsBoolean();
                state = donkeyState;
            } else {
                model = baby
                    ? new net.minecraft.client.model.animal.equine.BabyHorseModel(root)
                    : new net.minecraft.client.model.animal.equine.HorseModel(root);
                state = new net.minecraft.client.renderer.entity.state.EquineRenderState();
            }
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.eatAnimation = Oracle.f32(row, "eat");
            state.standAnimation = Oracle.f32(row, "stand");
            state.feedingAnimation = Oracle.f32(row, "feeding");
            state.animateTail = row.get("animate_tail").getAsBoolean();
            state.isInWater = row.get("in_water").getAsBoolean();
            state.ageScale = baby ? 0.5F : 1.0F;
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The camel's model posed by the reference's own `CamelModel.setupAnim` with its animation states, drawn. */
    static JsonObject camelModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.animal.camel.CamelModel model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.camel.BabyCamelModel(root)
                : new net.minecraft.client.model.animal.camel.AdultCamelModel(root);
            var state = new net.minecraft.client.renderer.entity.state.CamelRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.jumpCooldown = Oracle.f32(row, "jump_cooldown");
            var states = new net.minecraft.world.entity.AnimationState[] {
                state.sitAnimationState, state.sitPoseAnimationState, state.sitUpAnimationState,
                state.idleAnimationState, state.dashAnimationState};
            String[] names = {"sit", "sit_pose", "sit_up", "idle", "dash"};
            for (int i = 0; i < names.length; i++) {
                if (row.has(names[i])) {
                    states[i].start(row.get(names[i]).getAsInt());
                }
            }
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The llama's model from the layer table, posed by the reference's own `LlamaModel.setupAnim`, drawn. */
    static JsonObject llamaModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            var model = id.contains("_baby")
                ? new net.minecraft.client.model.animal.llama.BabyLlamaModel(root)
                : new net.minecraft.client.model.animal.llama.LlamaModel(root);
            var state = new net.minecraft.client.renderer.entity.state.LlamaRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.hasChest = row.get("has_chest").getAsBoolean();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
