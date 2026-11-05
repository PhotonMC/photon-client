import com.google.gson.JsonObject;

/** What the sound that follows an entity makes of it, tick by tick (`MinecartSoundInstance` and its kin). */
class MovementSoundExtras extends MovementMobExtras {
    /** A sound manager that keeps the ticking sounds it is handed, as the engine would queue them. */
    static final class QueuedSounds extends net.minecraft.client.sounds.SoundManager {
        net.minecraft.client.resources.sounds.TickableSoundInstance queued;

        QueuedSounds() {
            super(null);
        }

        /** What the entity itself plays (a sniffer's searching) is not asked here. */
        @Override
        public net.minecraft.client.sounds.SoundEngine.PlayResult play(
                net.minecraft.client.resources.sounds.SoundInstance instance) {
            return null;
        }

        @Override
        public void queueTickingSound(net.minecraft.client.resources.sounds.TickableSoundInstance instance) {
            queued = instance;
        }
    }

    /** The instance each row follows, made when the row first asks. */
    private static final java.util.IdentityHashMap<JsonObject, net.minecraft.client.resources.sounds.TickableSoundInstance>
        FOLLOWING = new java.util.IdentityHashMap<>();

    private static net.minecraft.client.resources.sounds.TickableSoundInstance made(LivingRow ctx) {
        var entity = ctx.entity();
        if (entity instanceof net.minecraft.world.entity.vehicle.minecart.AbstractMinecart minecart) {
            return new net.minecraft.client.resources.sounds.MinecartSoundInstance(minecart);
        } else if (entity instanceof net.minecraft.world.entity.animal.bee.Bee bee) {
            return ctx.row().has("aggressive_start") && ctx.row().get("aggressive_start").getAsBoolean()
                ? new net.minecraft.client.resources.sounds.BeeAggressiveSoundInstance(bee)
                : new net.minecraft.client.resources.sounds.BeeFlyingSoundInstance(bee);
        } else if (entity instanceof net.minecraft.world.entity.monster.Guardian guardian) {
            return new net.minecraft.client.resources.sounds.GuardianAttackSoundInstance(guardian);
        } else if (entity instanceof net.minecraft.world.entity.animal.sniffer.Sniffer sniffer) {
            return new net.minecraft.client.resources.sounds.SnifferSoundInstance(sniffer);
        }
        throw new IllegalArgumentException("No sound follows " + entity);
    }

    static void extraSound(LivingRow ctx, JsonObject described, int tick) throws Exception {
        // A sound that starts later (an event brings it) is not asked before.
        if (ctx.row().has("sound_from") && tick < ctx.row().get("sound_from").getAsInt()) {
            return;
        }
        var minecraft = ctx.scene().minecraft();
        // Made without its constructor, which starts an engine.
        var sounds = (QueuedSounds) unsafe.allocateInstance(QueuedSounds.class);
        setField(minecraft, "soundManager", sounds);
        var instance = FOLLOWING.computeIfAbsent(ctx.row(), row -> made(ctx));
        instance.tick();
        // A bee that changed its mind hands the buzz to the other sound, which ticks from the next tick on.
        var handedOver = sounds.queued != null;
        if (handedOver) {
            FOLLOWING.put(ctx.row(), sounds.queued);
        }
        var base = net.minecraft.client.resources.sounds.AbstractSoundInstance.class;
        described.addProperty("sound_event", instance.getIdentifier().toString());
        // The sound itself is not resolved here, so the fields are read as the instance holds them.
        described.addProperty("sound_volume", Oracle.bits(Reflect.field(base, "volume").getFloat(instance)));
        described.addProperty("sound_pitch", Oracle.bits(Reflect.field(base, "pitch").getFloat(instance)));
        described.addProperty("sound_x", Oracle.bits(instance.getX()));
        described.addProperty("sound_y", Oracle.bits(instance.getY()));
        described.addProperty("sound_z", Oracle.bits(instance.getZ()));
        described.addProperty("sound_stopped", instance.isStopped());
        described.addProperty("sound_handed_over", handedOver);
        described.addProperty("sound_can_play", instance.canPlaySound());
        described.addProperty("sound_loops", instance.isLooping());
        described.addProperty("sound_silent_start", instance.canStartSilent());
    }
}
