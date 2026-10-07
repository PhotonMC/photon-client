import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityFlyerOracle extends EntityFarmOracle {
    /**
     * The blaze, ghast and phantom models posed by the reference's own `setupAnim`, drawn, from the
     * age (and the look, for a blaze; the flap time, for a phantom).
     */
    static JsonObject airborneModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.Model<?> model;
            if (id.contains("blaze")) {
                var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                var blaze = new net.minecraft.client.model.monster.blaze.BlazeModel(root);
                blaze.setupAnim(state);
                model = blaze;
            } else if (id.contains("happy_ghast")) {
                var state = new net.minecraft.client.renderer.entity.state.HappyGhastRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.isRidden = row.get("ridden").getAsBoolean();
                if (row.get("body_item").getAsBoolean()) {
                    state.bodyItem = new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.HARNESS.white());
                }
                if (id.contains("harness")) {
                    var harness = new net.minecraft.client.model.animal.ghast.HappyGhastHarnessModel(root);
                    harness.setupAnim(state);
                    model = harness;
                } else {
                    var happy = new net.minecraft.client.model.animal.ghast.HappyGhastModel(root);
                    happy.setupAnim(state);
                    model = happy;
                }
            } else if (id.contains("breeze")) {
                var state = new net.minecraft.client.renderer.entity.state.BreezeRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                var animations = new Object[][] {
                    {"idle", state.idle}, {"shoot", state.shoot}, {"slide", state.slide},
                    {"slide_back", state.slideBack}, {"inhale", state.inhale}, {"long_jump", state.longJump},
                };
                for (var animation : animations) {
                    if (row.has((String) animation[0])) {
                        ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                    }
                }
                var breeze = new net.minecraft.client.model.monster.breeze.BreezeModel(root);
                breeze.setupAnim(state);
                model = breeze;
            } else if (id.contains("wind_charge")) {
                var state = new net.minecraft.client.renderer.entity.state.EntityRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                var charge = new net.minecraft.client.model.object.projectile.WindChargeModel(root);
                charge.setupAnim(state);
                model = charge;
            } else if (id.contains("shulker_bullet")) {
                var state = new net.minecraft.client.renderer.entity.state.ShulkerBulletRenderState();
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                var bullet = new net.minecraft.client.model.object.projectile.ShulkerBulletModel(root);
                bullet.setupAnim(state);
                model = bullet;
            } else if (id.contains("leash_knot")) {
                var knot = new net.minecraft.client.model.object.leash.LeashKnotModel(root);
                knot.setupAnim(new net.minecraft.client.renderer.entity.state.EntityRenderState());
                model = knot;
            } else if (id.contains("llama_spit")) {
                var spit = new net.minecraft.client.model.animal.llama.LlamaSpitModel(root);
                spit.setupAnim(new net.minecraft.client.renderer.entity.state.EntityRenderState());
                model = spit;
            } else if (id.contains("evoker_fangs")) {
                var state = new net.minecraft.client.renderer.entity.state.EvokerFangsRenderState();
                state.biteProgress = Oracle.f32(row, "bite_progress");
                var fangs = new net.minecraft.client.model.effects.EvokerFangsModel(root);
                fangs.setupAnim(state);
                model = fangs;
            } else if (id.contains("trident")) {
                var trident = new net.minecraft.client.model.object.projectile.TridentModel(root);
                trident.setupAnim(net.minecraft.util.Unit.INSTANCE);
                model = trident;
            } else if (id.contains("arrow")) {
                var state = new net.minecraft.client.renderer.entity.state.ArrowRenderState();
                state.shake = Oracle.f32(row, "shake");
                var arrow = new net.minecraft.client.model.object.projectile.ArrowModel(root);
                arrow.setupAnim(state);
                model = arrow;
            } else if (id.contains("end_crystal")) {
                var state = new net.minecraft.client.renderer.entity.state.EndCrystalRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.showsBottom = row.get("shows_bottom").getAsBoolean();
                var crystal = new net.minecraft.client.model.object.crystal.EndCrystalModel(root);
                crystal.setupAnim(state);
                model = crystal;
            } else if (id.contains("ender_dragon")) {
                var state = new net.minecraft.client.renderer.entity.state.EnderDragonRenderState();
                state.flapTime = Oracle.f32(row, "flap_time");
                state.partialTicks = Oracle.f32(row, "partial_ticks");
                state.isLandingOrTakingOff = row.get("landing_or_taking_off").getAsBoolean();
                state.isSitting = row.get("sitting").getAsBoolean();
                state.distanceToEgg = Oracle.f64(row, "distance_to_egg");
                for (var sample : row.getAsJsonArray("history")) {
                    var pair = sample.getAsJsonObject();
                    state.flightHistory.record(Oracle.f64(pair, "y"), Oracle.f32(pair, "y_rot"));
                }
                var dragon = new net.minecraft.client.model.monster.dragon.EnderDragonModel(root);
                dragon.setupAnim(state);
                model = dragon;
            } else if (id.contains("wither")) {
                var state = new net.minecraft.client.renderer.entity.state.WitherRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.bodyRot = Oracle.f32(row, "body_rot");
                state.xHeadRots[0] = Oracle.f32(row, "x_head_0");
                state.xHeadRots[1] = Oracle.f32(row, "x_head_1");
                state.yHeadRots[0] = Oracle.f32(row, "y_head_0");
                state.yHeadRots[1] = Oracle.f32(row, "y_head_1");
                var wither = new net.minecraft.client.model.monster.wither.WitherBossModel(root);
                wither.setupAnim(state);
                model = wither;
            } else if (id.contains("sniffer")) {
                var state = new net.minecraft.client.renderer.entity.state.SnifferRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.isSearching = row.get("searching").getAsBoolean();
                var animations = new Object[][] {
                    {"digging", state.diggingAnimationState}, {"sniffing", state.sniffingAnimationState},
                    {"rising", state.risingAnimationState}, {"happy", state.feelingHappyAnimationState},
                    {"scenting", state.scentingAnimationState},
                };
                for (var animation : animations) {
                    if (row.has((String) animation[0])) {
                        ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                    }
                }
                // The renderer makes the baby's model from the same class as the adult's.
                var sniffer = new net.minecraft.client.model.animal.sniffer.SnifferModel(root);
                sniffer.setupAnim(state);
                model = sniffer;
            } else if (id.contains("armadillo")) {
                var state = new net.minecraft.client.renderer.entity.state.ArmadilloRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.isHidingInShell = row.get("hiding").getAsBoolean();
                var animations = new Object[][] {
                    {"roll_out", state.rollOutAnimationState}, {"roll_up", state.rollUpAnimationState},
                    {"peek", state.peekAnimationState},
                };
                for (var animation : animations) {
                    if (row.has((String) animation[0])) {
                        ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                    }
                }
                net.minecraft.client.model.animal.armadillo.ArmadilloModel armadillo = id.contains("baby")
                    ? new net.minecraft.client.model.animal.armadillo.BabyArmadilloModel(root)
                    : new net.minecraft.client.model.animal.armadillo.AdultArmadilloModel(root);
                armadillo.setupAnim(state);
                model = armadillo;
            } else if (id.contains("creaking")) {
                var state = new net.minecraft.client.renderer.entity.state.CreakingRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.canMove = row.get("can_move").getAsBoolean();
                var animations = new Object[][] {
                    {"attack", state.attackAnimationState}, {"invulnerable", state.invulnerabilityAnimationState},
                    {"death", state.deathAnimationState},
                };
                for (var animation : animations) {
                    if (row.has((String) animation[0])) {
                        ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                    }
                }
                var creaking = new net.minecraft.client.model.monster.creaking.CreakingModel(root);
                creaking.setupAnim(state);
                model = creaking;
            } else if (id.contains("warden")) {
                var state = new net.minecraft.client.renderer.entity.state.WardenRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.tendrilAnimation = Oracle.f32(row, "tendril");
                var animations = new Object[][] {
                    {"roar", state.roarAnimationState}, {"sniff", state.sniffAnimationState},
                    {"emerge", state.emergeAnimationState}, {"digging", state.diggingAnimationState},
                    {"attack", state.attackAnimationState}, {"sonic_boom", state.sonicBoomAnimationState},
                };
                for (var animation : animations) {
                    if (row.has((String) animation[0])) {
                        ((net.minecraft.world.entity.AnimationState) animation[1]).start(row.get((String) animation[0]).getAsInt());
                    }
                }
                var warden = new net.minecraft.client.model.monster.warden.WardenModel(root);
                warden.setupAnim(state);
                model = warden;
            } else if (id.contains("parrot")) {
                var state = new net.minecraft.client.renderer.entity.state.ParrotRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.flapAngle = Oracle.f32(row, "flap_angle");
                state.pose = net.minecraft.client.model.animal.parrot.ParrotModel.Pose.valueOf(row.get("pose").getAsString());
                var parrot = new net.minecraft.client.model.animal.parrot.ParrotModel(root);
                parrot.setupAnim(state);
                model = parrot;
            } else if (id.contains("bat")) {
                var state = new net.minecraft.client.renderer.entity.state.BatRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.isResting = row.get("resting").getAsBoolean();
                if (row.has("fly")) {
                    state.flyAnimationState.start(row.get("fly").getAsInt());
                }
                if (row.has("rest")) {
                    state.restAnimationState.start(row.get("rest").getAsInt());
                }
                var bat = new net.minecraft.client.model.ambient.BatModel(root);
                bat.setupAnim(state);
                model = bat;
            } else if (id.contains("vex")) {
                var state = new net.minecraft.client.renderer.entity.state.VexRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.isCharging = row.get("charging").getAsBoolean();
                if (row.get("right_item").getAsBoolean()) {
                    state.rightHandItemState.newLayer();
                }
                if (row.get("left_item").getAsBoolean()) {
                    state.leftHandItemState.newLayer();
                }
                var vex = new net.minecraft.client.model.monster.vex.VexModel(root);
                vex.setupAnim(state);
                model = vex;
            } else if (id.contains("allay")) {
                var state = new net.minecraft.client.renderer.entity.state.AllayRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.yRot = Oracle.f32(row, "y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.walkAnimationPos = Oracle.f32(row, "walk_position");
                state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
                state.isDancing = row.get("dancing").getAsBoolean();
                state.isSpinning = row.get("spinning").getAsBoolean();
                state.spinningProgress = Oracle.f32(row, "spinning_progress");
                state.holdingAnimationProgress = Oracle.f32(row, "holding_progress");
                var allay = new net.minecraft.client.model.animal.allay.AllayModel(root);
                allay.setupAnim(state);
                model = allay;
            } else if (id.contains("ghast")) {
                var state = new net.minecraft.client.renderer.entity.state.GhastRenderState();
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                var ghast = new net.minecraft.client.model.monster.ghast.GhastModel(root);
                ghast.setupAnim(state);
                model = ghast;
            } else {
                var state = new net.minecraft.client.renderer.entity.state.PhantomRenderState();
                state.flapTime = Oracle.f32(row, "flap_time");
                var phantom = new net.minecraft.client.model.monster.phantom.PhantomModel(root);
                phantom.setupAnim(state);
                model = phantom;
            }
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A vex's or an allay's model posed by the reference's own `setupAnim`, and where its own
     * `ItemInHandLayer` puts what each hand holds: the pose (a matrix) each held item is
     * submitted with, before its own display transform, the right hand's first.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject flierItemPose(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            boolean vex = row.get("kind").getAsString().equals("vex");
            net.minecraft.client.renderer.entity.state.ArmedEntityRenderState state;
            net.minecraft.client.model.EntityModel model;
            if (vex) {
                var vexState = new net.minecraft.client.renderer.entity.state.VexRenderState();
                vexState.isCharging = row.get("charging").getAsBoolean();
                state = vexState;
            } else {
                var allayState = new net.minecraft.client.renderer.entity.state.AllayRenderState();
                allayState.holdingAnimationProgress = Oracle.f32(row, "holding_progress");
                state = allayState;
            }
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.mainArm = row.get("main_arm").getAsString().equals("left")
                ? net.minecraft.world.entity.HumanoidArm.LEFT
                : net.minecraft.world.entity.HumanoidArm.RIGHT;
            if (row.get("holds_right").getAsBoolean()) {
                state.rightHandItemState.newLayer();
            }
            if (row.get("holds_left").getAsBoolean()) {
                state.leftHandItemState.newLayer();
            }
            var root = bakedLayer(vex ? "minecraft:vex#main" : "minecraft:allay#main");
            if (vex) {
                var vexModel = new net.minecraft.client.model.monster.vex.VexModel(root);
                vexModel.setupAnim((net.minecraft.client.renderer.entity.state.VexRenderState) state);
                model = vexModel;
            } else {
                var allayModel = new net.minecraft.client.model.animal.allay.AllayModel(root);
                allayModel.setupAnim((net.minecraft.client.renderer.entity.state.AllayRenderState) state);
                model = allayModel;
            }
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
            var finalModel = model;
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> finalModel;
            new net.minecraft.client.renderer.entity.layers.ItemInHandLayer(parent)
                .submit(new com.mojang.blaze3d.vertex.PoseStack(), collector, 0, state, 0.0F, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("poses", poses);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The texture each parrot variant wears (`ParrotRenderer.getVariantTexture`), by id. */
    static JsonObject parrotTextures(final JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("texture", net.minecraft.client.renderer.entity.ParrotRenderer.getVariantTexture(
                net.minecraft.world.entity.animal.parrot.Parrot.Variant.byId(row.get("id").getAsInt())).toString());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
