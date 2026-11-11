import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import io.netty.buffer.ByteBuf;
import net.minecraft.core.particles.ParticleOptions;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.protocol.game.ClientboundExplodePacket;
import net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket;
import net.minecraft.network.protocol.game.ClientboundLoginPacket;
import net.minecraft.network.protocol.game.ClientboundRespawnPacket;
import net.minecraft.network.protocol.game.CommonPlayerSpawnInfo;

/** Play packets that carry particles, an explosion or where the player spawns, read with the game's codecs over a registry access. */
final class WirePlayStateOracle {
    private WirePlayStateOracle() {
    }

    /** The fields of `packet` read from `buffer`, or null when this class does not read that packet. */
    static JsonObject fields(String packet, ByteBuf buffer) {
        if (!packet.equals("level_particles") && !packet.equals("explode") && !packet.equals("respawn")
            && !packet.equals("login")) {
            return null;
        }
        var input = new RegistryFriendlyByteBuf(buffer, Game26_2.vanillaRegistries());
        try {
            return decoded(packet, input);
        } catch (IllegalArgumentException error) {
            // An id the codec does not know is a bad packet, not a bad request.
            throw new IllegalStateException(error);
        }
    }

    private static JsonObject decoded(String packet, RegistryFriendlyByteBuf input) {
        return switch (packet) {
            case "level_particles" -> particles(ClientboundLevelParticlesPacket.STREAM_CODEC.decode(input));
            case "explode" -> explosion(ClientboundExplodePacket.STREAM_CODEC.decode(input));
            case "respawn" -> respawn(ClientboundRespawnPacket.STREAM_CODEC.decode(input), input);
            default -> login(ClientboundLoginPacket.STREAM_CODEC.decode(input), input);
        };
    }

    private static String name(ParticleOptions options) {
        return BuiltInRegistries.PARTICLE_TYPE.getKey(options.getType()).getPath();
    }

    private static JsonObject particles(ClientboundLevelParticlesPacket packet) {
        var fields = new JsonObject();
        fields.addProperty("particle", name(packet.getParticle()));
        fields.addProperty("override", packet.isOverrideLimiter());
        fields.addProperty("always_show", packet.alwaysShow());
        fields.addProperty("x", Oracle.bits(packet.getX()));
        fields.addProperty("y", Oracle.bits(packet.getY()));
        fields.addProperty("z", Oracle.bits(packet.getZ()));
        fields.addProperty("x_dist", Oracle.bits(packet.getXDist()));
        fields.addProperty("y_dist", Oracle.bits(packet.getYDist()));
        fields.addProperty("z_dist", Oracle.bits(packet.getZDist()));
        fields.addProperty("x_speed", Oracle.bits(packet.getMaxSpeed()));
        fields.addProperty("y_speed", Oracle.bits(packet.getMaxSpeed()));
        fields.addProperty("z_speed", Oracle.bits(packet.getMaxSpeed()));
        fields.addProperty("count", packet.getCount());
        fields.addProperty("randomization", 0);
        return fields;
    }

    private static JsonObject explosion(ClientboundExplodePacket packet) {
        var fields = new JsonObject();
        fields.addProperty("x", Oracle.bits(packet.center().x));
        fields.addProperty("y", Oracle.bits(packet.center().y));
        fields.addProperty("z", Oracle.bits(packet.center().z));
        fields.addProperty("radius", Oracle.bits(packet.radius()));
        fields.addProperty("block_count", packet.blockCount());
        packet.playerKnockback().ifPresent(push -> {
            var knockback = new JsonObject();
            knockback.addProperty("x", Oracle.bits(push.x));
            knockback.addProperty("y", Oracle.bits(push.y));
            knockback.addProperty("z", Oracle.bits(push.z));
            fields.add("knockback", knockback);
        });
        fields.addProperty("particle", name(packet.explosionParticle()));
        packet.explosionSound().unwrapKey().ifPresent(key -> fields.addProperty("sound", key.identifier().toString()));
        var blocks = new JsonArray();
        for (var weighted : packet.blockParticles().unwrap()) {
            var row = new JsonObject();
            row.addProperty("particle", name(weighted.value().particle()));
            row.addProperty("scaling", Oracle.bits(weighted.value().scaling()));
            row.addProperty("speed", Oracle.bits(weighted.value().speed()));
            row.addProperty("weight", weighted.weight());
            blocks.add(row);
        }
        fields.add("block_particles", blocks);
        fields.addProperty("play_sound", true);
        return fields;
    }

    private static JsonObject spawn(CommonPlayerSpawnInfo info, RegistryFriendlyByteBuf input) {
        var fields = new JsonObject();
        var types = input.registryAccess().lookupOrThrow(Registries.DIMENSION_TYPE);
        fields.addProperty("dimension_type", types.getId(info.dimensionType().value()));
        fields.addProperty("dimension", info.dimension().identifier().toString());
        fields.addProperty("seed", Long.toString(info.seed()));
        fields.addProperty("game_type", info.gameType().getId());
        fields.addProperty("previous_game_type", info.previousGameType() == null ? -1 : info.previousGameType().getId());
        fields.addProperty("is_debug", info.isDebug());
        fields.addProperty("is_flat", info.isFlat());
        info.lastDeathLocation().ifPresent(death -> {
            fields.addProperty("death_dimension", death.dimension().identifier().toString());
            fields.addProperty("death_position", Long.toString(death.pos().asLong()));
        });
        fields.addProperty("portal_cooldown", info.portalCooldown());
        fields.addProperty("sea_level", info.seaLevel());
        return fields;
    }

    private static JsonObject respawn(ClientboundRespawnPacket packet, RegistryFriendlyByteBuf input) {
        var fields = spawn(packet.commonPlayerSpawnInfo(), input);
        fields.addProperty("data_to_keep", packet.dataToKeep());
        return fields;
    }

    private static JsonObject login(ClientboundLoginPacket packet, RegistryFriendlyByteBuf input) {
        var fields = spawn(packet.commonPlayerSpawnInfo(), input);
        fields.addProperty("player_id", packet.playerId());
        fields.addProperty("hardcore", packet.hardcore());
        fields.addProperty("level_count", packet.levels().size());
        fields.addProperty("max_players", packet.maxPlayers());
        fields.addProperty("chunk_radius", packet.chunkRadius());
        fields.addProperty("simulation_distance", packet.simulationDistance());
        fields.addProperty("reduced_debug_info", packet.reducedDebugInfo());
        fields.addProperty("show_death_screen", packet.showDeathScreen());
        fields.addProperty("limited_crafting", packet.doLimitedCrafting());
        fields.addProperty("online_mode", packet.onlineMode());
        fields.addProperty("enforces_secure_chat", packet.enforcesSecureChat());
        return fields;
    }
}
