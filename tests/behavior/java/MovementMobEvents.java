import com.google.gson.JsonObject;

/** Data-value events of monsters, golems and bosses. */
class MovementMobEvents extends MovementAnimalEvents {
    static void eventCreeperData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        if (event.has("swell_dir")) {
            data.add(creeperData("DATA_SWELL_DIR", event.get("swell_dir").getAsInt()));
        }
        if (event.has("ignited")) {
            data.add(creeperData("DATA_IS_IGNITED", event.get("ignited").getAsBoolean()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventCopperGolemData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var golemClass = net.minecraft.world.entity.animal.golem.CopperGolem.class;
        if (event.has("state")) {
            data.add(wolfData(golemClass, "COPPER_GOLEM_STATE",
                net.minecraft.world.entity.animal.golem.CopperGolemState.values()[event.get("state").getAsInt()]));
        }
        if (event.has("weather")) {
            data.add(wolfData(golemClass, "DATA_WEATHER_STATE",
                net.minecraft.world.level.block.WeatheringCopper.WeatherState.values()[event.get("weather").getAsInt()]));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventCopperGolemIdle(LivingRow ctx, JsonObject event) throws Exception {
        var entity = ctx.entity();
        // What the reference draws from an unseeded random source: fixed here.
        var random = net.minecraft.world.entity.Entity.class.getDeclaredField("random");
        random.setAccessible(true);
        ((net.minecraft.util.RandomSource) random.get(entity)).setSeed(event.get("seed").getAsLong());
    }

    static void eventShulkerData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var shulkerClass = net.minecraft.world.entity.monster.Shulker.class;
        if (event.has("face")) {
            data.add(wolfData(shulkerClass, "DATA_ATTACH_FACE_ID",
                net.minecraft.core.Direction.from3DDataValue(event.get("face").getAsInt())));
        }
        if (event.has("peek")) {
            data.add(wolfData(shulkerClass, "DATA_PEEK_ID", (byte) event.get("peek").getAsInt()));
        }
        if (event.has("color")) {
            data.add(wolfData(shulkerClass, "DATA_COLOR_ID", (byte) event.get("color").getAsInt()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventSulfurFuse(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        data.add(wolfData(net.minecraft.world.entity.monster.cubemob.SulfurCube.class, "MAX_FUSE",
            event.get("fuse").getAsInt()));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventGuardianRandom(LivingRow ctx, JsonObject event) throws Exception {
        var entity = ctx.entity();
        // What the constructor draws from an unseeded random source (and what
        // the spikes out of water are drawn from): fixed here.
        var tail = net.minecraft.world.entity.monster.Guardian.class
            .getDeclaredField("clientSideTailAnimation");
        tail.setAccessible(true);
        tail.setFloat(entity, Oracle.f32(event, "tail"));
        var tailOld = net.minecraft.world.entity.monster.Guardian.class
            .getDeclaredField("clientSideTailAnimationO");
        tailOld.setAccessible(true);
        tailOld.setFloat(entity, Oracle.f32(event, "tail"));
        var random = net.minecraft.world.entity.Entity.class.getDeclaredField("random");
        random.setAccessible(true);
        ((net.minecraft.util.RandomSource) random.get(entity)).setSeed(event.get("seed").getAsLong());
    }

    static void eventGuardianData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var guardianClass = net.minecraft.world.entity.monster.Guardian.class;
        if (event.has("moving")) {
            data.add(wolfData(guardianClass, "DATA_ID_MOVING", event.get("moving").getAsBoolean()));
        }
        if (event.has("target")) {
            data.add(wolfData(guardianClass, "DATA_ID_ATTACK_TARGET", event.get("target").getAsInt()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventCreakingData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        for (var field : new String[][] {{"can_move", "CAN_MOVE"}, {"active", "IS_ACTIVE"},
                {"tearing_down", "IS_TEARING_DOWN"}}) {
            if (event.has(field[0])) {
                data.add(wolfData(net.minecraft.world.entity.monster.creaking.Creaking.class, field[1],
                    event.get(field[0]).getAsBoolean()));
            }
        }
        if (event.has("home")) {
            data.add(wolfData(net.minecraft.world.entity.monster.creaking.Creaking.class, "HOME_POS",
                java.util.Optional.of(new net.minecraft.core.BlockPos(0, 0, 0))));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventWardenData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        data.add(wolfData(net.minecraft.world.entity.monster.warden.Warden.class, "CLIENT_ANGER_LEVEL",
            event.get("anger").getAsInt()));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventWitherData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        for (var field : new String[][] {{"target_b", "DATA_TARGET_B"}, {"target_c", "DATA_TARGET_C"},
                {"invulnerable", "DATA_ID_INV"}}) {
            if (event.has(field[0])) {
                data.add(wolfData(net.minecraft.world.entity.boss.wither.WitherBoss.class, field[1],
                    event.get(field[0]).getAsInt()));
            }
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventDragonData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        data.add(wolfData(net.minecraft.world.entity.boss.enderdragon.EnderDragon.class, "DATA_PHASE",
            event.get("phase").getAsInt()));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventCrystalData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var crystalClass = net.minecraft.world.entity.boss.enderdragon.EndCrystal.class;
        if (event.has("target")) {
            var target = event.getAsJsonArray("target");
            data.add(wolfData(crystalClass, "DATA_BEAM_TARGET", java.util.Optional.of(
                new net.minecraft.core.BlockPos(target.get(0).getAsInt(), target.get(1).getAsInt(),
                    target.get(2).getAsInt()))));
        }
        if (event.has("no_target")) {
            data.add(wolfData(crystalClass, "DATA_BEAM_TARGET", java.util.Optional.empty()));
        }
        if (event.has("bottom")) {
            data.add(wolfData(crystalClass, "DATA_SHOW_BOTTOM", event.get("bottom").getAsBoolean()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }
}
