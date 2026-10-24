import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityVillagerOracle extends EntityVehicleOracle {
    /** `.data/minecraft/assets/jar/assets/minecraft`, found from the game's jar. */
    static java.nio.file.Path jarAssets() throws Exception {
        var jar = java.nio.file.Path.of(net.minecraft.client.Minecraft.class
            .getProtectionDomain().getCodeSource().getLocation().toURI());
        return jar.getParent().getParent().resolve("assets/jar/assets/minecraft");
    }

    /** The hat a villager texture's metadata asks for, through the reference's own codec. */
    static net.minecraft.client.resources.metadata.animation.VillagerMetadataSection.Hat villagerHat(
        final String kind, final String name) throws Exception {
        var file = jarAssets().resolve("textures/entity/villager/" + kind + "/" + name + ".png.mcmeta");
        if (!java.nio.file.Files.exists(file)) {
            return net.minecraft.client.resources.metadata.animation.VillagerMetadataSection.Hat.NONE;
        }
        var json = com.google.gson.JsonParser.parseReader(java.nio.file.Files.newBufferedReader(file)).getAsJsonObject();
        return net.minecraft.client.resources.metadata.animation.VillagerMetadataSection.CODEC
            .parse(com.mojang.serialization.JsonOps.INSTANCE, json.get("villager")).getOrThrow().hat();
    }

    /**
     * The villager types and professions in the order the wire numbers them (the built-in
     * registries), and for each the hat its texture's metadata asks for.
     */
    static JsonObject villagerRegistries() throws Exception {
        Oracle.vanillaRegistries();
        var rows = new JsonArray();
        var types = new java.util.TreeMap<Integer, String>();
        for (var type : net.minecraft.core.registries.BuiltInRegistries.VILLAGER_TYPE) {
            types.put(net.minecraft.core.registries.BuiltInRegistries.VILLAGER_TYPE.getId(type),
                net.minecraft.core.registries.BuiltInRegistries.VILLAGER_TYPE.getKey(type).getPath());
        }
        var professions = new java.util.TreeMap<Integer, String>();
        for (var profession : net.minecraft.core.registries.BuiltInRegistries.VILLAGER_PROFESSION) {
            professions.put(net.minecraft.core.registries.BuiltInRegistries.VILLAGER_PROFESSION.getId(profession),
                net.minecraft.core.registries.BuiltInRegistries.VILLAGER_PROFESSION.getKey(profession).getPath());
        }
        for (var entry : new Object[][] {{"type", types}, {"profession", professions}}) {
            @SuppressWarnings("unchecked")
            var names = (java.util.TreeMap<Integer, String>) entry[1];
            int index = 0;
            for (var name : names.entrySet()) {
                var out = new JsonObject();
                out.addProperty("label", entry[0] + ":" + name.getValue());
                out.addProperty("kind", (String) entry[0]);
                out.addProperty("id", name.getKey());
                out.addProperty("name", name.getValue());
                out.addProperty("hat", villagerHat((String) entry[0], name.getValue()).getSerializedName());
                rows.add(out);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What `VillagerProfessionLayer` draws for a villager: the model (the hatted one or not) and
     * the texture and colour of each pass, from the reference's own layer with the hats its textures
     * ask for.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject villagerLayers(final JsonObject request) throws Exception {
        Oracle.vanillaRegistries();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            boolean baby = row.get("baby").getAsBoolean();
            String prefix = row.get("path").getAsString();
            boolean zombie = prefix.equals("zombie_villager");
            net.minecraft.client.model.EntityModel model = zombie
                ? new net.minecraft.client.model.monster.zombie.ZombieVillagerModel(bakedLayer("minecraft:" + prefix + "#main"))
                : new net.minecraft.client.model.npc.VillagerModel(bakedLayer("minecraft:" + prefix + "#main"));
            net.minecraft.client.model.EntityModel babyModel = zombie
                ? new net.minecraft.client.model.monster.zombie.BabyZombieVillagerModel(bakedLayer("minecraft:" + prefix + "_baby#main"))
                : new net.minecraft.client.model.npc.BabyVillagerModel(bakedLayer("minecraft:" + prefix + "_baby#main"));
            net.minecraft.client.model.EntityModel noHat = zombie
                ? new net.minecraft.client.model.monster.zombie.ZombieVillagerModel(bakedLayer("minecraft:" + prefix + "_no_hat#main"))
                : new net.minecraft.client.model.npc.VillagerModel(bakedLayer("minecraft:" + prefix + "_no_hat#main"));
            net.minecraft.client.model.EntityModel noHatBaby = zombie
                ? new net.minecraft.client.model.monster.zombie.BabyZombieVillagerModel(
                    bakedLayer("minecraft:" + prefix + "_baby_no_hat#main"))
                : new net.minecraft.client.model.npc.BabyVillagerModel(bakedLayer("minecraft:" + prefix + "_baby_no_hat#main"));
            var current = baby ? babyModel : model;
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> current;
            var layer = new net.minecraft.client.renderer.entity.layers.VillagerProfessionLayer(
                parent, null, prefix, noHat, noHatBaby);
            var typeKey = net.minecraft.resources.ResourceKey.create(net.minecraft.core.registries.Registries.VILLAGER_TYPE,
                net.minecraft.resources.Identifier.withDefaultNamespace(row.get("type").getAsString()));
            var professionKey = net.minecraft.resources.ResourceKey.create(
                net.minecraft.core.registries.Registries.VILLAGER_PROFESSION,
                net.minecraft.resources.Identifier.withDefaultNamespace(row.get("profession").getAsString()));
            var typeCache = Reflect.field(net.minecraft.client.renderer.entity.layers.VillagerProfessionLayer.class, "typeHatCache");
            ((java.util.Map) typeCache.get(layer)).put(typeKey, villagerHat("type", typeKey.identifier().getPath()));
            var professionCache = Reflect.field(net.minecraft.client.renderer.entity.layers.VillagerProfessionLayer.class, "professionHatCache");
            ((java.util.Map) professionCache.get(layer)).put(professionKey,
                villagerHat("profession", professionKey.identifier().getPath()));
            var villagerData = new net.minecraft.world.entity.npc.villager.VillagerData(
                net.minecraft.core.registries.BuiltInRegistries.VILLAGER_TYPE.getOrThrow(typeKey),
                net.minecraft.core.registries.BuiltInRegistries.VILLAGER_PROFESSION.getOrThrow(professionKey),
                row.get("level").getAsInt());
            net.minecraft.client.renderer.entity.state.LivingEntityRenderState state;
            if (zombie) {
                var zombieState = new net.minecraft.client.renderer.entity.state.ZombieVillagerRenderState();
                zombieState.villagerData = villagerData;
                state = zombieState;
            } else {
                var villagerState = new net.minecraft.client.renderer.entity.state.VillagerRenderState();
                villagerState.villagerData = villagerData;
                state = villagerState;
            }
            state.isBaby = baby;
            var passes = new JsonArray();
            var holder = new net.minecraft.client.renderer.SubmitNodeCollector[1];
            holder[0] = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("order")) {
                        return holder[0];
                    }
                    if (method.getName().equals("submitModel") && args.length == 10) {
                        var pass = new JsonObject();
                        pass.addProperty("model", args[0] == model ? "main" : args[0] == babyModel ? "main_baby"
                            : args[0] == noHat ? "no_hat" : "no_hat_baby");
                        var text = String.valueOf(args[3]);
                        var found = java.util.regex.Pattern.compile("minecraft:(textures/[a-z_/]+\\.png)").matcher(text);
                        pass.addProperty("texture", found.find() ? found.group(1) : text);
                        pass.addProperty("color", (int) args[6]);
                        passes.add(pass);
                    }
                    return null;
                });
            layer.submit(new com.mojang.blaze3d.vertex.PoseStack(), holder[0], 0, state, 0.0F, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("passes", passes);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A villager's models from the layer table (the villager, the wandering trader, grown and baby,
     * hatted or not), posed by the reference's own `VillagerModel.setupAnim`, drawn.
     */
    static JsonObject villagerModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            String id = row.get("layer").getAsString();
            var root = bakedLayer(id);
            net.minecraft.client.model.npc.VillagerModel model = id.contains("_baby")
                ? new net.minecraft.client.model.npc.BabyVillagerModel(root)
                : new net.minecraft.client.model.npc.VillagerModel(root);
            var state = new net.minecraft.client.renderer.entity.state.VillagerRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isUnhappy = row.get("unhappy").getAsBoolean();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * An illager's model from the layer table, posed by the reference's own
     * `IllagerModel.setupAnim` for an arm pose, a held item and a crossbow's charge, drawn.
     */
    static JsonObject illagerModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.illager.IllagerModel<
                net.minecraft.client.renderer.entity.state.IllagerRenderState>(bakedLayer(row.get("layer").getAsString()));
            model.getHat().visible = row.get("hat").getAsBoolean();
            var state = new net.minecraft.client.renderer.entity.state.IllagerRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.isRiding = row.get("riding").getAsBoolean();
            state.armPose = net.minecraft.world.entity.monster.illager.AbstractIllager.IllagerArmPose.valueOf(
                row.get("arm_pose").getAsString());
            state.mainArm = row.get("main_arm").getAsString().equals("left")
                ? net.minecraft.world.entity.HumanoidArm.LEFT : net.minecraft.world.entity.HumanoidArm.RIGHT;
            state.attackTime = Oracle.f32(row, "attack_time");
            state.attackAnim = state.attackTime;
            state.maxCrossbowChargeDuration = row.get("max_crossbow_charge").getAsInt();
            state.ticksUsingItem = Oracle.f32(row, "ticks_using_item");
            if (row.get("has_item").getAsBoolean()) {
                state.getMainHandItemState().ensureCapacity(1);
                state.getMainHandItemState().newLayer();
            }
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The witch's model posed by the reference's own `WitchModel.setupAnim`, drawn. */
    static JsonObject witchModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.witch.WitchModel(bakedLayer("minecraft:witch#main"));
            var state = new net.minecraft.client.renderer.entity.state.WitchRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.entityId = row.get("entity_id").getAsInt();
            state.isHoldingItem = row.get("holding_item").getAsBoolean();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The iron golem's model posed by the reference's own `IronGolemModel.setupAnim`, drawn. */
    static JsonObject ironGolemModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.animal.golem.IronGolemModel(bakedLayer("minecraft:iron_golem#main"));
            var state = new net.minecraft.client.renderer.entity.state.IronGolemRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.attackTicksRemaining = Oracle.f32(row, "attack_ticks");
            state.offerFlowerTick = row.get("offer_flower_tick").getAsInt();
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The snow golem's model posed by the reference's own `SnowGolemModel.setupAnim`, drawn. */
    static JsonObject snowGolemModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.animal.golem.SnowGolemModel(bakedLayer("minecraft:snow_golem#main"));
            var state = new net.minecraft.client.renderer.entity.state.LivingEntityRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The enderman's model posed by the reference's own `EndermanModel.setupAnim`, drawn; a
     * carried block makes the state's block non-empty through the reference's own class.
     */
    static JsonObject endermanModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.monster.enderman.EndermanModel<
                net.minecraft.client.renderer.entity.state.EndermanRenderState>(bakedLayer("minecraft:enderman#main"));
            var state = new net.minecraft.client.renderer.entity.state.EndermanRenderState();
            humanoidFields(state, row);
            state.isCreepy = row.get("creepy").getAsBoolean();
            if (row.get("carrying").getAsBoolean()) {
                state.carriedBlock.setupModel(new org.joml.Matrix4f(), false);
            }
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * What the iron golem's renderer adds to the pose while it walks (`setupRotations`), as the
     * matrix of the pose stack, and the crackiness level its health fraction gives.
     */
    static JsonObject ironGolemRender(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        var renderer = (net.minecraft.client.renderer.entity.IronGolemRenderer) UNSAFE.allocateInstance(
            net.minecraft.client.renderer.entity.IronGolemRenderer.class);
        var rotations = Reflect.method(net.minecraft.client.renderer.entity.IronGolemRenderer.class, "setupRotations",
            net.minecraft.client.renderer.entity.state.IronGolemRenderState.class,
            com.mojang.blaze3d.vertex.PoseStack.class, float.class, float.class);
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var state = new net.minecraft.client.renderer.entity.state.IronGolemRenderState();
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            rotations.invoke(renderer, state, stack, Oracle.f32(row, "body_rot"), 1.0F);
            var matrix = new float[16];
            stack.last().pose().get(matrix);
            var described = new JsonArray();
            for (float value : matrix) {
                described.add(Oracle.bits(value));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("pose", described);
            out.addProperty("crackiness",
                net.minecraft.world.entity.Crackiness.GOLEM.byFraction(Oracle.f32(row, "fraction")).name());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Where the layers that hold something on a villager, a trader or a witch put it, from the
     * reference's own layer classes: the matrix each is submitted with, from an identity entity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject villagerHeldLayers(final JsonObject request) throws Exception {
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
                        var described = new JsonArray();
                        for (float value : matrix) {
                            described.add(Oracle.bits(value));
                        }
                        poses.add(described);
                    }
                    return null;
                });
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            String kind = row.get("kind").getAsString();
            boolean baby = row.get("baby").getAsBoolean();
            if (kind.startsWith("witch")) {
                var model = new net.minecraft.client.model.monster.witch.WitchModel(bakedLayer("minecraft:witch#main"));
                var state = new net.minecraft.client.renderer.entity.state.WitchRenderState();
                state.yRot = Oracle.f32(row, "head_y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.entityId = row.get("entity_id").getAsInt();
                state.isHoldingItem = true;
                state.isHoldingPotion = kind.equals("witch_potion");
                model.setupAnim(state);
                state.heldItem.newLayer();
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
                new net.minecraft.client.renderer.entity.layers.WitchItemLayer(parent)
                    .submit(stack, collector, 0, state, 0.0F, 0.0F);
            } else {
                var model = baby
                    ? new net.minecraft.client.model.npc.BabyVillagerModel(bakedLayer("minecraft:villager_baby#main"))
                    : new net.minecraft.client.model.npc.VillagerModel(bakedLayer("minecraft:villager#main"));
                var state = new net.minecraft.client.renderer.entity.state.VillagerRenderState();
                state.yRot = Oracle.f32(row, "head_y_rot");
                state.xRot = Oracle.f32(row, "x_rot");
                state.ageInTicks = Oracle.f32(row, "age_in_ticks");
                state.isUnhappy = row.get("unhappy").getAsBoolean();
                model.setupAnim(state);
                net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
                if (kind.equals("villager_head")) {
                    state.headItem.newLayer();
                    var transforms = row.get("villager_transforms").getAsBoolean()
                        ? net.minecraft.client.renderer.entity.VillagerRenderer.CUSTOM_HEAD_TRANSFORMS
                        : net.minecraft.client.renderer.entity.layers.CustomHeadLayer.Transforms.DEFAULT;
                    new net.minecraft.client.renderer.entity.layers.CustomHeadLayer(parent, null, null, transforms)
                        .submit(stack, collector, 0, state, 0.0F, 0.0F);
                } else {
                    state.heldItem.newLayer();
                    new net.minecraft.client.renderer.entity.layers.CrossedArmsItemLayer(parent)
                        .submit(stack, collector, 0, state, 0.0F, 0.0F);
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
     * Where the layers a copper golem carries things with put them, from the reference's own layer
     * classes on the model as posed: the matrix each is submitted with, from an identity entity pose.
     */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static JsonObject copperGolemLayers(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var poses = new JsonArray();
            var holder = new net.minecraft.client.renderer.SubmitNodeCollector[1];
            holder[0] = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                EntityOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("order")) {
                        return holder[0];
                    }
                    if (method.getName().equals("submitItem") || method.getName().equals("submitBlockModel")) {
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
            var model = new net.minecraft.client.model.animal.golem.CopperGolemModel(
                bakedLayer("minecraft:copper_golem#main"));
            var state = new net.minecraft.client.renderer.entity.state.CopperGolemRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            state.copperGolemState = net.minecraft.world.entity.animal.golem.CopperGolemState.valueOf(
                row.get("action").getAsString());
            state.rightHandItemState.newLayer();
            state.leftHandItemState.newLayer();
            state.headItem.newLayer();
            fillBlock(state.blockOnAntenna);
            model.setupAnim(state);
            net.minecraft.client.renderer.entity.RenderLayerParent parent = () -> model;
            var stack = new com.mojang.blaze3d.vertex.PoseStack();
            String layer = row.get("layer").getAsString();
            if (layer.equals("hands")) {
                new net.minecraft.client.renderer.entity.layers.ItemInHandLayer(parent)
                    .submit(stack, holder[0], 0, state, 0.0F, 0.0F);
            } else if (layer.equals("head")) {
                new net.minecraft.client.renderer.entity.layers.CustomHeadLayer(parent, null, null,
                    net.minecraft.client.renderer.entity.layers.CustomHeadLayer.Transforms.DEFAULT)
                    .submit(stack, holder[0], 0, state, 0.0F, 0.0F);
            } else {
                new net.minecraft.client.renderer.entity.layers.BlockDecorationLayer(parent,
                    (java.util.function.Function<net.minecraft.client.renderer.entity.state.CopperGolemRenderState,
                        net.minecraft.client.renderer.block.BlockModelRenderState>) s -> s.blockOnAntenna,
                    (java.util.function.Consumer<com.mojang.blaze3d.vertex.PoseStack>) model::applyBlockOnAntennaTransform)
                    .submit(stack, holder[0], 0, state, 0.0F, 0.0F);
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

    /** The copper golem's model posed by the reference's own `CopperGolemModel.setupAnim`, drawn. */
    static JsonObject copperGolemModel(final JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var model = new net.minecraft.client.model.animal.golem.CopperGolemModel(
                bakedLayer("minecraft:copper_golem#main"));
            var state = new net.minecraft.client.renderer.entity.state.CopperGolemRenderState();
            state.yRot = Oracle.f32(row, "head_y_rot");
            state.xRot = Oracle.f32(row, "x_rot");
            state.walkAnimationPos = Oracle.f32(row, "walk_position");
            state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
            state.ageInTicks = Oracle.f32(row, "age_in_ticks");
            if (row.get("holds_item").getAsBoolean()) {
                state.rightHandItemState.newLayer();
            }
            var animations = new Object[][] {
                {"idle", state.idleAnimationState},
                {"get_item", state.interactionGetItem},
                {"get_no_item", state.interactionGetNoItem},
                {"drop_item", state.interactionDropItem},
                {"drop_no_item", state.interactionDropNoItem},
            };
            for (var animation : animations) {
                if (row.has((String) animation[0])) {
                    ((net.minecraft.world.entity.AnimationState) animation[1]).start(
                        row.get((String) animation[0]).getAsInt());
                }
            }
            model.setupAnim(state);
            rows.add(posedQuads(row.get("label").getAsString(), model));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
