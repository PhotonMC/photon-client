import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import io.netty.buffer.Unpooled;
import io.netty.handler.codec.DecoderException;
import io.netty.handler.codec.EncoderException;
import java.util.BitSet;
import java.util.HexFormat;
import java.util.Locale;
import java.util.UUID;
import net.minecraft.core.BlockPos;
import net.minecraft.network.FriendlyByteBuf;

/** Original test glue: all packing happens inside the pinned JAR. */
public final class BinaryOracle {
    public static JsonObject run(JsonObject request) {
        String codec = request.get("codec").getAsString();
        String operation = request.get("operation").getAsString();
        boolean read = switch (operation) {
            case "read" -> true;
            case "write" -> false;
            default -> throw new IllegalArgumentException("Unknown binary operation");
        };
        byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
        // Parse scenario input outside the catch block so malformed fixtures fail the run.
        byte[] input = null;
        UUID uuid = null;
        BlockPos pos = null;
        long[] longs = null;
        BitSet bits = null;
        int fixedSize = -1;
        if (read) {
            input = HexFormat.of().parseHex(request.get("hex").getAsString());
            if (codec.equals("fixedbitset")) {
                fixedSize = request.get("size").getAsInt();
                if (fixedSize < 0) {
                    throw new IllegalArgumentException("Size outside this corpus contract");
                }
            }
        } else if (codec.equals("uuid")) {
            long most = Long.parseUnsignedLong(request.get("most_hex").getAsString(), 16);
            long least = Long.parseUnsignedLong(request.get("least_hex").getAsString(), 16);
            uuid = new UUID(most, least);
        } else if (codec.equals("blockpos")) {
            int x = Integer.parseInt(request.get("x").getAsString());
            int y = Integer.parseInt(request.get("y").getAsString());
            int z = Integer.parseInt(request.get("z").getAsString());
            pos = new BlockPos(x, y, z);
        } else if (codec.equals("longarray")) {
            var values = request.getAsJsonArray("values");
            longs = new long[values.size()];
            for (int i = 0; i < longs.length; i++) {
                longs[i] = Long.parseLong(values.get(i).getAsString());
            }
        } else if (codec.equals("bitset")) {
            bits = new BitSet();
            for (var entry : request.getAsJsonArray("bits")) {
                int bit = entry.getAsInt();
                if (bit < 0) {
                    throw new IllegalArgumentException("Bit index outside this corpus contract");
                }
                bits.set(bit);
            }
        } else if (codec.equals("fixedbitset")) {
            fixedSize = request.get("size").getAsInt();
            if (fixedSize < 0) {
                throw new IllegalArgumentException("Size outside this corpus contract");
            }
            bits = new BitSet();
            for (var entry : request.getAsJsonArray("bits")) {
                int bit = entry.getAsInt();
                if (bit < 0) {
                    throw new IllegalArgumentException("Bit index outside this corpus contract");
                }
                bits.set(bit);
            }
        } else {
            throw new IllegalArgumentException("Unknown binary codec");
        }
        if (!codec.equals("uuid") && !codec.equals("blockpos")
                && !codec.equals("longarray") && !codec.equals("bitset")
                && !codec.equals("fixedbitset")) {
            throw new IllegalArgumentException("Unknown binary codec");
        }
        if (codec.equals("fixedbitset")) {
            return fixedBitSet(request, prefix, input, bits, fixedSize, read);
        }
        ByteBuf buffer = Unpooled.buffer();
        var result = new JsonObject();
        try {
            buffer.writeBytes(prefix);
            if (read) {
                buffer.writeBytes(input);
                buffer.readerIndex(prefix.length);
            }
            try {
                if (read && codec.equals("uuid")) {
                    UUID decoded = FriendlyByteBuf.readUUID(buffer);
                    result.addProperty("most_hex",
                        String.format(Locale.ROOT, "%016x", decoded.getMostSignificantBits()));
                    result.addProperty("least_hex",
                        String.format(Locale.ROOT, "%016x", decoded.getLeastSignificantBits()));
                } else if (read && codec.equals("blockpos")) {
                    BlockPos decoded = FriendlyByteBuf.readBlockPos(buffer);
                    result.addProperty("x", Integer.toString(decoded.getX()));
                    result.addProperty("y", Integer.toString(decoded.getY()));
                    result.addProperty("z", Integer.toString(decoded.getZ()));
                } else if (read && codec.equals("longarray")) {
                    long[] decoded_longs = FriendlyByteBuf.readLongArray(buffer);
                    var values = new JsonArray();
                    for (long value : decoded_longs) {
                        values.add(Long.toString(value));
                    }
                    result.add("values", values);
                } else if (read) {
                    long[] decoded_longs = FriendlyByteBuf.readLongArray(buffer);
                    BitSet decoded = BitSet.valueOf(decoded_longs);
                    var indices = new JsonArray();
                    decoded.stream().forEach(indices::add);
                    result.add("bits", indices);
                } else if (codec.equals("uuid")) {
                    FriendlyByteBuf.writeUUID(buffer, uuid);
                } else if (codec.equals("blockpos")) {
                    FriendlyByteBuf.writeBlockPos(buffer, pos);
                } else if (codec.equals("longarray")) {
                    FriendlyByteBuf.writeLongArray(buffer, longs);
                } else {
                    FriendlyByteBuf.writeLongArray(buffer, bits.toLongArray());
                }
                result.addProperty("outcome", "value");
            } catch (IndexOutOfBoundsException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", "buffer_underflow");
                result.addProperty("java_exception", error.getClass().getName());
            } catch (DecoderException | NegativeArraySizeException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", "decoder_error");
                result.addProperty("java_exception", error.getClass().getName());
            } catch (RuntimeException error) {
                if (error.getClass() != RuntimeException.class) {
                    throw error;
                }
                result.addProperty("outcome", "error");
                result.addProperty("error", "too_big");
                result.addProperty("java_exception", error.getClass().getName());
            }
            if (read) {
                result.addProperty("consumed", buffer.readerIndex() - prefix.length);
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
            } else {
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
            }
            return result;
        } finally {
            buffer.release();
        }
    }

    private static JsonObject fixedBitSet(
            JsonObject request, byte[] prefix, byte[] input, BitSet bits, int size, boolean read) {
        FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.buffer());
        var result = new JsonObject();
        try {
            buffer.writeBytes(prefix);
            if (read) {
                buffer.writeBytes(input);
                buffer.readerIndex(prefix.length);
            }
            try {
                if (read) {
                    BitSet decoded = buffer.readFixedBitSet(size);
                    var indices = new JsonArray();
                    decoded.stream().forEach(indices::add);
                    result.add("bits", indices);
                } else {
                    buffer.writeFixedBitSet(bits, size);
                }
                result.addProperty("outcome", "value");
            } catch (IndexOutOfBoundsException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", "buffer_underflow");
                result.addProperty("java_exception", error.getClass().getName());
            } catch (EncoderException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", "encoder_error");
                result.addProperty("java_exception", error.getClass().getName());
            }
            if (read) {
                result.addProperty("consumed", buffer.readerIndex() - prefix.length);
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
            } else {
                result.addProperty("written", buffer.writerIndex() - prefix.length);
                result.addProperty("hex", ByteBufUtil.hexDump(buffer));
            }
            return result;
        } finally {
            buffer.release();
        }
    }
}
