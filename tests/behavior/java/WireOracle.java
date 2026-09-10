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
import net.minecraft.network.Varint21FrameDecoder;
import net.minecraft.network.Varint21LengthFieldPrepender;
import net.minecraft.network.VarInt;
import net.minecraft.network.VarLong;

public final class WireOracle {
    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        if (operation.equals("frame_prepend")) {
            return framePrependResult(request);
        }
        if (operation.equals("frame_decode")) {
            return frameDecodeResult(request);
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
}
