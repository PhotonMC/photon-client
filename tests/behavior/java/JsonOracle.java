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
        if (!operation.equals("parse")) {
            throw new IllegalArgumentException("Unknown json operation: " + operation);
        }
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            rows.add(parse(element.getAsJsonObject()));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
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
