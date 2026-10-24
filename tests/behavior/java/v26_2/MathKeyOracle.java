import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.mojang.blaze3d.platform.InputConstants;
import java.util.Locale;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.ScrollWheelHandler;
import net.minecraft.client.ToggleKeyMapping;
import net.minecraft.client.input.CharacterEvent;
import net.minecraft.client.input.KeyEvent;
import net.minecraft.client.input.InputWithModifiers;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.client.input.MouseButtonInfo;
import net.minecraft.client.input.PreeditEvent;
import net.minecraft.client.player.KeyboardInput;
import net.minecraft.world.entity.player.Input;
import org.joml.Vector2i;
import sun.misc.Unsafe;

class MathKeyOracle extends MathInterpolationOracle {
    static JsonArray snapshotUtf16(String value) {
        var units = new JsonArray();
        for (int i = 0; i < value.length(); i++) {
            units.add(String.format(Locale.ROOT, "%04x", (int)value.charAt(i)));
        }
        return units;
    }

    static JsonObject snapshotPreedit(PreeditEvent event) {
        var state = new JsonObject();
        state.add("full_text", snapshotUtf16(event.fullText()));
        state.addProperty("caret_position", event.caretPosition());
        var blocks = new JsonArray();
        for (String block : event.blocks()) {
            blocks.add(snapshotUtf16(block));
        }
        state.add("blocks", blocks);
        state.addProperty("focused_block", event.focusedBlock());
        return state;
    }

    static JsonObject snapshotInputWithModifiers(InputWithModifiers input) {
        var state = new JsonObject();
        state.addProperty("input", input.input());
        state.addProperty("modifiers", input.modifiers());
        state.addProperty("is_selection", input.isSelection());
        state.addProperty("is_confirmation", input.isConfirmation());
        state.addProperty("is_escape", input.isEscape());
        state.addProperty("is_left", input.isLeft());
        state.addProperty("is_right", input.isRight());
        state.addProperty("is_up", input.isUp());
        state.addProperty("is_down", input.isDown());
        state.addProperty("is_cycle_focus", input.isCycleFocus());
        state.addProperty("digit", input.getDigit());
        state.addProperty("has_alt_down", input.hasAltDown());
        state.addProperty("has_shift_down", input.hasShiftDown());
        state.addProperty("has_control_down", input.hasControlDown());
        state.addProperty("has_control_down_with_quirk", input.hasControlDownWithQuirk());
        state.addProperty("is_select_all", input.isSelectAll());
        state.addProperty("is_copy", input.isCopy());
        state.addProperty("is_paste", input.isPaste());
        state.addProperty("is_cut", input.isCut());
        return state;
    }

    static InputConstants.Key inputKey(JsonObject value) {
        InputConstants.Type type = switch (value.get("type").getAsString()) {
            case "keysym" -> InputConstants.Type.KEYSYM;
            case "scancode" -> InputConstants.Type.SCANCODE;
            case "mouse" -> InputConstants.Type.MOUSE;
            default -> throw new IllegalArgumentException("Unknown input key type");
        };
        return type.getOrCreate(value.get("value").getAsInt());
    }

    static KeyMapping newKeyMapping(String name, InputConstants.Key key) {
        return new KeyMapping(name, key.getType(), key.getValue(), KeyMapping.Category.MOVEMENT);
    }

    static KeyMapping mapping(String target, KeyMapping primary, KeyMapping secondary, KeyMapping other) {
        return switch (target) {
            case "primary" -> primary;
            case "secondary" -> secondary;
            case "other" -> other;
            default -> throw new IllegalArgumentException("Unknown key mapping target");
        };
    }

    static InputConstants.Key actionKey(String target, InputConstants.Key primary,
                                                InputConstants.Key primaryDefault,
                                                InputConstants.Key secondary,
                                                InputConstants.Key other,
                                                InputConstants.Key rebind) {
        return switch (target) {
            case "primary" -> primary;
            case "primary_default" -> primaryDefault;
            case "secondary" -> secondary;
            case "other" -> other;
            case "rebind" -> rebind;
            default -> throw new IllegalArgumentException("Unknown action key");
        };
    }

    static JsonObject keyMappingState(KeyMapping primary, KeyMapping secondary, KeyMapping other) {
        var state = new JsonObject();
        state.addProperty("primary_down", primary.isDown());
        state.addProperty("secondary_down", secondary.isDown());
        state.addProperty("other_down", other.isDown());
        return state;
    }

    static JsonObject keyMappingProperties(
        KeyMapping primary,
        KeyMapping secondary,
        KeyMapping other,
        InputConstants.Key primaryKey,
        InputConstants.Key primaryDefaultKey,
        InputConstants.Key secondaryKey,
        InputConstants.Key secondaryDefaultKey,
        InputConstants.Key otherKey,
        InputConstants.Key otherDefaultKey,
        KeyEvent keyEvent,
        KeyEvent unknownKeyEvent,
        MouseButtonEvent mouseEvent
    ) {
        var state = new JsonObject();
        state.add("primary", keyMappingProperty(primary, primaryKey, primaryDefaultKey, keyEvent, unknownKeyEvent, mouseEvent));
        state.add("secondary", keyMappingProperty(secondary, secondaryKey, secondaryDefaultKey, keyEvent, unknownKeyEvent, mouseEvent));
        state.add("other", keyMappingProperty(other, otherKey, otherDefaultKey, keyEvent, unknownKeyEvent, mouseEvent));
        state.addProperty("primary_same_secondary", primary.same(secondary));
        state.addProperty("primary_same_other", primary.same(other));
        return state;
    }

    static JsonObject keyMappingProperty(
        KeyMapping mapping,
        InputConstants.Key key,
        InputConstants.Key defaultKey,
        KeyEvent keyEvent,
        KeyEvent unknownKeyEvent,
        MouseButtonEvent mouseEvent
    ) {
        var state = new JsonObject();
        state.addProperty("name", mapping.getName());
        state.addProperty("key", key.getName());
        state.addProperty("default_key", defaultKey.getName());
        state.addProperty("is_toggle", mapping instanceof ToggleKeyMapping);
        state.addProperty("is_default", mapping.isDefault());
        state.addProperty("is_unbound", mapping.isUnbound());
        state.addProperty("save", mapping.saveString());
        state.addProperty("matches_key_event", mapping.matches(keyEvent));
        state.addProperty("matches_unknown_key_event", mapping.matches(unknownKeyEvent));
        state.addProperty("matches_mouse", mapping.matchesMouse(mouseEvent));
        return state;
    }

    static JsonObject keyboardInputCase(JsonObject request) {
        var result = new JsonObject();
        Input keys = new Input(
            request.get("up").getAsBoolean(),
            request.get("down").getAsBoolean(),
            request.get("left").getAsBoolean(),
            request.get("right").getAsBoolean(),
            request.get("jump").getAsBoolean(),
            request.get("shift").getAsBoolean(),
            request.get("sprint").getAsBoolean());
        KeyboardInput keyboard = new KeyboardInput(optionsFor(keys));
        keyboard.tick();
        result.add("key_presses", snapshot(keyboard.keyPresses));
        result.add("move_vector", snapshot(keyboard.getMoveVector()));
        return result;
    }

    static JsonObject characterEventCase(JsonObject request) {
        var result = new JsonObject();
        CharacterEvent event = new CharacterEvent(request.get("codepoint").getAsInt());
        result.addProperty("codepoint", event.codepoint());
        result.add("utf16", snapshotUtf16(event.codepointAsString()));
        result.addProperty("is_allowed_chat_character", event.isAllowedChatCharacter());
        return result;
    }

    static JsonObject preeditEventCase(JsonObject request) {
        var result = new JsonObject();
        JsonArray codepointValues = request.getAsJsonArray("codepoints");
        JsonArray blockSizeValues = request.getAsJsonArray("block_sizes");
        int preeditSize = codepointValues.size();
        PreeditEvent event;
        if (preeditSize == 0) {
            event = PreeditEvent.createFromCallback(
                0, 0L, blockSizeValues.size(), 0L,
                request.get("focused_block").getAsInt(), request.get("caret").getAsInt());
        } else {
            Unsafe unsafe = unsafeInstance();
            long codepoints = unsafe.allocateMemory((long)preeditSize * Integer.BYTES);
            long blockSizes = unsafe.allocateMemory((long)blockSizeValues.size() * Integer.BYTES);
            try {
                for (int i = 0; i < codepointValues.size(); i++) {
                    unsafe.putInt(codepoints + (long)i * Integer.BYTES, codepointValues.get(i).getAsInt());
                }
                for (int i = 0; i < blockSizeValues.size(); i++) {
                    unsafe.putInt(blockSizes + (long)i * Integer.BYTES, blockSizeValues.get(i).getAsInt());
                }
                event = PreeditEvent.createFromCallback(
                    preeditSize, codepoints, blockSizeValues.size(), blockSizes,
                    request.get("focused_block").getAsInt(),
                    request.get("caret").getAsInt());
            } finally {
                unsafe.freeMemory(codepoints);
                unsafe.freeMemory(blockSizes);
            }
        }
        if (event == null) {
            result.add("event", JsonNull.INSTANCE);
        } else {
            result.add("event", snapshotPreedit(event));
        }
        return result;
    }

    static JsonObject scrollWheelCase(JsonObject request) {
        var result = new JsonObject();
        ScrollWheelHandler handler = new ScrollWheelHandler();
        var events = new JsonArray();
        for (JsonElement entry : request.getAsJsonArray("scrolls")) {
            JsonObject step = entry.getAsJsonObject();
            Vector2i wheel = handler.onMouseScroll(f64(step, "x"), f64(step, "y"));
            var state = new JsonObject();
            state.addProperty("x", wheel.x);
            state.addProperty("y", wheel.y);
            events.add(state);
        }
        result.add("scrolls", events);
        var selections = new JsonArray();
        for (JsonElement entry : request.getAsJsonArray("selections")) {
            JsonObject selection = entry.getAsJsonObject();
            double wheel = f64(selection, "wheel");
            int current = selection.get("current").getAsInt();
            int limit = selection.get("limit").getAsInt();
            var state = new JsonObject();
            state.addProperty("wheel", Oracle.bits(wheel));
            state.addProperty("current", current);
            state.addProperty("limit", limit);
            state.addProperty("selected", ScrollWheelHandler.getNextScrollWheelSelection(wheel, current, limit));
            selections.add(state);
        }
        result.add("selections", selections);
        return result;
    }

    static JsonObject inputWithModifiersCase(JsonObject request) {
        var result = new JsonObject();
        KeyEvent keyEvent = new KeyEvent(
            request.get("key").getAsInt(), request.get("scancode").getAsInt(), request.get("modifiers").getAsInt());
        MouseButtonInfo buttonInfo = new MouseButtonInfo(
            request.get("mouse_button").getAsInt(), request.get("modifiers").getAsInt());
        MouseButtonEvent mouseEvent = new MouseButtonEvent(
            f64(request, "mouse_x"), f64(request, "mouse_y"), buttonInfo);
        JsonObject key = snapshotInputWithModifiers(keyEvent);
        key.addProperty("key", keyEvent.key());
        key.addProperty("scancode", keyEvent.scancode());
        JsonObject info = snapshotInputWithModifiers(buttonInfo);
        info.addProperty("button", buttonInfo.button());
        JsonObject mouse = snapshotInputWithModifiers(mouseEvent);
        mouse.addProperty("x", Oracle.bits(mouseEvent.x()));
        mouse.addProperty("y", Oracle.bits(mouseEvent.y()));
        mouse.addProperty("button", mouseEvent.button());
        result.add("key_event", key);
        result.add("mouse_button_info", info);
        result.add("mouse_button_event", mouse);
        return result;
    }

    static JsonObject keyMappingCase(JsonObject request) {
        var result = new JsonObject();
        KeyMapping.releaseAll();
        String prefix = "behavior.key-mapping." + request.get("case_id").getAsString();
        InputConstants.Key primaryKey = inputKey(request.getAsJsonObject("primary"));
        InputConstants.Key primaryDefaultKey = primaryKey;
        InputConstants.Key secondaryKey = inputKey(request.getAsJsonObject("secondary"));
        InputConstants.Key secondaryDefaultKey = secondaryKey;
        InputConstants.Key otherKey = inputKey(request.getAsJsonObject("other"));
        InputConstants.Key otherDefaultKey = otherKey;
        InputConstants.Key rebindKey = inputKey(request.getAsJsonObject("rebind"));
        KeyMapping primary = newKeyMapping(prefix + ".primary", primaryKey);
        KeyMapping secondary = newKeyMapping(prefix + ".secondary", secondaryKey);
        KeyMapping other = newKeyMapping(prefix + ".other", otherKey);
        KeyEvent keyEvent = new KeyEvent(
            request.getAsJsonObject("key_event").get("key").getAsInt(),
            request.getAsJsonObject("key_event").get("scancode").getAsInt(), 0);
        KeyEvent unknownKeyEvent = new KeyEvent(
            request.getAsJsonObject("unknown_key_event").get("key").getAsInt(),
            request.getAsJsonObject("unknown_key_event").get("scancode").getAsInt(), 0);
        MouseButtonEvent mouseEvent = new MouseButtonEvent(
            0.0, 0.0,
            new MouseButtonInfo(request.get("mouse_button").getAsInt(), 0));
        result.add("initial", keyMappingState(primary, secondary, other));
        result.add("initial_properties", keyMappingProperties(
            primary, secondary, other, primaryKey, primaryDefaultKey, secondaryKey,
            secondaryDefaultKey, otherKey, otherDefaultKey, keyEvent, unknownKeyEvent, mouseEvent));
        var events = new JsonArray();
        for (var entry : request.getAsJsonArray("actions")) {
            JsonObject action = entry.getAsJsonObject();
            String kind = action.get("op").getAsString();
            Boolean consumed = null;
            switch (kind) {
                case "set_down" -> mapping(action.get("target").getAsString(), primary, secondary, other)
                    .setDown(action.get("value").getAsBoolean());
                case "set" -> KeyMapping.set(actionKey(action.get("key").getAsString(), primaryKey,
                    primaryDefaultKey, secondaryKey, otherKey, rebindKey), action.get("value").getAsBoolean());
                case "click" -> KeyMapping.click(actionKey(action.get("key").getAsString(), primaryKey,
                    primaryDefaultKey, secondaryKey, otherKey, rebindKey));
                case "consume_click" -> consumed = mapping(action.get("target").getAsString(),
                    primary, secondary, other).consumeClick();
                case "set_key" -> {
                    String target = action.get("target").getAsString();
                    InputConstants.Key replacement = actionKey(action.get("key").getAsString(), primaryKey,
                        primaryDefaultKey, secondaryKey, otherKey, rebindKey);
                    mapping(target, primary, secondary, other).setKey(replacement);
                    switch (target) {
                        case "primary" -> primaryKey = replacement;
                        case "secondary" -> secondaryKey = replacement;
                        case "other" -> otherKey = replacement;
                        default -> throw new IllegalArgumentException("Unknown key mapping target");
                    }
                }
                case "release_all" -> KeyMapping.releaseAll();
                default -> throw new IllegalArgumentException("Unknown key mapping action");
            }
            JsonObject event = keyMappingState(primary, secondary, other);
            if (consumed != null) {
                event.addProperty("consumed", consumed);
            }
            events.add(event);
        }
        result.add("events", events);
        result.add("properties", keyMappingProperties(
            primary, secondary, other, primaryKey, primaryDefaultKey, secondaryKey,
            secondaryDefaultKey, otherKey, otherDefaultKey, keyEvent, unknownKeyEvent, mouseEvent));
        return result;
    }

    static JsonObject toggleKeyMappingCase(JsonObject request) {
        var result = new JsonObject();
        KeyMapping.releaseAll();
        String prefix = "behavior.key-mapping." + request.get("case_id").getAsString();
        InputConstants.Key primaryKey = inputKey(request.getAsJsonObject("primary"));
        InputConstants.Key secondaryKey = inputKey(request.getAsJsonObject("secondary"));
        InputConstants.Key otherKey = inputKey(request.getAsJsonObject("other"));
        boolean primaryNeedsToggle = request.get("primary_needs_toggle").getAsBoolean();
        boolean primaryShouldRestore = request.get("primary_should_restore").getAsBoolean();
        boolean secondaryNeedsToggle = request.get("secondary_needs_toggle").getAsBoolean();
        boolean secondaryShouldRestore = request.get("secondary_should_restore").getAsBoolean();
        boolean otherNeedsToggle = request.get("other_needs_toggle").getAsBoolean();
        boolean otherShouldRestore = request.get("other_should_restore").getAsBoolean();
        KeyMapping primary = new ToggleKeyMapping(prefix + ".primary", primaryKey.getType(), primaryKey.getValue(),
            KeyMapping.Category.MOVEMENT, () -> primaryNeedsToggle, primaryShouldRestore);
        KeyMapping secondary = new ToggleKeyMapping(prefix + ".secondary", secondaryKey.getType(), secondaryKey.getValue(),
            KeyMapping.Category.MOVEMENT, () -> secondaryNeedsToggle, secondaryShouldRestore);
        KeyMapping other = new ToggleKeyMapping(prefix + ".other", otherKey.getType(), otherKey.getValue(),
            KeyMapping.Category.MOVEMENT, () -> otherNeedsToggle, otherShouldRestore);
        KeyEvent keyEvent = new KeyEvent(
            request.getAsJsonObject("key_event").get("key").getAsInt(),
            request.getAsJsonObject("key_event").get("scancode").getAsInt(), 0);
        KeyEvent unknownKeyEvent = new KeyEvent(
            request.getAsJsonObject("unknown_key_event").get("key").getAsInt(),
            request.getAsJsonObject("unknown_key_event").get("scancode").getAsInt(), 0);
        MouseButtonEvent mouseEvent = new MouseButtonEvent(
            0.0, 0.0,
            new MouseButtonInfo(request.get("mouse_button").getAsInt(), 0));
        result.add("initial", keyMappingState(primary, secondary, other));
        result.add("initial_properties", keyMappingProperties(
            primary, secondary, other, primaryKey, primaryKey, secondaryKey,
            secondaryKey, otherKey, otherKey, keyEvent, unknownKeyEvent, mouseEvent));
        var events = new JsonArray();
        for (var entry : request.getAsJsonArray("actions")) {
            JsonObject action = entry.getAsJsonObject();
            String kind = action.get("op").getAsString();
            Boolean consumed = null;
            Boolean shouldRestore = null;
            switch (kind) {
                case "set_down" -> mapping(action.get("target").getAsString(), primary, secondary, other)
                    .setDown(action.get("value").getAsBoolean());
                case "set" -> KeyMapping.set(actionKey(action.get("key").getAsString(), primaryKey,
                    primaryKey, secondaryKey, otherKey, primaryKey), action.get("value").getAsBoolean());
                case "click" -> KeyMapping.click(actionKey(action.get("key").getAsString(), primaryKey,
                    primaryKey, secondaryKey, otherKey, primaryKey));
                case "consume_click" -> consumed = mapping(action.get("target").getAsString(),
                    primary, secondary, other).consumeClick();
                case "release_all" -> KeyMapping.releaseAll();
                case "restore_toggle_states" -> KeyMapping.restoreToggleStatesOnScreenClosed();
                case "reset_toggle_keys" -> KeyMapping.resetToggleKeys();
                case "should_restore" -> shouldRestore = ((ToggleKeyMapping)mapping(action.get("target").getAsString(),
                    primary, secondary, other)).shouldRestoreStateOnScreenClosed();
                default -> throw new IllegalArgumentException("Unknown toggle key mapping action");
            }
            JsonObject event = keyMappingState(primary, secondary, other);
            if (consumed != null) {
                event.addProperty("consumed", consumed);
            }
            if (shouldRestore != null) {
                event.addProperty("should_restore", shouldRestore);
            }
            events.add(event);
        }
        result.add("events", events);
        result.add("properties", keyMappingProperties(
            primary, secondary, other, primaryKey, primaryKey, secondaryKey,
            secondaryKey, otherKey, otherKey, keyEvent, unknownKeyEvent, mouseEvent));
        return result;
    }
}
