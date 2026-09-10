import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import io.netty.buffer.Unpooled;
import io.netty.channel.ChannelHandlerContext;
import io.netty.handler.codec.CorruptedFrameException;
import io.netty.handler.codec.EncoderException;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.List;
import java.util.Locale;
import java.util.UUID;
import net.minecraft.network.Varint21FrameDecoder;
import net.minecraft.network.Varint21LengthFieldPrepender;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.VarInt;
import net.minecraft.network.VarLong;
import net.minecraft.network.protocol.handshake.ClientIntent;
import net.minecraft.network.protocol.handshake.ClientIntentionPacket;
import net.minecraft.network.protocol.login.ClientboundHelloPacket;
import net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket;
import net.minecraft.network.protocol.login.ServerboundHelloPacket;

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
