import java.util.ArrayList;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import net.minecraft.network.protocol.common.ClientboundKeepAlivePacket;
import net.minecraft.network.protocol.common.ClientboundPingPacket;
import net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket;
import net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket;
import io.netty.handler.codec.DecoderException;
import io.netty.handler.codec.EncoderException;
import net.minecraft.network.FriendlyByteBuf;
import java.util.HashMap;
import java.util.HexFormat;
import net.minecraft.resources.Identifier;
import net.minecraft.IdentifierException;
import it.unimi.dsi.fastutil.ints.IntArrayList;
import it.unimi.dsi.fastutil.ints.IntList;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistrySynchronization;
import net.minecraft.resources.ResourceKey;
import net.minecraft.network.protocol.common.ServerboundKeepAlivePacket;
import net.minecraft.network.protocol.common.ServerboundPongPacket;
import net.minecraft.nbt.Tag;
import net.minecraft.tags.TagNetworkSerialization;
import io.netty.buffer.Unpooled;

/** Common packets, tag updates and registry data. */
class WireRegistryOracle extends WireConfigurationOracle {
    static JsonObject commonPacketResult(JsonObject request) {
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

    static JsonObject updateTagsResult(JsonObject request) {
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

    static JsonObject registryDataResult(JsonObject request) {
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
}
