import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** A living entity, or any entity, ticked after the packets a row sends it. */
class MovementLivingOracle extends MovementObjectExtras {
    /**
     * A living entity of the reference through the packets that hurt or kill
     * it and the client ticks after them: `hurtTime`, `deathTime`, whether
     * its health is spent, and the limbs' walk animation.
     */
    static JsonObject livingEntityTicks(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        var listener = scene.listener();
        setField(scene.minecraft(), "packetProcessor",
            new net.minecraft.network.PacketProcessor(Thread.currentThread()));
        setField(listener, "level", level);
        setField(listener, "minecraft", scene.minecraft());
        // The level pushes entities against the client's player, and lets a
        // dying one fall over only near it: one a few blocks away.
        var player = new net.minecraft.client.player.LocalPlayer(
            scene.minecraft(), level, listener, null, null,
            net.minecraft.world.entity.player.Input.EMPTY, false, null);
        player.snapTo(row.has("player_x") ? Oracle.f64(row, "player_x") : 6.5, 64.0, 0.5, 0.0F, 0.0F);
        setField(scene.minecraft(), "player", player);
        // A sound an entity makes asks the renderer's camera how far it is: none is heard here.
        var soundRenderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        setField(soundRenderer, "mainCamera", new net.minecraft.client.Camera());
        setField(scene.minecraft(), "gameRenderer", soundRenderer);
        // What an entity adds to the air (a glow squid's sparks) is not drawn here.
        setField(scene.minecraft(), "particleEngine", unsafe.allocateInstance(DroppingParticleEngine.class));
        // The server's features: a minecart reads them as it is made (the new behaviour when it enabled `minecart_improvements`).
        setField(listener, "enabledFeatures", row.has("minecart_improvements")
            ? net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS.join(
                net.minecraft.world.flag.FeatureFlagSet.of(net.minecraft.world.flag.FeatureFlags.MINECART_IMPROVEMENTS))
            : net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
        // A boat asks for the teams of what it could push (no team here).
        setField(listener, "scoreboard", new net.minecraft.world.scores.Scoreboard());
        var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
            net.minecraft.resources.Identifier.parse(row.get("entity").getAsString()));
        net.minecraft.world.entity.Entity entity = type.create(level,
            new net.minecraft.world.entity.EntitySpawnRequest(
                net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
        // What a living entity has beyond an entity's own (a boat is not one).
        var living = entity instanceof net.minecraft.world.entity.LivingEntity livingEntity ? livingEntity : null;
        int entityId = row.get("entity_id").getAsInt();
        entity.setId(entityId);
        entity.snapTo(Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"), 0.0F, 0.0F);
        // A hanging entity's box comes from the block it is attached to and the way it faces
        // (`ClientboundAddEntityPacket.getData()`'s 3D direction value), set the way `setPos`
        // (public, overridden) and `setDirection` (protected) build it from a spawn.
        if (entity instanceof net.minecraft.world.entity.decoration.HangingEntity hanging) {
            hanging.setPos(entity.getX(), entity.getY(), entity.getZ());
            int data = row.has("data") ? row.get("data").getAsInt() : 0;
            var setDirection = Reflect.method(net.minecraft.world.entity.decoration.HangingEntity.class, "setDirection", net.minecraft.core.Direction.class);
            setDirection.invoke(hanging, net.minecraft.core.Direction.from3DDataValue(data));
        }
        // The constructor turns the head at random; the add-entity packet sets it (here: facing south).
        if (living != null) {
            living.yBodyRot = 0.0F;
            living.yHeadRot = 0.0F;
            living.yBodyRotO = 0.0F;
            living.yHeadRotO = 0.0F;
        }
        entity.syncPacketPositionCodec(entity.getX(), entity.getY(), entity.getZ());
        level.addEntity(entity);
        // Another entity that stays where it is: what a guardian's beam aims at.
        if (row.has("second")) {
            var second = row.getAsJsonObject("second");
            var secondType = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
                net.minecraft.resources.Identifier.parse(second.get("entity").getAsString()));
            var other = secondType.create(level, new net.minecraft.world.entity.EntitySpawnRequest(
                net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
            other.setId(second.get("entity_id").getAsInt());
            other.snapTo(Oracle.f64(second, "x"), Oracle.f64(second, "y"), Oracle.f64(second, "z"), 0.0F, 0.0F);
            if (other instanceof net.minecraft.world.entity.LivingEntity otherLiving) {
                otherLiving.yBodyRot = 0.0F;
                otherLiving.yHeadRot = 0.0F;
                otherLiving.yBodyRotO = 0.0F;
                otherLiving.yHeadRotO = 0.0F;
            }
            other.syncPacketPositionCodec(other.getX(), other.getY(), other.getZ());
            level.addEntity(other);
        }
        var ctx = new LivingRow(row, scene, level, listener, entity, living, entityId);
        var ticks = new JsonArray();
        for (int tick = 0; tick < row.get("tick_count").getAsInt(); tick++) {
            for (var element : row.getAsJsonArray("events")) {
                var event = element.getAsJsonObject();
                if (event.get("tick").getAsInt() != tick) {
                    continue;
                }
                switch (event.get("type").getAsString()) {
                    case "damage" -> eventDamage(ctx, event);
                    case "hurt" -> eventHurt(ctx, event);
                    case "death" -> eventDeath(ctx, event);
                    case "move" -> eventMove(ctx, event);
                    case "animate" -> eventAnimate(ctx, event);
                    case "creeper_data" -> eventCreeperData(ctx, event);
                    case "sync" -> eventSync(ctx, event);
                    case "game_time" -> eventGameTime(ctx, event);
                    case "camel_data" -> eventCamelData(ctx, event);
                    case "horse_data" -> eventHorseData(ctx, event);
                    case "seed" -> eventSeed(ctx, event);
                    case "bee_data" -> eventBeeData(ctx, event);
                    case "wolf_data" -> eventWolfData(ctx, event);
                    case "weather" -> eventWeather(ctx, event);
                    case "equipment" -> eventEquipment(ctx, event);
                    case "mob_flags" -> eventMobFlags(ctx, event);
                    case "strider_data" -> eventStriderData(ctx, event);
                    case "panda_data" -> eventPandaData(ctx, event);
                    case "fox_data" -> eventFoxData(ctx, event);
                    case "polar_bear_data" -> eventPolarBearData(ctx, event);
                    case "cat_data" -> eventCatData(ctx, event);
                    case "copper_golem_data" -> eventCopperGolemData(ctx, event);
                    case "entity_motion" -> eventEntityMotion(ctx, event);
                    case "glow_squid_data" -> eventGlowSquidData(ctx, event);
                    case "passengers" -> eventPassengers(ctx, event);
                    case "shulker_data" -> eventShulkerData(ctx, event);
                    case "sulfur_fuse" -> eventSulfurFuse(ctx, event);
                    case "head_rotation" -> eventHeadRotation(ctx, event);
                    case "cube_size" -> eventCubeSize(ctx, event);
                    case "baby_data" -> eventBabyData(ctx, event);
                    case "axolotl_data" -> eventAxolotlData(ctx, event);
                    case "boat_data" -> eventBoatData(ctx, event);
                    case "flier_data" -> eventFlierData(ctx, event);
                    case "guardian_random" -> eventGuardianRandom(ctx, event);
                    case "guardian_data" -> eventGuardianData(ctx, event);
                    case "frog_data" -> eventFrogData(ctx, event);
                    case "arrow_data" -> eventArrowData(ctx, event);
                    case "minecart_data" -> eventMinecartData(ctx, event);
                    case "crystal_data" -> eventCrystalData(ctx, event);
                    case "painting_data" -> eventPaintingData(ctx, event);
                    case "dragon_data" -> eventDragonData(ctx, event);
                    case "move_rot" -> eventMoveRot(ctx, event);
                    case "wither_data" -> eventWitherData(ctx, event);
                    case "sniffer_data" -> eventSnifferData(ctx, event);
                    case "armadillo_data" -> eventArmadilloData(ctx, event);
                    case "creaking_data" -> eventCreakingData(ctx, event);
                    case "warden_data" -> eventWardenData(ctx, event);
                    case "turtle_data" -> eventTurtleData(ctx, event);
                    case "fish_data" -> eventFishData(ctx, event);
                    case "squid_speed" -> eventSquidSpeed(ctx, event);
                    case "copper_golem_idle" -> eventCopperGolemIdle(ctx, event);
                    case "rabbit_idle" -> eventRabbitIdle(ctx, event);
                    case "attributes" -> eventAttributes(ctx, event);
                    case "entity_event" -> eventEntityEvent(ctx, event);
                    case "display_data" -> eventDisplayData(ctx, event);
                    case "minecart_steps" -> eventMinecartSteps(ctx, event);
                    default -> throw new IllegalArgumentException("Unknown event " + event);
                }
            }
            // The level ticks the entities that are not riding; a rider is ticked by its vehicle.
            level.tickNonPassenger(entity.isPassenger() ? entity.getRootVehicle() : entity);
            var described = new JsonObject();
            described.addProperty("hurt_time", living == null ? 0 : living.hurtTime);
            described.addProperty("death_time", living == null ? 0 : living.deathTime);
            described.addProperty("dead", living != null && living.isDeadOrDying());
            described.addProperty("walk_speed", Oracle.bits(living == null ? 0.0F : living.walkAnimation.speed()));
            described.addProperty("walk_position", Oracle.bits(living == null ? 0.0F : living.walkAnimation.position()));
            if (row.has("extras")) {
                for (var extra : row.getAsJsonArray("extras")) {
                    switch (extra.getAsString()) {
                        case "swing" -> extraSwing(ctx, described);
                        case "sheep" -> extraSheep(ctx, described);
                        case "chicken" -> extraChicken(ctx, described);
                        case "camel" -> extraCamel(ctx, described);
                        case "horse" -> extraHorse(ctx, described);
                        case "bee" -> extraBee(ctx, described);
                        case "wolf" -> extraWolf(ctx, described);
                        case "panda" -> extraPanda(ctx, described);
                        case "fox" -> extraFox(ctx, described);
                        case "strider" -> extraStrider(ctx, described);
                        case "ravager" -> extraRavager(ctx, described);
                        case "hoglin" -> extraHoglin(ctx, described);
                        case "polar_bear" -> extraPolarBear(ctx, described);
                        case "cat" -> extraCat(ctx, described);
                        case "rabbit" -> extraRabbit(ctx, described);
                        case "turtle" -> extraTurtle(ctx, described);
                        case "axolotl" -> extraAxolotl(ctx, described);
                        case "boat" -> extraBoat(ctx, described);
                        case "bat" -> extraBat(ctx, described);
                        case "breeze" -> extraBreeze(ctx, described);
                        case "arrow" -> extraArrow(ctx, described);
                        case "minecart" -> extraMinecart(ctx, described, tick);
                        case "display" -> extraDisplay(ctx, described);
                        case "end_crystal" -> extraEndCrystal(ctx, described);
                        case "ender_dragon" -> extraEnderDragon(ctx, described, tick);
                        case "wither" -> extraWither(ctx, described);
                        case "sniffer" -> extraSniffer(ctx, described);
                        case "armadillo" -> extraArmadillo(ctx, described);
                        case "creaking" -> extraCreaking(ctx, described);
                        case "warden" -> extraWarden(ctx, described);
                        case "parrot" -> extraParrot(ctx, described);
                        case "vex" -> extraVex(ctx, described);
                        case "allay" -> extraAllay(ctx, described);
                        case "guardian" -> extraGuardian(ctx, described);
                        case "frog" -> extraFrog(ctx, described);
                        case "dolphin" -> extraDolphin(ctx, described);
                        case "fish" -> extraFish(ctx, described);
                        case "shulker" -> extraShulker(ctx, described);
                        case "cube" -> extraCube(ctx, described);
                        case "squid" -> extraSquid(ctx, described);
                        case "copper_golem" -> extraCopperGolem(ctx, described);
                        case "attributes" -> extraAttributes(ctx, described);
                        case "goat" -> extraGoat(ctx, described);
                        case "iron_golem" -> extraIronGolem(ctx, described);
                        case "creeper" -> extraCreeper(ctx, described);
                        case "cull" -> extraCull(ctx, described);
                        default -> throw new IllegalArgumentException("Unknown extra " + extra);
                    }
                }
            }
            ticks.add(described);
        }
        entry.addProperty("outcome", "ticked");
        entry.add("ticks", ticks);
        return entry;
    }
}
