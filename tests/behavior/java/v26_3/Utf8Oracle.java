import com.google.gson.JsonObject;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import io.netty.buffer.Unpooled;
import io.netty.handler.codec.DecoderException;
import io.netty.handler.codec.EncoderException;
import java.util.HexFormat;
import java.util.Locale;
import net.minecraft.network.Utf8String;

/** Original test glue: all string encoding and decoding happens inside the pinned JAR. */
public final class Utf8Oracle {
    private static String fromUnits(String hex) {
        if (hex.length() % 4 != 0) {
            throw new IllegalArgumentException("Expected UTF-16 code units as four hex digits each");
        }
        char[] units = new char[hex.length() / 4];
        for (int i = 0; i < units.length; i++) {
            units[i] = (char) Integer.parseInt(hex.substring(i * 4, i * 4 + 4), 16);
        }
        return new String(units);
    }

    private static String toUnits(String value) {
        var hex = new StringBuilder(value.length() * 4);
        for (int i = 0; i < value.length(); i++) {
            hex.append(String.format(Locale.ROOT, "%04x", (int) value.charAt(i)));
        }
        return hex.toString();
    }

    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        boolean read = switch (operation) {
            case "read" -> true;
            case "write" -> false;
            default -> throw new IllegalArgumentException("Unknown UTF-8 operation");
        };
        int maxLength = request.get("max_length").getAsInt();
        if (maxLength < 0 || maxLength > 32767) {
            throw new IllegalArgumentException("Limit outside this corpus contract");
        }
        byte[] prefix = HexFormat.of().parseHex(request.get("prefix_hex").getAsString());
        // Parse scenario input outside the catch block so malformed fixtures fail the run.
        byte[] input = read ? HexFormat.of().parseHex(request.get("hex").getAsString()) : null;
        String value = read ? null : fromUnits(request.get("utf16_hex").getAsString());
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
                    String decoded = Utf8String.read(buffer, maxLength);
                    result.addProperty("utf16_hex", toUnits(decoded));
                    result.addProperty("utf16_length", decoded.length());
                } else {
                    Utf8String.write(buffer, value, maxLength);
                }
                result.addProperty("outcome", "value");
            } catch (RuntimeException error) {
                String category;
                if (read && error instanceof DecoderException) {
                    category = "decoder_error";
                } else if (!read && error instanceof EncoderException) {
                    category = "encoder_error";
                } else if (read && error instanceof IndexOutOfBoundsException) {
                    category = "buffer_underflow";
                } else if (read && error.getClass() == RuntimeException.class) {
                    category = "too_big";
                } else {
                    throw error;
                }
                result.addProperty("outcome", "error");
                result.addProperty("error", category);
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
