import com.google.gson.JsonObject;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import java.util.HexFormat;
import net.minecraft.commands.arguments.EntityAnchorArgument;
import net.minecraft.network.FriendlyByteBuf;
import com.google.gson.JsonArray;
import net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket;
import net.minecraft.network.protocol.game.ClientboundCustomChatCompletionsPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerCombatEndPacket;
import net.minecraft.network.protocol.game.ClientboundServerDataPacket;
import net.minecraft.network.protocol.game.ClientboundTagQueryPacket;
import net.minecraft.network.protocol.game.ClientboundDebugSamplePacket;
import net.minecraft.network.protocol.game.ClientboundGameTestHighlightPosPacket;
import net.minecraft.network.protocol.ping.ClientboundPongResponsePacket;
import net.minecraft.network.protocol.game.ClientboundLowDiskSpaceWarningPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerCombatEnterPacket;
import net.minecraft.network.protocol.game.ClientboundMoveVehiclePacket;
import net.minecraft.network.protocol.game.ClientboundPlayerLookAtPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerRotationPacket;
import net.minecraft.network.protocol.game.ClientboundProjectilePowerPacket;
import net.minecraft.world.entity.PositionMoveRotation;
import net.minecraft.world.entity.Relative;
import net.minecraft.world.phys.Vec3;

/** Play packets of the game protocol, read from bytes with the game's own codecs. */
final class WirePlayOracle {
    private WirePlayOracle() {
    }

    /** `play_packet`: the fields a packet's codec reads from `hex`; trailing bytes are an error, as the decoder makes them. */
    static JsonObject packetResult(JsonObject request) {
        ByteBuf buffer = Unpooled.wrappedBuffer(HexFormat.of().parseHex(request.get("hex").getAsString()));
        var result = new JsonObject();
        try {
            var input = new FriendlyByteBuf(buffer);
            var fields = switch (request.get("packet").getAsString()) {
                case "player_rotation" -> rotation(ClientboundPlayerRotationPacket.STREAM_CODEC.decode(input));
                case "player_look_at" -> lookAt(ClientboundPlayerLookAtPacket.STREAM_CODEC.decode(input));
                case "move_vehicle" -> vehicle(ClientboundMoveVehiclePacket.STREAM_CODEC.decode(input));
                case "projectile_power" -> power(ClientboundProjectilePowerPacket.STREAM_CODEC.decode(input));
                case "chunks_biomes" -> chunksBiomes(ClientboundChunksBiomesPacket.STREAM_CODEC.decode(input));
                case "server_data" -> serverData(ClientboundServerDataPacket.STREAM_CODEC.decode(buffer));
                case "tag_query" -> tagQuery(ClientboundTagQueryPacket.STREAM_CODEC.decode(input));
                case "combat_enter" -> combatEnter(ClientboundPlayerCombatEnterPacket.STREAM_CODEC.decode(input));
                case "low_disk_space" -> lowDisk(ClientboundLowDiskSpaceWarningPacket.STREAM_CODEC.decode(input));
                case "pong_response" -> pong(ClientboundPongResponsePacket.STREAM_CODEC.decode(input));
                case "debug_sample" -> debugSample(input);
                case "game_test_highlight" -> highlight(ClientboundGameTestHighlightPosPacket.STREAM_CODEC.decode(input));
                case "combat_end" -> combatEnd(ClientboundPlayerCombatEndPacket.STREAM_CODEC.decode(input));
                case "custom_chat_completions" -> completions(ClientboundCustomChatCompletionsPacket.STREAM_CODEC.decode(input));
                default -> throw new IllegalArgumentException("Unknown play packet");
            };
            if (buffer.isReadable()) {
                throw new IllegalStateException("trailing bytes");
            }
            result.addProperty("outcome", "value");
            result.add("fields", fields);
        } catch (IllegalArgumentException error) {
            throw error;
        } catch (RuntimeException error) {
            result.addProperty("outcome", "error");
        } finally {
            buffer.release();
        }
        return result;
    }

    private static JsonObject rotation(ClientboundPlayerRotationPacket packet) {
        var fields = new JsonObject();
        fields.addProperty("y_rot", Oracle.bits(packet.yRot()));
        fields.addProperty("relative_y", packet.relativeY());
        fields.addProperty("x_rot", Oracle.bits(packet.xRot()));
        fields.addProperty("relative_x", packet.relativeX());
        return fields;
    }

    private static JsonObject lookAt(ClientboundPlayerLookAtPacket packet) throws RuntimeException {
        try {
            var fields = new JsonObject();
            fields.addProperty("from", ((EntityAnchorArgument.Anchor) Reflect.get(packet, "fromAnchor")).ordinal());
            fields.addProperty("x", Oracle.bits((double) Reflect.get(packet, "x")));
            fields.addProperty("y", Oracle.bits((double) Reflect.get(packet, "y")));
            fields.addProperty("z", Oracle.bits((double) Reflect.get(packet, "z")));
            boolean atEntity = (boolean) Reflect.get(packet, "atEntity");
            fields.addProperty("at_entity", atEntity);
            if (atEntity) {
                fields.addProperty("entity", (int) Reflect.get(packet, "entity"));
                fields.addProperty("to", ((EntityAnchorArgument.Anchor) Reflect.get(packet, "toAnchor")).ordinal());
            }
            return fields;
        } catch (Exception error) {
            throw new IllegalStateException(error);
        }
    }

    private static JsonObject vehicle(ClientboundMoveVehiclePacket packet) {
        var fields = new JsonObject();
        var moving = packet.movingTo();
        fields.addProperty("x", Oracle.bits(moving.position().x));
        fields.addProperty("y", Oracle.bits(moving.position().y));
        fields.addProperty("z", Oracle.bits(moving.position().z));
        fields.addProperty("y_rot", Oracle.bits(moving.yRot()));
        fields.addProperty("x_rot", Oracle.bits(moving.xRot()));
        return fields;
    }

    private static JsonObject chunksBiomes(ClientboundChunksBiomesPacket packet) {
        var chunks = new JsonArray();
        for (var data : packet.chunkBiomeData()) {
            var chunk = new JsonObject();
            chunk.addProperty("x", data.pos().x());
            chunk.addProperty("z", data.pos().z());
            chunk.addProperty("buffer", HexFormat.of().formatHex(data.buffer()));
            chunks.add(chunk);
        }
        var fields = new JsonObject();
        fields.add("chunks", chunks);
        return fields;
    }

    private static JsonObject combatEnter(ClientboundPlayerCombatEnterPacket packet) {
        return new JsonObject();
    }

    private static JsonObject pong(ClientboundPongResponsePacket packet) {
        var fields = new JsonObject();
        fields.addProperty("time", Long.toString(packet.time()));
        return fields;
    }

    private static JsonObject debugSample(FriendlyByteBuf input) {
        // The sample type is an enum over registered debug subscriptions.
        Game26_3.bootstrap();
        var packet = ClientboundDebugSamplePacket.STREAM_CODEC.decode(input);
        var samples = new JsonArray();
        for (long value : packet.sample()) {
            samples.add(Long.toString(value));
        }
        var fields = new JsonObject();
        fields.add("sample", samples);
        fields.addProperty("type", packet.debugSampleType().ordinal());
        return fields;
    }

    private static JsonObject highlight(ClientboundGameTestHighlightPosPacket packet) {
        var fields = new JsonObject();
        fields.addProperty("absolute", packet.absolutePos().toShortString());
        fields.addProperty("relative", packet.relativePos().toShortString());
        return fields;
    }

    private static JsonObject lowDisk(ClientboundLowDiskSpaceWarningPacket packet) {
        return new JsonObject();
    }

    private static JsonObject combatEnd(ClientboundPlayerCombatEndPacket packet) {
        var fields = new JsonObject();
        try {
            fields.addProperty("duration", (Integer) Reflect.get(packet, "duration"));
        } catch (Exception error) {
            throw new IllegalStateException(error);
        }
        return fields;
    }

    private static JsonObject completions(ClientboundCustomChatCompletionsPacket packet) {
        var fields = new JsonObject();
        fields.addProperty("action", packet.action().ordinal());
        var entries = new JsonArray();
        packet.entries().forEach(entries::add);
        fields.add("entries", entries);
        return fields;
    }

    private static JsonObject serverData(ClientboundServerDataPacket packet) {
        var fields = new JsonObject();
        fields.addProperty("motd", packet.motd().getString());
        packet.iconBytes().ifPresent(icon -> fields.addProperty("icon", HexFormat.of().formatHex(icon)));
        return fields;
    }

    /** What F3+I writes of the tag: `Tag.toString` for a block, the pretty text less UUID and Pos for an entity. */
    private static JsonObject tagQuery(ClientboundTagQueryPacket packet) {
        var fields = new JsonObject();
        fields.addProperty("transaction", packet.getTransactionId());
        var tag = packet.getTag();
        if (tag != null) {
            fields.addProperty("compact", tag.toString());
            fields.addProperty("pretty", net.minecraft.nbt.NbtUtils.toPrettyComponent(tag).getString());
            tag.remove("UUID");
            tag.remove("Pos");
            fields.addProperty("entity", net.minecraft.nbt.NbtUtils.toPrettyComponent(tag).getString());
        }
        return fields;
    }

    private static JsonObject power(ClientboundProjectilePowerPacket packet) {
        var fields = new JsonObject();
        fields.addProperty("entity", packet.getId());
        fields.addProperty("power", Oracle.bits(packet.getAccelerationPower()));
        return fields;
    }

    /** An armor stand without a level, at a place: all the view code needs of an entity. */
    private static net.minecraft.world.entity.Entity fixture(Vec3 position) throws Exception {
        var entity = (net.minecraft.world.entity.Entity) Reflect.unsafe()
            .allocateInstance(net.minecraft.world.entity.decoration.ArmorStand.class);
        Reflect.set(entity, net.minecraft.world.entity.Entity.class, "position", position);
        Reflect.set(entity, net.minecraft.world.entity.Entity.class, "deltaMovement", Vec3.ZERO);
        Reflect.set(entity, net.minecraft.world.entity.Entity.class, "interpolationHandler",
            net.minecraft.world.entity.InterpolationHandler.NO_OP);
        return entity;
    }

    /** `rotate_player`: the view `handleRotatePlayer` leaves, from the one the player had. */
    static JsonObject rotatePlayerResult(JsonObject request) {
        try {
            var entity = fixture(Vec3.ZERO);
            entity.setYRot(Oracle.f32(request, "current_y"));
            entity.setXRot(Oracle.f32(request, "current_x"));
            var relatives = Relative.rotation(request.get("relative_y").getAsBoolean(), request.get("relative_x").getAsBoolean());
            var current = PositionMoveRotation.of(entity);
            var change = current.withRotation(Oracle.f32(request, "y_rot"), Oracle.f32(request, "x_rot"));
            var absolute = PositionMoveRotation.calculateAbsolute(current, change, relatives);
            entity.setYRot(absolute.yRot());
            entity.setXRot(absolute.xRot());
            entity.setOldRot();
            var result = new JsonObject();
            result.addProperty("y_rot", Oracle.bits(entity.getYRot()));
            result.addProperty("x_rot", Oracle.bits(entity.getXRot()));
            return result;
        } catch (Exception error) {
            throw new IllegalStateException(error);
        }
    }

    /** `look_at`: the view `Entity.lookAt` leaves for a player at `position` with eyes at `eye_height`. */
    static JsonObject lookAtResult(JsonObject request) {
        try {
            var position = new Vec3(
                Oracle.f64(request, "x"), Oracle.f64(request, "y"), Oracle.f64(request, "z"));
            var entity = fixture(position);
            Reflect.set(entity, net.minecraft.world.entity.Entity.class, "eyeHeight", Oracle.f32(request, "eye_height"));
            var target = new Vec3(
                Oracle.f64(request, "at_x"), Oracle.f64(request, "at_y"), Oracle.f64(request, "at_z"));
            var anchor = request.get("eyes").getAsBoolean() ? EntityAnchorArgument.Anchor.EYES : EntityAnchorArgument.Anchor.FEET;
            entity.lookAt(anchor, target);
            var result = new JsonObject();
            result.addProperty("y_rot", Oracle.bits(entity.getYRot()));
            result.addProperty("x_rot", Oracle.bits(entity.getXRot()));
            return result;
        } catch (Exception error) {
            throw new IllegalStateException(error);
        }
    }
}
