import com.google.gson.JsonObject;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import io.netty.buffer.Unpooled;
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
        if (read) {
            input = HexFormat.of().parseHex(request.get("hex").getAsString());
        } else if (codec.equals("uuid")) {
            long most = Long.parseUnsignedLong(request.get("most_hex").getAsString(), 16);
            long least = Long.parseUnsignedLong(request.get("least_hex").getAsString(), 16);
            uuid = new UUID(most, least);
        } else if (codec.equals("blockpos")) {
            int x = Integer.parseInt(request.get("x").getAsString());
            int y = Integer.parseInt(request.get("y").getAsString());
            int z = Integer.parseInt(request.get("z").getAsString());
            pos = new BlockPos(x, y, z);
        } else {
            throw new IllegalArgumentException("Unknown binary codec");
        }
        if (!codec.equals("uuid") && !codec.equals("blockpos")) {
            throw new IllegalArgumentException("Unknown binary codec");
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
                } else if (read) {
                    BlockPos decoded = FriendlyByteBuf.readBlockPos(buffer);
                    result.addProperty("x", Integer.toString(decoded.getX()));
                    result.addProperty("y", Integer.toString(decoded.getY()));
                    result.addProperty("z", Integer.toString(decoded.getZ()));
                } else if (codec.equals("uuid")) {
                    FriendlyByteBuf.writeUUID(buffer, uuid);
                } else {
                    FriendlyByteBuf.writeBlockPos(buffer, pos);
                }
                result.addProperty("outcome", "value");
            } catch (IndexOutOfBoundsException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", "buffer_underflow");
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
