import com.google.gson.JsonObject;

/** Events that need no particular kind of entity: damage, motion, position and rotation packets. */
class MovementPacketEvents extends MovementLivingRow {
    static void eventDamage(LivingRow ctx, JsonObject event) throws Exception {
        var level = ctx.level();
        var listener = ctx.listener();
        var entity = ctx.entity();
        listener.handleDamageEvent(
            new net.minecraft.network.protocol.game.ClientboundDamageEventPacket(
                entity, level.damageSources().generic()));
    }

    static void eventHurt(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var living = ctx.living();
        listener.handleHurtAnimation(
            new net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket(living));
    }

    static void eventDeath(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entity = ctx.entity();
        listener.handleEntityEvent(
            new net.minecraft.network.protocol.game.ClientboundEntityEventPacket(entity, (byte) 3));
    }

    static void eventMove(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        listener.handleMoveEntity(
            new net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.Pos(entityId,
                (short) event.get("xa").getAsInt(), (short) event.get("ya").getAsInt(),
                (short) event.get("za").getAsInt(),
                !event.has("on_ground") || event.get("on_ground").getAsBoolean()));
    }

    static void eventAnimate(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entity = ctx.entity();
        listener.handleAnimate(
            new net.minecraft.network.protocol.game.ClientboundAnimatePacket(
                entity, event.get("action").getAsInt()));
    }

    static void eventSync(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        listener.handleEntityPositionSync(
            new net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket(entityId,
                new net.minecraft.world.entity.PositionMoveRotation(
                    new net.minecraft.world.phys.Vec3(Oracle.f64(event, "x"),
                        Oracle.f64(event, "y"), Oracle.f64(event, "z")),
                    new net.minecraft.world.phys.Vec3(Oracle.f64(event, "motion_x"),
                        Oracle.f64(event, "motion_y"), Oracle.f64(event, "motion_z")),
                    0.0F, 0.0F),
                !event.has("on_ground") || event.get("on_ground").getAsBoolean()));
    }

    static void eventGameTime(LivingRow ctx, JsonObject event) throws Exception {
        var level = ctx.level();
        level.setTimeFromServer(event.get("time").getAsLong());
    }

    static void eventEntityMotion(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        listener.handleSetEntityMotion(
            new net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket(entityId,
                new net.minecraft.world.phys.Vec3(Oracle.f64(event, "x"), Oracle.f64(event, "y"),
                    Oracle.f64(event, "z"))));
    }

    static void eventHeadRotation(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entity = ctx.entity();
        listener.handleRotateMob(
            new net.minecraft.network.protocol.game.ClientboundRotateHeadPacket(entity,
                (byte) event.get("angle").getAsInt()));
    }

    static void eventMoveRot(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        listener.handleMoveEntity(
            new net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.PosRot(entityId,
                (short) event.get("xa").getAsInt(), (short) event.get("ya").getAsInt(),
                (short) event.get("za").getAsInt(), (byte) event.get("y_rot").getAsInt(),
                (byte) event.get("x_rot").getAsInt(), true));
    }

    static void eventEntityEvent(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entity = ctx.entity();
        listener.handleEntityEvent(
            new net.minecraft.network.protocol.game.ClientboundEntityEventPacket(
                entity, (byte) event.get("event").getAsInt()));
    }

    static void eventAttributes(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var instances = new java.util.ArrayList<net.minecraft.world.entity.ai.attributes.AttributeInstance>();
        for (var item : event.getAsJsonArray("attributes")) {
            var changed = item.getAsJsonObject();
            var holder = net.minecraft.core.registries.BuiltInRegistries.ATTRIBUTE.get(
                net.minecraft.resources.Identifier.parse(changed.get("name").getAsString())).orElseThrow();
            var instance = new net.minecraft.world.entity.ai.attributes.AttributeInstance(holder, ignored -> {});
            instance.setBaseValue(Oracle.f64(changed, "base"));
            for (var modifier : changed.getAsJsonArray("modifiers")) {
                var described = modifier.getAsJsonObject();
                instance.addTransientModifier(new net.minecraft.world.entity.ai.attributes.AttributeModifier(
                    net.minecraft.resources.Identifier.parse(described.get("id").getAsString()),
                    Oracle.f64(described, "amount"),
                    net.minecraft.world.entity.ai.attributes.AttributeModifier.Operation.BY_ID.apply(
                        described.get("operation").getAsInt())));
            }
            instances.add(instance);
        }
        listener.handleUpdateAttributes(
            new net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket(entityId, instances));
    }

    static void eventWeather(LivingRow ctx, JsonObject event) throws Exception {
        var level = ctx.level();
        level.setRainLevel(Oracle.f32(event, "rain"));
        level.setThunderLevel(Oracle.f32(event, "thunder"));
    }

    static void eventEquipment(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var stack = event.get("item").isJsonNull()
            ? net.minecraft.world.item.ItemStack.EMPTY
            : new net.minecraft.world.item.ItemStack(net.minecraft.core.registries.BuiltInRegistries.ITEM
                .getValue(net.minecraft.resources.Identifier.parse(event.get("item").getAsString())));
        listener.handleSetEquipment(new net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket(
            entityId, java.util.List.of(com.mojang.datafixers.util.Pair.of(
                event.has("slot") && event.get("slot").getAsString().equals("head")
                    ? net.minecraft.world.entity.EquipmentSlot.HEAD
                    : net.minecraft.world.entity.EquipmentSlot.MAINHAND, stack))));
    }

    static void eventPassengers(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        // The packet as the wire carries it: the vehicle and the ids that ride it.
        var buffer = new net.minecraft.network.FriendlyByteBuf(io.netty.buffer.Unpooled.buffer());
        buffer.writeVarInt(event.get("vehicle").getAsInt());
        var riders = event.getAsJsonArray("riders");
        var ids = new int[riders.size()];
        for (int i = 0; i < ids.length; i++) {
            ids[i] = riders.get(i).getAsInt();
        }
        buffer.writeVarIntArray(ids);
        listener.handleSetEntityPassengersPacket(
            net.minecraft.network.protocol.game.ClientboundSetPassengersPacket.STREAM_CODEC.decode(buffer));
    }

    static void eventSeed(LivingRow ctx, JsonObject event) throws Exception {
        var entity = ctx.entity();
        // What the reference draws from an unseeded random source: fixed here.
        var random = Reflect.field(net.minecraft.world.entity.Entity.class, "random");
        ((net.minecraft.util.RandomSource) random.get(entity)).setSeed(event.get("seed").getAsLong());
    }

    static void eventMobFlags(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        data.add(wolfData(net.minecraft.world.entity.Mob.class, "DATA_MOB_FLAGS_ID",
            (byte) event.get("flags").getAsInt()));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventBabyData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        data.add(wolfData(net.minecraft.world.entity.AgeableMob.class, "DATA_BABY_ID",
            event.get("baby").getAsBoolean()));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventCubeSize(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        data.add(wolfData(net.minecraft.world.entity.monster.cubemob.AbstractCubeMob.class, "ID_SIZE",
            event.get("size").getAsInt()));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }
}
