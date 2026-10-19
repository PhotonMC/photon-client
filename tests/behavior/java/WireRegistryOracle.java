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

    /**
     * `ClientboundAwardStatsPacket.STREAM_CODEC` on bytes: each statistic as its stat type's id, its value's
     * id in the registry that type counts, and the number, in id order.
     */
    @SuppressWarnings("unchecked")
    static JsonObject awardStatsResult(JsonObject request) {
        Oracle.vanillaRegistries();
        ByteBuf buffer = Unpooled.buffer();
        try {
            buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
            var result = new JsonObject();
            try {
                var access = net.minecraft.core.RegistryAccess.fromRegistryOfRegistries(
                    net.minecraft.core.registries.BuiltInRegistries.REGISTRY);
                var packet = net.minecraft.network.protocol.game.ClientboundAwardStatsPacket.STREAM_CODEC
                    .decode(new net.minecraft.network.RegistryFriendlyByteBuf(buffer, access));
                var rows = new java.util.ArrayList<int[]>();
                for (var entry : packet.stats().object2IntEntrySet()) {
                    var stat = entry.getKey();
                    var registry = (Registry<Object>) stat.getType().getRegistry();
                    rows.add(new int[] {
                        net.minecraft.core.registries.BuiltInRegistries.STAT_TYPE.getId(stat.getType()),
                        registry.getId(stat.getValue()), entry.getIntValue()});
                }
                rows.sort(java.util.Comparator.<int[]>comparingInt(row -> row[0]).thenComparingInt(row -> row[1]));
                var stats = new JsonArray();
                for (var row : rows) {
                    var stat = new JsonArray();
                    stat.add(row[0]);
                    stat.add(row[1]);
                    stat.add(row[2]);
                    stats.add(stat);
                }
                result.add("stats", stats);
                result.addProperty("outcome", "value");
            } catch (RuntimeException error) {
                result.addProperty("outcome", "error");
            }
            return result;
        } finally {
            buffer.release();
        }
    }

    /**
     * `ClientboundUpdateAdvancementsPacket.STREAM_CODEC` on bytes: the flags, each advancement (its id, parent,
     * requirements and display with the title and description as text, the icon's item id and count and how many
     * components it patches, the frame, the flags, the background and the place as bits), the ids removed and the
     * progress of each criterion; the advancements in the order they came, the progress by id.
     */
    static JsonObject updateAdvancementsResult(JsonObject request) {
        Oracle.vanillaRegistries();
        ByteBuf buffer = Unpooled.buffer();
        try {
            buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
            var result = new JsonObject();
            try {
                var access = net.minecraft.core.RegistryAccess.fromRegistryOfRegistries(
                    net.minecraft.core.registries.BuiltInRegistries.REGISTRY);
                var packet = net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket.STREAM_CODEC
                    .decode(new net.minecraft.network.RegistryFriendlyByteBuf(buffer, access));
                result.addProperty("reset", packet.shouldReset());
                result.addProperty("show", packet.shouldShowAdvancements());
                var added = new JsonArray();
                for (var holder : packet.getAdded()) {
                    var advancement = holder.value();
                    var entry = new JsonObject();
                    entry.addProperty("id", holder.id().toString());
                    entry.add("parent", advancement.parent().map(id -> (JsonElement) new com.google.gson.JsonPrimitive(id.toString()))
                        .orElse(JsonNull.INSTANCE));
                    var groups = new JsonArray();
                    for (var group : advancement.requirements().requirements()) {
                        var names = new JsonArray();
                        group.forEach(names::add);
                        groups.add(names);
                    }
                    entry.add("requirements", groups);
                    entry.addProperty("telemetry", advancement.sendsTelemetryEvent());
                    if (advancement.display().isPresent()) {
                        var display = advancement.display().get();
                        var shown = new JsonObject();
                        shown.addProperty("title", display.getTitle().getString());
                        shown.addProperty("description", display.getDescription().getString());
                        shown.addProperty("icon", net.minecraft.core.registries.BuiltInRegistries.ITEM.getId(display.getIcon().item().value()));
                        shown.addProperty("icon_count", display.getIcon().count());
                        shown.addProperty("icon_patched", display.getIcon().components().size());
                        shown.addProperty("frame", display.getType().ordinal());
                        shown.addProperty("show_toast", display.shouldShowToast());
                        shown.addProperty("hidden", display.isHidden());
                        shown.add("background", display.getBackground().map(asset -> (JsonElement) new com.google.gson.JsonPrimitive(asset.id().toString()))
                            .orElse(JsonNull.INSTANCE));
                        shown.addProperty("x", Oracle.bits(display.getX()));
                        shown.addProperty("y", Oracle.bits(display.getY()));
                        entry.add("display", shown);
                    } else {
                        entry.add("display", JsonNull.INSTANCE);
                    }
                    added.add(entry);
                }
                result.add("added", added);
                var removed = new JsonArray();
                packet.getRemoved().forEach(id -> removed.add(id.toString()));
                result.add("removed", removed);
                var progress = new JsonArray();
                var ids = new java.util.ArrayList<>(packet.getProgress().keySet());
                ids.sort(java.util.Comparator.comparing(Identifier::toString));
                var criteriaField = net.minecraft.advancements.AdvancementProgress.class.getDeclaredField("criteria");
                criteriaField.setAccessible(true);
                var obtainedField = net.minecraft.advancements.CriterionProgress.class.getDeclaredField("obtained");
                obtainedField.setAccessible(true);
                for (var id : ids) {
                    var criteria = (Map<String, net.minecraft.advancements.CriterionProgress>) criteriaField.get(packet.getProgress().get(id));
                    var names = new java.util.ArrayList<>(criteria.keySet());
                    java.util.Collections.sort(names);
                    var list = new JsonArray();
                    for (var name : names) {
                        var pair = new JsonArray();
                        pair.add(name);
                        var obtained = (java.time.Instant) obtainedField.get(criteria.get(name));
                        pair.add(obtained == null ? JsonNull.INSTANCE : new com.google.gson.JsonPrimitive(obtained.toEpochMilli()));
                        list.add(pair);
                    }
                    var one = new JsonObject();
                    one.addProperty("id", id.toString());
                    one.add("criteria", list);
                    progress.add(one);
                }
                result.add("progress", progress);
                result.addProperty("outcome", "value");
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(error);
            } catch (RuntimeException error) {
                result.addProperty("outcome", "error");
            }
            return result;
        } finally {
            buffer.release();
        }
    }

    /** `ClientboundSelectAdvancementsTabPacket.STREAM_CODEC` on bytes: the tab, if there is one. */
    static JsonObject selectAdvancementsTabResult(JsonObject request) {
        ByteBuf buffer = Unpooled.buffer();
        try {
            buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
            var result = new JsonObject();
            try {
                var packet = net.minecraft.network.protocol.game.ClientboundSelectAdvancementsTabPacket.STREAM_CODEC
                    .decode(new FriendlyByteBuf(buffer));
                var tab = packet.getTab();
                result.add("tab", tab == null ? JsonNull.INSTANCE : new com.google.gson.JsonPrimitive(tab.toString()));
                result.addProperty("outcome", "value");
            } catch (RuntimeException error) {
                result.addProperty("outcome", "error");
            }
            return result;
        } finally {
            buffer.release();
        }
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
