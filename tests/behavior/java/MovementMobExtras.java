import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** What a row reads back from monsters, golems and bosses after a tick. */
class MovementMobExtras extends MovementAnimalExtras {
    static void extraShulker(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var shulker = (net.minecraft.world.entity.monster.Shulker) entity;
        described.addProperty("face", shulker.getAttachFace().get3DDataValue());
        described.addProperty("color",
            shulker.getColor() == null ? 16 : shulker.getColor().getId());
        described.addProperty("peek_half", Oracle.bits(shulker.getClientPeekAmount(0.5F)));
        described.addProperty("peek_start", Oracle.bits(shulker.getClientPeekAmount(0.0F)));
        described.addProperty("peek_end", Oracle.bits(shulker.getClientPeekAmount(1.0F)));
        described.addProperty("y_rot", Oracle.bits(shulker.getYRot()));
        described.addProperty("y_body_rot", Oracle.bits(shulker.yBodyRot));
        described.addProperty("y_body_rot_old", Oracle.bits(shulker.yBodyRotO));
        described.addProperty("y_head_rot", Oracle.bits(shulker.yHeadRot));
        described.addProperty("pos_x", Oracle.bits(shulker.getX()));
        described.addProperty("pos_y", Oracle.bits(shulker.getY()));
        described.addProperty("pos_z", Oracle.bits(shulker.getZ()));
        described.addProperty("old_x", Oracle.bits(shulker.xOld));
        described.addProperty("old_y", Oracle.bits(shulker.yOld));
        described.addProperty("old_z", Oracle.bits(shulker.zOld));
        described.addProperty("riding", shulker.isPassenger());
        var shulkerBox = shulker.getBoundingBox();
        for (var side : new Object[][] {{"min_x", shulkerBox.minX}, {"min_y", shulkerBox.minY},
            {"min_z", shulkerBox.minZ}, {"max_x", shulkerBox.maxX}, {"max_y", shulkerBox.maxY},
            {"max_z", shulkerBox.maxZ}}) {
            described.addProperty("box_" + side[0], Oracle.bits((double) side[1]));
        }
        for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
            var offset = shulker.getRenderPosition(partial);
            described.addProperty("render_" + partial,
                offset == null ? "none"
                    : Oracle.bits(offset.x) + "," + Oracle.bits(offset.y) + "," + Oracle.bits(offset.z));
        }
    }

    static void extraCube(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var cube = (net.minecraft.world.entity.monster.cubemob.AbstractCubeMob) entity;
        described.addProperty("size", cube.getSize());
        described.addProperty("squish", Oracle.bits(cube.squish));
        described.addProperty("old_squish", Oracle.bits(cube.oSquish));
        described.addProperty("target_squish", Oracle.bits(cube.targetSquish));
        described.addProperty("squish_half",
            Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, cube.oSquish, cube.squish)));
        described.addProperty("y_rot", Oracle.bits(cube.getYRot()));
        described.addProperty("y_body_rot", Oracle.bits(cube.yBodyRot));
        described.addProperty("on_ground", cube.onGround());
        if (cube instanceof net.minecraft.world.entity.monster.cubemob.SulfurCube sulfur) {
            described.addProperty("fuse", sulfur.getFuse());
        }
        var box = cube.getBoundingBox();
        for (var side : new Object[][] {{"min_x", box.minX}, {"min_y", box.minY}, {"min_z", box.minZ},
            {"max_x", box.maxX}, {"max_y", box.maxY}, {"max_z", box.maxZ}}) {
            described.addProperty("box_" + side[0], Oracle.bits((double) side[1]));
        }
    }

    static void extraCopperGolem(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var golem = (net.minecraft.world.entity.animal.golem.CopperGolem) entity;
        var playing = new Object[][] {
            {"idle", golem.getIdleAnimationState()},
            {"get_item", golem.getInteractionGetItemAnimationState()},
            {"get_no_item", golem.getInteractionGetNoItemAnimationState()},
            {"drop_item", golem.getInteractionDropItemAnimationState()},
            {"drop_no_item", golem.getInteractionDropNoItemAnimationState()},
        };
        for (var animation : playing) {
            var state = (net.minecraft.world.entity.AnimationState) animation[1];
            described.addProperty(animation[0] + "_started", state.isStarted());
            described.addProperty(animation[0] + "_millis", state.getTimeInMillis(golem.tickCount));
        }
        described.addProperty("state", golem.getState().ordinal());
        described.addProperty("weather", golem.getWeatherState().ordinal());
    }

    static void extraIronGolem(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var golem = (net.minecraft.world.entity.animal.golem.IronGolem) entity;
        described.addProperty("attack_tick", golem.getAttackAnimationTick());
        described.addProperty("offer_flower_tick", golem.getOfferFlowerTick());
    }

    static void extraCreeper(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var creeper = (net.minecraft.world.entity.monster.Creeper) entity;
        for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
            described.addProperty("swelling_" + partial, Oracle.bits(creeper.getSwelling(partial)));
        }
    }

    static void extraCreaking(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var creaking = (net.minecraft.world.entity.monster.creaking.Creaking) entity;
        var states = new Object[][] {
            {"attack", creaking.attackAnimationState},
            {"invulnerable", creaking.invulnerabilityAnimationState},
            {"death", creaking.deathAnimationState},
        };
        for (var described2 : states) {
            var animation = (net.minecraft.world.entity.AnimationState) described2[1];
            described.addProperty(described2[0] + "_started", animation.isStarted());
            described.addProperty(described2[0] + "_millis",
                animation.isStarted() ? animation.getTimeInMillis((float) creaking.tickCount) : 0L);
        }
        described.addProperty("can_move", creaking.canMove());
        described.addProperty("active", creaking.isActive());
        described.addProperty("tearing_down", creaking.isTearingDown());
        described.addProperty("heart_bound", creaking.isHeartBound());
    }

    static void extraWarden(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var warden = (net.minecraft.world.entity.monster.warden.Warden) entity;
        var states = new Object[][] {
            {"roar", warden.roarAnimationState}, {"sniff", warden.sniffAnimationState},
            {"emerge", warden.emergeAnimationState}, {"digging", warden.diggingAnimationState},
            {"attack", warden.attackAnimationState}, {"sonic_boom", warden.sonicBoomAnimationState},
        };
        for (var described2 : states) {
            var animation = (net.minecraft.world.entity.AnimationState) described2[1];
            described.addProperty(described2[0] + "_started", animation.isStarted());
            described.addProperty(described2[0] + "_millis",
                animation.isStarted() ? animation.getTimeInMillis((float) warden.tickCount) : 0L);
        }
        // The pulses at the start and at the end of the tick, as they lerp.
        described.addProperty("tendril_old", Oracle.bits(warden.getTendrilAnimation(0.0F)));
        described.addProperty("tendril", Oracle.bits(warden.getTendrilAnimation(1.0F)));
        described.addProperty("heart_old", Oracle.bits(warden.getHeartAnimation(0.0F)));
        described.addProperty("heart", Oracle.bits(warden.getHeartAnimation(1.0F)));
    }

    static void extraWither(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var wither = (net.minecraft.world.entity.boss.wither.WitherBoss) entity;
        described.addProperty("invulnerable", wither.getInvulnerableTicks());
        for (int i = 0; i < 2; i++) {
            described.addProperty("x_head_" + i, Oracle.bits(wither.getHeadXRots()[i]));
            described.addProperty("y_head_" + i, Oracle.bits(wither.getHeadYRots()[i]));
        }
        described.addProperty("y_rot", Oracle.bits(wither.getYRot()));
        described.addProperty("y_body_rot", Oracle.bits(wither.yBodyRot));
    }

    static void extraEnderDragon(LivingRow ctx, JsonObject described, int tick) throws Exception {
        var entity = ctx.entity();
        var row = ctx.row();
        var dragon = (net.minecraft.world.entity.boss.enderdragon.EnderDragon) entity;
        // Found a tick in ten, by the entity's own random source: only the last tick says.
        described.addProperty("nearest_crystal", tick + 1 == row.get("tick_count").getAsInt()
            ? (dragon.nearestCrystal == null ? 0 : dragon.nearestCrystal.getId()) : -1);
        described.addProperty("flap_time", Oracle.bits(dragon.flapTime));
        described.addProperty("old_flap_time", Oracle.bits(dragon.oFlapTime));
        var samples = new JsonArray();
        for (int delay = 0; delay < 64; delay++) {
            var sample = dragon.flightHistory.get(delay);
            var pair = new JsonObject();
            pair.addProperty("y", Oracle.bits(sample.y()));
            pair.addProperty("y_rot", Oracle.bits(sample.yRot()));
            samples.add(pair);
        }
        described.add("history", samples);
        described.addProperty("y_rot", Oracle.bits(dragon.getYRot()));
        described.addProperty("dragon_death_time", dragon.dragonDeathTime);
        var parts = new JsonArray();
        for (var part : dragon.getSubEntities()) {
            parts.add(Oracle.bits(part.getX()) + Oracle.bits(part.getY()) + Oracle.bits(part.getZ()));
        }
        described.add("parts", parts);
    }

    static void extraEndCrystal(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var crystal = (net.minecraft.world.entity.boss.enderdragon.EndCrystal) entity;
        var target = crystal.getBeamTarget();
        described.addProperty("beam_target", target == null ? "none"
            : target.getX() + "," + target.getY() + "," + target.getZ());
        described.addProperty("shows_bottom", crystal.showsBottom());
    }

    static void extraGuardian(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var guardian = (net.minecraft.world.entity.monster.Guardian) entity;
        described.addProperty("in_water", guardian.isInWater());
        for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
            described.addProperty("tail_" + partial, Oracle.bits(guardian.getTailAnimation(partial)));
            described.addProperty("spikes_" + partial, Oracle.bits(guardian.getSpikesAnimation(partial)));
        }
        described.addProperty("attack_time", Oracle.bits(guardian.getClientSideAttackTime()));
        // The turn of the head and body, the flop and the draws the tick took.
        described.addProperty("y_rot", Oracle.bits(guardian.getYRot()));
        described.addProperty("x_rot", Oracle.bits(guardian.getXRot()));
        described.addProperty("head_y", Oracle.bits(guardian.getYHeadRot()));
        described.addProperty("body_y", Oracle.bits(guardian.yBodyRot));
        described.addProperty("on_ground", guardian.onGround());
        described.addProperty("pos_x", Oracle.bits(guardian.getX()));
        described.addProperty("pos_y", Oracle.bits(guardian.getY()));
        described.addProperty("pos_z", Oracle.bits(guardian.getZ()));
        described.addProperty("motion_x", Oracle.bits(guardian.getDeltaMovement().x));
        described.addProperty("motion_y", Oracle.bits(guardian.getDeltaMovement().y));
        described.addProperty("motion_z", Oracle.bits(guardian.getDeltaMovement().z));
        var randomField = net.minecraft.world.entity.Entity.class.getDeclaredField("random");
        randomField.setAccessible(true);
        var source = randomField.get(guardian);
        var seedField = source.getClass().getDeclaredField("seed");
        seedField.setAccessible(true);
        described.addProperty("random_state",
            ((java.util.concurrent.atomic.AtomicLong) seedField.get(source)).get());
    }

    static void extraAttributes(LivingRow ctx, JsonObject described) throws Exception {
        var living = ctx.living();
        described.addProperty("max_health", Oracle.bits(living.getMaxHealth()));
        described.addProperty("scale", Oracle.bits(living.getScale()));
    }
}
