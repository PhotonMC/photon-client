import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import com.google.gson.JsonObject;
import io.netty.buffer.Unpooled;
import net.minecraft.network.VarInt;
import net.minecraft.network.VarLong;


public final class WireOracle extends WireRegistryOracle {
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
        if (operation.equals("login_acknowledged")) {
            return loginAcknowledgedResult(request);
        }
        if (operation.equals("login_finished")) {
            return loginFinishedResult(request);
        }
        if (operation.equals("configuration_unit")) {
            return configurationUnitResult(request);
        }
        if (operation.equals("configuration_misc")) {
            return configurationMiscResult(request);
        }
        if (operation.equals("protocol_metadata")) {
            return protocolMetadataResult(request);
        }
        if (operation.equals("client_information")) {
            return clientInformationResult(request);
        }
        if (operation.equals("common_packet")) {
            return commonPacketResult(request);
        }
        if (operation.equals("enabled_features")) {
            return enabledFeaturesResult(request);
        }
        if (operation.equals("update_advancements")) {
            return updateAdvancementsResult(request);
        }
        if (operation.equals("select_advancements_tab")) {
            return selectAdvancementsTabResult(request);
        }
        if (operation.equals("award_stats")) {
            return awardStatsResult(request);
        }
        if (operation.equals("update_tags")) {
            return updateTagsResult(request);
        }
        if (operation.equals("registry_data")) {
            return registryDataResult(request);
        }
        if (operation.equals("status_request")) {
            return statusRequestResult(request);
        }
        if (operation.equals("ping")) {
            return pingResult(request);
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
}
