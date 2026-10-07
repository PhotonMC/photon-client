import com.google.gson.JsonObject;

/** Data-value events of animals. */
class MovementAnimalEvents extends MovementPacketEvents {
    static void eventCamelData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var camel = net.minecraft.world.entity.animal.camel.Camel.class;
        if (event.has("dash")) {
            data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
                net.minecraft.world.entity.animal.camel.Camel.DASH, event.get("dash").getAsBoolean()));
        }
        if (event.has("last_pose")) {
            data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
                net.minecraft.world.entity.animal.camel.Camel.LAST_POSE_CHANGE_TICK,
                event.get("last_pose").getAsLong()));
        }
        if (event.has("pose")) {
            var pose = net.minecraft.world.entity.Entity.class.getDeclaredField("DATA_POSE");
            pose.setAccessible(true);
            data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
                (net.minecraft.network.syncher.EntityDataAccessor<net.minecraft.world.entity.Pose>) pose.get(null),
                net.minecraft.world.entity.Pose.valueOf(event.get("pose").getAsString())));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventHorseData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var flags = net.minecraft.world.entity.animal.equine.AbstractHorse.class.getDeclaredField("DATA_ID_FLAGS");
        flags.setAccessible(true);
        data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
            (net.minecraft.network.syncher.EntityDataAccessor<Byte>) flags.get(null),
            (byte) event.get("flags").getAsInt()));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventBeeData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        if (event.has("flags")) {
            data.add(beeData("DATA_FLAGS_ID", (byte) event.get("flags").getAsInt()));
        }
        if (event.has("anger_end")) {
            data.add(beeData("DATA_ANGER_END_TIME", event.get("anger_end").getAsLong()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventWolfData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var living = ctx.living();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        if (event.has("flags")) {
            data.add(wolfData(net.minecraft.world.entity.TamableAnimal.class, "DATA_FLAGS_ID",
                (byte) event.get("flags").getAsInt()));
        }
        if (event.has("interested")) {
            data.add(wolfData(net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_INTERESTED_ID",
                event.get("interested").getAsBoolean()));
        }
        if (event.has("collar")) {
            data.add(wolfData(net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_COLLAR_COLOR",
                event.get("collar").getAsInt()));
        }
        if (event.has("anger_end")) {
            data.add(wolfData(net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_ANGER_END_TIME",
                event.get("anger_end").getAsLong()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
        if (event.has("health")) {
            living.setHealth(Oracle.f32(event, "health"));
        }
    }

    static void eventStriderData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        data.add(wolfData(net.minecraft.world.entity.monster.Strider.class, "DATA_SUFFOCATING",
            event.get("suffocating").getAsBoolean()));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventPandaData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var pandaClass = net.minecraft.world.entity.animal.panda.Panda.class;
        for (var counter : new String[][] {{"unhappy", "UNHAPPY_COUNTER"}, {"sneeze", "SNEEZE_COUNTER"}, {"eat", "EAT_COUNTER"}}) {
            if (event.has(counter[0])) {
                data.add(wolfData(pandaClass, counter[1], event.get(counter[0]).getAsInt()));
            }
        }
        for (var gene : new String[][] {{"main_gene", "MAIN_GENE_ID"}, {"hidden_gene", "HIDDEN_GENE_ID"}, {"flags", "DATA_ID_FLAGS"}}) {
            if (event.has(gene[0])) {
                data.add(wolfData(pandaClass, gene[1], (byte) event.get(gene[0]).getAsInt()));
            }
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventFoxData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var foxClass = net.minecraft.world.entity.animal.fox.Fox.class;
        if (event.has("variant")) {
            data.add(wolfData(foxClass, "DATA_TYPE_ID", event.get("variant").getAsInt()));
        }
        if (event.has("flags")) {
            data.add(wolfData(foxClass, "DATA_FLAGS_ID", (byte) event.get("flags").getAsInt()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventPolarBearData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        data.add(wolfData(net.minecraft.world.entity.animal.polarbear.PolarBear.class, "DATA_STANDING_ID",
            event.get("standing").getAsBoolean()));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventCatData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var catClass = net.minecraft.world.entity.animal.feline.Cat.class;
        if (event.has("flags")) {
            data.add(wolfData(net.minecraft.world.entity.TamableAnimal.class, "DATA_FLAGS_ID",
                (byte) event.get("flags").getAsInt()));
        }
        if (event.has("lying")) {
            data.add(wolfData(catClass, "IS_LYING", event.get("lying").getAsBoolean()));
        }
        if (event.has("relaxed")) {
            data.add(wolfData(catClass, "RELAX_STATE_ONE", event.get("relaxed").getAsBoolean()));
        }
        if (event.has("collar")) {
            data.add(wolfData(catClass, "DATA_COLLAR_COLOR", event.get("collar").getAsInt()));
        }
        if (event.has("pose")) {
            var pose = net.minecraft.world.entity.Entity.class.getDeclaredField("DATA_POSE");
            pose.setAccessible(true);
            data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
                (net.minecraft.network.syncher.EntityDataAccessor<net.minecraft.world.entity.Pose>) pose.get(null),
                net.minecraft.world.entity.Pose.valueOf(event.get("pose").getAsString())));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventTurtleData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        if (event.has("has_egg")) {
            data.add(wolfData(net.minecraft.world.entity.animal.turtle.Turtle.class, "HAS_EGG",
                event.get("has_egg").getAsBoolean()));
        }
        if (event.has("laying_egg")) {
            data.add(wolfData(net.minecraft.world.entity.animal.turtle.Turtle.class, "LAYING_EGG",
                event.get("laying_egg").getAsBoolean()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventFishData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        // A salmon's size or a pufferfish's puff state, which also resize its box.
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        boolean salmon = event.get("fish").getAsString().equals("salmon");
        data.add(wolfData(salmon ? net.minecraft.world.entity.animal.fish.Salmon.class
                : net.minecraft.world.entity.animal.fish.Pufferfish.class,
            salmon ? "DATA_TYPE" : "PUFF_STATE", event.get("value").getAsInt()));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventSquidSpeed(LivingRow ctx, JsonObject event) throws Exception {
        var entity = ctx.entity();
        // What the constructor draws from an unseeded random source: fixed here.
        var speed = net.minecraft.world.entity.animal.squid.Squid.class
            .getDeclaredField("tentacleSpeed");
        speed.setAccessible(true);
        speed.setFloat(entity, Oracle.f32(event, "speed"));
    }

    static void eventRabbitIdle(LivingRow ctx, JsonObject event) throws Exception {
        var entity = ctx.entity();
        // What the reference draws from an unseeded random source: fixed here.
        var rabbit = (net.minecraft.world.entity.animal.rabbit.Rabbit) entity;
        var timeout = net.minecraft.world.entity.animal.rabbit.Rabbit.class
            .getDeclaredField("idleAnimationTimeout");
        timeout.setAccessible(true);
        timeout.setInt(rabbit, event.get("timeout").getAsInt());
        var random = net.minecraft.world.entity.Entity.class.getDeclaredField("random");
        random.setAccessible(true);
        ((net.minecraft.util.RandomSource) random.get(rabbit)).setSeed(event.get("seed").getAsLong());
    }

    static void eventAxolotlData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        if (event.has("variant")) {
            data.add(wolfData(net.minecraft.world.entity.animal.axolotl.Axolotl.class, "DATA_VARIANT",
                event.get("variant").getAsInt()));
        }
        if (event.has("playing_dead")) {
            data.add(wolfData(net.minecraft.world.entity.animal.axolotl.Axolotl.class, "DATA_PLAYING_DEAD",
                event.get("playing_dead").getAsBoolean()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventFrogData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var pose = net.minecraft.world.entity.Entity.class.getDeclaredField("DATA_POSE");
        pose.setAccessible(true);
        data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
            (net.minecraft.network.syncher.EntityDataAccessor<net.minecraft.world.entity.Pose>) pose.get(null),
            net.minecraft.world.entity.Pose.valueOf(event.get("pose").getAsString())));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventSnifferData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        data.add(wolfData(net.minecraft.world.entity.animal.sniffer.Sniffer.class, "DATA_STATE",
            net.minecraft.world.entity.animal.sniffer.Sniffer.State.values()[event.get("state").getAsInt()]));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventArmadilloData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        data.add(wolfData(net.minecraft.world.entity.animal.armadillo.Armadillo.class, "ARMADILLO_STATE",
            net.minecraft.world.entity.animal.armadillo.Armadillo.ArmadilloState.values()[event.get("state").getAsInt()]));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventGlowSquidData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        data.add(wolfData(net.minecraft.world.entity.animal.squid.GlowSquid.class,
            "DATA_DARK_TICKS_REMAINING", event.get("dark_ticks").getAsInt()));
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventFlierData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        if (event.has("parrot_variant")) {
            data.add(wolfData(net.minecraft.world.entity.animal.parrot.Parrot.class, "DATA_VARIANT_ID",
                event.get("parrot_variant").getAsInt()));
        }
        if (event.has("bat_flags")) {
            data.add(wolfData(net.minecraft.world.entity.ambient.Bat.class, "DATA_ID_FLAGS",
                (byte) event.get("bat_flags").getAsInt()));
        }
        if (event.has("vex_flags")) {
            data.add(wolfData(net.minecraft.world.entity.monster.Vex.class, "DATA_FLAGS_ID",
                (byte) event.get("vex_flags").getAsInt()));
        }
        if (event.has("allay_dancing")) {
            data.add(wolfData(net.minecraft.world.entity.animal.allay.Allay.class, "DATA_DANCING",
                event.get("allay_dancing").getAsBoolean()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }
}
