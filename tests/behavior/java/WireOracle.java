import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.google.common.collect.ArrayListMultimap;
import com.mojang.authlib.GameProfile;
import com.mojang.authlib.properties.Property;
import com.mojang.authlib.properties.PropertyMap;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import io.netty.buffer.Unpooled;
import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.CorruptedFrameException;
import io.netty.handler.codec.DecoderException;
import io.netty.handler.codec.EncoderException;
import it.unimi.dsi.fastutil.ints.IntArrayList;
import it.unimi.dsi.fastutil.ints.IntList;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import net.minecraft.IdentifierException;
import net.minecraft.core.Registry;
import net.minecraft.network.Varint21FrameDecoder;
import net.minecraft.network.Varint21LengthFieldPrepender;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.VarInt;
import net.minecraft.network.VarLong;
import net.minecraft.network.protocol.handshake.ClientIntent;
import net.minecraft.network.protocol.handshake.ClientIntentionPacket;
import net.minecraft.network.protocol.login.ClientboundHelloPacket;
import net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket;
import net.minecraft.network.protocol.login.ClientboundLoginFinishedPacket;
import net.minecraft.network.protocol.login.ServerboundLoginAcknowledgedPacket;
import net.minecraft.network.protocol.login.ServerboundHelloPacket;
import net.minecraft.network.protocol.common.ClientboundKeepAlivePacket;
import net.minecraft.network.protocol.common.ClientboundPingPacket;
import net.minecraft.network.protocol.common.ServerboundKeepAlivePacket;
import net.minecraft.network.protocol.common.ServerboundPongPacket;
import net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket;
import net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket;
import net.minecraft.network.protocol.ping.ClientboundPongResponsePacket;
import net.minecraft.network.protocol.ping.ServerboundPingRequestPacket;
import net.minecraft.network.protocol.status.ServerboundStatusRequestPacket;
import net.minecraft.network.protocol.common.ServerboundClientInformationPacket;
import net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket;
import net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket;
import net.minecraft.network.protocol.configuration.ClientboundFinishConfigurationPacket;
import net.minecraft.network.protocol.configuration.ClientboundResetChatPacket;
import net.minecraft.core.RegistrySynchronization;
import net.minecraft.network.protocol.configuration.ServerboundAcceptCodeOfConductPacket;
import net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket;
import net.minecraft.server.level.ClientInformation;
import net.minecraft.server.level.ParticleStatus;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.nbt.ByteArrayTag;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.IntArrayTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.LongArrayTag;
import net.minecraft.nbt.Tag;
import net.minecraft.tags.TagNetworkSerialization;
import net.minecraft.world.entity.HumanoidArm;
import net.minecraft.world.entity.player.ChatVisiblity;

public final class WireOracle {
    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        if (operation.equals("frame_prepend")) {
            return framePrependResult(request);
        }
        if (operation.equals("frame_decode")) {
            return frameDecodeResult(request);
        }
        if (operation.equals("handshake_intention")) {
            return handshakeIntentionResult(request);
        }
        if (operation.equals("login_hello")) {
            return loginHelloResult(request);
        }
        if (operation.equals("login_compression")) {
            return loginCompressionResult(request);
        }
        if (operation.equals("login_server_hello")) {
            return loginServerHelloResult(request);
        }
        if (operation.equals("login_acknowledged")) {
            return loginAcknowledgedResult(request);
        }
        if (operation.equals("login_finished")) {
            return loginFinishedResult(request);
        }
        if (operation.equals("configuration_unit")) {
            return configurationUnitResult(request);
        }
        if (operation.equals("configuration_misc")) {
            return configurationMiscResult(request);
        }
        if (operation.equals("client_information")) {
            return clientInformationResult(request);
        }
        if (operation.equals("common_packet")) {
            return commonPacketResult(request);
        }
        if (operation.equals("enabled_features")) {
            return enabledFeaturesResult(request);
        }
        if (operation.equals("update_tags")) {
            return updateTagsResult(request);
        }
        if (operation.equals("registry_data")) {
            return registryDataResult(request);
        }
        if (operation.equals("status_request")) {
            return statusRequestResult(request);
        }
        if (operation.equals("ping")) {
            return pingResult(request);
        }
        boolean wide = switch (request.get("codec").getAsString()) {
            case "varint" -> false;
            case "varlong" -> true;
            default -> throw new IllegalArgumentException("Unknown codec");
        };
        var result = new JsonObject();
        ByteBuf buffer = Unpooled.buffer();
        try {
            if (operation.equals("write")) {
                // Parse at the declared width: invalid test input must not silently truncate.
                long value = wide ? Long.parseLong(request.get("value").getAsString())
                    : Integer.parseInt(request.get("value").getAsString());
                if (wide) {
                    VarLong.write(buffer, value);
                    result.addProperty("size", VarLong.getByteSize(value));
                } else {
                    VarInt.write(buffer, (int) value);
                    result.addProperty("size", VarInt.getByteSize((int) value));
                }
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                result.addProperty("decoded", Long.toString(wide ? VarLong.read(buffer) : VarInt.read(buffer)));
                result.addProperty("outcome", "value");
            } else if (operation.equals("read")) {
                buffer.writeBytes(java.util.HexFormat.of().parseHex(request.get("hex").getAsString()));
                try {
                    result.addProperty("value", Long.toString(wide ? VarLong.read(buffer) : VarInt.read(buffer)));
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (RuntimeException error) {
                    if (error.getClass() != RuntimeException.class) {
                        throw error;
                    }
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "too_big");
                    result.addProperty("java_exception", error.getClass().getName());
                }
            } else {
                throw new IllegalArgumentException("Unknown wire operation");
            }
            result.addProperty("consumed", buffer.readerIndex());
            result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
            return result;
        } finally {
            buffer.release();
        }
    }

    private static JsonObject framePrependResult(JsonObject request) {
        int bodyLength = request.get("body_length").getAsInt();
        int readerSkip = request.get("reader_skip").getAsInt();
        ByteBuf message = Unpooled.buffer(readerSkip + bodyLength);
        ByteBuf output = Unpooled.buffer();
        try {
            message.writeZero(readerSkip);
            for (int index = 0; index < bodyLength; index++) {
                message.writeByte(index);
            }
            message.readerIndex(readerSkip);

            var result = new JsonObject();
            try {
                new PrependerProbe().apply(message, output);
                result.addProperty("outcome", "value");
            } catch (EncoderException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", "encoder_error");
                result.addProperty("java_exception", error.getClass().getName());
            }
            result.addProperty("written", output.readableBytes());
            result.addProperty("hex", ByteBufUtil.hexDump(output));
            return result;
        } finally {
            message.release();
            output.release();
        }
    }

    private static JsonObject frameDecodeResult(JsonObject request) {
        var decoder = new DecoderProbe();
        ByteBuf input = Unpooled.buffer();
        JsonArray steps = new JsonArray();
        try {
            for (var chunk : request.getAsJsonArray("chunks")) {
                input.writeBytes(HexFormat.of().parseHex(chunk.getAsString()));
                var step = new JsonObject();
                var frames = new JsonArray();
                boolean failed = false;
                try {
                    List<Object> decoded = new ArrayList<>();
                    decoder.apply(input, decoded);
                    for (Object value : decoded) {
                        ByteBuf frame = (ByteBuf)value;
                        try {
                            frames.add(ByteBufUtil.hexDump(frame));
                        } finally {
                            frame.release();
                        }
                    }
                    step.addProperty("outcome", "value");
                } catch (CorruptedFrameException error) {
                    step.addProperty("outcome", "error");
                    step.addProperty("error", "decoder_error");
                    failed = true;
                }
                step.add("frames", frames);
                step.addProperty("consumed", input.readerIndex());
                step.addProperty("remaining_hex", ByteBufUtil.hexDump(input));
                steps.add(step);
                if (failed) {
                    break;
                }
            }
            var result = new JsonObject();
            result.add("steps", steps);
            return result;
        } finally {
            decoder.close();
            input.release();
        }
    }

    private static final class PrependerProbe extends Varint21LengthFieldPrepender {
        void apply(ByteBuf message, ByteBuf output) {
            super.encode((ChannelHandlerContext)null, message, output);
        }
    }

    private static final class DecoderProbe extends Varint21FrameDecoder {
        DecoderProbe() {
            super(null);
        }

        void apply(ByteBuf input, List<Object> output) {
            super.decode((ChannelHandlerContext)null, input, output);
        }

        void close() {
            super.handlerRemoved0((ChannelHandlerContext)null);
        }
    }

    private static JsonObject handshakeIntentionResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            int protocolVersion = request.get("protocol_version").getAsInt();
            String hostName = fromUtf16Hex(request.get("host_utf16_hex").getAsString());
            int port = request.get("port").getAsInt();
            ClientIntent intention = clientIntent(request.get("intention").getAsString());
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    ClientIntentionPacket.STREAM_CODEC.encode(
                        new FriendlyByteBuf(buffer),
                        new ClientIntentionPacket(protocolVersion, hostName, port, intention)
                    );
                    result.addProperty("outcome", "value");
                } catch (EncoderException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "encoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    ClientIntentionPacket packet = ClientIntentionPacket.STREAM_CODEC.decode(new FriendlyByteBuf(buffer));
                    result.addProperty("protocol_version", packet.protocolVersion());
                    result.addProperty("host_utf16_hex", toUtf16Hex(packet.hostName()));
                    result.addProperty("port", packet.port());
                    result.addProperty("intention", packet.intention().name().toLowerCase(Locale.ROOT));
                    result.addProperty("intention_id", packet.intention().id());
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (RuntimeException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", error.getClass() == RuntimeException.class ? "too_big" : "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown handshake direction");
    }

    private static ClientIntent clientIntent(String name) {
        return switch (name) {
            case "status" -> ClientIntent.STATUS;
            case "login" -> ClientIntent.LOGIN;
            case "transfer" -> ClientIntent.TRANSFER;
            default -> throw new IllegalArgumentException("Unknown client intent");
        };
    }

    private static JsonObject loginHelloResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            String name = fromUtf16Hex(request.get("name_utf16_hex").getAsString());
            UUID profileId = new UUID(
                Long.parseUnsignedLong(request.get("profile_most_hex").getAsString(), 16),
                Long.parseUnsignedLong(request.get("profile_least_hex").getAsString(), 16)
            );
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    ServerboundHelloPacket.STREAM_CODEC.encode(
                        new FriendlyByteBuf(buffer),
                        new ServerboundHelloPacket(name, profileId)
                    );
                    result.addProperty("outcome", "value");
                } catch (EncoderException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "encoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    ServerboundHelloPacket packet = ServerboundHelloPacket.STREAM_CODEC.decode(new FriendlyByteBuf(buffer));
                    result.addProperty("name_utf16_hex", toUtf16Hex(packet.name()));
                    result.addProperty("profile_most_hex", String.format(Locale.ROOT, "%016x", packet.profileId().getMostSignificantBits()));
                    result.addProperty("profile_least_hex", String.format(Locale.ROOT, "%016x", packet.profileId().getLeastSignificantBits()));
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (RuntimeException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", error.getClass() == RuntimeException.class ? "too_big" : "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown login direction");
    }

    private static JsonObject loginCompressionResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            int threshold = request.get("threshold").getAsInt();
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                ClientboundLoginCompressionPacket.STREAM_CODEC.encode(
                    new FriendlyByteBuf(buffer),
                    new ClientboundLoginCompressionPacket(threshold)
                );
                result.addProperty("outcome", "value");
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    ClientboundLoginCompressionPacket packet = ClientboundLoginCompressionPacket.STREAM_CODEC.decode(new FriendlyByteBuf(buffer));
                    result.addProperty("threshold", packet.getCompressionThreshold());
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (RuntimeException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", error.getClass() == RuntimeException.class ? "too_big" : "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown compression direction");
    }

    private static JsonObject loginServerHelloResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            String serverId = fromUtf16Hex(request.get("server_id_utf16_hex").getAsString());
            byte[] publicKey = HexFormat.of().parseHex(request.get("public_key_hex").getAsString());
            byte[] challenge = HexFormat.of().parseHex(request.get("challenge_hex").getAsString());
            boolean shouldAuthenticate = request.get("should_authenticate").getAsBoolean();
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    ClientboundHelloPacket.STREAM_CODEC.encode(
                        new FriendlyByteBuf(buffer),
                        new ClientboundHelloPacket(serverId, publicKey, challenge, shouldAuthenticate)
                    );
                    result.addProperty("outcome", "value");
                } catch (EncoderException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "encoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    ClientboundHelloPacket packet = ClientboundHelloPacket.STREAM_CODEC.decode(new FriendlyByteBuf(buffer));
                    result.addProperty("server_id_utf16_hex", toUtf16Hex(packet.getServerId()));
                    result.addProperty("public_key_hex", ByteBufUtil.hexDump(rawPublicKey(packet)));
                    result.addProperty("challenge_hex", ByteBufUtil.hexDump(packet.getChallenge()));
                    result.addProperty("should_authenticate", packet.shouldAuthenticate());
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (RuntimeException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", error.getClass() == RuntimeException.class ? "too_big" : "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown server hello direction");
    }

    private static JsonObject loginAcknowledgedResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                ServerboundLoginAcknowledgedPacket.STREAM_CODEC.encode(
                    buffer, ServerboundLoginAcknowledgedPacket.INSTANCE
                );
                var result = new JsonObject();
                result.addProperty("outcome", "value");
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                ServerboundLoginAcknowledgedPacket packet = ServerboundLoginAcknowledgedPacket.STREAM_CODEC.decode(buffer);
                var result = new JsonObject();
                result.addProperty("singleton", packet == ServerboundLoginAcknowledgedPacket.INSTANCE);
                result.addProperty("outcome", "value");
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown login acknowledgement direction");
    }

    private static JsonObject loginFinishedResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            GameProfile profile = profileFromRequest(request);
            UUID sessionId = uuidFromHex(request, "session");
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    ClientboundLoginFinishedPacket.STREAM_CODEC.encode(
                        buffer, new ClientboundLoginFinishedPacket(profile, sessionId)
                    );
                    result.addProperty("outcome", "value");
                } catch (EncoderException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "encoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    ClientboundLoginFinishedPacket packet = ClientboundLoginFinishedPacket.STREAM_CODEC.decode(buffer);
                    GameProfile profile = packet.gameProfile();
                    result.addProperty("profile_most_hex", String.format(Locale.ROOT, "%016x", profile.id().getMostSignificantBits()));
                    result.addProperty("profile_least_hex", String.format(Locale.ROOT, "%016x", profile.id().getLeastSignificantBits()));
                    result.addProperty("name_utf16_hex", toUtf16Hex(profile.name()));
                    result.add("properties", propertiesToJson(profile.properties()));
                    result.addProperty("session_most_hex", String.format(Locale.ROOT, "%016x", packet.sessionId().getMostSignificantBits()));
                    result.addProperty("session_least_hex", String.format(Locale.ROOT, "%016x", packet.sessionId().getLeastSignificantBits()));
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (RuntimeException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", error.getClass() == RuntimeException.class ? "too_big" : "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown login finished direction");
    }

    private static JsonObject configurationUnitResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        String packet = request.get("packet").getAsString();
        if (direction.equals("write")) {
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                switch (packet) {
                    case "server-finish" -> ServerboundFinishConfigurationPacket.STREAM_CODEC.encode(
                        buffer, ServerboundFinishConfigurationPacket.INSTANCE
                    );
                    case "client-finish" -> ClientboundFinishConfigurationPacket.STREAM_CODEC.encode(
                        buffer, ClientboundFinishConfigurationPacket.INSTANCE
                    );
                    case "accept-code-of-conduct" -> ServerboundAcceptCodeOfConductPacket.STREAM_CODEC.encode(
                        buffer, ServerboundAcceptCodeOfConductPacket.INSTANCE
                    );
                    default -> throw new IllegalArgumentException("Unknown configuration unit packet");
                }
                var result = new JsonObject();
                result.addProperty("outcome", "value");
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                boolean singleton = switch (packet) {
                    case "server-finish" -> ServerboundFinishConfigurationPacket.STREAM_CODEC.decode(buffer)
                        == ServerboundFinishConfigurationPacket.INSTANCE;
                    case "client-finish" -> ClientboundFinishConfigurationPacket.STREAM_CODEC.decode(buffer)
                        == ClientboundFinishConfigurationPacket.INSTANCE;
                    case "accept-code-of-conduct" -> ServerboundAcceptCodeOfConductPacket.STREAM_CODEC.decode(buffer)
                        == ServerboundAcceptCodeOfConductPacket.INSTANCE;
                    default -> throw new IllegalArgumentException("Unknown configuration unit packet");
                };
                var result = new JsonObject();
                result.addProperty("singleton", singleton);
                result.addProperty("outcome", "value");
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown configuration unit direction");
    }

    private static JsonObject configurationMiscResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        String packet = request.get("packet").getAsString();
        if (direction.equals("write")) {
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    switch (packet) {
                        case "code-of-conduct" -> ClientboundCodeOfConductPacket.STREAM_CODEC.encode(
                            buffer, new ClientboundCodeOfConductPacket(
                                fromUtf16Hex(request.get("code_of_conduct_utf16_hex").getAsString())
                            )
                        );
                        case "reset-chat" -> ClientboundResetChatPacket.STREAM_CODEC.encode(
                            buffer, ClientboundResetChatPacket.INSTANCE
                        );
                        default -> throw new IllegalArgumentException("Unknown configuration misc packet");
                    }
                    result.addProperty("outcome", "value");
                } catch (EncoderException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "encoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    if (packet.equals("code-of-conduct")) {
                        ClientboundCodeOfConductPacket decoded = ClientboundCodeOfConductPacket.STREAM_CODEC
                            .decode(buffer);
                        result.addProperty("code_of_conduct_utf16_hex", toUtf16Hex(decoded.codeOfConduct()));
                    } else if (packet.equals("reset-chat")) {
                        result.addProperty(
                            "singleton", ClientboundResetChatPacket.STREAM_CODEC.decode(buffer)
                                == ClientboundResetChatPacket.INSTANCE
                        );
                    } else {
                        throw new IllegalArgumentException("Unknown configuration misc packet");
                    }
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (RuntimeException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", error.getClass() == RuntimeException.class ? "too_big" : "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown configuration misc direction");
    }

    private static JsonObject clientInformationResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            ClientInformation information = new ClientInformation(
                fromUtf16Hex(request.get("language_utf16_hex").getAsString()),
                request.get("view_distance").getAsInt(),
                chatVisibility(request.get("chat_visibility").getAsString()),
                request.get("chat_colors").getAsBoolean(),
                request.get("model_customisation").getAsInt(),
                humanoidArm(request.get("main_hand").getAsString()),
                request.get("text_filtering_enabled").getAsBoolean(),
                request.get("allows_listing").getAsBoolean(),
                particleStatus(request.get("particle_status").getAsString())
            );
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    ServerboundClientInformationPacket.STREAM_CODEC.encode(
                        new FriendlyByteBuf(buffer), new ServerboundClientInformationPacket(information)
                    );
                    result.addProperty("outcome", "value");
                } catch (EncoderException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "encoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    ClientInformation information = ServerboundClientInformationPacket.STREAM_CODEC
                        .decode(new FriendlyByteBuf(buffer)).information();
                    result.addProperty("language_utf16_hex", toUtf16Hex(information.language()));
                    result.addProperty("view_distance", information.viewDistance());
                    result.addProperty("chat_visibility", information.chatVisibility().name().toLowerCase(Locale.ROOT));
                    result.addProperty("chat_colors", information.chatColors());
                    result.addProperty("model_customisation", information.modelCustomisation());
                    result.addProperty("main_hand", information.mainHand().name().toLowerCase(Locale.ROOT));
                    result.addProperty("text_filtering_enabled", information.textFilteringEnabled());
                    result.addProperty("allows_listing", information.allowsListing());
                    result.addProperty("particle_status", information.particleStatus().name().toLowerCase(Locale.ROOT));
                    result.addProperty("outcome", "value");
                } catch (ArrayIndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (RuntimeException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", error.getClass() == RuntimeException.class ? "too_big" : "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown client information direction");
    }

    private static ChatVisiblity chatVisibility(String value) {
        return switch (value) {
            case "full" -> ChatVisiblity.FULL;
            case "system" -> ChatVisiblity.SYSTEM;
            case "hidden" -> ChatVisiblity.HIDDEN;
            default -> throw new IllegalArgumentException("Unknown chat visibility");
        };
    }

    private static HumanoidArm humanoidArm(String value) {
        return switch (value) {
            case "left" -> HumanoidArm.LEFT;
            case "right" -> HumanoidArm.RIGHT;
            default -> throw new IllegalArgumentException("Unknown main hand");
        };
    }

    private static ParticleStatus particleStatus(String value) {
        return switch (value) {
            case "all" -> ParticleStatus.ALL;
            case "decreased" -> ParticleStatus.DECREASED;
            case "minimal" -> ParticleStatus.MINIMAL;
            default -> throw new IllegalArgumentException("Unknown particle status");
        };
    }

    private static GameProfile profileFromRequest(JsonObject request) {
        ArrayListMultimap<String, Property> properties = ArrayListMultimap.create();
        for (var element : request.getAsJsonArray("properties")) {
            JsonObject value = element.getAsJsonObject();
            String name = fromUtf16Hex(value.get("name_utf16_hex").getAsString());
            String propertyValue = fromUtf16Hex(value.get("value_utf16_hex").getAsString());
            JsonElement signatureValue = value.get("signature_utf16_hex");
            Property property = signatureValue == null || signatureValue.isJsonNull()
                ? new Property(name, propertyValue)
                : new Property(name, propertyValue, fromUtf16Hex(signatureValue.getAsString()));
            properties.put(name, property);
        }
        return new GameProfile(
            uuidFromHex(request, "profile"),
            fromUtf16Hex(request.get("name_utf16_hex").getAsString()),
            new PropertyMap(properties)
        );
    }

    private static UUID uuidFromHex(JsonObject request, String prefix) {
        return new UUID(
            Long.parseUnsignedLong(request.get(prefix + "_most_hex").getAsString(), 16),
            Long.parseUnsignedLong(request.get(prefix + "_least_hex").getAsString(), 16)
        );
    }

    private static JsonArray propertiesToJson(PropertyMap properties) {
        JsonArray result = new JsonArray();
        for (Property property : properties.values()) {
            var value = new JsonObject();
            value.addProperty("name_utf16_hex", toUtf16Hex(property.name()));
            value.addProperty("value_utf16_hex", toUtf16Hex(property.value()));
            if (property.signature() == null) {
                value.add("signature_utf16_hex", com.google.gson.JsonNull.INSTANCE);
            } else {
                value.addProperty("signature_utf16_hex", toUtf16Hex(property.signature()));
            }
            result.add(value);
        }
        return result;
    }

    private static JsonObject statusRequestResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                ServerboundStatusRequestPacket.STREAM_CODEC.encode(buffer, ServerboundStatusRequestPacket.INSTANCE);
                var result = new JsonObject();
                result.addProperty("outcome", "value");
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                ServerboundStatusRequestPacket packet = ServerboundStatusRequestPacket.STREAM_CODEC.decode(buffer);
                result.addProperty("singleton", packet == ServerboundStatusRequestPacket.INSTANCE);
                result.addProperty("outcome", "value");
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown status request direction");
    }

    private static JsonObject commonPacketResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        String packet = request.get("packet").getAsString();
        if (direction.equals("write")) {
            long id = Long.parseLong(request.get("id").getAsString());
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                FriendlyByteBuf friendly = new FriendlyByteBuf(buffer);
                switch (packet) {
                    case "client-ping" -> ClientboundPingPacket.STREAM_CODEC.encode(
                        friendly, new ClientboundPingPacket(Math.toIntExact(id))
                    );
                    case "server-pong" -> ServerboundPongPacket.STREAM_CODEC.encode(
                        friendly, new ServerboundPongPacket(Math.toIntExact(id))
                    );
                    case "client-keepalive" -> ClientboundKeepAlivePacket.STREAM_CODEC.encode(
                        friendly, new ClientboundKeepAlivePacket(id)
                    );
                    case "server-keepalive" -> ServerboundKeepAlivePacket.STREAM_CODEC.encode(
                        friendly, new ServerboundKeepAlivePacket(id)
                    );
                    default -> throw new IllegalArgumentException("Unknown common packet");
                }
                var result = new JsonObject();
                result.addProperty("outcome", "value");
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    long id = switch (packet) {
                        case "client-ping" -> ClientboundPingPacket.STREAM_CODEC
                            .decode(new FriendlyByteBuf(buffer)).getId();
                        case "server-pong" -> ServerboundPongPacket.STREAM_CODEC
                            .decode(new FriendlyByteBuf(buffer)).getId();
                        case "client-keepalive" -> ClientboundKeepAlivePacket.STREAM_CODEC
                            .decode(new FriendlyByteBuf(buffer)).getId();
                        case "server-keepalive" -> ServerboundKeepAlivePacket.STREAM_CODEC
                            .decode(new FriendlyByteBuf(buffer)).getId();
                        default -> throw new IllegalArgumentException("Unknown common packet");
                    };
                    result.addProperty("id", Long.toString(id));
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown common packet direction");
    }

    private static JsonObject enabledFeaturesResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            Set<net.minecraft.resources.Identifier> features = new HashSet<>();
            for (JsonElement element : request.getAsJsonArray("features")) {
                features.add(net.minecraft.resources.Identifier.parse(element.getAsString()));
            }
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    ClientboundUpdateEnabledFeaturesPacket.STREAM_CODEC.encode(
                        new FriendlyByteBuf(buffer), new ClientboundUpdateEnabledFeaturesPacket(features)
                    );
                    result.addProperty("outcome", "value");
                } catch (EncoderException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "encoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    Set<net.minecraft.resources.Identifier> features = ClientboundUpdateEnabledFeaturesPacket.STREAM_CODEC
                        .decode(new FriendlyByteBuf(buffer)).features();
                    var values = new JsonArray();
                    for (net.minecraft.resources.Identifier feature : features) {
                        values.add(feature.toString());
                    }
                    result.add("features", values);
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (DecoderException | IdentifierException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (RuntimeException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", error.getClass() == RuntimeException.class ? "too_big" : "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown enabled features direction");
    }

    private static JsonObject updateTagsResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            Map<ResourceKey<? extends Registry<?>>, TagNetworkSerialization.NetworkPayload> registries = new HashMap<>();
            for (JsonElement registryElement : request.getAsJsonArray("registries")) {
                JsonObject registryValue = registryElement.getAsJsonObject();
                Map<Identifier, IntList> tags = new HashMap<>();
                for (JsonElement tagElement : registryValue.getAsJsonArray("tags")) {
                    JsonObject tagValue = tagElement.getAsJsonObject();
                    IntList values = new IntArrayList();
                    for (JsonElement value : tagValue.getAsJsonArray("values")) {
                        values.add(value.getAsInt());
                    }
                    tags.put(Identifier.parse(tagValue.get("id").getAsString()), values);
                }
                Identifier registry = Identifier.parse(registryValue.get("registry").getAsString());
                registries.put(ResourceKey.createRegistryKey(registry), new TagNetworkSerialization.NetworkPayload(tags));
            }
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    ClientboundUpdateTagsPacket.STREAM_CODEC.encode(
                        new FriendlyByteBuf(buffer), new ClientboundUpdateTagsPacket(registries)
                    );
                    result.addProperty("outcome", "value");
                } catch (EncoderException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "encoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    Map<ResourceKey<? extends Registry<?>>, TagNetworkSerialization.NetworkPayload> registries = ClientboundUpdateTagsPacket.STREAM_CODEC
                        .decode(new FriendlyByteBuf(buffer)).getTags();
                    var registryValues = new JsonArray();
                    for (var registryEntry : registries.entrySet()) {
                        var registryValue = new JsonObject();
                        registryValue.addProperty("registry", registryEntry.getKey().identifier().toString());
                        var tagValues = new JsonArray();
                        for (var tagEntry : networkPayloadTags(registryEntry.getValue()).entrySet()) {
                            var tagValue = new JsonObject();
                            tagValue.addProperty("id", tagEntry.getKey().toString());
                            var values = new JsonArray();
                            tagEntry.getValue().forEach(value -> values.add(value));
                            tagValue.add("values", values);
                            tagValues.add(tagValue);
                        }
                        registryValue.add("tags", tagValues);
                        registryValues.add(registryValue);
                    }
                    result.add("registries", registryValues);
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (DecoderException | IdentifierException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (RuntimeException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", error.getClass() == RuntimeException.class ? "too_big" : "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown update tags direction");
    }

    private static JsonObject registryDataResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            Identifier registryId = Identifier.parse(request.get("registry").getAsString());
            List<RegistrySynchronization.PackedRegistryEntry> entries = new ArrayList<>();
            for (JsonElement element : request.getAsJsonArray("entries")) {
                JsonObject entry = element.getAsJsonObject();
                Identifier id = Identifier.parse(entry.get("id").getAsString());
                JsonElement data = entry.get("data");
                Optional<Tag> tag = data == null || data.isJsonNull()
                    ? Optional.empty()
                    : Optional.of(buildTag(data.getAsJsonObject()));
                entries.add(new RegistrySynchronization.PackedRegistryEntry(id, tag));
            }
            ResourceKey<? extends Registry<?>> registry = ResourceKey.createRegistryKey(registryId);
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    ClientboundRegistryDataPacket.STREAM_CODEC.encode(
                        new FriendlyByteBuf(buffer), new ClientboundRegistryDataPacket(registry, entries)
                    );
                    result.addProperty("outcome", "value");
                } catch (EncoderException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "encoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    ClientboundRegistryDataPacket packet = ClientboundRegistryDataPacket.STREAM_CODEC
                        .decode(new FriendlyByteBuf(buffer));
                    result.addProperty("registry", packet.registry().identifier().toString());
                    var entryValues = new JsonArray();
                    for (RegistrySynchronization.PackedRegistryEntry entry : packet.entries()) {
                        var entryValue = new JsonObject();
                        entryValue.addProperty("id", entry.id().toString());
                        if (entry.data().isPresent()) {
                            entryValue.add("data", describeTag(entry.data().get()));
                        } else {
                            entryValue.add("data", JsonNull.INSTANCE);
                        }
                        entryValues.add(entryValue);
                    }
                    result.add("entries", entryValues);
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (EncoderException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "encoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (DecoderException | IdentifierException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                } catch (RuntimeException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", error.getClass() == RuntimeException.class ? "too_big" : "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown registry data direction");
    }

    private static Tag buildTag(JsonObject node) {
        return switch (node.get("type").getAsString()) {
            case "byte" -> net.minecraft.nbt.ByteTag.valueOf(Byte.parseByte(node.get("value").getAsString()));
            case "short" -> net.minecraft.nbt.ShortTag.valueOf(Short.parseShort(node.get("value").getAsString()));
            case "int" -> net.minecraft.nbt.IntTag.valueOf(Integer.parseInt(node.get("value").getAsString()));
            case "long" -> net.minecraft.nbt.LongTag.valueOf(Long.parseLong(node.get("value").getAsString()));
            case "float" -> net.minecraft.nbt.FloatTag.valueOf(Float.intBitsToFloat(
                Integer.parseUnsignedInt(node.get("value").getAsString(), 16)));
            case "double" -> net.minecraft.nbt.DoubleTag.valueOf(Double.longBitsToDouble(
                Long.parseUnsignedLong(node.get("value").getAsString(), 16)));
            case "string" -> net.minecraft.nbt.StringTag.valueOf(node.get("value").getAsString());
            case "byte-array" -> {
                var values = node.getAsJsonArray("value");
                byte[] data = new byte[values.size()];
                for (int i = 0; i < data.length; i++) {
                    data[i] = Byte.parseByte(values.get(i).getAsString());
                }
                yield new ByteArrayTag(data);
            }
            case "int-array" -> {
                var values = node.getAsJsonArray("value");
                int[] data = new int[values.size()];
                for (int i = 0; i < data.length; i++) {
                    data[i] = Integer.parseInt(values.get(i).getAsString());
                }
                yield new IntArrayTag(data);
            }
            case "long-array" -> {
                var values = node.getAsJsonArray("value");
                long[] data = new long[values.size()];
                for (int i = 0; i < data.length; i++) {
                    data[i] = Long.parseLong(values.get(i).getAsString());
                }
                yield new LongArrayTag(data);
            }
            case "list" -> {
                var list = new ListTag();
                for (JsonElement value : node.getAsJsonArray("value")) {
                    list.add(list.size(), buildTag(value.getAsJsonObject()));
                }
                yield list;
            }
            case "compound" -> {
                var compound = new CompoundTag();
                JsonObject members = node.getAsJsonObject("value");
                for (String key : members.keySet()) {
                    compound.put(key, buildTag(members.getAsJsonObject(key)));
                }
                yield compound;
            }
            default -> throw new IllegalArgumentException("Unknown NBT node type");
        };
    }

    private static JsonObject describeTag(Tag tag) {
        var node = new JsonObject();
        switch (tag.getId()) {
            case 1 -> {
                node.addProperty("type", "byte");
                node.addProperty("value", Byte.toString(((net.minecraft.nbt.ByteTag)tag).value()));
            }
            case 2 -> {
                node.addProperty("type", "short");
                node.addProperty("value", Short.toString(((net.minecraft.nbt.ShortTag)tag).value()));
            }
            case 3 -> {
                node.addProperty("type", "int");
                node.addProperty("value", Integer.toString(((net.minecraft.nbt.IntTag)tag).value()));
            }
            case 4 -> {
                node.addProperty("type", "long");
                node.addProperty("value", Long.toString(((net.minecraft.nbt.LongTag)tag).value()));
            }
            case 5 -> {
                node.addProperty("type", "float");
                node.addProperty("value", String.format("%08x", Float.floatToRawIntBits(
                    ((net.minecraft.nbt.FloatTag)tag).value())));
            }
            case 6 -> {
                node.addProperty("type", "double");
                node.addProperty("value", String.format("%016x", Double.doubleToRawLongBits(
                    ((net.minecraft.nbt.DoubleTag)tag).value())));
            }
            case 7 -> {
                node.addProperty("type", "byte-array");
                var values = new JsonArray();
                for (byte value : ((ByteArrayTag)tag).getAsByteArray()) {
                    values.add(Byte.toString(value));
                }
                node.add("value", values);
            }
            case 8 -> {
                node.addProperty("type", "string");
                node.addProperty("value", ((net.minecraft.nbt.StringTag)tag).value());
            }
            case 9 -> {
                node.addProperty("type", "list");
                var values = new JsonArray();
                ListTag list = (ListTag)tag;
                for (int i = 0; i < list.size(); i++) {
                    values.add(describeTag(list.get(i)));
                }
                node.add("value", values);
            }
            case 10 -> {
                node.addProperty("type", "compound");
                var members = new JsonObject();
                CompoundTag compound = (CompoundTag)tag;
                List<String> keys = new ArrayList<>(compound.keySet());
                keys.sort(String::compareTo);
                for (String key : keys) {
                    members.add(key, describeTag(compound.get(key)));
                }
                node.add("value", members);
            }
            case 11 -> {
                node.addProperty("type", "int-array");
                var values = new JsonArray();
                for (int value : ((IntArrayTag)tag).getAsIntArray()) {
                    values.add(Integer.toString(value));
                }
                node.add("value", values);
            }
            case 12 -> {
                node.addProperty("type", "long-array");
                var values = new JsonArray();
                for (long value : ((LongArrayTag)tag).getAsLongArray()) {
                    values.add(Long.toString(value));
                }
                node.add("value", values);
            }
            default -> throw new IllegalStateException("Unexpected tag in registry data");
        }
        return node;
    }

    @SuppressWarnings("unchecked")
    private static Map<Identifier, IntList> networkPayloadTags(TagNetworkSerialization.NetworkPayload payload) {
        try {
            var field = TagNetworkSerialization.NetworkPayload.class.getDeclaredField("tags");
            field.setAccessible(true);
            return (Map<Identifier, IntList>)field.get(payload);
        } catch (ReflectiveOperationException error) {
            throw new RuntimeException(error);
        }
    }

    private static JsonObject pingResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        String packet = request.get("packet").getAsString();
        if (direction.equals("write")) {
            long time = Long.parseLong(request.get("time").getAsString());
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                if (packet.equals("request")) {
                    ServerboundPingRequestPacket.STREAM_CODEC.encode(buffer, new ServerboundPingRequestPacket(time));
                } else if (packet.equals("response")) {
                    ClientboundPongResponsePacket.STREAM_CODEC.encode(new FriendlyByteBuf(buffer), new ClientboundPongResponsePacket(time));
                } else {
                    throw new IllegalArgumentException("Unknown ping packet");
                }
                var result = new JsonObject();
                result.addProperty("outcome", "value");
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        if (direction.equals("read")) {
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
                var result = new JsonObject();
                try {
                    long time;
                    if (packet.equals("request")) {
                        time = ServerboundPingRequestPacket.STREAM_CODEC.decode(buffer).getTime();
                    } else if (packet.equals("response")) {
                        time = ClientboundPongResponsePacket.STREAM_CODEC.decode(new FriendlyByteBuf(buffer)).time();
                    } else {
                        throw new IllegalArgumentException("Unknown ping packet");
                    }
                    result.addProperty("time", Long.toString(time));
                    result.addProperty("outcome", "value");
                } catch (IndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "buffer_underflow");
                    result.addProperty("java_exception", error.getClass().getName());
                }
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown ping direction");
    }

    private static String fromUtf16Hex(String value) {
        if (value.length() % 4 != 0) {
            throw new IllegalArgumentException("UTF-16 hex must contain complete units");
        }
        char[] units = new char[value.length() / 4];
        for (int index = 0; index < units.length; index++) {
            units[index] = (char)Integer.parseInt(value.substring(index * 4, index * 4 + 4), 16);
        }
        return new String(units);
    }

    private static String toUtf16Hex(String value) {
        StringBuilder result = new StringBuilder(value.length() * 4);
        for (int index = 0; index < value.length(); index++) {
            result.append(String.format(Locale.ROOT, "%04x", (int)value.charAt(index)));
        }
        return result.toString();
    }

    private static byte[] rawPublicKey(ClientboundHelloPacket packet) {
        try {
            var field = ClientboundHelloPacket.class.getDeclaredField("publicKey");
            field.setAccessible(true);
            return ((byte[])field.get(packet)).clone();
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("Cannot inspect decoded public key", error);
        }
    }
}
