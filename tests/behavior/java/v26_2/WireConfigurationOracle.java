import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import net.minecraft.server.level.ClientInformation;
import net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket;
import net.minecraft.network.protocol.configuration.ClientboundFinishConfigurationPacket;
import net.minecraft.network.protocol.configuration.ClientboundResetChatPacket;
import net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket;
import io.netty.handler.codec.DecoderException;
import io.netty.handler.codec.EncoderException;
import net.minecraft.network.FriendlyByteBuf;
import java.util.HashSet;
import java.util.HexFormat;
import net.minecraft.IdentifierException;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.util.Locale;
import net.minecraft.network.protocol.configuration.ServerboundAcceptCodeOfConductPacket;
import net.minecraft.network.protocol.common.ServerboundClientInformationPacket;
import net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket;
import java.util.Set;
import io.netty.buffer.Unpooled;

/** Configuration packets: unit, miscellaneous, client information and features. */
class WireConfigurationOracle extends WireLoginOracle {
    static JsonObject configurationUnitResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        String packet = request.get("packet").getAsString();
        if (direction.equals("write")) {
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                switch (packet) {
                    case "server-finish" -> ServerboundFinishConfigurationPacket.STREAM_CODEC.encode(
                        buffer, ServerboundFinishConfigurationPacket.INSTANCE
                    );
                    case "client-finish" -> ClientboundFinishConfigurationPacket.STREAM_CODEC.encode(
                        buffer, ClientboundFinishConfigurationPacket.INSTANCE
                    );
                    case "accept-code-of-conduct" -> ServerboundAcceptCodeOfConductPacket.STREAM_CODEC.encode(
                        buffer, ServerboundAcceptCodeOfConductPacket.INSTANCE
                    );
                    default -> throw new IllegalArgumentException("Unknown configuration unit packet");
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
                boolean singleton = switch (packet) {
                    case "server-finish" -> ServerboundFinishConfigurationPacket.STREAM_CODEC.decode(buffer)
                        == ServerboundFinishConfigurationPacket.INSTANCE;
                    case "client-finish" -> ClientboundFinishConfigurationPacket.STREAM_CODEC.decode(buffer)
                        == ClientboundFinishConfigurationPacket.INSTANCE;
                    case "accept-code-of-conduct" -> ServerboundAcceptCodeOfConductPacket.STREAM_CODEC.decode(buffer)
                        == ServerboundAcceptCodeOfConductPacket.INSTANCE;
                    default -> throw new IllegalArgumentException("Unknown configuration unit packet");
                };
                var result = new JsonObject();
                result.addProperty("singleton", singleton);
                result.addProperty("outcome", "value");
                result.addProperty("consumed", buffer.readerIndex());
                result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
                return result;
            } finally {
                buffer.release();
            }
        }

        throw new IllegalArgumentException("Unknown configuration unit direction");
    }

    static JsonObject configurationMiscResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        String packet = request.get("packet").getAsString();
        if (direction.equals("write")) {
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    switch (packet) {
                        case "code-of-conduct" -> ClientboundCodeOfConductPacket.STREAM_CODEC.encode(
                            buffer, new ClientboundCodeOfConductPacket(
                                fromUtf16Hex(request.get("code_of_conduct_utf16_hex").getAsString())
                            )
                        );
                        case "reset-chat" -> ClientboundResetChatPacket.STREAM_CODEC.encode(
                            buffer, ClientboundResetChatPacket.INSTANCE
                        );
                        default -> throw new IllegalArgumentException("Unknown configuration misc packet");
                    }
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
                    if (packet.equals("code-of-conduct")) {
                        ClientboundCodeOfConductPacket decoded = ClientboundCodeOfConductPacket.STREAM_CODEC
                            .decode(buffer);
                        result.addProperty("code_of_conduct_utf16_hex", toUtf16Hex(decoded.codeOfConduct()));
                    } else if (packet.equals("reset-chat")) {
                        result.addProperty(
                            "singleton", ClientboundResetChatPacket.STREAM_CODEC.decode(buffer)
                                == ClientboundResetChatPacket.INSTANCE
                        );
                    } else {
                        throw new IllegalArgumentException("Unknown configuration misc packet");
                    }
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

        throw new IllegalArgumentException("Unknown configuration misc direction");
    }

    static JsonObject clientInformationResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            ClientInformation information = new ClientInformation(
                fromUtf16Hex(request.get("language_utf16_hex").getAsString()),
                request.get("view_distance").getAsInt(),
                chatVisibility(request.get("chat_visibility").getAsString()),
                request.get("chat_colors").getAsBoolean(),
                request.get("model_customisation").getAsInt(),
                humanoidArm(request.get("main_hand").getAsString()),
                request.get("text_filtering_enabled").getAsBoolean(),
                request.get("allows_listing").getAsBoolean(),
                particleStatus(request.get("particle_status").getAsString())
            );
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    ServerboundClientInformationPacket.STREAM_CODEC.encode(
                        new FriendlyByteBuf(buffer), new ServerboundClientInformationPacket(information)
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
                    ClientInformation information = ServerboundClientInformationPacket.STREAM_CODEC
                        .decode(new FriendlyByteBuf(buffer)).information();
                    result.addProperty("language_utf16_hex", toUtf16Hex(information.language()));
                    result.addProperty("view_distance", information.viewDistance());
                    result.addProperty("chat_visibility", information.chatVisibility().name().toLowerCase(Locale.ROOT));
                    result.addProperty("chat_colors", information.chatColors());
                    result.addProperty("model_customisation", information.modelCustomisation());
                    result.addProperty("main_hand", information.mainHand().name().toLowerCase(Locale.ROOT));
                    result.addProperty("text_filtering_enabled", information.textFilteringEnabled());
                    result.addProperty("allows_listing", information.allowsListing());
                    result.addProperty("particle_status", information.particleStatus().name().toLowerCase(Locale.ROOT));
                    result.addProperty("outcome", "value");
                } catch (ArrayIndexOutOfBoundsException error) {
                    result.addProperty("outcome", "error");
                    result.addProperty("error", "decoder_error");
                    result.addProperty("java_exception", error.getClass().getName());
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

        throw new IllegalArgumentException("Unknown client information direction");
    }

    static JsonObject enabledFeaturesResult(JsonObject request) {
        String direction = request.get("direction").getAsString();
        if (direction.equals("write")) {
            Set<net.minecraft.resources.Identifier> features = new HashSet<>();
            for (JsonElement element : request.getAsJsonArray("features")) {
                features.add(net.minecraft.resources.Identifier.parse(element.getAsString()));
            }
            byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
            ByteBuf buffer = Unpooled.buffer();
            try {
                buffer.writeBytes(prefix);
                var result = new JsonObject();
                try {
                    ClientboundUpdateEnabledFeaturesPacket.STREAM_CODEC.encode(
                        new FriendlyByteBuf(buffer), new ClientboundUpdateEnabledFeaturesPacket(features)
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
                    Set<net.minecraft.resources.Identifier> features = ClientboundUpdateEnabledFeaturesPacket.STREAM_CODEC
                        .decode(new FriendlyByteBuf(buffer)).features();
                    var values = new JsonArray();
                    for (net.minecraft.resources.Identifier feature : features) {
                        values.add(feature.toString());
                    }
                    result.add("features", values);
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

        throw new IllegalArgumentException("Unknown enabled features direction");
    }
}
