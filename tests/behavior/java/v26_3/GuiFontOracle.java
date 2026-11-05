import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/**
 * Text measured by the game's own Font: a row's text in the default font (or the named one), plain
 * and bold, as the width in pixels and as its glyphs' advances.
 */
final class GuiFontOracle {
    private GuiFontOracle() {}

    static JsonObject fontWidth(JsonObject request) throws Exception {
        var font = GuiFonts.loadedFont();
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            String text = row.get("text").getAsString();
            var plain = net.minecraft.network.chat.Component.literal(text);
            var bold = net.minecraft.network.chat.Component.literal(text)
                .withStyle(net.minecraft.ChatFormatting.BOLD);
            var out = new JsonObject();
            out.addProperty("width", font.width(plain));
            out.addProperty("bold_width", font.width(bold));
            var advances = new JsonArray();
            text.codePoints().forEach(codepoint -> {
                var info = GuiFonts.glyphSource("minecraft:default").getGlyph(codepoint).info();
                advances.add(Oracle.bits(info.getAdvance(false)));
                advances.add(Oracle.bits(info.getAdvance(true)));
                advances.add(Oracle.bits(info.getShadowOffset()));
            });
            out.add("advances", advances);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
