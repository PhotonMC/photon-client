import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import net.minecraft.network.protocol.handshake.ClientIntent;
import net.minecraft.network.protocol.handshake.ClientIntentionPacket;
import net.minecraft.network.protocol.login.ClientboundHelloPacket;
import net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket;
import net.minecraft.network.protocol.login.ClientboundLoginFinishedPacket;
import io.netty.handler.codec.EncoderException;
import net.minecraft.network.FriendlyByteBuf;
import com.mojang.authlib.GameProfile;
import java.util.HexFormat;
import com.google.gson.JsonObject;
import java.util.Locale;
import net.minecraft.network.protocol.login.ServerboundHelloPacket;
import net.minecraft.network.protocol.login.ServerboundLoginAcknowledgedPacket;
import java.util.UUID;
import io.netty.buffer.Unpooled;

/** The handshake and login packets. */
class WireLoginOracle extends WireFrameOracle {
    static JsonObject handshakeIntentionResult(JsonObject request) {
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

    static JsonObject loginHelloResult(JsonObject request) {
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

    static JsonObject loginCompressionResult(JsonObject request) {
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

    static JsonObject loginServerHelloResult(JsonObject request) {
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

    static JsonObject loginAcknowledgedResult(JsonObject request) {
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

    static JsonObject loginFinishedResult(JsonObject request) {
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
}
