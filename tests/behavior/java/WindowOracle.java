import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.mojang.blaze3d.platform.Window;
import java.lang.reflect.Field;
import sun.misc.Unsafe;

/**
 * Test glue only: invokes the original JAR's window sizing, never reimplements it.
 *
 * {@code Window.calculateScale} and {@code Window.setGuiScale} read and write
 * plain instance fields and never touch the GLFW handle, so the scenario
 * allocates the original class without running its constructor and drives those
 * two methods directly. That keeps the reference headless while still calling
 * the shipped bytecode.
 */
public final class WindowOracle {
    public static JsonObject run(JsonObject request) throws Exception {
        String operation = request.get("operation").getAsString();
        return switch (operation) {
            case "gui_scale" -> guiScale(request);
            case "base_size" -> baseSize();
            case "min_size" -> minSize(request);
            default -> throw new IllegalArgumentException("Unknown window operation: " + operation);
        };
    }

    private static JsonObject baseSize() {
        var result = new JsonObject();
        result.addProperty("base_width", Window.BASE_WIDTH);
        result.addProperty("base_height", Window.BASE_HEIGHT);
        return result;
    }

    private static JsonObject minSize(JsonObject request) throws Exception {
        var method = Window.class.getDeclaredMethod("allowedWindowMinSize", int.class);
        method.setAccessible(true);
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            int requested = row.get("requested").getAsInt();
            var entry = new JsonObject();
            entry.addProperty("label", row.get("label").getAsString());
            entry.addProperty("allowed", (Integer)method.invoke(null, requested));
            rows.add(entry);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static JsonObject guiScale(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            int framebufferWidth = row.get("framebuffer_width").getAsInt();
            int framebufferHeight = row.get("framebuffer_height").getAsInt();
            int maxScale = row.get("max_scale").getAsInt();
            boolean enforceUnicode = row.get("enforce_unicode").getAsBoolean();

            Window window = allocateWindow();
            setInt(window, "framebufferWidth", framebufferWidth);
            setInt(window, "framebufferHeight", framebufferHeight);

            int calculated = window.calculateScale(maxScale, enforceUnicode);
            window.setGuiScale(calculated);

            var entry = new JsonObject();
            entry.addProperty("label", row.get("label").getAsString());
            entry.addProperty("calculated_scale", calculated);
            entry.addProperty("gui_scale", window.getGuiScale());
            entry.addProperty("gui_scaled_width", window.getGuiScaledWidth());
            entry.addProperty("gui_scaled_height", window.getGuiScaledHeight());
            rows.add(entry);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static Window allocateWindow() throws Exception {
        var unsafeField = Unsafe.class.getDeclaredField("theUnsafe");
        unsafeField.setAccessible(true);
        Unsafe unsafe = (Unsafe)unsafeField.get(null);
        return (Window)unsafe.allocateInstance(Window.class);
    }

    private static void setInt(Window window, String fieldName, int value) throws Exception {
        Field field = Window.class.getDeclaredField(fieldName);
        field.setAccessible(true);
        field.setInt(window, value);
    }
}
