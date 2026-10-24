import java.util.ArrayList;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import net.minecraft.network.protocol.ping.ClientboundPongResponsePacket;
import net.minecraft.network.ConnectionProtocol;
import io.netty.handler.codec.CorruptedFrameException;
import io.netty.handler.codec.EncoderException;
import net.minecraft.network.FriendlyByteBuf;
import java.util.HexFormat;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.List;
import net.minecraft.network.protocol.PacketFlow;
import net.minecraft.network.protocol.ping.ServerboundPingRequestPacket;
import net.minecraft.network.protocol.status.ServerboundStatusRequestPacket;
import io.netty.buffer.Unpooled;

/** Frames, status, ping and protocol metadata. */
class WireFrameOracle extends WireSupport {
    static JsonObject framePrependResult(JsonObject request) {
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

    static JsonObject frameDecodeResult(JsonObject request) {
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

    static JsonObject statusRequestResult(JsonObject request) {
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

    static JsonObject pingResult(JsonObject request) {
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

    static JsonObject protocolMetadataResult(JsonObject request) {
        String kind = request.get("kind").getAsString();
        String name = request.get("name").getAsString();
        JsonObject result = new JsonObject();
        if (kind.equals("connection")) {
            ConnectionProtocol protocol = ConnectionProtocol.valueOf(name);
            result.addProperty("id", protocol.id());
            result.addProperty("name", protocol.name());
            result.addProperty("ordinal", protocol.ordinal());
        } else if (kind.equals("flow")) {
            PacketFlow flow = PacketFlow.valueOf(name);
            result.addProperty("id", flow.id());
            result.addProperty("name", flow.name());
            result.addProperty("opposite", flow.getOpposite().name());
            result.addProperty("ordinal", flow.ordinal());
        } else {
            throw new IllegalArgumentException("Unknown protocol metadata kind");
        }
        result.addProperty("outcome", "value");
        return result;
    }
}
