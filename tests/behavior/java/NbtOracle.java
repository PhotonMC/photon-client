import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import io.netty.buffer.Unpooled;
import io.netty.handler.codec.DecoderException;
import io.netty.handler.codec.EncoderException;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.List;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.ReportedNbtException;
import net.minecraft.nbt.Tag;
import net.minecraft.network.FriendlyByteBuf;

/** Original test glue: all NBT work happens inside the pinned JAR. */
public final class NbtOracle {
    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        boolean read = switch (operation) {
            case "read" -> true;
            case "write" -> false;
            default -> throw new IllegalArgumentException("Unknown NBT operation");
        };
        byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
        byte[] input = null;
        CompoundTag root = null;
        boolean nullRoot = false;
        if (read) {
            input = HexFormat.of().parseHex(request.get("hex").getAsString());
        } else if (request.has("null_root") && request.get("null_root").getAsBoolean()) {
            nullRoot = true;
        } else {
            root = new CompoundTag();
            for (var entry : request.getAsJsonArray("fields")) {
                JsonObject field = entry.getAsJsonObject();
                String key = field.get("key").getAsString();
                switch (field.get("type").getAsString()) {
                    case "byte" -> root.putByte(key, Byte.parseByte(field.get("value").getAsString()));
                    case "short" -> root.putShort(key, Short.parseShort(field.get("value").getAsString()));
                    case "int" -> root.putInt(key, Integer.parseInt(field.get("value").getAsString()));
                    case "long" -> root.putLong(key, Long.parseLong(field.get("value").getAsString()));
                    case "float" -> root.putFloat(key,
                        Float.intBitsToFloat(Integer.parseUnsignedInt(field.get("value").getAsString(), 16)));
                    case "double" -> root.putDouble(key, Double.longBitsToDouble(
                        Long.parseUnsignedLong(field.get("value").getAsString(), 16)));
                    case "string" -> root.putString(key, field.get("value").getAsString());
                    default -> throw new IllegalArgumentException("Unknown NBT field type");
                }
            }
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
                if (read) {
                    CompoundTag decoded = FriendlyByteBuf.readNbt(buffer);
                    if (decoded == null) {
                        result.addProperty("null", true);
                    } else {
                        var fields = new JsonArray();
                        List<String> keys = new ArrayList<>(decoded.keySet());
                        keys.sort(String::compareTo);
                        for (String key : keys) {
                            Tag tag = decoded.get(key);
                            var field = new JsonObject();
                            field.addProperty("key", key);
                            switch (tag.getId()) {
                                case 1 -> {
                                    field.addProperty("type", "byte");
                                    field.addProperty("value", Byte.toString(decoded.getByteOr(key, (byte) 0)));
                                }
                                case 2 -> {
                                    field.addProperty("type", "short");
                                    field.addProperty("value", Short.toString(decoded.getShortOr(key, (short) 0)));
                                }
                                case 3 -> {
                                    field.addProperty("type", "int");
                                    field.addProperty("value", Integer.toString(decoded.getIntOr(key, 0)));
                                }
                                case 4 -> {
                                    field.addProperty("type", "long");
                                    field.addProperty("value", Long.toString(decoded.getLongOr(key, 0L)));
                                }
                                case 5 -> {
                                    field.addProperty("type", "float");
                                    field.addProperty("value", String.format("%08x",
                                        Float.floatToRawIntBits(decoded.getFloatOr(key, 0f))));
                                }
                                case 6 -> {
                                    field.addProperty("type", "double");
                                    field.addProperty("value", String.format("%016x",
                                        Double.doubleToRawLongBits(decoded.getDoubleOr(key, 0d))));
                                }
                                case 8 -> {
                                    field.addProperty("type", "string");
                                    field.addProperty("value", decoded.getStringOr(key, ""));
                                }
                                default -> throw new IllegalStateException("Unexpected tag in this corpus: " + tag.getId());
                            }
                            fields.add(field);
                        }
                        result.add("fields", fields);
                    }
                } else if (nullRoot) {
                    FriendlyByteBuf.writeNbt(buffer, null);
                } else {
                    FriendlyByteBuf.writeNbt(buffer, root);
                }
                result.addProperty("outcome", "value");
            } catch (IndexOutOfBoundsException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", "buffer_underflow");
                result.addProperty("java_exception", error.getClass().getName());
            } catch (DecoderException | ReportedNbtException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", "decoder_error");
                result.addProperty("java_exception", error.getClass().getName());
            } catch (EncoderException error) {
                // Note: readNbt maps IO failures to EncoderException.
                result.addProperty("outcome", "error");
                result.addProperty("error", "encoder_error");
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
}
