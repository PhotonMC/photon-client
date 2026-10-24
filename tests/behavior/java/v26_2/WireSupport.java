import java.util.ArrayList;
import com.google.common.collect.ArrayListMultimap;
import net.minecraft.nbt.ByteArrayTag;
import io.netty.buffer.ByteBuf;
import io.netty.channel.ChannelHandlerContext;
import net.minecraft.world.entity.player.ChatVisiblity;
import net.minecraft.network.protocol.handshake.ClientIntent;
import net.minecraft.network.protocol.login.ClientboundHelloPacket;
import net.minecraft.nbt.CompoundTag;
import com.mojang.authlib.GameProfile;
import net.minecraft.world.entity.HumanoidArm;
import net.minecraft.resources.Identifier;
import net.minecraft.nbt.IntArrayTag;
import it.unimi.dsi.fastutil.ints.IntList;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.util.List;
import net.minecraft.nbt.ListTag;
import java.util.Locale;
import net.minecraft.nbt.LongArrayTag;
import java.util.Map;
import net.minecraft.server.level.ParticleStatus;
import com.mojang.authlib.properties.Property;
import com.mojang.authlib.properties.PropertyMap;
import net.minecraft.nbt.Tag;
import net.minecraft.tags.TagNetworkSerialization;
import java.util.UUID;
import net.minecraft.network.Varint21FrameDecoder;
import net.minecraft.network.Varint21LengthFieldPrepender;

/** Hex, profile and tag helpers the wire rows share. */
class WireSupport {
    static String fromUtf16Hex(String value) {
        if (value.length() % 4 != 0) {
            throw new IllegalArgumentException("UTF-16 hex must contain complete units");
        }
        char[] units = new char[value.length() / 4];
        for (int index = 0; index < units.length; index++) {
            units[index] = (char)Integer.parseInt(value.substring(index * 4, index * 4 + 4), 16);
        }
        return new String(units);
    }

    static String toUtf16Hex(String value) {
        StringBuilder result = new StringBuilder(value.length() * 4);
        for (int index = 0; index < value.length(); index++) {
            result.append(String.format(Locale.ROOT, "%04x", (int)value.charAt(index)));
        }
        return result.toString();
    }

    static byte[] rawPublicKey(ClientboundHelloPacket packet) {
        try {
            var field = Reflect.field(ClientboundHelloPacket.class, "publicKey");
            return ((byte[])field.get(packet)).clone();
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("Cannot inspect decoded public key", error);
        }
    }

    static UUID uuidFromHex(JsonObject request, String prefix) {
        return new UUID(
            Long.parseUnsignedLong(request.get(prefix + "_most_hex").getAsString(), 16),
            Long.parseUnsignedLong(request.get(prefix + "_least_hex").getAsString(), 16)
        );
    }

    static GameProfile profileFromRequest(JsonObject request) {
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

    static JsonArray propertiesToJson(PropertyMap properties) {
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

    static final class PrependerProbe extends Varint21LengthFieldPrepender {
        void apply(ByteBuf message, ByteBuf output) {
            super.encode((ChannelHandlerContext)null, message, output);
        }
    }

    static final class DecoderProbe extends Varint21FrameDecoder {
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

    static ClientIntent clientIntent(String name) {
        return switch (name) {
            case "status" -> ClientIntent.STATUS;
            case "login" -> ClientIntent.LOGIN;
            case "transfer" -> ClientIntent.TRANSFER;
            default -> throw new IllegalArgumentException("Unknown client intent");
        };
    }

    static ChatVisiblity chatVisibility(String value) {
        return switch (value) {
            case "full" -> ChatVisiblity.FULL;
            case "system" -> ChatVisiblity.SYSTEM;
            case "hidden" -> ChatVisiblity.HIDDEN;
            default -> throw new IllegalArgumentException("Unknown chat visibility");
        };
    }

    static HumanoidArm humanoidArm(String value) {
        return switch (value) {
            case "left" -> HumanoidArm.LEFT;
            case "right" -> HumanoidArm.RIGHT;
            default -> throw new IllegalArgumentException("Unknown main hand");
        };
    }

    static ParticleStatus particleStatus(String value) {
        return switch (value) {
            case "all" -> ParticleStatus.ALL;
            case "decreased" -> ParticleStatus.DECREASED;
            case "minimal" -> ParticleStatus.MINIMAL;
            default -> throw new IllegalArgumentException("Unknown particle status");
        };
    }

    @SuppressWarnings("unchecked")
    static Map<Identifier, IntList> networkPayloadTags(TagNetworkSerialization.NetworkPayload payload) {
        try {
            var field = Reflect.field(TagNetworkSerialization.NetworkPayload.class, "tags");
            return (Map<Identifier, IntList>)field.get(payload);
        } catch (ReflectiveOperationException error) {
            throw new RuntimeException(error);
        }
    }

    static Tag buildTag(JsonObject node) {
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

    static JsonObject describeTag(Tag tag) {
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
}
