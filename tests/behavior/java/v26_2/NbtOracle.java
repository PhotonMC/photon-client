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
import net.minecraft.nbt.ByteArrayTag;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.IntArrayTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.LongArrayTag;
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
        } else if (request.has("tree")) {
            Tag tree = buildTag(request.getAsJsonObject("tree"));
            if (!(tree instanceof CompoundTag compound)) {
                throw new IllegalArgumentException("Root must be a compound in this corpus");
            }
            root = compound;
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
                            JsonObject described = describeTag(decoded.get(key));
                            var field = new JsonObject();
                            field.addProperty("key", key);
                            field.addProperty("type", described.get("type").getAsString());
                            field.add("value", described.get("value"));
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
                for (var entry : node.getAsJsonArray("value")) {
                    list.add(list.size(), buildTag(entry.getAsJsonObject()));
                }
                yield list;
            }
            case "compound" -> {
                var compound = new CompoundTag();
                JsonObject members = node.getAsJsonObject("value");
                for (var key : members.keySet()) {
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
                node.addProperty("value",
                    Byte.toString(((net.minecraft.nbt.ByteTag) tag).value()));
            }
            case 2 -> {
                node.addProperty("type", "short");
                node.addProperty("value",
                    Short.toString(((net.minecraft.nbt.ShortTag) tag).value()));
            }
            case 3 -> {
                node.addProperty("type", "int");
                node.addProperty("value",
                    Integer.toString(((net.minecraft.nbt.IntTag) tag).value()));
            }
            case 4 -> {
                node.addProperty("type", "long");
                node.addProperty("value",
                    Long.toString(((net.minecraft.nbt.LongTag) tag).value()));
            }
            case 5 -> {
                node.addProperty("type", "float");
                node.addProperty("value", String.format("%08x", Float.floatToRawIntBits(
                    ((net.minecraft.nbt.FloatTag) tag).value())));
            }
            case 6 -> {
                node.addProperty("type", "double");
                node.addProperty("value", String.format("%016x", Double.doubleToRawLongBits(
                    ((net.minecraft.nbt.DoubleTag) tag).value())));
            }
            case 7 -> {
                node.addProperty("type", "byte-array");
                var values = new JsonArray();
                for (byte value : ((ByteArrayTag) tag).getAsByteArray()) {
                    values.add(Byte.toString(value));
                }
                node.add("value", values);
            }
            case 8 -> {
                node.addProperty("type", "string");
                node.addProperty("value", ((net.minecraft.nbt.StringTag) tag).value());
            }
            case 9 -> {
                node.addProperty("type", "list");
                var values = new JsonArray();
                ListTag list = (ListTag) tag;
                for (int i = 0; i < list.size(); i++) {
                    values.add(describeTag(list.get(i)));
                }
                node.add("value", values);
            }
            case 10 -> {
                node.addProperty("type", "compound");
                var members = new JsonObject();
                CompoundTag compound = (CompoundTag) tag;
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
                for (int value : ((IntArrayTag) tag).getAsIntArray()) {
                    values.add(Integer.toString(value));
                }
                node.add("value", values);
            }
            case 12 -> {
                node.addProperty("type", "long-array");
                var values = new JsonArray();
                for (long value : ((LongArrayTag) tag).getAsLongArray()) {
                    values.add(Long.toString(value));
                }
                node.add("value", values);
            }
            default -> throw new IllegalStateException("Unexpected tag in this corpus: " + tag.getId());
        }
        return node;
    }
}
