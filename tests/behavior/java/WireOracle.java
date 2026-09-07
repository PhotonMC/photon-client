import com.google.gson.JsonObject;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import io.netty.buffer.Unpooled;
import net.minecraft.network.VarInt;
import net.minecraft.network.VarLong;

public final class WireOracle {
    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
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
