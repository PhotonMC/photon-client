import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.BlockPos;

/** The original client player in a scene: ticks, attacks and breaking. */
class MovementPlayerOracle extends MovementWorldOracle {
    static JsonObject localPlayerTicks(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());

        var scene = buildScene(row);
        var listener = scene.listener();
        var wire = scene.wire();
        var level = scene.level();
        var minecraft = scene.minecraft();
        var options = scene.options();

        var player = new net.minecraft.client.player.LocalPlayer(
            minecraft, level, listener, null, null,
            net.minecraft.world.entity.player.Input.EMPTY, false, null);
        // The server hands out the entity id at login; the scenario names it,
        // because the sprint command packet carries it.
        player.setId(row.get("entity_id").getAsInt());
        // The entity's random starts from nowhere in particular; the port seeds it from the id.
        player.getRandom().setSeed(player.getId());
        setField(minecraft, "player", player);
        var info = new net.minecraft.client.multiplayer.PlayerInfo(listener.getLocalGameProfile(), false);
        var gameType = net.minecraft.world.level.GameType.byName(row.get("game_mode").getAsString());
        var setMode = Reflect.method(net.minecraft.client.multiplayer.PlayerInfo.class, 
            "setGameMode", net.minecraft.world.level.GameType.class);
        setMode.invoke(info, gameType);
        setField(player, "playerInfo", info);
        var gameMode = new net.minecraft.client.multiplayer.MultiPlayerGameMode(minecraft, listener);
        gameMode.setLocalMode(gameType);
        gameMode.adjustPlayer(player);
        setField(minecraft, "gameMode", gameMode);
        var renderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        var camera = new net.minecraft.client.Camera();
        setField(renderer, "mainCamera", camera);
        setField(minecraft, "gameRenderer", renderer);
        camera.setEntity(player);
        player.input = new net.minecraft.client.player.KeyboardInput(options);
        player.snapTo(
            Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"),
            Oracle.f32(row, "y_rot"), Oracle.f32(row, "x_rot"));
        // The constructor turns a new living entity to a random facing; the
        // scene starts with head and body where the player looks instead.
        player.yHeadRot = player.getYRot();
        player.yHeadRotO = player.getYRot();
        player.yBodyRot = player.getYRot();
        player.yBodyRotO = player.getYRot();

        boolean breaking = row.has("breaking");
        var rockets = new java.util.ArrayList<net.minecraft.world.entity.Entity>();
        if (breaking) {
            // The client's own player is one of its level's entities, and
            // placing a block checks the level's entities for room.
            level.addEntity(player);
            setField(minecraft, "level", level);
            var sprite = (net.minecraft.client.renderer.texture.TextureAtlasSprite)
                unsafe.allocateInstance(BlankSprite.class);
            BlankModels.sprite = sprite;
            BlankModelManager.models = (net.minecraft.client.renderer.block.BlockStateModelSet)
                unsafe.allocateInstance(BlankModels.class);
            setField(minecraft, "modelManager", unsafe.allocateInstance(BlankModelManager.class));
            setField(minecraft, "particleEngine", unsafe.allocateInstance(DroppingParticleEngine.class));
            setField(minecraft, "blockColors", net.minecraft.client.color.block.BlockColors.createDefault());
            setField(level, "levelExtractor", unsafe.allocateInstance(NoRebuilds.class));
            setField(listener, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
            var breaking_ = row.getAsJsonObject("breaking");
            if (breaking_.has("main_hand")) {
                var item = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                    net.minecraft.resources.Identifier.parse(breaking_.get("main_hand").getAsString()));
                int count = breaking_.has("main_hand_count") ? breaking_.get("main_hand_count").getAsInt() : 1;
                var stack = new net.minecraft.world.item.ItemStack(item, count);
                if (breaking_.has("main_hand_use_cooldown")) {
                    stack.set(net.minecraft.core.component.DataComponents.USE_COOLDOWN,
                        new net.minecraft.world.item.component.UseCooldown(breaking_.get("main_hand_use_cooldown").getAsFloat()));
                }
                if (breaking_.has("main_hand_use_remainder")) {
                    stack.set(net.minecraft.core.component.DataComponents.USE_REMAINDER,
                        new net.minecraft.world.item.component.UseRemainder(new net.minecraft.world.item.ItemStackTemplate(
                            net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                                net.minecraft.resources.Identifier.parse(breaking_.get("main_hand_use_remainder").getAsString())))));
                }
                player.getInventory().setItem(0, stack);
            }
            if (breaking_.has("off_hand")) {
                var item = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                    net.minecraft.resources.Identifier.parse(breaking_.get("off_hand").getAsString()));
                player.getInventory().setItem(40, new net.minecraft.world.item.ItemStack(item));
            }
            if (breaking_.has("chest")) {
                var item = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                    net.minecraft.resources.Identifier.parse(breaking_.get("chest").getAsString()));
                player.setItemSlot(net.minecraft.world.entity.EquipmentSlot.CHEST,
                    new net.minecraft.world.item.ItemStack(item));
            }
            if (breaking_.has("rockets")) {
                // Fireworks that fly with the player, as the server's use of a rocket makes them.
                for (int i = 0; i < breaking_.get("rockets").getAsInt(); i++) {
                    var rocket = new net.minecraft.world.entity.projectile.FireworkRocketEntity(level,
                        new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.FIREWORK_ROCKET), player);
                    rocket.setId(900 + i);
                    level.addEntity(rocket);
                    rockets.add(rocket);
                }
            }
            setField(getField(minecraft, "gameRenderer"), "itemInHandRenderer",
                unsafe.allocateInstance(net.minecraft.client.renderer.ItemInHandRenderer.class));
            if (breaking_.has("effects")) {
                for (var effectElement : breaking_.getAsJsonArray("effects")) {
                    var effect = effectElement.getAsJsonObject();
                    var holder = net.minecraft.core.registries.BuiltInRegistries.MOB_EFFECT.get(
                        net.minecraft.resources.Identifier.parse(effect.get("effect").getAsString()))
                        .orElseThrow();
                    player.forceAddEffect(new net.minecraft.world.effect.MobEffectInstance(
                        holder, 1000000, effect.get("amplifier").getAsInt()), null);
                }
            }
        }
        if (row.has("entities")) {
            // Entities the server told of: made as the type makes them, put
            // where the scene says and added to the level.
            for (var element : row.getAsJsonArray("entities")) {
                var described = element.getAsJsonObject();
                var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
                    net.minecraft.resources.Identifier.parse(described.get("type").getAsString()));
                var entity = type.create(level, new net.minecraft.world.entity.EntitySpawnRequest(
                    net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
                entity.setId(described.get("id").getAsInt());
                entity.snapTo(Oracle.f64(described, "x"), Oracle.f64(described, "y"),
                    Oracle.f64(described, "z"), 0.0F, 0.0F);
                if (entity instanceof net.minecraft.world.entity.decoration.HangingEntity hanging) {
                    // The way a hanging entity faces comes with the spawn (3D direction value).
                    hanging.setPos(entity.getX(), entity.getY(), entity.getZ());
                    var setDirection = Reflect.method(net.minecraft.world.entity.decoration.HangingEntity.class, "setDirection", net.minecraft.core.Direction.class);
                    setDirection.invoke(hanging, net.minecraft.core.Direction.from3DDataValue(
                        described.has("data") ? described.get("data").getAsInt() : 0));
                }
                level.addEntity(entity);
                if (row.has("camera_entity") && row.get("camera_entity").getAsInt() == entity.getId()) {
                    // The server's SetCamera: the view is another entity's, so the player is not
                    // the controlled camera.
                    camera.setEntity(entity);
                }
            }
        }
        // A vehicle the player rides, as the server's passengers packet makes it: the player is put
        // aboard and, on a boat, turned the way the boat faces.
        net.minecraft.world.entity.Entity vehicle = null;
        if (row.has("ride")) {
            var ride = row.getAsJsonObject("ride");
            // A boat asks (and a minecart, which reads the features as it is made) for the teams of what it could push (no team here), and sounds ask the camera.
            setField(listener, "scoreboard", new net.minecraft.world.scores.Scoreboard());
            setField(listener, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
            setField(minecraft, "particleEngine", unsafe.allocateInstance(DroppingParticleEngine.class));
            var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
                net.minecraft.resources.Identifier.parse(ride.get("type").getAsString()));
            vehicle = type.create(level, new net.minecraft.world.entity.EntitySpawnRequest(
                net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
            vehicle.setId(ride.get("id").getAsInt());
            if (ride.has("saddled") && ride.get("saddled").getAsBoolean()
                    && vehicle instanceof net.minecraft.world.entity.Mob saddled) {
                saddled.setItemSlot(net.minecraft.world.entity.EquipmentSlot.SADDLE,
                    new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.SADDLE));
            }
            if (ride.has("holding")) {
                // What the rider holds in the selected slot (a stick steers a pig).
                var held = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                    net.minecraft.resources.Identifier.parse(ride.get("holding").getAsString()));
                player.getInventory().setItem(0, new net.minecraft.world.item.ItemStack(held));
            }
            if (vehicle instanceof net.minecraft.world.entity.animal.camel.Camel camel) {
                // The game's clock, and how long ago it last changed its pose (sitting counts negative).
                level.setTimeFromServer(1000L);
                if (ride.has("sit_age")) {
                    camel.setPose(net.minecraft.world.entity.Pose.SITTING);
                    camel.resetLastPoseChangeTick(-(1000L - ride.get("sit_age").getAsLong()));
                } else {
                    camel.resetLastPoseChangeTick(0L);
                }
            }
            if (vehicle instanceof net.minecraft.world.entity.animal.equine.AbstractHorse horse) {
                // A tamed horse in a saddle: the one a player controls.
                horse.setTamed(true);
                if (!ride.has("unsaddled")) {
                    horse.setItemSlot(net.minecraft.world.entity.EquipmentSlot.SADDLE,
                        new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.SADDLE));
                }
            }
            vehicle.snapTo(Oracle.f64(ride, "x"), Oracle.f64(ride, "y"), Oracle.f64(ride, "z"),
                Oracle.f32(ride, "y_rot"), 0.0F);
            if (vehicle instanceof net.minecraft.world.entity.LivingEntity living) {
                // The constructor turns the head at random; the add-entity packet sets it.
                float facing = Oracle.f32(ride, "y_rot");
                living.yBodyRot = facing;
                living.yHeadRot = facing;
                living.yBodyRotO = facing;
                living.yHeadRotO = facing;
            }
            if (ride.has("motion_x")) {
                vehicle.setDeltaMovement(
                    Oracle.f64(ride, "motion_x"), Oracle.f64(ride, "motion_y"), Oracle.f64(ride, "motion_z"));
            }
            vehicle.syncPacketPositionCodec(vehicle.getX(), vehicle.getY(), vehicle.getZ());
            level.addEntity(vehicle);
            level.addEntity(player);
            setField(minecraft, "level", level);
            player.startRiding(vehicle, true, false);
            if (vehicle instanceof net.minecraft.world.entity.vehicle.boat.AbstractBoat) {
                player.yRotO = vehicle.getYRot();
                player.setYRot(vehicle.getYRot());
                player.setYHeadRot(vehicle.getYRot());
            }
        }
        boolean pistons = row.has("block_events");
        if (pistons) {
            // Block events of pistons move the blocks around the player, and the level's block
            // entities push it: the player is one of the level's entities.
            if (!breaking && vehicle == null) {
                level.addEntity(player);
                setField(minecraft, "level", level);
                setField(minecraft, "particleEngine", unsafe.allocateInstance(DroppingParticleEngine.class));
            }
            setField(listener, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
            level.setTimeFromServer(1000L);
        }
        int tickIndex = 0;
        var startAttack = Reflect.method(net.minecraft.client.Minecraft.class, "startAttack");
        var continueAttack = Reflect.method(net.minecraft.client.Minecraft.class, 
            "continueAttack", boolean.class);

        var ticks = new JsonArray();
        for (var element : row.get("ticks").getAsJsonArray()) {
            var input = element.getAsJsonObject();
            options.keyUp.setDown(input.get("forward").getAsBoolean());
            options.keyDown.setDown(input.get("backward").getAsBoolean());
            options.keyLeft.setDown(input.get("left").getAsBoolean());
            options.keyRight.setDown(input.get("right").getAsBoolean());
            options.keyJump.setDown(input.get("jump").getAsBoolean());
            options.keyShift.setDown(input.get("sneak").getAsBoolean());
            options.keySprint.setDown(input.get("sprint").getAsBoolean());
            if (input.has("y_rot")) {
                player.setYRot(Oracle.f32(input, "y_rot"));
            }
            if (input.has("x_rot")) {
                player.setXRot(Oracle.f32(input, "x_rot"));
            }
            wire.sent.clear();
            // Sounds of the setup (and of earlier scenes) are not this tick's.
            SilentSoundManager.PLAYED.clear();
            if (pistons) {
                for (var eventElement : row.getAsJsonArray("block_events")) {
                    var event = eventElement.getAsJsonObject();
                    if (event.get("tick").getAsInt() == tickIndex) {
                        var at = new BlockPos(event.get("x").getAsInt(), event.get("y").getAsInt(),
                            event.get("z").getAsInt());
                        level.blockEvent(at, level.getBlockState(at).getBlock(), event.get("b0").getAsInt(),
                            event.get("b1").getAsInt());
                    }
                }
            }
            if (input.has("damage")) {
                var type = net.minecraft.resources.ResourceKey.create(
                    net.minecraft.core.registries.Registries.DAMAGE_TYPE,
                    net.minecraft.resources.Identifier.parse(input.get("damage").getAsString()));
                var holder = Game26_2.vanillaRegistries()
                    .lookupOrThrow(net.minecraft.core.registries.Registries.DAMAGE_TYPE).getOrThrow(type);
                player.handleDamageEvent(new net.minecraft.world.damagesource.DamageSource(holder));
            }
            if (input.has("death")) {
                player.handleEntityEvent((byte) 3);
            }
            if (input.has("finish_use")) {
                player.handleEntityEvent((byte) 9);
            }
            if (input.has("attack")) {
                attack(minecraft, level, input, startAttack, continueAttack);
            }
            // The level ticks its entities, not the entity itself: it saves
            // the old position and rotation and counts the tick first, and
            // both are read during the tick.
            level.tickNonPassenger(vehicle != null ? vehicle : player);
            // The rockets were added after the player, so the level ticks them after it.
            for (var rocket : rockets) {
                level.tickNonPassenger(rocket);
            }
            if (pistons) {
                // `Minecraft.tick`: the block entities tick after the entities, and the level's
                // clock moves on with the tick.
                level.tickBlockEntities();
                level.setTimeFromServer(level.getGameTime() + 1L);
            }
            tickIndex++;
            var described = describe(player, wire.sent);
            var rocketPositions = new JsonArray();
            for (var rocket : rockets) {
                var at = new JsonObject();
                at.addProperty("x", Oracle.bits(rocket.getX()));
                at.addProperty("y", Oracle.bits(rocket.getY()));
                at.addProperty("z", Oracle.bits(rocket.getZ()));
                rocketPositions.add(at);
            }
            described.add("rockets", rocketPositions);
            if (vehicle != null) {
                // The fall of a vehicle that lands is passed on to its passenger (its sounds); that
                // is not ported, so a ridden tick's sounds are not compared.
                described.add("sounds", new JsonArray());
                described.add("vehicle", describeVehicle(vehicle));
            }
            if (input.has("attack")) {
                described.add("breaking", breakingState(minecraft, level, gameMode, player,
                    input.getAsJsonObject("attack")));
            }
            ticks.add(described);
        }
        entry.addProperty("outcome", "ticked");
        entry.add("ticks", ticks);
        return entry;
    }

    /**
     * One tick of what `Minecraft.runTick` does before the level ticks the
     * player: the server's block packets first, then the attack key as
     * `handleKeybinds` spends it, then the miss wait counting down. The aim
     * is given, not picked: the scene names the block face or a miss.
     */
    static void attack(
        net.minecraft.client.Minecraft minecraft,
        net.minecraft.client.multiplayer.ClientLevel level,
        JsonObject input,
        java.lang.reflect.Method startAttack,
        java.lang.reflect.Method continueAttack) throws Exception {
        var attack = input.getAsJsonObject("attack");
        if (attack.has("server")) {
            for (var element : attack.getAsJsonArray("server")) {
                var event = element.getAsJsonObject();
                if (event.has("ack")) {
                    level.handleBlockChangedAck(event.get("ack").getAsInt());
                } else {
                    level.setServerVerifiedBlockState(
                        new BlockPos(event.get("x").getAsInt(), event.get("y").getAsInt(),
                            event.get("z").getAsInt()),
                        state(event), 19);
                }
            }
        }
        // `Minecraft.tick`: the right click delay counts down first.
        int rightClickDelay = (int) getField(minecraft, "rightClickDelay");
        if (rightClickDelay > 0) {
            setField(minecraft, "rightClickDelay", rightClickDelay - 1);
        }
        if (attack.has("pick")) {
            // `Minecraft.pick`: where the crosshair lands, as the player picks it.
            minecraft.hitResult = minecraft.player.raycastHitResult(1.0F, minecraft.player);
        } else if (attack.has("target")) {
            var target = attack.getAsJsonObject("target");
            var pos = new BlockPos(target.get("x").getAsInt(), target.get("y").getAsInt(),
                target.get("z").getAsInt());
            var face = net.minecraft.core.Direction.byName(target.get("face").getAsString());
            var location = target.has("location")
                ? new net.minecraft.world.phys.Vec3(
                    Oracle.f64(target.getAsJsonObject("location"), "x"),
                    Oracle.f64(target.getAsJsonObject("location"), "y"),
                    Oracle.f64(target.getAsJsonObject("location"), "z"))
                : net.minecraft.world.phys.Vec3.atCenterOf(pos);
            minecraft.hitResult = new net.minecraft.world.phys.BlockHitResult(
                location, face, pos, target.has("inside") && target.get("inside").getAsBoolean());
        } else {
            minecraft.hitResult = net.minecraft.world.phys.BlockHitResult.miss(
                net.minecraft.world.phys.Vec3.ZERO, net.minecraft.core.Direction.UP, BlockPos.ZERO);
        }
        // `handleKeybinds`: a player in the middle of a use releases it when the use key is up, and
        // spends no attack, use or pick click.
        boolean using = minecraft.player.isUsingItem();
        if (using && attack.has("use") && !attack.getAsJsonObject("use").get("down").getAsBoolean()) {
            minecraft.gameMode.releaseUsingItem(minecraft.player);
        }
        for (int click = 0; !using && click < attack.get("clicks").getAsInt(); click++) {
            startAttack.invoke(minecraft);
        }
        if (attack.has("use")) {
            var use = attack.getAsJsonObject("use");
            var startUseItem = Reflect.method(net.minecraft.client.Minecraft.class, "startUseItem");
            for (int click = 0; !using && click < use.get("clicks").getAsInt(); click++) {
                startUseItem.invoke(minecraft);
            }
            // The pick item key is spent after the use clicks, before a held use.
            if (attack.has("pick_item")) {
                var pickItem = Reflect.method(net.minecraft.client.Minecraft.class, "pickBlockOrEntity");
                for (int click = 0; !using && click < attack.getAsJsonObject("pick_item").get("clicks").getAsInt(); click++) {
                    pickItem.invoke(minecraft);
                }
            }
            if (use.get("down").getAsBoolean() && (int) getField(minecraft, "rightClickDelay") == 0
                && !minecraft.player.isUsingItem()) {
                startUseItem.invoke(minecraft);
            }
        }
        continueAttack.invoke(minecraft, attack.get("down").getAsBoolean());
        int missTime = (int) getField(minecraft, "missTime");
        if (missTime > 0) {
            setField(minecraft, "missTime", missTime - 1);
        }
    }

    /** What breaking left behind: the game mode's own state and the level's. */
    static JsonObject breakingState(
        net.minecraft.client.Minecraft minecraft,
        net.minecraft.client.multiplayer.ClientLevel level,
        net.minecraft.client.multiplayer.MultiPlayerGameMode gameMode,
        net.minecraft.client.player.LocalPlayer player,
        JsonObject attack) throws Exception {
        var out = new JsonObject();
        out.addProperty("destroying", gameMode.isDestroying());
        out.addProperty("progress", Oracle.bits((float) getField(gameMode, "destroyProgress")));
        out.addProperty("destroy_ticks", Oracle.bits((float) getField(gameMode, "destroyTicks")));
        out.addProperty("delay", (int) getField(gameMode, "destroyDelay"));
        out.addProperty("miss_time", (int) getField(minecraft, "missTime"));
        var destroying = (it.unimi.dsi.fastutil.ints.Int2ObjectMap<?>) getField(level, "destroyingBlocks");
        var entry = (net.minecraft.server.level.BlockDestructionProgress) destroying.get(player.getId());
        out.addProperty("stage", entry == null ? -1 : entry.getProgress());
        var handler = getField(level, "blockStatePredictionHandler");
        out.addProperty("sequence", (int) getField(handler, "currentSequenceNr"));
        out.addProperty("attack_strength_ticker", (int) getField(player, "attackStrengthTicker"));
        out.addProperty("right_click_delay", (int) getField(minecraft, "rightClickDelay"));
        out.addProperty("main_hand_count", player.getMainHandItem().getCount());
        out.addProperty("main_hand_item", net.minecraft.core.registries.BuiltInRegistries.ITEM
            .getKey(player.getMainHandItem().getItem()).toString());
        var contents = player.getMainHandItem().get(net.minecraft.core.component.DataComponents.POTION_CONTENTS);
        out.addProperty("main_hand_potion", contents == null ? "" : contents.potion()
            .flatMap(holder -> holder.unwrapKey()).map(key -> key.identifier().toString()).orElse(""));
        out.addProperty("main_hand_cooldown",
            !player.getMainHandItem().isEmpty() && player.getCooldowns().isOnCooldown(player.getMainHandItem()));
        out.addProperty("off_hand_count", player.getOffhandItem().getCount());
        var states = new JsonArray();
        for (var element : attack.getAsJsonArray("watch")) {
            var watched = element.getAsJsonObject();
            states.add(net.minecraft.world.level.block.Block.getId(level.getBlockState(new BlockPos(
                watched.get("x").getAsInt(), watched.get("y").getAsInt(), watched.get("z").getAsInt()))));
        }
        out.add("watched_states", states);
        return out;
    }

    /** What the level keeps of a vehicle the player rides, after a tick. */
    static JsonObject describeVehicle(net.minecraft.world.entity.Entity vehicle) throws Exception {
        var out = new JsonObject();
        out.addProperty("x", Oracle.bits(vehicle.getX()));
        out.addProperty("y", Oracle.bits(vehicle.getY()));
        out.addProperty("z", Oracle.bits(vehicle.getZ()));
        out.addProperty("motion_x", Oracle.bits(vehicle.getDeltaMovement().x));
        out.addProperty("motion_y", Oracle.bits(vehicle.getDeltaMovement().y));
        out.addProperty("motion_z", Oracle.bits(vehicle.getDeltaMovement().z));
        out.addProperty("y_rot", Oracle.bits(vehicle.getYRot()));
        out.addProperty("x_rot", Oracle.bits(vehicle.getXRot()));
        out.addProperty("on_ground", vehicle.onGround());
        out.addProperty("horizontal_collision", vehicle.horizontalCollision);
        out.addProperty("vertical_collision", vehicle.verticalCollision);
        out.addProperty("fall_distance", Oracle.bits(vehicle.fallDistance));
        if (vehicle instanceof net.minecraft.world.entity.animal.equine.AbstractHorse) {
            out.addProperty("pending_jump", Oracle.bits((float) getField(vehicle, "playerJumpPendingScale")));
            if (vehicle instanceof net.minecraft.world.entity.animal.camel.Camel camel) {
                out.addProperty("dash_cooldown", camel.getJumpCooldown());
                out.addProperty("camel_sitting", camel.isCamelSitting());
                out.addProperty("dashing", camel.isDashing());
            }
            out.addProperty("y_head_rot", Oracle.bits(((net.minecraft.world.entity.LivingEntity) vehicle).yHeadRot));
        }
        if (vehicle instanceof net.minecraft.world.entity.vehicle.boat.AbstractBoat boat) {
            out.addProperty("status", String.valueOf(getField(boat, "status")));
            out.addProperty("delta_rotation", Oracle.bits((float) getField(boat, "deltaRotation")));
            out.addProperty("paddle_left", boat.getPaddleState(0));
            out.addProperty("paddle_right", boat.getPaddleState(1));
            out.addProperty("water_level", Oracle.bits((double) getField(boat, "waterLevel")));
            out.addProperty("land_friction", Oracle.bits((float) getField(boat, "landFriction")));
            out.addProperty("last_yd", Oracle.bits((double) getField(boat, "lastYd")));
        }
        return out;
    }

    static JsonObject describe(
        net.minecraft.client.player.LocalPlayer player,
        java.util.List<net.minecraft.network.protocol.Packet<?>> sent) throws Exception {
        var tick = new JsonObject();
        tick.addProperty("x", Oracle.bits(player.getX()));
        tick.addProperty("y", Oracle.bits(player.getY()));
        tick.addProperty("z", Oracle.bits(player.getZ()));
        tick.addProperty("motion_x", Oracle.bits(player.getDeltaMovement().x));
        tick.addProperty("motion_y", Oracle.bits(player.getDeltaMovement().y));
        tick.addProperty("motion_z", Oracle.bits(player.getDeltaMovement().z));
        tick.addProperty("y_rot", Oracle.bits(player.getYRot()));
        tick.addProperty("x_rot", Oracle.bits(player.getXRot()));
        tick.addProperty("on_ground", player.onGround());
        tick.addProperty("horizontal_collision", player.horizontalCollision);
        tick.addProperty("vertical_collision", player.verticalCollision);
        tick.addProperty("minor_horizontal_collision", player.minorHorizontalCollision);
        tick.addProperty("sprinting", player.isSprinting());
        tick.addProperty("shift_key_down", player.isShiftKeyDown());
        tick.addProperty("pose", player.getPose().name());
        tick.addProperty("fall_flying", player.isFallFlying());
        tick.addProperty("fall_distance", Oracle.bits(player.fallDistance));
        tick.addProperty("water_vision", Oracle.bits(player.getWaterVision()));
        tick.addProperty("flying", player.getAbilities().flying);
        // What the view bob reads, half way through the next frame.
        var avatar = player.avatarState();
        // The portal effect that spins the view, now and a tick ago.
        tick.addProperty("portal_effect", Oracle.bits(player.portalEffectIntensity));
        tick.addProperty("portal_effect_old", Oracle.bits(player.oPortalEffectIntensity));
        tick.addProperty("bob_walk", Oracle.bits(avatar.getBackwardsInterpolatedWalkDistance(0.5F)));
        tick.addProperty("bob", Oracle.bits(avatar.getInterpolatedBob(0.5F)));
        // What the player model is posed from, half way through the frame.
        tick.addProperty("body_rot", Oracle.bits(net.minecraft.util.Mth.rotLerp(0.5F, player.yBodyRotO, player.yBodyRot)));
        tick.addProperty("head_rot", Oracle.bits(net.minecraft.util.Mth.rotLerp(0.5F, player.yHeadRotO, player.yHeadRot)));
        tick.addProperty("walk_position", Oracle.bits(player.walkAnimation.position(0.5F)));
        tick.addProperty("walk_speed", Oracle.bits(player.walkAnimation.speed(0.5F)));
        tick.addProperty("swim_amount", Oracle.bits(player.getSwimAmount(0.5F)));
        tick.addProperty("hand_x_bob", Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, player.xBobO, player.xBob)));
        tick.addProperty("hand_y_bob", Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, player.yBobO, player.yBob)));
        // The swing and the use of an item the arms are posed from.
        tick.addProperty("swinging", player.swinging);
        tick.addProperty("swing_time", player.swingTime);
        tick.addProperty("swing_off_hand", player.swingingArm == net.minecraft.world.InteractionHand.OFF_HAND);
        tick.addProperty("attack_anim", Oracle.bits(player.getAttackAnim(0.5F)));
        tick.addProperty("using_item", player.isUsingItem());
        tick.addProperty("using_off_hand", player.getUsedItemHand() == net.minecraft.world.InteractionHand.OFF_HAND);
        tick.addProperty("ticks_using_item", Oracle.bits(player.getTicksUsingItem(0.5F)));
        // The block the screen effects cover the view with (`getViewBlockingState`), as a state number.
        var viewBlock = Reflect.method(net.minecraft.client.renderer.ScreenEffectRenderer.class, 
            "getViewBlockingState", net.minecraft.world.entity.player.Player.class);
        var blocking = (net.minecraft.world.level.block.state.BlockState) viewBlock.invoke(null, player);
        tick.addProperty("view_block", blocking == null ? -1 : net.minecraft.world.level.block.Block.getId(blocking));
        var packets = new JsonArray();
        for (var packet : sent) {
            var described = new JsonObject();
            described.addProperty("type", packet.type().id().toString());
            described.addProperty("bytes", encode(packet));
            packets.add(described);
        }
        tick.add("sent", packets);
        // The sounds the tick asked for, in a fixed order (the order they were asked in is not compared).
        var sounds = new JsonArray();
        SilentSoundManager.PLAYED.stream().sorted().forEach(sounds::add);
        SilentSoundManager.PLAYED.clear();
        tick.add("sounds", sounds);
        return tick;
    }
}
