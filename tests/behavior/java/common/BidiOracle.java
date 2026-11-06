import com.google.gson.JsonObject;
import java.util.Locale;
import net.minecraft.client.resources.language.FormattedBidiReorder;
import net.minecraft.network.chat.FormattedText;

/** Original test glue: reordering, mirroring and shaping all happen inside the pinned JAR. */
public final class BidiOracle {
    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        if (!operation.equals("reorder")) {
            throw new IllegalArgumentException("Unknown bidi operation");
        }
        String hex = request.get("utf16_hex").getAsString();
        if (hex.length() % 4 != 0) {
            throw new IllegalArgumentException("Expected UTF-16 code units as four hex digits each");
        }
        char[] units = new char[hex.length() / 4];
        for (int i = 0; i < units.length; i++) {
            units[i] = (char) Integer.parseInt(hex.substring(i * 4, i * 4 + 4), 16);
        }
        boolean rtl = request.get("right_to_left").getAsBoolean();
        var result = new JsonObject();
        var out = new StringBuilder();
        FormattedBidiReorder.reorder(FormattedText.of(new String(units)), rtl).accept((position, style, codepoint) -> {
            out.append(String.format(Locale.ROOT, "%06x", codepoint));
            return true;
        });
        result.addProperty("codepoints_hex", out.toString());
        return result;
    }
}
