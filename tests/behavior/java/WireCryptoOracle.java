import com.google.gson.JsonObject;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import io.netty.buffer.Unpooled;
import java.math.BigInteger;
import java.security.PublicKey;
import java.util.HexFormat;
import javax.crypto.Cipher;
import javax.crypto.SecretKey;
import javax.crypto.spec.SecretKeySpec;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.protocol.login.ServerboundKeyPacket;
import net.minecraft.util.Crypt;

/** The login encryption: the server hash, the CFB8 stream and the key packet. */
final class WireCryptoOracle {
    static JsonObject serverHashResult(JsonObject request) {
        try {
            String serverId = WireSupport.fromUtf16Hex(request.get("server_id_utf16_hex").getAsString());
            byte[] secret = HexFormat.of().parseHex(request.get("secret_hex").getAsString());
            PublicKey publicKey = Crypt.byteToPublicKey(HexFormat.of().parseHex(request.get("public_key_hex").getAsString()));
            SecretKey key = new SecretKeySpec(secret, "AES");
            var result = new JsonObject();
            result.addProperty("hash", new BigInteger(Crypt.digestData(serverId, publicKey, key)).toString(16));
            return result;
        } catch (Exception error) {
            throw new IllegalStateException(error);
        }
    }

    /** A stream of writes through one cipher, each split encrypted in turn, then decrypted back. */
    static JsonObject streamResult(JsonObject request) {
        try {
            SecretKey key = new SecretKeySpec(HexFormat.of().parseHex(request.get("secret_hex").getAsString()), "AES");
            boolean encrypt = request.get("mode").getAsString().equals("encrypt");
            Cipher cipher = Crypt.getCipher(encrypt ? 1 : 2, key);
            var out = new StringBuilder();
            for (var chunk : request.getAsJsonArray("chunks_hex")) {
                byte[] input = HexFormat.of().parseHex(chunk.getAsString());
                byte[] done = cipher.update(input);
                out.append(done == null ? "" : HexFormat.of().formatHex(done));
            }
            var result = new JsonObject();
            result.addProperty("hex", out.toString());
            return result;
        } catch (Exception error) {
            throw new IllegalStateException(error);
        }
    }

    /** The key packet, decoded and written back. */
    static JsonObject keyPacketResult(JsonObject request) {
        ByteBuf buffer = Unpooled.buffer();
        ByteBuf rewritten = Unpooled.buffer();
        try {
            buffer.writeBytes(HexFormat.of().parseHex(request.get("hex").getAsString()));
            var result = new JsonObject();
            try {
                ServerboundKeyPacket packet = ServerboundKeyPacket.STREAM_CODEC.decode(new FriendlyByteBuf(buffer));
                ServerboundKeyPacket.STREAM_CODEC.encode(new FriendlyByteBuf(rewritten), packet);
                result.addProperty("rewritten_hex", ByteBufUtil.hexDump(rewritten));
                result.addProperty("outcome", "value");
            } catch (IndexOutOfBoundsException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", "buffer_underflow");
            } catch (RuntimeException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", error.getClass() == RuntimeException.class ? "too_big" : "decoder_error");
            }
            result.addProperty("consumed", buffer.readerIndex());
            result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
            return result;
        } finally {
            buffer.release();
            rewritten.release();
        }
    }
}
