import com.google.gson.Gson;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Locale;

/** Test glue only: invokes the original JAR; never implements game algorithms. */
public final class Oracle {
    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        var gson = new Gson();
        try (var input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8))) {
            for (String line; (line = input.readLine()) != null;) {
                JsonObject request = JsonParser.parseString(line).getAsJsonObject();
                if (request.get("schema_version").getAsInt() != 1) {
                    throw new IllegalArgumentException("Unsupported scenario schema");
                }
                String handler = switch (request.get("family").getAsString()) {
                    case "wire" -> "WireOracle";
                    case "utf8" -> "Utf8Oracle";
                    case "math" -> "MathOracle";
                    case "timer" -> "TimerOracle";
                    case "binary" -> "BinaryOracle";
                    case "nbt" -> "NbtOracle";
                    case "resources" -> "ResourceOracle";
                    default -> throw new IllegalArgumentException("Unknown scenario family");
                };
                var response = new JsonObject();
                response.addProperty("schema_version", 1);
                response.add("case_id", request.get("case_id"));
                JsonObject result = (JsonObject) Class.forName(handler)
                    .getMethod("run", JsonObject.class).invoke(null, request);
                if (result.has("java_exception")) {
                    var diagnostics = new JsonObject();
                    diagnostics.add("java_exception", result.remove("java_exception"));
                    response.add("diagnostics", diagnostics);
                }
                response.add("result", result);
                System.out.println(gson.toJson(response));
            }
        }
    }

    static float f32(JsonObject object, String name) {
        return Float.intBitsToFloat(Integer.parseUnsignedInt(object.get(name).getAsString(), 16));
    }

    static String bits(float value) {
        return String.format(Locale.ROOT, "%08x", Float.floatToRawIntBits(value));
    }

    static String bits(double value) {
        return String.format(Locale.ROOT, "%016x", Double.doubleToRawLongBits(value));
    }
}
