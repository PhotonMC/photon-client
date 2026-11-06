import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.Strictness;
import com.google.gson.stream.JsonReader;
import java.io.StringReader;
import java.util.HexFormat;

/** Test glue only: parses through the original Gson configuration. */
public final class JsonOracle {
    private static final Gson GSON = new GsonBuilder().create();

    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            switch (operation) {
                case "parse" -> rows.add(parse(element.getAsJsonObject()));
                case "number" -> rows.add(number(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException("Unknown json operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** What the reference turns one number literal into, per requested type. */
    private static JsonObject number(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        String literal = row.get("literal").getAsString();
        JsonElement parsed;
        try {
            JsonReader reader = new JsonReader(new StringReader(literal));
            reader.setStrictness(Strictness.STRICT);
            parsed = GSON.getAdapter(JsonElement.class).read(reader);
        } catch (Exception error) {
            entry.addProperty("outcome", "rejected");
            return entry;
        }
        var primitive = parsed.getAsJsonPrimitive();
        entry.addProperty("outcome", "converted");
        entry.addProperty("literal", primitive.getAsString());
        entry.addProperty("as_int", conversion(() -> String.valueOf(primitive.getAsInt())));
        entry.addProperty("as_long", conversion(() -> String.valueOf(primitive.getAsLong())));
        entry.addProperty("as_float", conversion(
            () -> String.format(java.util.Locale.ROOT, "%08x", Float.floatToRawIntBits(primitive.getAsFloat()))));
        entry.addProperty("as_double", conversion(
            () -> String.format(java.util.Locale.ROOT, "%016x", Double.doubleToRawLongBits(primitive.getAsDouble()))));
        return entry;
    }

    private static String conversion(java.util.function.Supplier<String> supplier) {
        try {
            return supplier.get();
        } catch (RuntimeException error) {
            return "error";
        }
    }

    private static JsonObject parse(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        String text = new String(
            HexFormat.of().parseHex(row.get("text").getAsString()),
            java.nio.charset.StandardCharsets.UTF_8
        );
        try {
            JsonReader reader = new JsonReader(new StringReader(text));
            reader.setStrictness(Strictness.STRICT);
            JsonElement parsed = GSON.getAdapter(JsonElement.class).read(reader);
            if (parsed == null) {
                entry.addProperty("outcome", "rejected");
                return entry;
            }
            // The reference's readers consume one value and stop; trailing
            // content is what distinguishes a document from a prefix.
            if (reader.peek() != com.google.gson.stream.JsonToken.END_DOCUMENT) {
                entry.addProperty("outcome", "rejected");
                return entry;
            }
            entry.addProperty("outcome", "parsed");
            entry.addProperty("canonical", canonical(parsed));
        } catch (Exception error) {
            entry.addProperty("outcome", "rejected");
        }
        return entry;
    }

    /** A deterministic rendering, so both sides compare shape, order and text. */
    private static String canonical(JsonElement element) {
        var out = new StringBuilder();
        write(element, out);
        return out.toString();
    }

    private static void write(JsonElement element, StringBuilder out) {
        if (element.isJsonNull()) {
            out.append("null");
        } else if (element.isJsonObject()) {
            out.append('{');
            boolean first = true;
            for (var field : element.getAsJsonObject().entrySet()) {
                if (!first) {
                    out.append(',');
                }
                first = false;
                writeString(field.getKey(), out);
                out.append(':');
                write(field.getValue(), out);
            }
            out.append('}');
        } else if (element.isJsonArray()) {
            out.append('[');
            boolean first = true;
            for (JsonElement item : element.getAsJsonArray()) {
                if (!first) {
                    out.append(',');
                }
                first = false;
                write(item, out);
            }
            out.append(']');
        } else {
            var primitive = element.getAsJsonPrimitive();
            if (primitive.isBoolean()) {
                out.append(primitive.getAsBoolean() ? "true" : "false");
            } else if (primitive.isNumber()) {
                // The literal as written, which is what Gson keeps until a
                // caller asks for a specific numeric type.
                out.append('n').append(primitive.getAsString());
            } else {
                writeString(primitive.getAsString(), out);
            }
        }
    }

    /** Strings go out as UTF-16 code units, which pins escape and surrogate
     *  handling without the rendering itself needing escapes. */
    private static void writeString(String value, StringBuilder out) {
        out.append('s');
        for (int index = 0; index < value.length(); index++) {
            out.append(String.format("%04x", (int)value.charAt(index)));
        }
        out.append(';');
    }
}
