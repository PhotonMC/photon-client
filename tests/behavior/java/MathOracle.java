import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.mojang.blaze3d.platform.InputConstants;
import com.mojang.math.OctahedralGroup;
import com.mojang.serialization.DataResult;
import com.mojang.serialization.JsonOps;
import it.unimi.dsi.fastutil.longs.LongConsumer;
import java.util.ArrayList;
import java.util.EnumSet;
import java.util.Locale;
import java.util.UUID;
import java.util.function.IntPredicate;
import java.util.stream.Stream;
import net.minecraft.DetectedVersion;
import net.minecraft.SharedConstants;
import net.minecraft.WorldVersion;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.GraphicsPreset;
import net.minecraft.client.Options;
import net.minecraft.client.OptionInstance;
import net.minecraft.client.ScrollWheelHandler;
import net.minecraft.client.ToggleKeyMapping;
import net.minecraft.client.input.CharacterEvent;
import net.minecraft.client.input.KeyEvent;
import net.minecraft.client.input.InputWithModifiers;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.client.input.MouseButtonInfo;
import net.minecraft.client.input.PreeditEvent;
import net.minecraft.client.player.KeyboardInput;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.AxisCycle;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.SectionPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.util.Mth;
import net.minecraft.util.RandomSource;
import net.minecraft.util.SmoothDouble;
import net.minecraft.world.entity.Avatar;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EntityDimensions;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.entity.monster.Shulker;
import net.minecraft.world.entity.vehicle.boat.Boat;
import net.minecraft.world.entity.vehicle.minecart.Minecart;
import net.minecraft.world.level.BlockCollisions;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.CollisionGetter;
import net.minecraft.world.level.EntityGetter;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.LadderBlock;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.block.SlabBlock;
import net.minecraft.world.level.block.StairBlock;
import net.minecraft.world.level.block.TrapDoorBlock;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.SlabType;
import net.minecraft.world.level.block.state.properties.StairsShape;
import net.minecraft.world.level.levelgen.PositionalRandomFactory;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.level.border.WorldBorder;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.entity.LevelCallback;
import net.minecraft.world.level.entity.LevelEntityGetter;
import net.minecraft.world.level.entity.TransientEntitySectionManager;
import net.minecraft.world.level.block.state.properties.AttachFace;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.shapes.BooleanOp;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.VoxelShape;
import org.apache.commons.lang3.math.Fraction;
import org.joml.Quaternionf;
import org.joml.Vector3f;
import org.joml.Vector2i;
import sun.misc.Unsafe;

public final class MathOracle {
    private static final WorldVersion TEST_VERSION = DetectedVersion.createBuiltIn("26.2", "26.2");

    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        var result = new JsonObject();
        if (operation.equals("vec2")) {
            Vec2 vector = new Vec2(Oracle.f32(request, "x"), Oracle.f32(request, "y"));
            var events = new JsonArray();
            events.add(snapshot(vector));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                vector = switch (step.get("action").getAsString()) {
                    case "normalize" -> vector.normalized();
                    case "scale" -> vector.scale(Oracle.f32(step, "factor"));
                    default -> throw new IllegalArgumentException("Unknown Vec2 action");
                };
                events.add(snapshot(vector));
            }
            result.add("events", events);
        } else if (operation.equals("keyboard_input")) {
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
        } else if (operation.equals("character_event")) {
            CharacterEvent event = new CharacterEvent(request.get("codepoint").getAsInt());
            result.addProperty("codepoint", event.codepoint());
            result.add("utf16", snapshotUtf16(event.codepointAsString()));
            result.addProperty("is_allowed_chat_character", event.isAllowedChatCharacter());
        } else if (operation.equals("preedit_event")) {
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
        } else if (operation.equals("option_values")) {
            OptionInstance.IntRange range = new OptionInstance.IntRange(
                request.get("min").getAsInt(), request.get("max").getAsInt(),
                request.get("apply_value_immediately").getAsBoolean());
            result.addProperty("min", range.minInclusive());
            result.addProperty("max", range.maxInclusive());
            result.addProperty("apply_value_immediately", range.applyValueImmediately());
            var intValues = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("int_values")) {
                int value = entry.getAsInt();
                var state = new JsonObject();
                state.addProperty("value", value);
                state.addProperty("valid", range.validateValue(value).isPresent());
                state.addProperty("next", range.next(value).get());
                state.addProperty("previous", range.previous(value).get());
                state.addProperty("to_slider", Oracle.bits(range.toSliderValue(value)));
                intValues.add(state);
            }
            result.add("int_values", intValues);
            var sliders = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("sliders")) {
                double slider = f64(entry.getAsString());
                var state = new JsonObject();
                state.addProperty("slider", Oracle.bits(slider));
                state.addProperty("from_slider", range.fromSliderValue(slider));
                sliders.add(state);
            }
            result.add("sliders", sliders);
            var unitValues = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("unit_values")) {
                double value = f64(entry.getAsString());
                var state = new JsonObject();
                state.addProperty("value", Oracle.bits(value));
                var validated = OptionInstance.UnitDouble.INSTANCE.validateValue(value);
                state.addProperty("valid", validated.isPresent());
                validated.ifPresent(item -> state.addProperty("validated", Oracle.bits(item)));
                state.addProperty("to_slider", Oracle.bits(OptionInstance.UnitDouble.INSTANCE.toSliderValue(value)));
                state.addProperty("from_slider", Oracle.bits(OptionInstance.UnitDouble.INSTANCE.fromSliderValue(value)));
                unitValues.add(state);
            }
            result.add("unit_values", unitValues);
        } else if (operation.equals("sliderable_enum")) {
            OptionInstance.SliderableEnum<GraphicsPreset> valueSet = new OptionInstance.SliderableEnum<>(
                java.util.List.of(GraphicsPreset.values()), GraphicsPreset.CODEC);
            var values = new JsonArray();
            for (GraphicsPreset value : GraphicsPreset.values()) {
                values.add(value.name());
            }
            result.add("values", values);
            var probes = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("probes")) {
                GraphicsPreset value = GraphicsPreset.valueOf(entry.getAsJsonObject().get("value").getAsString());
                var state = new JsonObject();
                state.addProperty("value", value.name());
                state.addProperty("valid", valueSet.validateValue(value).isPresent());
                state.addProperty("to_slider", Oracle.bits(valueSet.toSliderValue(value)));
                state.addProperty("next", valueSet.next(value).orElseThrow().name());
                state.addProperty("previous", valueSet.previous(value).orElseThrow().name());
                probes.add(state);
            }
            result.add("probes", probes);
            var sliders = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("sliders")) {
                double slider = f64(entry.getAsString());
                var state = new JsonObject();
                state.addProperty("slider", Oracle.bits(slider));
                state.addProperty("from_slider", valueSet.fromSliderValue(slider).name());
                sliders.add(state);
            }
            result.add("sliders", sliders);
        } else if (operation.equals("cycleable_enum")) {
            var values = new ArrayList<String>();
            for (JsonElement entry : request.getAsJsonArray("values")) {
                values.add(entry.getAsString());
            }
            OptionInstance.Enum<String> valueSet = new OptionInstance.Enum<>(values, com.mojang.serialization.Codec.STRING);
            var valueListSupplier = valueSet.valueListSupplier();
            result.add("values", new JsonArray());
            for (String value : valueListSupplier.getDefaultList()) {
                result.getAsJsonArray("values").add(value);
            }
            result.add("selected_values", new JsonArray());
            for (String value : valueListSupplier.getSelectedList()) {
                result.getAsJsonArray("selected_values").add(value);
            }
            var probes = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("probes")) {
                String value = entry.getAsString();
                var state = new JsonObject();
                state.addProperty("value", value);
                var validated = valueSet.validateValue(value);
                state.addProperty("valid", validated.isPresent());
                validated.ifPresent(item -> state.addProperty("validated", item));
                probes.add(state);
            }
            result.add("probes", probes);
        } else if (operation.equals("alt_enum")) {
            var values = new ArrayList<String>();
            for (JsonElement entry : request.getAsJsonArray("values")) {
                values.add(entry.getAsString());
            }
            var altValues = new ArrayList<String>();
            for (JsonElement entry : request.getAsJsonArray("alt_values")) {
                altValues.add(entry.getAsString());
            }
            var conditions = new ArrayList<Boolean>();
            for (JsonElement entry : request.getAsJsonArray("conditions")) {
                conditions.add(entry.getAsBoolean());
            }
            int[] conditionCalls = {0};
            java.util.function.BooleanSupplier altCondition = () -> {
                int index = Math.min(conditionCalls[0]++, conditions.size() - 1);
                return conditions.get(index);
            };
            OptionInstance.AltEnum<String> valueSet = new OptionInstance.AltEnum<>(
                values, altValues, altCondition, (instance, value) -> {}, com.mojang.serialization.Codec.STRING);
            var valueListSupplier = valueSet.valueListSupplier();
            result.add("default_values", new JsonArray());
            for (String value : valueListSupplier.getDefaultList()) {
                result.getAsJsonArray("default_values").add(value);
            }
            result.add("alt_values", new JsonArray());
            for (String value : altValues) {
                result.getAsJsonArray("alt_values").add(value);
            }
            result.add("selected_values", new JsonArray());
            for (int i = 0; i < conditions.size(); i++) {
                result.getAsJsonArray("selected_values").add(new JsonArray());
                for (String value : valueListSupplier.getSelectedList()) {
                    result.getAsJsonArray("selected_values").get(i).getAsJsonArray().add(value);
                }
            }
            var probes = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("probes")) {
                String value = entry.getAsString();
                var state = new JsonObject();
                state.addProperty("value", value);
                var validated = valueSet.validateValue(value);
                state.addProperty("valid", validated.isPresent());
                validated.ifPresent(item -> state.addProperty("validated", item));
                probes.add(state);
            }
            result.add("probes", probes);
        } else if (operation.equals("lazy_enum")) {
            var valueLists = new ArrayList<java.util.List<String>>();
            for (JsonElement listElement : request.getAsJsonArray("value_lists")) {
                var values = new ArrayList<String>();
                for (JsonElement entry : listElement.getAsJsonArray()) {
                    values.add(entry.getAsString());
                }
                valueLists.add(values);
            }
            int[] supplierCalls = {0};
            java.util.function.Supplier<java.util.List<String>> supplier = () -> {
                int index = Math.min(supplierCalls[0]++, valueLists.size() - 1);
                return new ArrayList<>(valueLists.get(index));
            };
            var validValues = new ArrayList<String>();
            for (JsonElement entry : request.getAsJsonArray("valid_values")) {
                validValues.add(entry.getAsString());
            }
            java.util.function.Function<String, java.util.Optional<String>> validator = value ->
                validValues.contains(value) ? java.util.Optional.of(value) : java.util.Optional.empty();
            OptionInstance.LazyEnum<String> valueSet = new OptionInstance.LazyEnum<>(
                supplier, validator, com.mojang.serialization.Codec.STRING);
            var firstSupplier = valueSet.valueListSupplier();
            var secondSupplier = valueSet.valueListSupplier();
            result.add("first_default", new JsonArray());
            for (String value : firstSupplier.getDefaultList()) {
                result.getAsJsonArray("first_default").add(value);
            }
            result.add("first_selected", new JsonArray());
            for (String value : firstSupplier.getSelectedList()) {
                result.getAsJsonArray("first_selected").add(value);
            }
            result.add("second_default", new JsonArray());
            for (String value : secondSupplier.getDefaultList()) {
                result.getAsJsonArray("second_default").add(value);
            }
            result.add("second_selected", new JsonArray());
            for (String value : secondSupplier.getSelectedList()) {
                result.getAsJsonArray("second_selected").add(value);
            }
            var probes = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("probes")) {
                String value = entry.getAsString();
                var state = new JsonObject();
                state.addProperty("value", value);
                var validated = valueSet.validateValue(value);
                state.addProperty("valid", validated.isPresent());
                validated.ifPresent(item -> state.addProperty("validated", item));
                probes.add(state);
            }
            result.add("probes", probes);
        } else if (operation.equals("clamping_lazy_max_int_range")) {
            var maxValues = new ArrayList<Integer>();
            for (JsonElement entry : request.getAsJsonArray("max_values")) {
                maxValues.add(entry.getAsInt());
            }
            int[] maxCalls = {0};
            java.util.function.IntSupplier maxSupplier = () -> {
                int index = Math.min(maxCalls[0]++, maxValues.size() - 1);
                return maxValues.get(index);
            };
            OptionInstance.ClampingLazyMaxIntRange range = new OptionInstance.ClampingLazyMaxIntRange(
                request.get("min").getAsInt(), maxSupplier, request.get("encodable_max").getAsInt());
            result.addProperty("min", range.minInclusive());
            result.addProperty("encodable_max", range.encodableMaxInclusive());
            var maxima = new JsonArray();
            maxima.add(range.maxInclusive());
            maxima.add(range.maxInclusive());
            result.add("maxima", maxima);
            result.addProperty("create_cycle_button", range.createCycleButton());
            var valueListSupplier = range.valueListSupplier();
            result.add("values", new JsonArray());
            for (Integer value : valueListSupplier.getDefaultList()) {
                result.getAsJsonArray("values").add(value);
            }
            result.add("selected_values", new JsonArray());
            for (Integer value : valueListSupplier.getSelectedList()) {
                result.getAsJsonArray("selected_values").add(value);
            }
            var probes = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("probes")) {
                int value = entry.getAsInt();
                var state = new JsonObject();
                state.addProperty("value", value);
                state.addProperty("validated", range.validateValue(value).orElseThrow());
                state.addProperty("next", range.next(value).orElseThrow());
                state.addProperty("previous", range.previous(value).orElseThrow());
                state.addProperty("to_slider", Oracle.bits(range.toSliderValue(value)));
                probes.add(state);
            }
            result.add("probes", probes);
            var sliders = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("sliders")) {
                double slider = f64(entry.getAsString());
                var state = new JsonObject();
                state.addProperty("slider", Oracle.bits(slider));
                state.addProperty("from_slider", range.fromSliderValue(slider));
                sliders.add(state);
            }
            result.add("sliders", sliders);
        } else if (operation.equals("mapped_int_range")) {
            OptionInstance.IntRange range = new OptionInstance.IntRange(
                request.get("min").getAsInt(), request.get("max").getAsInt(), true);
            boolean discrete = request.get("discrete").getAsBoolean();
            OptionInstance.SliderableValueSet<String> mapped = range.xmap(
                Integer::toString, Integer::parseInt, discrete);
            var values = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("values")) {
                String value = entry.getAsString();
                var state = new JsonObject();
                state.addProperty("value", value);
                var validated = mapped.validateValue(value);
                state.addProperty("valid", validated.isPresent());
                validated.ifPresent(item -> state.addProperty("validated", item));
                state.addProperty("to_slider", Oracle.bits(mapped.toSliderValue(value)));
                state.add("next", JsonNull.INSTANCE);
                mapped.next(value).ifPresent(item -> state.addProperty("next", item));
                state.add("previous", JsonNull.INSTANCE);
                mapped.previous(value).ifPresent(item -> state.addProperty("previous", item));
                values.add(state);
            }
            result.add("values", values);
            var sliders = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("sliders")) {
                double slider = f64(entry.getAsString());
                var state = new JsonObject();
                state.addProperty("slider", Oracle.bits(slider));
                state.addProperty("from_slider", mapped.fromSliderValue(slider));
                sliders.add(state);
            }
            result.add("sliders", sliders);
        } else if (operation.equals("mapped_unit_double")) {
            OptionInstance.SliderableValueSet<Double> mapped = OptionInstance.UnitDouble.INSTANCE.xmap(
                Mth::square, Math::sqrt);
            var values = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("values")) {
                double value = f64(entry.getAsString());
                var state = new JsonObject();
                state.addProperty("value", Oracle.bits(value));
                var validated = mapped.validateValue(value);
                state.addProperty("valid", validated.isPresent());
                validated.ifPresent(item -> state.addProperty("validated", Oracle.bits(item)));
                state.addProperty("to_slider", Oracle.bits(mapped.toSliderValue(value)));
                values.add(state);
            }
            result.add("values", values);
            var sliders = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("sliders")) {
                double slider = f64(entry.getAsString());
                var state = new JsonObject();
                state.addProperty("slider", Oracle.bits(slider));
                state.addProperty("from_slider", Oracle.bits(mapped.fromSliderValue(slider)));
                sliders.add(state);
            }
            result.add("sliders", sliders);
        } else if (operation.equals("scroll_wheel")) {
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
        } else if (operation.equals("smooth_double")) {
            SmoothDouble smooth = new SmoothDouble();
            var events = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                String op = step.get("op").getAsString();
                if (op.equals("reset")) {
                    smooth.reset();
                    var state = new JsonObject();
                    state.addProperty("op", "reset");
                    events.add(state);
                } else if (op.equals("delta")) {
                    double value = smooth.getNewDeltaValue(f64(step, "target_delta"), f64(step, "time"));
                    var state = new JsonObject();
                    state.addProperty("op", "delta");
                    state.addProperty("value", Oracle.bits(value));
                    events.add(state);
                } else {
                    throw new IllegalArgumentException("Unknown SmoothDouble operation");
                }
            }
            result.add("events", events);
        } else if (operation.equals("input_with_modifiers")) {
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
        } else if (operation.equals("key_mapping")) {
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
        } else if (operation.equals("toggle_key_mapping")) {
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
        } else if (operation.equals("direction")) {
            Direction facing = Direction.byName(request.get("name").getAsString());
            if (facing == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            var events = new JsonArray();
            events.add(snapshot(facing));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                Direction.Axis axis = Direction.Axis.byName(step.get("axis").getAsString());
                if (axis == null) {
                    throw new IllegalArgumentException("Unknown axis name");
                }
                facing = step.get("clockwise").getAsBoolean()
                    ? facing.getClockWise(axis) : facing.getCounterClockWise(axis);
                events.add(snapshot(facing));
            }
            result.add("events", events);
        } else if (operation.equals("direction_axial_rotation")) {
            Direction facing = Direction.byName(request.get("name").getAsString());
            if (facing == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            result.add("clockwise", snapshot(facing.getClockWise(axis)));
            result.add("counter_clockwise", snapshot(facing.getCounterClockWise(axis)));
        } else if (operation.equals("direction_y_rotation")) {
            Direction facing = Direction.byName(request.get("name").getAsString());
            if (facing == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            var clockwise = new JsonObject();
            try {
                clockwise.addProperty("outcome", "value");
                clockwise.add("value", snapshot(facing.getClockWise()));
            } catch (IllegalStateException error) {
                clockwise.addProperty("outcome", "error");
                clockwise.addProperty("error", error.getMessage());
            }
            var counterClockwise = new JsonObject();
            try {
                counterClockwise.addProperty("outcome", "value");
                counterClockwise.add("value", snapshot(facing.getCounterClockWise()));
            } catch (IllegalStateException error) {
                counterClockwise.addProperty("outcome", "error");
                counterClockwise.addProperty("error", error.getMessage());
            }
            result.add("clockwise", clockwise);
            result.add("counter_clockwise", counterClockwise);
        } else if (operation.equals("direction_data")) {
            int value = Integer.parseInt(request.get("value").getAsString());
            Direction facing = switch (request.get("type").getAsString()) {
                case "3d" -> Direction.from3DDataValue(value);
                case "2d" -> Direction.from2DDataValue(value);
                default -> throw new IllegalArgumentException("Unknown Direction data type");
            };
            result.add("value", snapshot(facing));
        } else if (operation.equals("direction_y_rot")) {
            result.add("value", snapshot(Direction.fromYRot(f64(request, "value"))));
        } else if (operation.equals("direction_axis")) {
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            Direction.AxisDirection direction = switch (request.get("direction").getAsString()) {
                case "positive" -> Direction.AxisDirection.POSITIVE;
                case "negative" -> Direction.AxisDirection.NEGATIVE;
                default -> throw new IllegalArgumentException("Unknown axis direction");
            };
            result.add("value", snapshot(Direction.fromAxisAndDirection(axis, direction)));
        } else if (operation.equals("direction_get")) {
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            Direction.AxisDirection direction = switch (request.get("direction").getAsString()) {
                case "positive" -> Direction.AxisDirection.POSITIVE;
                case "negative" -> Direction.AxisDirection.NEGATIVE;
                default -> throw new IllegalArgumentException("Unknown axis direction");
            };
            result.add("value", snapshot(Direction.get(direction, axis)));
        } else if (operation.equals("direction_unit_vectors")) {
            Direction facing = Direction.byName(request.get("name").getAsString());
            if (facing == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            var unitVec3i = new JsonObject();
            unitVec3i.addProperty("x", facing.getUnitVec3i().getX());
            unitVec3i.addProperty("y", facing.getUnitVec3i().getY());
            unitVec3i.addProperty("z", facing.getUnitVec3i().getZ());
            result.add("unit_vec3i", unitVec3i);
            result.add("unit_vec3", snapshot(facing.getUnitVec3()));
            result.add("unit_vec3f", snapshot(new Vector3f(facing.getUnitVec3f())));
        } else if (operation.equals("direction_step")) {
            Direction facing = Direction.byName(request.get("name").getAsString());
            if (facing == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            result.addProperty("step_x", facing.getStepX());
            result.addProperty("step_y", facing.getStepY());
            result.addProperty("step_z", facing.getStepZ());
            result.add("step", snapshot(facing.step()));
        } else if (operation.equals("direction_axis_orientation")) {
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            result.addProperty("vertical", axis.isVertical());
            result.addProperty("horizontal", axis.isHorizontal());
        } else if (operation.equals("direction_axis_directions")) {
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            var directions = new JsonArray();
            for (Direction direction : axis.getDirections()) {
                directions.add(snapshot(direction));
            }
            result.add("positive", snapshot(axis.getPositive()));
            result.add("negative", snapshot(axis.getNegative()));
            result.add("directions", directions);
        } else if (operation.equals("direction_axis_direction_values")) {
            Direction.AxisDirection direction = switch (request.get("direction").getAsString()) {
                case "positive" -> Direction.AxisDirection.POSITIVE;
                case "negative" -> Direction.AxisDirection.NEGATIVE;
                default -> throw new IllegalArgumentException("Unknown axis direction");
            };
            result.addProperty("step", direction.getStep());
            result.addProperty("name", direction.getName());
            result.addProperty("opposite", direction.opposite().getName());
        } else if (operation.equals("direction_axis_choose")) {
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            int intValue = axis.choose(
                Integer.parseInt(request.get("int_x").getAsString()),
                Integer.parseInt(request.get("int_y").getAsString()),
                Integer.parseInt(request.get("int_z").getAsString()));
            double doubleValue = axis.choose(
                f64(request, "double_x"), f64(request, "double_y"), f64(request, "double_z"));
            boolean booleanValue = axis.choose(
                request.get("bool_x").getAsBoolean(), request.get("bool_y").getAsBoolean(),
                request.get("bool_z").getAsBoolean());
            result.addProperty("int", intValue);
            result.addProperty("double", Oracle.bits(doubleValue));
            result.addProperty("boolean", booleanValue);
        } else if (operation.equals("direction_axis_test")) {
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            Direction input = request.get("direction").getAsString().equals("none")
                ? null : Direction.byName(request.get("direction").getAsString());
            if (!request.get("direction").getAsString().equals("none") && input == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            result.addProperty("value", axis.test(input));
        } else if (operation.equals("direction_axis_plane")) {
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            result.addProperty("plane", axis.getPlane() == Direction.Plane.HORIZONTAL
                ? "horizontal" : "vertical");
        } else if (operation.equals("direction_plane_test")) {
            Direction.Plane plane = switch (request.get("plane").getAsString()) {
                case "horizontal" -> Direction.Plane.HORIZONTAL;
                case "vertical" -> Direction.Plane.VERTICAL;
                default -> throw new IllegalArgumentException("Unknown plane");
            };
            Direction input = request.get("direction").getAsString().equals("none")
                ? null : Direction.byName(request.get("direction").getAsString());
            if (!request.get("direction").getAsString().equals("none") && input == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            result.addProperty("value", plane.test(input));
        } else if (operation.equals("direction_plane_order")) {
            Direction.Plane plane = switch (request.get("plane").getAsString()) {
                case "horizontal" -> Direction.Plane.HORIZONTAL;
                case "vertical" -> Direction.Plane.VERTICAL;
                default -> throw new IllegalArgumentException("Unknown plane");
            };
            var faces = new JsonArray();
            for (Direction direction : plane) {
                faces.add(direction.getName());
            }
            var streamFaces = new JsonArray();
            plane.stream().forEach(direction -> streamFaces.add(direction.getName()));
            var axes = new JsonArray();
            if (plane == Direction.Plane.HORIZONTAL) {
                axes.add(Direction.Axis.X.getName());
                axes.add(Direction.Axis.Z.getName());
            } else {
                axes.add(Direction.Axis.Y.getName());
            }
            result.add("faces", faces);
            result.add("stream_faces", streamFaces);
            result.add("axes", axes);
            result.addProperty("length", plane.length());
        } else if (operation.equals("direction_random")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            Direction.Plane plane = switch (request.get("plane").getAsString()) {
                case "horizontal" -> Direction.Plane.HORIZONTAL;
                case "vertical" -> Direction.Plane.VERTICAL;
                default -> throw new IllegalArgumentException("Unknown plane");
            };
            result.add("direction", snapshot(Direction.getRandom(random)));
            result.addProperty("axis", Direction.Axis.getRandom(random).getName());
            result.add("plane_direction", snapshot(plane.getRandomDirection(random)));
            result.addProperty("plane_axis", plane.getRandomAxis(random).getName());
            result.addProperty("tail", Integer.toString(random.nextInt()));
        } else if (operation.equals("direction_plane_shuffle")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            Direction.Plane plane = switch (request.get("plane").getAsString()) {
                case "horizontal" -> Direction.Plane.HORIZONTAL;
                case "vertical" -> Direction.Plane.VERTICAL;
                default -> throw new IllegalArgumentException("Unknown plane");
            };
            var faces = new JsonArray();
            for (Direction direction : plane.shuffledCopy(random)) {
                faces.add(direction.getName());
            }
            result.add("faces", faces);
            result.addProperty("tail", Integer.toString(random.nextInt()));
        } else if (operation.equals("direction_all_shuffle")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            var faces = new JsonArray();
            for (Direction direction : Direction.allShuffled(random)) {
                faces.add(direction.getName());
            }
            result.add("faces", faces);
            result.addProperty("tail", Integer.toString(random.nextInt()));
        } else if (operation.equals("direction_stream")) {
            var faces = new JsonArray();
            Direction.stream().forEach(direction -> faces.add(direction.getName()));
            result.add("faces", faces);
        } else if (operation.equals("direction_to_y_rot")) {
            Direction facing = Direction.byName(request.get("name").getAsString());
            if (facing == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            result.addProperty("bits", Oracle.bits(facing.toYRot()));
        } else if (operation.equals("direction_get_y_rot")) {
            Direction facing = Direction.byName(request.get("name").getAsString());
            if (facing == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            try {
                result.addProperty("outcome", "value");
                result.addProperty("bits", Oracle.bits(Direction.getYRot(facing)));
            } catch (IllegalStateException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", error.getMessage());
            }
        } else if (operation.equals("direction_rotation")) {
            Direction facing = Direction.byName(request.get("name").getAsString());
            if (facing == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            result.add("value", snapshot(facing.getRotation()));
        } else if (operation.equals("direction_name_lookup")) {
            String name = request.get("name").getAsString();
            if (request.get("kind").getAsString().equals("direction")) {
                Direction facing = Direction.byName(name);
                result.addProperty("found", facing != null);
                if (facing != null) {
                    result.addProperty("name", facing.getName());
                    result.addProperty("serialized_name", facing.getSerializedName());
                    result.addProperty("text", facing.toString());
                }
            } else if (request.get("kind").getAsString().equals("axis")) {
                Direction.Axis axis = Direction.Axis.byName(name);
                result.addProperty("found", axis != null);
                if (axis != null) {
                    result.addProperty("name", axis.getName());
                    result.addProperty("serialized_name", axis.getSerializedName());
                    result.addProperty("text", axis.toString());
                }
            } else {
                throw new IllegalArgumentException("Unknown name lookup kind");
            }
        } else if (operation.equals("direction_enum_text")) {
            String name = request.get("name").getAsString();
            if (request.get("kind").getAsString().equals("axis_direction")) {
                Direction.AxisDirection direction = switch (name) {
                    case "positive" -> Direction.AxisDirection.POSITIVE;
                    case "negative" -> Direction.AxisDirection.NEGATIVE;
                    default -> throw new IllegalArgumentException("Unknown axis direction");
                };
                result.addProperty("enum_name", direction.name());
                result.addProperty("name", direction.getName());
                result.addProperty("text", direction.toString());
            } else if (request.get("kind").getAsString().equals("plane")) {
                Direction.Plane plane = switch (name) {
                    case "horizontal" -> Direction.Plane.HORIZONTAL;
                    case "vertical" -> Direction.Plane.VERTICAL;
                    default -> throw new IllegalArgumentException("Unknown plane");
                };
                result.addProperty("enum_name", plane.name());
                result.addProperty("text", plane.toString());
            } else {
                throw new IllegalArgumentException("Unknown enum text kind");
            }
        } else if (operation.equals("direction_enum_name")) {
            String name = request.get("name").getAsString();
            if (request.get("kind").getAsString().equals("direction")) {
                Direction facing = Direction.byName(name);
                if (facing == null) {
                    throw new IllegalArgumentException("Unknown direction name");
                }
                result.addProperty("enum_name", facing.name());
                result.addProperty("name", facing.getName());
                result.addProperty("serialized_name", facing.getSerializedName());
                result.addProperty("text", facing.toString());
            } else if (request.get("kind").getAsString().equals("axis")) {
                Direction.Axis axis = Direction.Axis.byName(name);
                if (axis == null) {
                    throw new IllegalArgumentException("Unknown axis name");
                }
                result.addProperty("enum_name", axis.name());
                result.addProperty("name", axis.getName());
                result.addProperty("serialized_name", axis.getSerializedName());
                result.addProperty("text", axis.toString());
            } else {
                throw new IllegalArgumentException("Unknown enum name kind");
            }
        } else if (operation.equals("direction_approximate_nearest")) {
            result.add("value", snapshot(Direction.getApproximateNearest(
                f64(request, "x"), f64(request, "y"), f64(request, "z"))));
        } else if (operation.equals("direction_approximate_nearest_overloads")) {
            result.add("f32", snapshot(Direction.getApproximateNearest(
                Oracle.f32(request, "f32_x"), Oracle.f32(request, "f32_y"), Oracle.f32(request, "f32_z"))));
            result.add("f64", snapshot(Direction.getApproximateNearest(
                f64(request, "x"), f64(request, "y"), f64(request, "z"))));
            result.add("vec3", snapshot(Direction.getApproximateNearest(vec3(request))));
        } else if (operation.equals("direction_nearest")) {
            Direction fallback = directionOrElse(request);
            Direction facing;
            if (request.get("kind").getAsString().equals("components")) {
                facing = Direction.getNearest(
                    Integer.parseInt(request.get("x").getAsString()),
                    Integer.parseInt(request.get("y").getAsString()),
                    Integer.parseInt(request.get("z").getAsString()),
                    fallback);
            } else if (request.get("kind").getAsString().equals("vec3i")) {
                facing = Direction.getNearest(blockPos(request), fallback);
            } else {
                throw new IllegalArgumentException("Unknown Direction nearest kind");
            }
            if (facing == null) {
                result.add("value", null);
            } else {
                result.add("value", snapshot(facing));
            }
        } else if (operation.equals("direction_axis_step_order")) {
            var axes = new JsonArray();
            for (Direction.Axis axis : Direction.axisStepOrder(vec3(request))) {
                axes.add(axis.getName());
            }
            result.add("axes", axes);
        } else if (operation.equals("direction_is_facing_angle")) {
            Direction facing = Direction.byName(request.get("name").getAsString());
            if (facing == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            result.addProperty("value", facing.isFacingAngle(Oracle.f32(request, "angle")));
        } else if (operation.equals("coord")) {
            int value = Integer.parseInt(request.get("value").getAsString());
            String fn = request.get("fn").getAsString();
            int out = switch (fn) {
                case "block_to_section" -> SectionPos.blockToSectionCoord(value);
                case "section_to_block" -> SectionPos.sectionToBlockCoord(value);
                case "section_relative" -> SectionPos.sectionRelative(value);
                default -> throw new IllegalArgumentException("Unknown coord function");
            };
            result.addProperty("value", Integer.toString(out));
            return result;
        } else if (operation.equals("blockpos")) {
            BlockPos pos = blockPos(request);
            var events = new JsonArray();
            events.add(snapshot(pos));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                pos = switch (step.get("action").getAsString()) {
                    case "offset" -> pos.offset(
                        Integer.parseInt(step.get("x").getAsString()),
                        Integer.parseInt(step.get("y").getAsString()),
                        Integer.parseInt(step.get("z").getAsString()));
                    case "min" -> BlockPos.min(pos, blockPos(step));
                    case "max" -> BlockPos.max(pos, blockPos(step));
                    default -> throw new IllegalArgumentException("Unknown BlockPos action");
                };
                events.add(snapshot(pos));
            }
            result.add("events", events);
        } else if (operation.equals("blockpos_packed_helpers")) {
            BlockPos pos = blockPos(request);
            long packed = pos.asLong();
            long neighborNode = Long.parseUnsignedLong(request.get("neighbor_node").getAsString(), 16);
            Direction direction = Direction.byName(request.get("direction").getAsString());
            if (direction == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            int stepX = Integer.parseInt(request.get("step_x").getAsString());
            int stepY = Integer.parseInt(request.get("step_y").getAsString());
            int stepZ = Integer.parseInt(request.get("step_z").getAsString());
            var accessors = new JsonObject();
            accessors.addProperty("x", Integer.toString(BlockPos.getX(packed)));
            accessors.addProperty("y", Integer.toString(BlockPos.getY(packed)));
            accessors.addProperty("z", Integer.toString(BlockPos.getZ(packed)));
            result.add("zero", snapshot(BlockPos.ZERO));
            result.addProperty("packed_horizontal_length", BlockPos.PACKED_HORIZONTAL_LENGTH);
            result.addProperty("packed_y_length", BlockPos.PACKED_Y_LENGTH);
            result.addProperty("max_horizontal_coordinate", BlockPos.MAX_HORIZONTAL_COORDINATE);
            result.addProperty("packed", Long.toUnsignedString(packed, 16));
            result.addProperty("coordinate_packed", Long.toUnsignedString(BlockPos.asLong(
                pos.getX(), pos.getY(), pos.getZ()), 16));
            result.add("decoded", snapshot(BlockPos.of(packed)));
            result.add("accessors", accessors);
            result.addProperty("flat_index", Long.toUnsignedString(BlockPos.getFlatIndex(neighborNode), 16));
            result.addProperty("offset_direction", Long.toUnsignedString(
                BlockPos.offset(neighborNode, direction), 16));
            result.addProperty("offset_steps", Long.toUnsignedString(
                BlockPos.offset(neighborNode, stepX, stepY, stepZ), 16));
        } else if (operation.equals("blockpos_relative_helpers")) {
            BlockPos pos = blockPos(request);
            Direction direction = Direction.byName(request.get("direction").getAsString());
            if (direction == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            int steps = Integer.parseInt(request.get("steps").getAsString());
            var cardinal = new JsonObject();
            cardinal.add("above", snapshot(pos.above()));
            cardinal.add("above_steps", snapshot(pos.above(steps)));
            cardinal.add("below", snapshot(pos.below()));
            cardinal.add("below_steps", snapshot(pos.below(steps)));
            cardinal.add("north", snapshot(pos.north()));
            cardinal.add("north_steps", snapshot(pos.north(steps)));
            cardinal.add("south", snapshot(pos.south()));
            cardinal.add("south_steps", snapshot(pos.south(steps)));
            cardinal.add("west", snapshot(pos.west()));
            cardinal.add("west_steps", snapshot(pos.west(steps)));
            cardinal.add("east", snapshot(pos.east()));
            cardinal.add("east_steps", snapshot(pos.east(steps)));
            result.add("relative_direction", snapshot(pos.relative(direction)));
            result.add("relative_direction_steps", snapshot(pos.relative(direction, steps)));
            result.add("relative_axis_steps", snapshot(pos.relative(axis, steps)));
            result.add("cardinal", cardinal);
        } else if (operation.equals("blockpos_vector_helpers")) {
            BlockPos pos = blockPos(request);
            BlockPos delta = new BlockPos(
                Integer.parseInt(request.get("dx").getAsString()),
                Integer.parseInt(request.get("dy").getAsString()),
                Integer.parseInt(request.get("dz").getAsString()));
            int scale = Integer.parseInt(request.get("scale").getAsString());
            result.add("offset_vec3i", snapshot(pos.offset(delta)));
            result.add("subtract_vec3i", snapshot(pos.subtract(delta)));
            result.add("multiply", snapshot(pos.multiply(scale)));
        } else if (operation.equals("blockpos_geometry_helpers")) {
            BlockPos pos = blockPos(request);
            Rotation rotation = switch (request.get("rotation").getAsString()) {
                case "none" -> Rotation.NONE;
                case "clockwise_90" -> Rotation.CLOCKWISE_90;
                case "180" -> Rotation.CLOCKWISE_180;
                case "counterclockwise_90" -> Rotation.COUNTERCLOCKWISE_90;
                default -> throw new IllegalArgumentException("Unknown BlockPos rotation");
            };
            BlockPos upVector = new BlockPos(
                Integer.parseInt(request.get("up_x").getAsString()),
                Integer.parseInt(request.get("up_y").getAsString()),
                Integer.parseInt(request.get("up_z").getAsString()));
            int y = Integer.parseInt(request.get("at_y").getAsString());
            result.add("rotate", snapshot(pos.rotate(rotation)));
            result.add("cross", snapshot(pos.cross(upVector)));
            result.add("at_y", snapshot(pos.atY(y)));
        } else if (operation.equals("blockpos_distance_helpers")) {
            BlockPos pos = blockPos(request);
            BlockPos other = blockPos(request.getAsJsonObject("other"));
            JsonObject point = request.getAsJsonObject("point");
            double distance = f64(request, "distance");
            result.addProperty("dist_sqr", Oracle.bits(pos.distSqr(other)));
            result.addProperty("dist_low_corner_sqr", Oracle.bits(pos.distToLowCornerSqr(
                f64(point, "x"), f64(point, "y"), f64(point, "z"))));
            result.addProperty("dist_center_sqr", Oracle.bits(pos.distToCenterSqr(
                f64(point, "x"), f64(point, "y"), f64(point, "z"))));
            result.addProperty("closer_than", pos.closerThan(other, distance));
        } else if (operation.equals("blockpos_integer_helpers")) {
            BlockPos pos = blockPos(request);
            BlockPos other = blockPos(request.getAsJsonObject("other"));
            var selected = new JsonObject();
            selected.addProperty("x", pos.get(Direction.Axis.X));
            selected.addProperty("y", pos.get(Direction.Axis.Y));
            selected.addProperty("z", pos.get(Direction.Axis.Z));
            result.addProperty("dist_manhattan", pos.distManhattan(other));
            result.addProperty("dist_chessboard", pos.distChessboard(other));
            result.add("get", selected);
        } else if (operation.equals("blockpos_clamp_location")) {
            BlockPos pos = blockPos(request);
            result.add("value", snapshot(pos.clampLocationWithin(
                vec3(request.getAsJsonObject("location")))));
        } else if (operation.equals("blockpos_between_closed")) {
            BlockPos from = blockPos(request.getAsJsonObject("from"));
            BlockPos to = blockPos(request.getAsJsonObject("to"));
            var fromTo = new JsonArray();
            for (BlockPos pos : BlockPos.betweenClosed(from, to)) {
                fromTo.add(snapshot(pos));
            }
            int minX = Integer.parseInt(request.get("min_x").getAsString());
            int minY = Integer.parseInt(request.get("min_y").getAsString());
            int minZ = Integer.parseInt(request.get("min_z").getAsString());
            int maxX = Integer.parseInt(request.get("max_x").getAsString());
            int maxY = Integer.parseInt(request.get("max_y").getAsString());
            int maxZ = Integer.parseInt(request.get("max_z").getAsString());
            var coordinates = new JsonArray();
            for (BlockPos pos : BlockPos.betweenClosed(minX, minY, minZ, maxX, maxY, maxZ)) {
                coordinates.add(snapshot(pos));
            }
            result.add("from_to", fromTo);
            result.add("coordinates", coordinates);
        } else if (operation.equals("blockpos_neighbor_column")) {
            int startX = Integer.parseInt(request.get("start_x").getAsString());
            int startY = Integer.parseInt(request.get("start_y").getAsString());
            int startZ = Integer.parseInt(request.get("start_z").getAsString());
            int endY = Integer.parseInt(request.get("end_y").getAsString());
            var positions = new JsonArray();
            for (BlockPos pos : BlockPos.neighborColumn(startX, startY, startZ, endY)) {
                positions.add(snapshot(pos));
            }
            result.add("positions", positions);
        } else if (operation.equals("blockpos_square_out_south_east")) {
            BlockPos from = blockPos(request);
            var positions = new JsonArray();
            positions.add(snapshot(from));
            positions.add(snapshot(from.south()));
            positions.add(snapshot(from.east()));
            positions.add(snapshot(from.south().east()));
            result.add("positions", positions);
        } else if (operation.equals("blockpos_random_ranges")) {
            long seed = Long.parseLong(request.get("seed").getAsString());
            int limit = Integer.parseInt(request.get("limit").getAsString());
            int minX = Integer.parseInt(request.get("min_x").getAsString());
            int minY = Integer.parseInt(request.get("min_y").getAsString());
            int minZ = Integer.parseInt(request.get("min_z").getAsString());
            int maxX = Integer.parseInt(request.get("max_x").getAsString());
            int maxY = Integer.parseInt(request.get("max_y").getAsString());
            int maxZ = Integer.parseInt(request.get("max_z").getAsString());
            RandomSource betweenRandom = RandomSource.create(seed);
            var between = new JsonArray();
            for (BlockPos pos : BlockPos.randomBetweenClosed(
                betweenRandom, limit, minX, minY, minZ, maxX, maxY, maxZ)) {
                between.add(snapshot(pos));
            }
            int afterBetween = betweenRandom.nextInt();
            BlockPos center = blockPos(request.getAsJsonObject("center"));
            int size = Integer.parseInt(request.get("size").getAsString());
            RandomSource cubeRandom = RandomSource.create(seed);
            var cube = new JsonArray();
            for (BlockPos pos : BlockPos.randomInCube(cubeRandom, limit, center, size)) {
                cube.add(snapshot(pos));
            }
            int afterCube = cubeRandom.nextInt();
            result.add("between", between);
            result.addProperty("after_between", Integer.toString(afterBetween));
            result.add("cube", cube);
            result.addProperty("after_cube", Integer.toString(afterCube));
        } else if (operation.equals("blockpos_between_closed_aabb")) {
            AABB aabb = box(request.getAsJsonObject("aabb"));
            var positions = new JsonArray();
            for (BlockPos pos : BlockPos.betweenClosed(aabb)) {
                positions.add(snapshot(pos));
            }
            result.add("positions", positions);
        } else if (operation.equals("blockpos_within_manhattan")) {
            BlockPos origin = blockPos(request.getAsJsonObject("origin"));
            int reachX = Integer.parseInt(request.get("reach_x").getAsString());
            int reachY = Integer.parseInt(request.get("reach_y").getAsString());
            int reachZ = Integer.parseInt(request.get("reach_z").getAsString());
            var positions = new JsonArray();
            for (BlockPos pos : BlockPos.withinManhattan(origin, reachX, reachY, reachZ)) {
                positions.add(snapshot(pos));
            }
            var streamPositions = new JsonArray();
            BlockPos.withinManhattanStream(origin, reachX, reachY, reachZ)
                .forEach(pos -> streamPositions.add(snapshot(pos)));
            result.add("positions", positions);
            result.add("stream_positions", streamPositions);
        } else if (operation.equals("blockpos_spiral_around")) {
            BlockPos center = blockPos(request.getAsJsonObject("center"));
            int radius = Integer.parseInt(request.get("radius").getAsString());
            Direction firstDirection = Direction.byName(request.get("first_direction").getAsString());
            Direction secondDirection = Direction.byName(request.get("second_direction").getAsString());
            var positions = new JsonArray();
            for (BlockPos.MutableBlockPos pos : BlockPos.spiralAround(center, radius, firstDirection, secondDirection)) {
                positions.add(snapshot(pos));
            }
            result.add("positions", positions);
        } else if (operation.equals("blockpos_find_closest_match")) {
            BlockPos startPos = blockPos(request.getAsJsonObject("start"));
            int horizontalRadius = Integer.parseInt(request.get("horizontal_radius").getAsString());
            int verticalRadius = Integer.parseInt(request.get("vertical_radius").getAsString());
            BlockPos target = blockPos(request.getAsJsonObject("target"));
            var match = BlockPos.findClosestMatch(
                startPos, horizontalRadius, verticalRadius, pos -> pos.equals(target));
            if (match.isPresent()) {
                result.add("value", snapshot(match.get()));
            } else {
                result.add("value", com.google.gson.JsonNull.INSTANCE);
            }
        } else if (operation.equals("blockpos_between_closed_streams")) {
            BlockPos from = blockPos(request.getAsJsonObject("from"));
            BlockPos to = blockPos(request.getAsJsonObject("to"));
            var blockPositions = new JsonArray();
            BlockPos.betweenClosedStream(from, to)
                .forEach(pos -> blockPositions.add(snapshot(pos)));
            JsonObject bounds = request.getAsJsonObject("bounding_box");
            BoundingBox boundingBox = new BoundingBox(
                Integer.parseInt(bounds.get("min_x").getAsString()),
                Integer.parseInt(bounds.get("min_y").getAsString()),
                Integer.parseInt(bounds.get("min_z").getAsString()),
                Integer.parseInt(bounds.get("max_x").getAsString()),
                Integer.parseInt(bounds.get("max_y").getAsString()),
                Integer.parseInt(bounds.get("max_z").getAsString()));
            var boundingPositions = new JsonArray();
            BlockPos.betweenClosedStream(boundingBox)
                .forEach(pos -> boundingPositions.add(snapshot(pos)));
            AABB aabb = box(request.getAsJsonObject("aabb"));
            var aabbPositions = new JsonArray();
            BlockPos.betweenClosedStream(aabb)
                .forEach(pos -> aabbPositions.add(snapshot(pos)));
            int minX = Integer.parseInt(request.get("min_x").getAsString());
            int minY = Integer.parseInt(request.get("min_y").getAsString());
            int minZ = Integer.parseInt(request.get("min_z").getAsString());
            int maxX = Integer.parseInt(request.get("max_x").getAsString());
            int maxY = Integer.parseInt(request.get("max_y").getAsString());
            int maxZ = Integer.parseInt(request.get("max_z").getAsString());
            var coordinatePositions = new JsonArray();
            BlockPos.betweenClosedStream(minX, minY, minZ, maxX, maxY, maxZ)
                .forEach(pos -> coordinatePositions.add(snapshot(pos)));
            result.add("block", blockPositions);
            result.add("bounding_box", boundingPositions);
            result.add("aabb", aabbPositions);
            result.add("coordinates", coordinatePositions);
        } else if (operation.equals("blockpos_coordinate_helpers")) {
            double x = f64(request, "x");
            double y = f64(request, "y");
            double z = f64(request, "z");
            Vec3 point = vec3(request.getAsJsonObject("point"));
            BlockPos first = blockPos(request.getAsJsonObject("first"));
            BlockPos second = blockPos(request.getAsJsonObject("second"));
            result.add("containing", snapshot(BlockPos.containing(x, y, z)));
            result.add("containing_position", snapshot(BlockPos.containing(point)));
            result.add("min", snapshot(BlockPos.min(first, second)));
            result.add("max", snapshot(BlockPos.max(first, second)));
        } else if (operation.equals("blockpos_between_corners_direction")) {
            AABB aabb = box(request.getAsJsonObject("aabb"));
            BlockPos first = blockPos(request.getAsJsonObject("first"));
            BlockPos second = blockPos(request.getAsJsonObject("second"));
            int firstX = Integer.parseInt(request.get("first_x").getAsString());
            int firstY = Integer.parseInt(request.get("first_y").getAsString());
            int firstZ = Integer.parseInt(request.get("first_z").getAsString());
            int secondX = Integer.parseInt(request.get("second_x").getAsString());
            int secondY = Integer.parseInt(request.get("second_y").getAsString());
            int secondZ = Integer.parseInt(request.get("second_z").getAsString());
            Vec3 direction = vec3(request.getAsJsonObject("direction"));
            var aabbPositions = new JsonArray();
            for (BlockPos pos : BlockPos.betweenCornersInDirection(aabb, direction)) {
                aabbPositions.add(snapshot(pos));
            }
            var blockPositions = new JsonArray();
            for (BlockPos pos : BlockPos.betweenCornersInDirection(first, second, direction)) {
                blockPositions.add(snapshot(pos));
            }
            var coordinatePositions = new JsonArray();
            for (BlockPos pos : BlockPos.betweenCornersInDirection(
                firstX, firstY, firstZ, secondX, secondY, secondZ, direction)) {
                coordinatePositions.add(snapshot(pos));
            }
            result.add("aabb", aabbPositions);
            result.add("block_pos", blockPositions);
            result.add("coordinates", coordinatePositions);
        } else if (operation.equals("blockpos_breadth_first_traversal")) {
            BlockPos start = blockPos(request.getAsJsonObject("start"));
            int maxDepth = Integer.parseInt(request.get("max_depth").getAsString());
            int maxCount = Integer.parseInt(request.get("max_count").getAsString());
            JsonArray rules = request.getAsJsonArray("nodes");
            var processed = new JsonArray();
            var providerCalls = new JsonArray();
            int count = BlockPos.breadthFirstTraversal(
                start,
                maxDepth,
                maxCount,
                (pos, consumer) -> {
                    providerCalls.add(snapshot(pos));
                    for (JsonElement entry : rules) {
                        JsonObject rule = entry.getAsJsonObject();
                        if (pos.equals(blockPos(rule.getAsJsonObject("position")))) {
                            for (JsonElement neighbour : rule.getAsJsonArray("neighbors")) {
                                consumer.accept(blockPos(neighbour.getAsJsonObject()));
                            }
                            break;
                        }
                    }
                },
                pos -> {
                    processed.add(snapshot(pos));
                    for (JsonElement entry : rules) {
                        JsonObject rule = entry.getAsJsonObject();
                        if (pos.equals(blockPos(rule.getAsJsonObject("position")))) {
                            return switch (rule.get("status").getAsString()) {
                                case "accept" -> BlockPos.TraversalNodeStatus.ACCEPT;
                                case "skip" -> BlockPos.TraversalNodeStatus.SKIP;
                                case "stop" -> BlockPos.TraversalNodeStatus.STOP;
                                default -> throw new IllegalArgumentException("Unknown traversal status");
                            };
                        }
                    }
                    return BlockPos.TraversalNodeStatus.ACCEPT;
                });
            result.addProperty("count", count);
            result.add("processed", processed);
            result.add("provider_calls", providerCalls);
        } else if (operation.equals("blockpos_mutable_helpers")) {
            BlockPos initial = blockPos(request.getAsJsonObject("initial"));
            BlockPos.MutableBlockPos mutable = initial.mutable();
            var constructors = new JsonObject();
            constructors.add("immutable", snapshot(initial.immutable()));
            constructors.add("mutable", snapshot(initial.mutable()));
            constructors.add("double", snapshot(new BlockPos.MutableBlockPos(
                f64(request, "double_x"), f64(request, "double_y"), f64(request, "double_z"))));
            var events = new JsonArray();
            for (JsonElement entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                JsonObject event = new JsonObject();
                switch (step.get("op").getAsString()) {
                    case "set" -> mutable.set(
                        Integer.parseInt(step.get("x").getAsString()),
                        Integer.parseInt(step.get("y").getAsString()),
                        Integer.parseInt(step.get("z").getAsString()));
                    case "set_double" -> mutable.set(
                        f64(step, "x"), f64(step, "y"), f64(step, "z"));
                    case "set_block_pos" -> mutable.set(blockPos(step.getAsJsonObject("value")));
                    case "set_packed" -> mutable.set(Long.parseUnsignedLong(step.get("value").getAsString(), 16));
                    case "set_with_offset_direction" -> {
                        BlockPos pos = blockPos(step.getAsJsonObject("position"));
                        Direction direction = Direction.byName(step.get("direction").getAsString());
                        mutable.setWithOffset(pos, direction);
                    }
                    case "set_with_offset_components" -> mutable.setWithOffset(
                        blockPos(step.getAsJsonObject("position")),
                        Integer.parseInt(step.get("x").getAsString()),
                        Integer.parseInt(step.get("y").getAsString()),
                        Integer.parseInt(step.get("z").getAsString()));
                    case "set_with_offset_block_pos" -> mutable.setWithOffset(
                        blockPos(step.getAsJsonObject("position")),
                        blockPos(step.getAsJsonObject("offset")));
                    case "move_direction" -> mutable.move(
                        Direction.byName(step.get("direction").getAsString()));
                    case "move_direction_steps" -> mutable.move(
                        Direction.byName(step.get("direction").getAsString()),
                        Integer.parseInt(step.get("steps").getAsString()));
                    case "move_components" -> mutable.move(
                        Integer.parseInt(step.get("x").getAsString()),
                        Integer.parseInt(step.get("y").getAsString()),
                        Integer.parseInt(step.get("z").getAsString()));
                    case "move_block_pos" -> mutable.move(blockPos(step.getAsJsonObject("value")));
                    case "clamp" -> mutable.clamp(
                        Direction.Axis.byName(step.get("axis").getAsString()),
                        Integer.parseInt(step.get("minimum").getAsString()),
                        Integer.parseInt(step.get("maximum").getAsString()));
                    case "set_x" -> mutable.setX(Integer.parseInt(step.get("value").getAsString()));
                    case "set_y" -> mutable.setY(Integer.parseInt(step.get("value").getAsString()));
                    case "set_z" -> mutable.setZ(Integer.parseInt(step.get("value").getAsString()));
                    case "offset_return" -> event.add("returned", snapshot(mutable.offset(
                        Integer.parseInt(step.get("x").getAsString()),
                        Integer.parseInt(step.get("y").getAsString()),
                        Integer.parseInt(step.get("z").getAsString()))));
                    case "multiply_return" -> event.add("returned", snapshot(mutable.multiply(
                        Integer.parseInt(step.get("scale").getAsString()))));
                    case "relative_return" -> event.add("returned", snapshot(mutable.relative(
                        Direction.byName(step.get("direction").getAsString()),
                        Integer.parseInt(step.get("steps").getAsString()))));
                    case "relative_axis_return" -> event.add("returned", snapshot(mutable.relative(
                        Direction.Axis.byName(step.get("axis").getAsString()),
                        Integer.parseInt(step.get("steps").getAsString()))));
                    case "rotate_return" -> {
                        Rotation rotation = switch (step.get("rotation").getAsString()) {
                            case "none" -> Rotation.NONE;
                            case "clockwise_90" -> Rotation.CLOCKWISE_90;
                            case "180" -> Rotation.CLOCKWISE_180;
                            case "counterclockwise_90" -> Rotation.COUNTERCLOCKWISE_90;
                            default -> throw new IllegalArgumentException("Unknown mutable rotation");
                        };
                        event.add("returned", snapshot(mutable.rotate(rotation)));
                    }
                    default -> throw new IllegalArgumentException("Unknown mutable BlockPos operation");
                }
                event.add("state", snapshot(mutable));
                events.add(event);
            }
            result.add("constructors", constructors);
            result.add("events", events);
        } else if (operation.equals("axis_cycle_helpers")) {
            int x = Integer.parseInt(request.get("x").getAsString());
            int y = Integer.parseInt(request.get("y").getAsString());
            int z = Integer.parseInt(request.get("z").getAsString());
            double doubleX = f64(request, "double_x");
            double doubleY = f64(request, "double_y");
            double doubleZ = f64(request, "double_z");
            var cycles = new JsonObject();
            for (AxisCycle cycle : AxisCycle.values()) {
                var value = new JsonObject();
                value.addProperty("int_x", cycle.cycle(x, y, z, Direction.Axis.X));
                value.addProperty("int_y", cycle.cycle(x, y, z, Direction.Axis.Y));
                value.addProperty("int_z", cycle.cycle(x, y, z, Direction.Axis.Z));
                value.addProperty("double_x", Oracle.bits(cycle.cycle(doubleX, doubleY, doubleZ, Direction.Axis.X)));
                value.addProperty("double_y", Oracle.bits(cycle.cycle(doubleX, doubleY, doubleZ, Direction.Axis.Y)));
                value.addProperty("double_z", Oracle.bits(cycle.cycle(doubleX, doubleY, doubleZ, Direction.Axis.Z)));
                value.addProperty("axis_x", cycle.cycle(Direction.Axis.X).getName());
                value.addProperty("axis_y", cycle.cycle(Direction.Axis.Y).getName());
                value.addProperty("axis_z", cycle.cycle(Direction.Axis.Z).getName());
                value.addProperty("inverse", cycle.inverse().name().toLowerCase(Locale.ROOT));
                cycles.add(cycle.name().toLowerCase(Locale.ROOT), value);
            }
            var between = new JsonArray();
            for (Direction.Axis from : AxisCycle.AXIS_VALUES) {
                for (Direction.Axis to : AxisCycle.AXIS_VALUES) {
                    var value = new JsonObject();
                    value.addProperty("from", from.getName());
                    value.addProperty("to", to.getName());
                    value.addProperty("cycle", AxisCycle.between(from, to).name().toLowerCase(Locale.ROOT));
                    between.add(value);
                }
            }
            var mutableSet = new JsonObject();
            for (AxisCycle cycle : AxisCycle.values()) {
                BlockPos.MutableBlockPos value = new BlockPos.MutableBlockPos();
                value.set(cycle, x, y, z);
                mutableSet.add(cycle.name().toLowerCase(Locale.ROOT), snapshot(value));
            }
            result.add("cycles", cycles);
            result.add("between", between);
            result.add("mutable_set", mutableSet);
        } else if (operation.equals("section_relative")) {
            short relative = SectionPos.sectionRelativePos(blockPos(request));
            result.addProperty("relative", Integer.toString(Short.toUnsignedInt(relative)));
            result.addProperty("x", Integer.toString(SectionPos.sectionRelativeX(relative)));
            result.addProperty("y", Integer.toString(SectionPos.sectionRelativeY(relative)));
            result.addProperty("z", Integer.toString(SectionPos.sectionRelativeZ(relative)));
        } else if (operation.equals("vec3_codec")) {
            String mode = request.get("mode").getAsString();
            if (mode.equals("decode")) {
                var input = new JsonArray();
                for (var entry : request.getAsJsonArray("values")) {
                    input.add(Double.longBitsToDouble(Long.parseUnsignedLong(entry.getAsString(), 16)));
                }
                DataResult<Vec3> decoded = Vec3.CODEC.parse(JsonOps.INSTANCE, input);
                result = codecDecodeResult(decoded);
            } else if (mode.equals("encode")) {
                DataResult<JsonElement> encoded = Vec3.CODEC.encodeStart(JsonOps.INSTANCE, vec3(request));
                result = codecEncodeResult(encoded);
            } else {
                throw new IllegalArgumentException("Unknown Vec3 codec mode");
            }
        } else if (operation.equals("chunkpos_codec")) {
            String mode = request.get("mode").getAsString();
            if (mode.equals("decode")) {
                var input = new JsonArray();
                for (var entry : request.getAsJsonArray("values")) {
                    input.add(Integer.parseInt(entry.getAsString()));
                }
                DataResult<ChunkPos> decoded = ChunkPos.CODEC.parse(JsonOps.INSTANCE, input);
                result = chunkPosCodecDecodeResult(decoded);
            } else if (mode.equals("encode")) {
                DataResult<JsonElement> encoded = ChunkPos.CODEC.encodeStart(JsonOps.INSTANCE, chunkPos(request));
                result = chunkPosCodecEncodeResult(encoded);
            } else {
                throw new IllegalArgumentException("Unknown ChunkPos codec mode");
            }
        } else if (operation.equals("chunkpos_block_coordinates")) {
            ChunkPos pos = chunkPos(request);
            int offset = Integer.parseInt(request.get("offset").getAsString());
            result.add("value", chunkPosBlockCoordinates(pos, offset));
        } else if (operation.equals("chunkpos_region_coordinates")) {
            ChunkPos pos = chunkPos(request);
            int regionX = Integer.parseInt(request.get("region_x").getAsString());
            int regionZ = Integer.parseInt(request.get("region_z").getAsString());
            result.add("value", chunkPosRegionCoordinates(pos, regionX, regionZ));
        } else if (operation.equals("chunkpos_block_positions")) {
            ChunkPos pos = chunkPos(request);
            int blockX = Integer.parseInt(request.get("block_x").getAsString());
            int y = Integer.parseInt(request.get("y").getAsString());
            int blockZ = Integer.parseInt(request.get("block_z").getAsString());
            result.add("value", chunkPosBlockPositions(pos, blockX, y, blockZ, request.getAsJsonArray("contains")));
        } else if (operation.equals("chunkpos_representation")) {
            ChunkPos pos = chunkPos(request);
            int hashX = Integer.parseInt(request.get("hash_x").getAsString());
            int hashZ = Integer.parseInt(request.get("hash_z").getAsString());
            result.add("value", chunkPosRepresentation(pos, hashX, hashZ));
        } else if (operation.equals("chunkpos_from_blockpos")) {
            BlockPos pos = blockPos(request);
            result.add("value", chunkPosFromBlockPos(pos));
        } else if (operation.equals("chunkpos_distances")) {
            ChunkPos pos = chunkPos(request);
            int otherX = Integer.parseInt(request.get("other_x").getAsString());
            int otherZ = Integer.parseInt(request.get("other_z").getAsString());
            result.add("value", chunkPosDistances(pos, otherX, otherZ));
        } else if (operation.equals("chunkpos_from_section_node")) {
            long sectionNode = Long.parseUnsignedLong(request.get("section_node").getAsString(), 16);
            result.addProperty("value", Long.toUnsignedString(ChunkPos.fromSectionNode(sectionNode), 16));
        } else if (operation.equals("chunkpos_range_closed")) {
            result.add("value", chunkPosRangeClosed(request));
        } else if (operation.equals("chunkpos_constant")) {
            result.add("value", chunkPosConstant(request));
        } else if (operation.equals("chunkpos_packed_accessors")) {
            long packed = Long.parseUnsignedLong(request.get("packed").getAsString(), 16);
            var value = new JsonObject();
            value.addProperty("x", Integer.toString(ChunkPos.getX(packed)));
            value.addProperty("z", Integer.toString(ChunkPos.getZ(packed)));
            result.add("value", value);
        } else if (operation.equals("sectionpos_packed_accessors")) {
            long packed = Long.parseUnsignedLong(request.get("packed").getAsString(), 16);
            var value = new JsonObject();
            value.addProperty("x", Integer.toString(SectionPos.x(packed)));
            value.addProperty("y", Integer.toString(SectionPos.y(packed)));
            value.addProperty("z", Integer.toString(SectionPos.z(packed)));
            result.add("value", value);
        } else if (operation.equals("sectionpos_block_coordinates")) {
            SectionPos pos = SectionPos.of(
                Integer.parseInt(request.get("x").getAsString()),
                Integer.parseInt(request.get("y").getAsString()),
                Integer.parseInt(request.get("z").getAsString()));
            var value = new JsonObject();
            value.addProperty("min_block_x", Integer.toString(pos.minBlockX()));
            value.addProperty("min_block_y", Integer.toString(pos.minBlockY()));
            value.addProperty("min_block_z", Integer.toString(pos.minBlockZ()));
            value.addProperty("max_block_x", Integer.toString(pos.maxBlockX()));
            value.addProperty("max_block_y", Integer.toString(pos.maxBlockY()));
            value.addProperty("max_block_z", Integer.toString(pos.maxBlockZ()));
            result.add("value", value);
        } else if (operation.equals("sectionpos_relative_to_block")) {
            SectionPos pos = SectionPos.of(
                Integer.parseInt(request.get("x").getAsString()),
                Integer.parseInt(request.get("y").getAsString()),
                Integer.parseInt(request.get("z").getAsString()));
            short relative = Short.parseShort(request.get("relative").getAsString());
            var value = new JsonObject();
            value.addProperty("relative_x", Integer.toString(SectionPos.sectionRelativeX(relative)));
            value.addProperty("relative_y", Integer.toString(SectionPos.sectionRelativeY(relative)));
            value.addProperty("relative_z", Integer.toString(SectionPos.sectionRelativeZ(relative)));
            value.addProperty("block_x", Integer.toString(pos.relativeToBlockX(relative)));
            value.addProperty("block_y", Integer.toString(pos.relativeToBlockY(relative)));
            value.addProperty("block_z", Integer.toString(pos.relativeToBlockZ(relative)));
            value.add("block_position", snapshot(pos.relativeToBlockPos(relative)));
            result.add("value", value);
        } else if (operation.equals("sectionpos_spatial")) {
            SectionPos pos = SectionPos.of(
                Integer.parseInt(request.get("x").getAsString()),
                Integer.parseInt(request.get("y").getAsString()),
                Integer.parseInt(request.get("z").getAsString()));
            var value = new JsonObject();
            value.add("origin", snapshot(pos.origin()));
            value.add("center", snapshot(pos.center()));
            value.add("chunk", snapshot(pos.chunk()));
            result.add("value", value);
        } else if (operation.equals("sectionpos_packed_transformations")) {
            long blockNode = Long.parseUnsignedLong(request.get("block_node").getAsString(), 16);
            long sectionNode = Long.parseUnsignedLong(request.get("section_node").getAsString(), 16);
            int x = Integer.parseInt(request.get("x").getAsString());
            int z = Integer.parseInt(request.get("z").getAsString());
            var value = new JsonObject();
            value.addProperty("block_to_section", Long.toUnsignedString(SectionPos.blockToSection(blockNode), 16));
            value.addProperty("zero_node", Long.toUnsignedString(SectionPos.getZeroNode(sectionNode), 16));
            value.addProperty("zero_node_coordinates", Long.toUnsignedString(SectionPos.getZeroNode(x, z), 16));
            value.addProperty("section_to_chunk", Long.toUnsignedString(SectionPos.sectionToChunk(sectionNode), 16));
            result.add("value", value);
        } else if (operation.equals("sectionpos_as_long")) {
            long blockNode = Long.parseUnsignedLong(request.get("block_node").getAsString(), 16);
            int x = Integer.parseInt(request.get("x").getAsString());
            int y = Integer.parseInt(request.get("y").getAsString());
            int z = Integer.parseInt(request.get("z").getAsString());
            int sectionX = Integer.parseInt(request.get("section_x").getAsString());
            int sectionY = Integer.parseInt(request.get("section_y").getAsString());
            int sectionZ = Integer.parseInt(request.get("section_z").getAsString());
            var value = new JsonObject();
            value.addProperty("from_block_pos", Long.toUnsignedString(SectionPos.asLong(BlockPos.of(blockNode)), 16));
            value.addProperty("coordinates", Long.toUnsignedString(SectionPos.asLong(x, y, z), 16));
            value.addProperty("instance", Long.toUnsignedString(SectionPos.of(sectionX, sectionY, sectionZ).asLong(), 16));
            result.add("value", value);
        } else if (operation.equals("sectionpos_offset")) {
            long sectionNode = Long.parseUnsignedLong(request.get("section_node").getAsString(), 16);
            Direction direction = Direction.byName(request.get("direction").getAsString());
            if (direction == null) {
                throw new IllegalArgumentException("Unknown SectionPos offset direction");
            }
            int stepX = Integer.parseInt(request.get("step_x").getAsString());
            int stepY = Integer.parseInt(request.get("step_y").getAsString());
            int stepZ = Integer.parseInt(request.get("step_z").getAsString());
            int x = Integer.parseInt(request.get("x").getAsString());
            int y = Integer.parseInt(request.get("y").getAsString());
            int z = Integer.parseInt(request.get("z").getAsString());
            SectionPos instance = SectionPos.of(x, y, z).offset(stepX, stepY, stepZ);
            var value = new JsonObject();
            value.addProperty("packed_direction", Long.toUnsignedString(SectionPos.offset(sectionNode, direction), 16));
            value.addProperty("packed_steps", Long.toUnsignedString(SectionPos.offset(sectionNode, stepX, stepY, stepZ), 16));
            value.addProperty("instance_x", Integer.toString(instance.x()));
            value.addProperty("instance_y", Integer.toString(instance.y()));
            value.addProperty("instance_z", Integer.toString(instance.z()));
            value.addProperty("instance_packed", Long.toUnsignedString(instance.asLong(), 16));
            result.add("value", value);
        } else if (operation.equals("sectionpos_ranges")) {
            result.add("value", sectionPosRanges(request));
        } else if (operation.equals("sectionpos_blocks_inside")) {
            result.add("value", sectionPosBlocksInside(request));
        } else if (operation.equals("sectionpos_around_and_at_block_pos")) {
            result.add("value", sectionPosAroundAndAtBlockPos(request));
        } else if (operation.equals("sectionpos_double_coordinates")) {
            double value = f64(request, "value");
            String fn = request.get("fn").getAsString();
            int out = switch (fn) {
                case "pos_to_section" -> SectionPos.posToSectionCoord(value);
                case "block_to_section" -> SectionPos.blockToSectionCoord(value);
                default -> throw new IllegalArgumentException("Unknown SectionPos double coordinate function");
            };
            result.addProperty("value", Integer.toString(out));
        } else if (operation.equals("sectionpos_of_overloads")) {
            SectionPos value = switch (request.get("mode").getAsString()) {
                case "blockpos" -> SectionPos.of(blockPos(request));
                case "chunkpos" -> SectionPos.of(chunkPos(request),
                    Integer.parseInt(request.get("section_y").getAsString()));
                case "position" -> SectionPos.of(vec3(request));
                case "packed" -> SectionPos.of(Long.parseUnsignedLong(
                    request.get("section_node").getAsString(), 16));
                default -> throw new IllegalArgumentException("Unknown SectionPos of overload mode");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("chunkpos_validity")) {
            ChunkPos pos = chunkPos(request);
            result.add("value", chunkPosValidity(pos));
        } else if (operation.equals("mth_constants")) {
            switch (request.get("name").getAsString()) {
                case "pi" -> result.addProperty("bits", Oracle.bits(Mth.PI));
                case "half_pi" -> result.addProperty("bits", Oracle.bits(Mth.HALF_PI));
                case "two_pi" -> result.addProperty("bits", Oracle.bits(Mth.TWO_PI));
                case "deg_to_rad" -> result.addProperty("bits", Oracle.bits(Mth.DEG_TO_RAD));
                case "rad_to_deg" -> result.addProperty("bits", Oracle.bits(Mth.RAD_TO_DEG));
                case "epsilon" -> result.addProperty("bits", Oracle.bits(Mth.EPSILON));
                case "sqrt_of_two" -> result.addProperty("bits", Oracle.bits(Mth.SQRT_OF_TWO));
                case "y_axis" -> result.add("value", snapshot(new Vector3f(Mth.Y_AXIS)));
                case "x_axis" -> result.add("value", snapshot(new Vector3f(Mth.X_AXIS)));
                case "z_axis" -> result.add("value", snapshot(new Vector3f(Mth.Z_AXIS)));
                default -> throw new IllegalArgumentException("Unknown Mth constant");
            }
        } else if (operation.equals("mth_floating_primitives")) {
            String function = request.get("fn").getAsString();
            if (function.equals("sin")) {
                result.addProperty("bits", Oracle.bits(Mth.sin(f64(request, "value"))));
            } else if (function.equals("cos")) {
                result.addProperty("bits", Oracle.bits(Mth.cos(f64(request, "value"))));
            } else if (function.equals("sqrt")) {
                result.addProperty("bits", Oracle.bits(Mth.sqrt(Oracle.f32(request, "value"))));
            } else if (function.equals("abs")) {
                result.addProperty("bits", Oracle.bits(Mth.abs(Oracle.f32(request, "value"))));
            } else if (function.equals("clamp")) {
                String width = request.get("type").getAsString();
                if (width.equals("f32")) {
                    result.addProperty("bits", Oracle.bits(Mth.clamp(
                        Oracle.f32(request, "value"), Oracle.f32(request, "min"),
                        Oracle.f32(request, "max"))));
                } else if (width.equals("f64")) {
                    result.addProperty("bits", Oracle.bits(Mth.clamp(
                        f64(request, "value"), f64(request, "min"), f64(request, "max"))));
                } else {
                    throw new IllegalArgumentException("Unknown Mth clamp type");
                }
            } else {
                throw new IllegalArgumentException("Unknown Mth floating primitive");
            }
        } else if (operation.equals("mth_power_helpers")) {
            String width = request.get("type").getAsString();
            String function = request.get("fn").getAsString();
            if (width.equals("i32")) {
                int input = Integer.parseInt(request.get("input").getAsString());
                if (function.equals("is_power_of_two")) {
                    result.addProperty("value", Mth.isPowerOfTwo(input));
                } else {
                    int value = switch (function) {
                        case "smallest_encompassing_power_of_two" -> Mth.smallestEncompassingPowerOfTwo(input);
                        case "smallest_square_side" -> Mth.smallestSquareSide(input);
                        case "ceillog2" -> Mth.ceillog2(input);
                        case "log2" -> Mth.log2(input);
                        default -> throw new IllegalArgumentException("Unknown Mth i32 power helper");
                    };
                    result.addProperty("value", Integer.toString(value));
                }
            } else if (width.equals("i64")) {
                if (!function.equals("is_power_of_two")) {
                    throw new IllegalArgumentException("Unknown Mth i64 power helper");
                }
                result.addProperty("value", Mth.isPowerOfTwo(
                    Long.parseLong(request.get("input").getAsString())));
            } else {
                throw new IllegalArgumentException("Unknown Mth power helper type");
            }
        } else if (operation.equals("mth_frac")) {
            String width = request.get("type").getAsString();
            if (!request.get("fn").getAsString().equals("frac")) {
                throw new IllegalArgumentException("Unknown Mth frac function");
            }
            if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.frac(Oracle.f32(request, "input"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.frac(f64(request, "input"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth frac type");
            }
        } else if (operation.equals("mth_create_insecure_uuid")) {
            RandomSource random = RandomSource.create(
                Long.parseLong(request.get("seed").getAsString()));
            UUID uuid = Mth.createInsecureUUID(random);
            result.addProperty("most_hex", String.format(Locale.ROOT, "%016x",
                uuid.getMostSignificantBits()));
            result.addProperty("least_hex", String.format(Locale.ROOT, "%016x",
                uuid.getLeastSignificantBits()));
        } else if (operation.equals("mth_murmur_hash3_mixer")) {
            int hash = Integer.parseInt(request.get("input").getAsString());
            result.addProperty("value", Integer.toString(Mth.murmurHash3Mixer(hash)));
        } else if (operation.equals("mth_binary_search")) {
            int from = Integer.parseInt(request.get("from").getAsString());
            int to = Integer.parseInt(request.get("to").getAsString());
            int threshold = Integer.parseInt(request.get("threshold").getAsString());
            IntPredicate condition = switch (request.get("condition").getAsString()) {
                case "ge" -> value -> value >= threshold;
                case "gt" -> value -> value > threshold;
                default -> throw new IllegalArgumentException("Unknown Mth binary search condition");
            };
            result.addProperty("value", Integer.toString(Mth.binarySearch(from, to, condition)));
        } else if (operation.equals("mth_ray_intersects_aabb")) {
            Vec3 rayStart = vec3(request.getAsJsonObject("ray_start"));
            Vec3 rayDir = vec3(request.getAsJsonObject("ray_dir"));
            AABB aabb = box(request.getAsJsonObject("aabb"));
            result.addProperty("value", Mth.rayIntersectsAABB(rayStart, rayDir, aabb));
        } else if (operation.equals("mth_fast_inv_sqrt")) {
            result.addProperty("bits", Oracle.bits(Mth.fastInvSqrt(f64(request, "input"))));
        } else if (operation.equals("mth_fast_inv_cube_root")) {
            result.addProperty("bits", Oracle.bits(Mth.fastInvCubeRoot(
                Oracle.f32(request, "input"))));
        } else if (operation.equals("mth_hsv_to_color")) {
            try {
                float hue = Oracle.f32(request, "hue");
                float saturation = Oracle.f32(request, "saturation");
                float value = Oracle.f32(request, "value");
                int alpha = Integer.parseInt(request.get("alpha").getAsString());
                int color = switch (request.get("mode").getAsString()) {
                    case "rgb" -> Mth.hsvToRgb(hue, saturation, value);
                    case "argb" -> Mth.hsvToArgb(hue, saturation, value, alpha);
                    default -> throw new IllegalArgumentException("Unknown Mth HSV color mode");
                };
                result.addProperty("value", Integer.toString(color));
            } catch (RuntimeException error) {
                if (error.getClass() != RuntimeException.class) {
                    throw error;
                }
                result.addProperty("outcome", "error");
                result.addProperty("error", "hue_out_of_range");
                result.addProperty("java_exception", error.getClass().getName());
            }
        } else if (operation.equals("mth_angle_wrap_difference")) {
            if (!request.get("type").getAsString().equals("f32")) {
                throw new IllegalArgumentException("Unknown Mth angle wrap/difference type");
            }
            String function = request.get("fn").getAsString();
            float value = switch (function) {
                case "wrap_degrees_90" -> Mth.wrapDegrees90(Oracle.f32(request, "value"));
                case "degrees_difference" -> Mth.degreesDifference(
                    Oracle.f32(request, "from"), Oracle.f32(request, "to"));
                case "degrees_difference_abs" -> Mth.degreesDifferenceAbs(
                    Oracle.f32(request, "from"), Oracle.f32(request, "to"));
                default -> throw new IllegalArgumentException("Unknown Mth angle wrap/difference function");
            };
            result.addProperty("bits", Oracle.bits(value));
        } else if (operation.equals("mth_angle_approach")) {
            if (!request.get("type").getAsString().equals("f32")) {
                throw new IllegalArgumentException("Unknown Mth angle approach type");
            }
            String function = request.get("fn").getAsString();
            float value = switch (function) {
                case "rotate_if_necessary" -> Mth.rotateIfNecessary(
                    Oracle.f32(request, "base"), Oracle.f32(request, "target"),
                    Oracle.f32(request, "max"));
                case "approach" -> Mth.approach(
                    Oracle.f32(request, "current"), Oracle.f32(request, "target"),
                    Oracle.f32(request, "increment"));
                case "approach_degrees" -> Mth.approachDegrees(
                    Oracle.f32(request, "current"), Oracle.f32(request, "target"),
                    Oracle.f32(request, "increment"));
                default -> throw new IllegalArgumentException("Unknown Mth angle approach function");
            };
            result.addProperty("bits", Oracle.bits(value));
        } else if (operation.equals("mth_get_int")) {
            JsonElement inputElement = request.get("input");
            String input = inputElement.isJsonNull() ? null : inputElement.getAsString();
            int defaultValue = Integer.parseInt(request.get("default").getAsString());
            result.addProperty("value", Integer.toString(Mth.getInt(input, defaultValue)));
        } else if (operation.equals("mth_get_seed")) {
            int x = request.get("x").getAsInt();
            int y = request.get("y").getAsInt();
            int z = request.get("z").getAsInt();
            result.addProperty("value", Long.toString(Mth.getSeed(x, y, z)));
        } else if (operation.equals("mth_atan2")) {
            result.addProperty("bits", Oracle.bits(Mth.atan2(
                f64(request, "y"), f64(request, "x"))));
        } else if (operation.equals("mth_inv_sqrt")) {
            if (request.get("type").getAsString().equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.invSqrt(
                    Oracle.f32(request, "input"))));
            } else if (request.get("type").getAsString().equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.invSqrt(
                    f64(request, "input"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth invSqrt type");
            }
        } else if (operation.equals("mth_integer_helpers")) {
            String width = request.get("type").getAsString();
            String function = request.get("fn").getAsString();
            if (width.equals("i32")) {
                int input = Integer.parseInt(request.get("input").getAsString());
                int divisor = Integer.parseInt(request.get("divisor").getAsString());
                int value = switch (function) {
                    case "floor_div" -> Mth.floorDiv(input, divisor);
                    case "positive_modulo" -> Mth.positiveModulo(input, divisor);
                    case "positive_ceil_div" -> Mth.positiveCeilDiv(input, divisor);
                    case "round_toward" -> Mth.roundToward(input, divisor);
                    default -> throw new IllegalArgumentException("Unknown Mth integer function");
                };
                result.addProperty("value", Integer.toString(value));
            } else if (width.equals("i64")) {
                long input = Long.parseLong(request.get("input").getAsString());
                long divisor = Long.parseLong(request.get("divisor").getAsString());
                long value = switch (function) {
                    case "positive_ceil_div" -> Mth.positiveCeilDiv(input, divisor);
                    case "round_toward" -> Mth.roundToward(input, divisor);
                    default -> throw new IllegalArgumentException("Unknown Mth long function");
                };
                result.addProperty("value", Long.toString(value));
            } else {
                throw new IllegalArgumentException("Unknown Mth integer type");
            }
        } else if (operation.equals("mth_integer_basics")) {
            String width = request.get("type").getAsString();
            String function = request.get("fn").getAsString();
            if (width.equals("i32")) {
                int input = Integer.parseInt(request.get("input").getAsString());
                if (function.equals("is_multiple")) {
                    result.addProperty("value", Mth.isMultipleOf(input,
                        Integer.parseInt(request.get("divisor").getAsString())));
                } else {
                    int value = switch (function) {
                        case "abs" -> Mth.abs(input);
                        case "clamp" -> Mth.clamp(input,
                            Integer.parseInt(request.get("min").getAsString()),
                            Integer.parseInt(request.get("max").getAsString()));
                        case "wrap_degrees" -> Mth.wrapDegrees(input);
                        default -> throw new IllegalArgumentException("Unknown Mth integer basic function");
                    };
                    result.addProperty("value", Integer.toString(value));
                }
            } else if (width.equals("i64")) {
                long input = Long.parseLong(request.get("input").getAsString());
                if (!function.equals("clamp")) {
                    throw new IllegalArgumentException("Unknown Mth long basic function");
                }
                long value = Mth.clamp(input,
                    Long.parseLong(request.get("min").getAsString()),
                    Long.parseLong(request.get("max").getAsString()));
                result.addProperty("value", Long.toString(value));
            } else {
                throw new IllegalArgumentException("Unknown Mth integer basic type");
            }
        } else if (operation.equals("mth_angle_helpers")) {
            String width = request.get("type").getAsString();
            String function = request.get("fn").getAsString();
            if (function.equals("pack_degrees")) {
                if (!width.equals("f32")) {
                    throw new IllegalArgumentException("Unknown Mth packDegrees type");
                }
                byte value = Mth.packDegrees(Oracle.f32(request, "input"));
                result.addProperty("value", Byte.toString(value));
            } else if (function.equals("unpack_degrees")) {
                if (!width.equals("i8")) {
                    throw new IllegalArgumentException("Unknown Mth unpackDegrees type");
                }
                byte input = Byte.parseByte(request.get("input").getAsString());
                result.addProperty("bits", Oracle.bits(Mth.unpackDegrees(input)));
            } else if (function.equals("wrap_degrees")) {
                if (!width.equals("i64")) {
                    throw new IllegalArgumentException("Unknown Mth long wrapDegrees type");
                }
                long input = Long.parseLong(request.get("input").getAsString());
                result.addProperty("bits", Oracle.bits(Mth.wrapDegrees(input)));
            } else {
                throw new IllegalArgumentException("Unknown Mth angle function");
            }
        } else if (operation.equals("mth_rounding_helpers")) {
            String width = request.get("type").getAsString();
            String function = request.get("fn").getAsString();
            if (function.equals("ceil")) {
                int value;
                if (width.equals("f32")) {
                    value = Mth.ceil(Oracle.f32(request, "input"));
                } else if (width.equals("f64")) {
                    value = Mth.ceil(f64(request, "input"));
                } else {
                    throw new IllegalArgumentException("Unknown Mth ceil type");
                }
                result.addProperty("value", Integer.toString(value));
            } else if (function.equals("lfloor")) {
                if (!width.equals("f64")) {
                    throw new IllegalArgumentException("Unknown Mth lfloor type");
                }
                result.addProperty("value", Long.toString(Mth.lfloor(f64(request, "input"))));
            } else if (function.equals("ceil_long")) {
                if (!width.equals("f64")) {
                    throw new IllegalArgumentException("Unknown Mth ceilLong type");
                }
                result.addProperty("value", Long.toString(Mth.ceilLong(f64(request, "input"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth rounding function");
            }
        } else if (operation.equals("mth_positive_modulo")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.positiveModulo(
                    Oracle.f32(request, "input"), Oracle.f32(request, "mod"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.positiveModulo(
                    f64(request, "input"), f64(request, "mod"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth positiveModulo type");
            }
        } else if (operation.equals("mth_abs_max")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.absMax(
                    Oracle.f32(request, "first"), Oracle.f32(request, "second"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.absMax(
                    f64(request, "first"), f64(request, "second"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth absMax type");
            }
        } else if (operation.equals("mth_equal")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("equal", Mth.equal(
                    Oracle.f32(request, "first"), Oracle.f32(request, "second")));
            } else if (width.equals("f64")) {
                result.addProperty("equal", Mth.equal(
                    f64(request, "first"), f64(request, "second")));
            } else {
                throw new IllegalArgumentException("Unknown Mth equal type");
            }
        } else if (operation.equals("mth_clamped_lerp")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.clampedLerp(
                    Oracle.f32(request, "factor"), Oracle.f32(request, "min"),
                    Oracle.f32(request, "max"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.clampedLerp(
                    f64(request, "factor"), f64(request, "min"), f64(request, "max"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth clampedLerp type");
            }
        } else if (operation.equals("mth_next_random")) {
            RandomSource random = RandomSource.create(
                Long.parseLong(request.get("seed").getAsString()));
            String width = request.get("type").getAsString();
            if (width.equals("i32")) {
                result.addProperty("value", Integer.toString(Mth.nextInt(random,
                    Integer.parseInt(request.get("min").getAsString()),
                    Integer.parseInt(request.get("max").getAsString()))));
            } else if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.nextFloat(random,
                    Oracle.f32(request, "min"), Oracle.f32(request, "max"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.nextDouble(random,
                    f64(request, "min"), f64(request, "max"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth next random type");
            }
        } else if (operation.equals("mth_random_helpers")) {
            RandomSource random = RandomSource.create(
                Long.parseLong(request.get("seed").getAsString()));
            String function = request.get("fn").getAsString();
            if (function.equals("inclusive")) {
                int value = Mth.randomBetweenInclusive(random,
                    Integer.parseInt(request.get("min").getAsString()),
                    Integer.parseInt(request.get("max").getAsString()));
                result.addProperty("value", Integer.toString(value));
                result.addProperty("follow_up", Integer.toString(random.nextInt()));
            } else if (function.equals("between")) {
                float value = Mth.randomBetween(random,
                    Oracle.f32(request, "min"), Oracle.f32(request, "max"));
                result.addProperty("bits", Oracle.bits(value));
                result.addProperty("follow_up_bits", Oracle.bits(random.nextFloat()));
            } else if (function.equals("normal")) {
                float value = Mth.normal(random,
                    Oracle.f32(request, "mean"), Oracle.f32(request, "deviation"));
                result.addProperty("bits", Oracle.bits(value));
                result.addProperty("follow_up_bits", Oracle.bits(random.nextGaussian()));
            } else {
                throw new IllegalArgumentException("Unknown Mth random helper");
            }
        } else if (operation.equals("mth_length")) {
            String type = request.get("type").getAsString();
            String function = request.get("fn").getAsString();
            if (type.equals("f64_2")) {
                double x = f64(request, "x");
                double y = f64(request, "y");
                double value = switch (function) {
                    case "length_squared" -> Mth.lengthSquared(x, y);
                    case "length" -> Mth.length(x, y);
                    default -> throw new IllegalArgumentException("Unknown Mth length function");
                };
                result.addProperty("bits", Oracle.bits(value));
            } else if (type.equals("f32_2")) {
                if (!function.equals("length")) {
                    throw new IllegalArgumentException("Unknown Mth length function");
                }
                result.addProperty("bits", Oracle.bits(Mth.length(
                    Oracle.f32(request, "x"), Oracle.f32(request, "y"))));
            } else if (type.equals("f64_3")) {
                double x = f64(request, "x");
                double y = f64(request, "y");
                double z = f64(request, "z");
                double value = switch (function) {
                    case "length_squared" -> Mth.lengthSquared(x, y, z);
                    case "length" -> Mth.length(x, y, z);
                    default -> throw new IllegalArgumentException("Unknown Mth length function");
                };
                result.addProperty("bits", Oracle.bits(value));
            } else if (type.equals("f32_3")) {
                if (!function.equals("length_squared")) {
                    throw new IllegalArgumentException("Unknown Mth length function");
                }
                result.addProperty("bits", Oracle.bits(Mth.lengthSquared(
                    Oracle.f32(request, "x"), Oracle.f32(request, "y"),
                    Oracle.f32(request, "z"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth length type");
            }
        } else if (operation.equals("mth_quantize")) {
            result.addProperty("value", Integer.toString(Mth.quantize(
                f64(request, "value"), Integer.parseInt(request.get("resolution").getAsString()))));
        } else if (operation.equals("mth_out_from_origin")) {
            int origin = Integer.parseInt(request.get("origin").getAsString());
            int lowerBound = Integer.parseInt(request.get("lower").getAsString());
            int upperBound = Integer.parseInt(request.get("upper").getAsString());
            try {
                var positions = switch (request.get("mode").getAsString()) {
                    case "default" -> Mth.outFromOrigin(origin, lowerBound, upperBound);
                    case "step" -> Mth.outFromOrigin(origin, lowerBound, upperBound,
                        Integer.parseInt(request.get("step").getAsString()));
                    default -> throw new IllegalArgumentException("Unknown Mth outFromOrigin mode");
                };
                var values = new JsonArray();
                positions.forEach(value -> values.add(value));
                result.addProperty("outcome", "value");
                result.add("value", values);
            } catch (IllegalArgumentException error) {
                result.addProperty("outcome", "error");
                result.addProperty("error", error.getMessage());
                result.addProperty("java_exception", error.getClass().getName());
            }
        } else if (operation.equals("mth_mul_and_truncate")) {
            Fraction fraction = Fraction.getFraction(
                Integer.parseInt(request.get("numerator").getAsString()),
                Integer.parseInt(request.get("denominator").getAsString()));
            result.addProperty("value", Integer.toString(Mth.mulAndTruncate(
                fraction, Integer.parseInt(request.get("factor").getAsString()))));
        } else if (operation.equals("mth_rotation_around_axis")) {
            Vector3f axis = new Vector3f(
                Oracle.f32(request, "axis_x"),
                Oracle.f32(request, "axis_y"),
                Oracle.f32(request, "axis_z"));
            Quaternionf rotation = new Quaternionf(
                Oracle.f32(request, "rotation_x"),
                Oracle.f32(request, "rotation_y"),
                Oracle.f32(request, "rotation_z"),
                Oracle.f32(request, "rotation_w"));
            result.add("value", snapshot(Mth.rotationAroundAxis(
                axis, rotation, new Quaternionf())));
        } else if (operation.equals("mth_inverse_lerp")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.inverseLerp(
                    Oracle.f32(request, "value"), Oracle.f32(request, "min"),
                    Oracle.f32(request, "max"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.inverseLerp(
                    f64(request, "value"), f64(request, "min"), f64(request, "max"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth inverseLerp type");
            }
        } else if (operation.equals("mth_lerp")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.lerp(
                    Oracle.f32(request, "alpha"), Oracle.f32(request, "p0"),
                    Oracle.f32(request, "p1"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.lerp(
                    f64(request, "alpha"), f64(request, "p0"), f64(request, "p1"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth lerp type");
            }
        } else if (operation.equals("mth_lerp_int")) {
            result.addProperty("value", Integer.toString(Mth.lerpInt(
                Oracle.f32(request, "alpha"),
                Integer.parseInt(request.get("p0").getAsString()),
                Integer.parseInt(request.get("p1").getAsString()))));
        } else if (operation.equals("mth_lerp_discrete")) {
            result.addProperty("value", Integer.toString(Mth.lerpDiscrete(
                Oracle.f32(request, "alpha"),
                Integer.parseInt(request.get("p0").getAsString()),
                Integer.parseInt(request.get("p1").getAsString()))));
        } else if (operation.equals("mth_smoothstep")) {
            result.addProperty("bits", Oracle.bits(Mth.smoothstep(
                f64(request, "x"))));
        } else if (operation.equals("mth_smoothstep_derivative")) {
            result.addProperty("bits", Oracle.bits(Mth.smoothstepDerivative(
                f64(request, "x"))));
        } else if (operation.equals("mth_sign")) {
            result.addProperty("value", Integer.toString(Mth.sign(f64(request, "x"))));
        } else if (operation.equals("mth_rot_lerp")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.rotLerp(
                    Oracle.f32(request, "alpha"), Oracle.f32(request, "from"),
                    Oracle.f32(request, "to"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.rotLerp(
                    f64(request, "alpha"), f64(request, "from"), f64(request, "to"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth rotLerp type");
            }
        } else if (operation.equals("mth_rot_lerp_rad")) {
            result.addProperty("bits", Oracle.bits(Mth.rotLerpRad(
                Oracle.f32(request, "alpha"), Oracle.f32(request, "from"),
                Oracle.f32(request, "to"))));
        } else if (operation.equals("mth_triangle_wave")) {
            result.addProperty("bits", Oracle.bits(Mth.triangleWave(
                Oracle.f32(request, "index"), Oracle.f32(request, "period"))));
        } else if (operation.equals("mth_square_f32")) {
            result.addProperty("bits", Oracle.bits(Mth.square(Oracle.f32(request, "x"))));
        } else if (operation.equals("mth_cube_f32")) {
            result.addProperty("bits", Oracle.bits(Mth.cube(Oracle.f32(request, "x"))));
        } else if (operation.equals("mth_square_f64")) {
            result.addProperty("bits", Oracle.bits(Mth.square(f64(request, "x"))));
        } else if (operation.equals("mth_square_i32")) {
            result.addProperty("value", Integer.toString(Mth.square(
                Integer.parseInt(request.get("x").getAsString()))));
        } else if (operation.equals("mth_square_i64")) {
            result.addProperty("value", Long.toString(Mth.square(
                Long.parseLong(request.get("x").getAsString()))));
        } else if (operation.equals("mth_clamped_map_f64")) {
            result.addProperty("bits", Oracle.bits(Mth.clampedMap(
                f64(request, "value"), f64(request, "from_min"), f64(request, "from_max"),
                f64(request, "to_min"), f64(request, "to_max"))));
        } else if (operation.equals("mth_clamped_map_f32")) {
            result.addProperty("bits", Oracle.bits(Mth.clampedMap(
                Oracle.f32(request, "value"), Oracle.f32(request, "from_min"),
                Oracle.f32(request, "from_max"), Oracle.f32(request, "to_min"),
                Oracle.f32(request, "to_max"))));
        } else if (operation.equals("mth_map_f64")) {
            result.addProperty("bits", Oracle.bits(Mth.map(
                f64(request, "value"), f64(request, "from_min"), f64(request, "from_max"),
                f64(request, "to_min"), f64(request, "to_max"))));
        } else if (operation.equals("mth_map_f32")) {
            result.addProperty("bits", Oracle.bits(Mth.map(
                Oracle.f32(request, "value"), Oracle.f32(request, "from_min"),
                Oracle.f32(request, "from_max"), Oracle.f32(request, "to_min"),
                Oracle.f32(request, "to_max"))));
        } else if (operation.equals("mth_wobble_f64")) {
            result.addProperty("bits", Oracle.bits(Mth.wobble(
                f64(request, "coord"))));
        } else if (operation.equals("mth_interpolation")) {
            String function = request.get("fn").getAsString();
            switch (function) {
                case "lerp2" -> result.addProperty("bits", Oracle.bits(Mth.lerp2(
                    f64(request, "alpha1"), f64(request, "alpha2"),
                    f64(request, "x00"), f64(request, "x10"),
                    f64(request, "x01"), f64(request, "x11"))));
                case "lerp3" -> result.addProperty("bits", Oracle.bits(Mth.lerp3(
                    f64(request, "alpha1"), f64(request, "alpha2"),
                    f64(request, "alpha3"), f64(request, "x000"),
                    f64(request, "x100"), f64(request, "x010"),
                    f64(request, "x110"), f64(request, "x001"),
                    f64(request, "x101"), f64(request, "x011"),
                    f64(request, "x111"))));
                case "catmullrom" -> result.addProperty("bits", Oracle.bits(Mth.catmullrom(
                    Oracle.f32(request, "alpha"), Oracle.f32(request, "p0"),
                    Oracle.f32(request, "p1"), Oracle.f32(request, "p2"),
                    Oracle.f32(request, "p3"))));
                default -> throw new IllegalArgumentException("Unknown Mth interpolation function");
            }
        } else if (operation.equals("vec3")) {
            Vec3 vector = new Vec3(f64(request, "x"), f64(request, "y"), f64(request, "z"));
            var events = new JsonArray();
            events.add(snapshot(vector));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                vector = switch (step.get("action").getAsString()) {
                    case "normalize" -> vector.normalize();
                    case "scale" -> vector.scale(f64(step, "factor"));
                    case "reverse" -> vector.reverse();
                    case "horizontal" -> vector.horizontal();
                    case "multiply_vec3" -> vector.multiply(vec3(step.getAsJsonObject("scale")));
                    case "multiply_components" -> vector.multiply(
                        f64(step, "x"), f64(step, "y"), f64(step, "z"));
                    case "add_vec3" -> vector.add(vec3(step.getAsJsonObject("delta")));
                    case "add_scalar" -> vector.add(f64(step, "value"));
                    case "add_components" -> vector.add(
                        f64(step, "x"), f64(step, "y"), f64(step, "z"));
                    case "subtract_vec3" -> vector.subtract(vec3(step.getAsJsonObject("delta")));
                    case "subtract_scalar" -> vector.subtract(f64(step, "value"));
                    case "subtract_components" -> vector.subtract(
                        f64(step, "x"), f64(step, "y"), f64(step, "z"));
                    case "vector_to" -> vector.vectorTo(vec3(step.getAsJsonObject("target")));
                    case "cross" -> vector.cross(vec3(step.getAsJsonObject("other")));
                    default -> throw new IllegalArgumentException("Unknown Vec3 action");
                };
                events.add(snapshot(vector));
            }
            result.add("events", events);
        } else if (operation.equals("vec3_dot")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.addProperty("value", Oracle.bits(first.dot(second)));
        } else if (operation.equals("vec3_distance_to_sqr")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.addProperty("value", Oracle.bits(first.distanceToSqr(second)));
        } else if (operation.equals("vec3_distance_to")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.addProperty("value", Oracle.bits(first.distanceTo(second)));
        } else if (operation.equals("vec3_distance_to_sqr_components")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.addProperty("value", Oracle.bits(first.distanceToSqr(
                f64(request, "x"), f64(request, "y"), f64(request, "z"))));
        } else if (operation.equals("vec3_horizontal_distances")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.addProperty("distance", Oracle.bits(first.horizontalDistance()));
            result.addProperty("distance_squared", Oracle.bits(first.horizontalDistanceSqr()));
        } else if (operation.equals("vec3_closer_than")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.addProperty("position", first.closerThan(second, f64(request, "distance")));
            result.addProperty("xz_y", first.closerThan(
                second, f64(request, "distance_xz"), f64(request, "distance_y")));
        } else if (operation.equals("vec3_offset_random")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            float offset = Oracle.f32(request, "offset");
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            result.add("offset_random", snapshot(first.offsetRandom(random, offset)));
            RandomSource randomXZ = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            result.add("offset_random_xz", snapshot(first.offsetRandomXZ(randomXZ, offset)));
        } else if (operation.equals("vec3_lerp")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.add("value", snapshot(first.lerp(second, f64(request, "alpha"))));
        } else if (operation.equals("vec3_rotate_clockwise_90")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.add("value", snapshot(first.rotateClockwise90()));
        } else if (operation.equals("vec3_is_finite")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.addProperty("value", first.isFinite());
        } else if (operation.equals("vec3_get")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            result.addProperty("value", Oracle.bits(first.get(axis)));
        } else if (operation.equals("vec3_with")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            result.add("value", snapshot(first.with(axis, f64(request, "value"))));
        } else if (operation.equals("vec3_relative")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Direction direction = Direction.byName(request.get("direction").getAsString());
            if (direction == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            result.add("value", snapshot(first.relative(direction, f64(request, "distance"))));
        } else if (operation.equals("vec3_align")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            EnumSet<Direction.Axis> axes = EnumSet.noneOf(Direction.Axis.class);
            for (var entry : request.getAsJsonArray("axes")) {
                Direction.Axis axis = Direction.Axis.byName(entry.getAsString());
                if (axis == null) {
                    throw new IllegalArgumentException("Unknown axis name");
                }
                axes.add(axis);
            }
            result.add("value", snapshot(first.align(axes)));
        } else if (operation.equals("vec3_projected_on")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 onto = vec3(request.getAsJsonObject("onto"));
            result.add("value", snapshot(first.projectedOn(onto)));
        } else if (operation.equals("vec3_to_vector3f")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.add("value", snapshot(first.toVector3f()));
        } else if (operation.equals("vec3_coordinates")) {
            BlockPos pos = blockPos(request);
            String kind = request.get("kind").getAsString();
            Vec3 value = switch (kind) {
                case "lower_corner" -> Vec3.atLowerCornerOf(pos);
                case "lower_corner_offset" -> Vec3.atLowerCornerWithOffset(
                    pos, f64(request, "offset_x"), f64(request, "offset_y"), f64(request, "offset_z"));
                case "center" -> Vec3.atCenterOf(pos);
                case "bottom_center" -> Vec3.atBottomCenterOf(pos);
                case "up_from_bottom_center" -> Vec3.upFromBottomCenterOf(pos, f64(request, "y_offset"));
                default -> throw new IllegalArgumentException("Unknown Vec3 coordinate helper");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("vec3_equality")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.addProperty("equals", first.equals(second));
            result.addProperty("first_hash", Integer.toString(first.hashCode()));
            result.addProperty("second_hash", Integer.toString(second.hashCode()));
        } else if (operation.equals("vec3_to_string")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.addProperty("value", first.toString());
        } else if (operation.equals("vec3_rotate")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            float radians = Oracle.f32(request, "radians");
            Vec3 value = switch (request.get("axis").getAsString()) {
                case "x" -> first.xRot(radians);
                case "y" -> first.yRot(radians);
                case "z" -> first.zRot(radians);
                default -> throw new IllegalArgumentException("Unknown Vec3 rotation axis");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("vec3_direction_from_rotation")) {
            Vec3 value = switch (request.get("kind").getAsString()) {
                case "components" -> Vec3.directionFromRotation(
                    Oracle.f32(request, "rot_x"), Oracle.f32(request, "rot_y"));
                case "vec2" -> {
                    JsonObject rotation = request.getAsJsonObject("rotation");
                    yield Vec3.directionFromRotation(new Vec2(
                        Oracle.f32(rotation, "x"), Oracle.f32(rotation, "y")));
                }
                default -> throw new IllegalArgumentException("Unknown Vec3 direction rotation kind");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("vec3_local_coordinates")) {
            Vec3 value = switch (request.get("kind").getAsString()) {
                case "apply" -> Vec3.applyLocalCoordinatesToRotation(
                    new Vec2(
                        Oracle.f32(request.getAsJsonObject("rotation"), "x"),
                        Oracle.f32(request.getAsJsonObject("rotation"), "y")),
                    vec3(request.getAsJsonObject("direction")));
                case "add" -> vec3(request.getAsJsonObject("first"))
                    .addLocalCoordinates(vec3(request.getAsJsonObject("direction")));
                default -> throw new IllegalArgumentException("Unknown Vec3 local coordinate kind");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("vec3_rotation")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.add("value", snapshot(first.rotation()));
        } else if (operation.equals("vec2_rotate")) {
            JsonObject first = request.getAsJsonObject("first");
            Vec2 vector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            result.add("value", snapshot(vector.rotate(f64(request, "angle"))));
        } else if (operation.equals("vec2_dot")) {
            JsonObject first = request.getAsJsonObject("first");
            JsonObject second = request.getAsJsonObject("second");
            Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            Vec2 secondVector = new Vec2(Oracle.f32(second, "x"), Oracle.f32(second, "y"));
            result.addProperty("value", Oracle.bits(firstVector.dot(secondVector)));
        } else if (operation.equals("vec2_add_vec2")) {
            JsonObject first = request.getAsJsonObject("first");
            JsonObject second = request.getAsJsonObject("second");
            Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            Vec2 secondVector = new Vec2(Oracle.f32(second, "x"), Oracle.f32(second, "y"));
            result.add("value", snapshot(firstVector.add(secondVector)));
        } else if (operation.equals("vec2_add_scalar")) {
            JsonObject first = request.getAsJsonObject("first");
            Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            result.add("value", snapshot(firstVector.add(Oracle.f32(request, "value"))));
        } else if (operation.equals("vec2_distance_to_sqr")) {
            JsonObject first = request.getAsJsonObject("first");
            JsonObject second = request.getAsJsonObject("second");
            Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            Vec2 secondVector = new Vec2(Oracle.f32(second, "x"), Oracle.f32(second, "y"));
            result.addProperty("value", Oracle.bits(firstVector.distanceToSqr(secondVector)));
        } else if (operation.equals("vec2_negated")) {
            JsonObject first = request.getAsJsonObject("first");
            Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            result.add("value", snapshot(firstVector.negated()));
        } else if (operation.equals("vec2_equality")) {
            JsonObject firstObject = request.getAsJsonObject("first");
            JsonObject secondObject = request.getAsJsonObject("second");
            Vec2 first = new Vec2(Oracle.f32(firstObject, "x"), Oracle.f32(firstObject, "y"));
            Vec2 second = new Vec2(Oracle.f32(secondObject, "x"), Oracle.f32(secondObject, "y"));
            result.addProperty("equals", first.equals(second));
            result.addProperty("first_hash", Integer.toString(first.hashCode()));
            result.addProperty("second_hash", Integer.toString(second.hashCode()));
        } else if (operation.equals("vec2_constant")) {
            Vec2 value = switch (request.get("name").getAsString()) {
                case "zero" -> Vec2.ZERO;
                case "one" -> Vec2.ONE;
                case "unit_x" -> Vec2.UNIT_X;
                case "neg_unit_x" -> Vec2.NEG_UNIT_X;
                case "unit_y" -> Vec2.UNIT_Y;
                case "neg_unit_y" -> Vec2.NEG_UNIT_Y;
                case "max" -> Vec2.MAX;
                case "min" -> Vec2.MIN;
                default -> throw new IllegalArgumentException("Unknown Vec2 constant");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("vec3_constant")) {
            Vec3 value = switch (request.get("name").getAsString()) {
                case "zero" -> Vec3.ZERO;
                case "x_axis" -> Vec3.X_AXIS;
                case "y_axis" -> Vec3.Y_AXIS;
                case "z_axis" -> Vec3.Z_AXIS;
                default -> throw new IllegalArgumentException("Unknown Vec3 constant");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("shape")) {
            String kind = request.get("kind").getAsString();
            if (kind.equals("single")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var events = new JsonArray();
                events.add(snapshot(shape));
                for (var entry : request.getAsJsonArray("steps")) {
                    JsonObject step = entry.getAsJsonObject();
                    shape = switch (step.get("op").getAsString()) {
                        case "move" -> shape.move(f64(step, "dx"), f64(step, "dy"), f64(step, "dz"));
                        case "move_vec3" -> shape.move(vec3(step.getAsJsonObject("delta")));
                        case "move_block_pos" -> shape.move(blockPos(step.getAsJsonObject("delta")));
                        default -> throw new IllegalArgumentException("Unknown VoxelShape op");
                    };
                    events.add(snapshot(shape));
                }
                result.add("events", events);
            } else if (kind.equals("join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                BooleanOp op = switch (request.get("op").getAsString()) {
                    case "false" -> BooleanOp.FALSE;
                    case "or" -> BooleanOp.OR;
                    case "and" -> BooleanOp.AND;
                    case "only_first" -> BooleanOp.ONLY_FIRST;
                    case "only_second" -> BooleanOp.ONLY_SECOND;
                    case "not_same" -> BooleanOp.NOT_SAME;
                    case "second" -> BooleanOp.SECOND;
                    case "first" -> BooleanOp.FIRST;
                    default -> throw new IllegalArgumentException("Unknown VoxelShape boolean op");
                };
                result.add("shape", snapshot(Shapes.join(first, second, op)));
                result.addProperty("join_is_not_empty", Shapes.joinIsNotEmpty(first, second, op));
            } else if (kind.equals("join_many")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("join_many needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                result.add("shape", snapshot(Shapes.or(first, tail)));
            } else if (kind.equals("single_encompassing")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("single_encompassing needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                result.add("shape", snapshot(Shapes.or(first, tail).singleEncompassing()));
            } else if (kind.equals("optimize")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("optimize needs at least one shape");
                }
                VoxelShape shape = voxelShape(entries.get(0).getAsJsonObject());
                for (int i = 1; i < entries.size(); i++) {
                    shape = Shapes.joinUnoptimized(
                        shape,
                        voxelShape(entries.get(i).getAsJsonObject()),
                        BooleanOp.OR
                    );
                }
                var events = new JsonArray();
                events.add(snapshotWithCoords(shape));
                events.add(snapshotWithCoords(shape.optimize()));
                result.add("events", events);
            } else if (kind.equals("edges")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("edges needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                VoxelShape shape = Shapes.or(first, tail);
                var edges = new JsonArray();
                shape.forAllEdges((x1, y1, z1, x2, y2, z2) -> {
                    var edge = new JsonArray();
                    edge.add(Oracle.bits(x1));
                    edge.add(Oracle.bits(y1));
                    edge.add(Oracle.bits(z1));
                    edge.add(Oracle.bits(x2));
                    edge.add(Oracle.bits(y2));
                    edge.add(Oracle.bits(z2));
                    edges.add(edge);
                });
                result.add("edges", edges);
            } else if (kind.equals("join_coords")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                BooleanOp op = switch (request.get("op").getAsString()) {
                    case "or" -> BooleanOp.OR;
                    case "only_first" -> BooleanOp.ONLY_FIRST;
                    case "only_second" -> BooleanOp.ONLY_SECOND;
                    case "and" -> BooleanOp.AND;
                    case "not_same" -> BooleanOp.NOT_SAME;
                    case "first" -> BooleanOp.FIRST;
                    case "second" -> BooleanOp.SECOND;
                    default -> throw new IllegalArgumentException("Unknown VoxelShape boolean op");
                };
                result.add("unoptimized", snapshotWithCoords(Shapes.joinUnoptimized(first, second, op)));
                result.add("optimized", snapshotWithCoords(Shapes.join(first, second, op)));
            } else if (kind.equals("boxes")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("boxes needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                VoxelShape shape = Shapes.or(first, tail);
                var boxes = new JsonArray();
                shape.forAllBoxes((x1, y1, z1, x2, y2, z2) -> {
                    var box = new JsonArray();
                    box.add(Oracle.bits(x1));
                    box.add(Oracle.bits(y1));
                    box.add(Oracle.bits(z1));
                    box.add(Oracle.bits(x2));
                    box.add(Oracle.bits(y2));
                    box.add(Oracle.bits(z2));
                    boxes.add(box);
                });
                result.add("boxes", boxes);
            } else if (kind.equals("create_aabb")) {
                AABB aabb = box(request.getAsJsonObject("aabb"));
                result.add("shape", snapshot(Shapes.create(aabb)));
            } else if (kind.equals("create_coords")) {
                JsonObject box = request.getAsJsonObject("box");
                result.add("shape", snapshot(Shapes.create(
                    f64(box, "min_x"), f64(box, "min_y"), f64(box, "min_z"),
                    f64(box, "max_x"), f64(box, "max_y"), f64(box, "max_z"))));
            } else if (kind.equals("infinity")) {
                VoxelShape infinity = Shapes.INFINITY;
                String view = request.get("view").getAsString();
                if (view.equals("snapshot")) {
                    result.add("shape", snapshot(infinity));
                } else if (view.equals("boxes")) {
                    var boxes = new JsonArray();
                    infinity.forAllBoxes((x1, y1, z1, x2, y2, z2) -> {
                        var box = new JsonArray();
                        box.add(Oracle.bits(x1));
                        box.add(Oracle.bits(y1));
                        box.add(Oracle.bits(z1));
                        box.add(Oracle.bits(x2));
                        box.add(Oracle.bits(y2));
                        box.add(Oracle.bits(z2));
                        boxes.add(box);
                    });
                    result.add("boxes", boxes);
                } else if (view.equals("not_same_self")) {
                    result.add("shape", snapshot(Shapes.join(infinity, infinity, BooleanOp.NOT_SAME)));
                } else {
                    throw new IllegalArgumentException("Unknown infinity view");
                }
            } else if (kind.equals("constants")) {
                String name = request.get("name").getAsString();
                if (name.equals("epsilon")) {
                    result.addProperty("epsilon", Oracle.bits(Shapes.EPSILON));
                } else if (name.equals("big_epsilon")) {
                    result.addProperty("big_epsilon", Oracle.bits(Shapes.BIG_EPSILON));
                } else if (name.equals("both")) {
                    result.addProperty("epsilon", Oracle.bits(Shapes.EPSILON));
                    result.addProperty("big_epsilon", Oracle.bits(Shapes.BIG_EPSILON));
                } else {
                    throw new IllegalArgumentException("Unknown shape constant");
                }
            } else if (kind.equals("axis_limits")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("axis_limits needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                VoxelShape shape = Shapes.or(first, tail);
                result.addProperty("min_x", Oracle.bits(shape.min(Direction.Axis.X)));
                result.addProperty("max_x", Oracle.bits(shape.max(Direction.Axis.X)));
                result.addProperty("min_y", Oracle.bits(shape.min(Direction.Axis.Y)));
                result.addProperty("max_y", Oracle.bits(shape.max(Direction.Axis.Y)));
                result.addProperty("min_z", Oracle.bits(shape.min(Direction.Axis.Z)));
                result.addProperty("max_z", Oracle.bits(shape.max(Direction.Axis.Z)));
            } else if (kind.equals("axis_limits_at")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("axis_limits_at needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                VoxelShape shape = Shapes.or(first, tail);
                var values = new JsonArray();
                for (var entry : request.getAsJsonArray("queries")) {
                    JsonObject query = entry.getAsJsonObject();
                    Direction.Axis axis = Direction.Axis.byName(query.get("axis").getAsString());
                    if (axis == null) {
                        throw new IllegalArgumentException("Unknown shape axis");
                    }
                    double value = query.get("op").getAsString().equals("min")
                        ? shape.min(axis, f64(query, "b"), f64(query, "c"))
                        : shape.max(axis, f64(query, "b"), f64(query, "c"));
                    values.add(Oracle.bits(value));
                }
                result.add("values", values);
            } else if (kind.equals("equal")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                result.addProperty("equal", Shapes.equal(first, second));
            } else if (kind.equals("object_equals")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                VoxelShape sameGeometry = voxelShape(request.getAsJsonObject("same"));
                VoxelShape differentGeometry = voxelShape(request.getAsJsonObject("different"));
                result.addProperty("self", shape.equals(shape));
                result.addProperty("same_geometry", shape.equals(sameGeometry));
                result.addProperty("different_geometry", shape.equals(differentGeometry));
            } else if (kind.equals("object_equals_constants")) {
                VoxelShape empty = Shapes.empty();
                VoxelShape block = Shapes.block();
                VoxelShape infinity = Shapes.INFINITY;
                result.addProperty("empty_self", empty.equals(Shapes.empty()));
                result.addProperty("empty_thin", empty.equals(Shapes.box(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)));
                result.addProperty("empty_move", empty.equals(empty.move(1.0, 2.0, 3.0)));
                result.addProperty("block_self", block.equals(Shapes.block()));
                result.addProperty("block_box", block.equals(Shapes.box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)));
                result.addProperty("block_move", block.equals(block.move(0.0, 0.0, 0.0)));
                result.addProperty("infinity_self", infinity.equals(Shapes.INFINITY));
                result.addProperty("infinity_box", infinity.equals(Shapes.box(
                    Double.NEGATIVE_INFINITY,
                    Double.NEGATIVE_INFINITY,
                    Double.NEGATIVE_INFINITY,
                    Double.POSITIVE_INFINITY,
                    Double.POSITIVE_INFINITY,
                    Double.POSITIVE_INFINITY
                )));
            } else if (kind.equals("to_string")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                result.addProperty("value", shape.toString());
            } else if (kind.equals("rotate")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                OctahedralGroup rotation = OctahedralGroup.valueOf(request.get("rotation").getAsString());
                result.add("shape", snapshot(Shapes.rotate(shape, rotation)));
            } else if (kind.equals("rotate_join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                OctahedralGroup rotation = OctahedralGroup.valueOf(request.get("rotation").getAsString());
                result.add("shape", snapshot(Shapes.rotate(Shapes.or(first, second), rotation)));
            } else if (kind.equals("rotate_center")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                OctahedralGroup rotation = OctahedralGroup.valueOf(request.get("rotation").getAsString());
                result.add("shape", snapshot(Shapes.rotate(
                    shape, rotation, vec3(request.getAsJsonObject("center")))));
            } else if (kind.equals("rotate_horizontal_axis")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var axes = Shapes.rotateHorizontalAxis(shape);
                var snapshots = new JsonArray();
                for (Direction.Axis axis : Direction.Axis.values()) {
                    if (axes.containsKey(axis)) {
                        var entry = new JsonObject();
                        entry.addProperty("axis", axis.getName());
                        entry.add("shape", snapshot(axes.get(axis)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_horizontal_axis_center")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var axes = Shapes.rotateHorizontalAxis(shape, vec3(request.getAsJsonObject("center")));
                var snapshots = new JsonArray();
                for (Direction.Axis axis : Direction.Axis.values()) {
                    if (axes.containsKey(axis)) {
                        var entry = new JsonObject();
                        entry.addProperty("axis", axis.getName());
                        entry.add("shape", snapshot(axes.get(axis)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_all_axis")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var axes = Shapes.rotateAllAxis(shape);
                var snapshots = new JsonArray();
                for (Direction.Axis axis : Direction.Axis.values()) {
                    if (axes.containsKey(axis)) {
                        var entry = new JsonObject();
                        entry.addProperty("axis", axis.getName());
                        entry.add("shape", snapshot(axes.get(axis)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_all_axis_center")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var axes = Shapes.rotateAllAxis(shape, vec3(request.getAsJsonObject("center")));
                var snapshots = new JsonArray();
                for (Direction.Axis axis : Direction.Axis.values()) {
                    if (axes.containsKey(axis)) {
                        var entry = new JsonObject();
                        entry.addProperty("axis", axis.getName());
                        entry.add("shape", snapshot(axes.get(axis)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_horizontal")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var directions = Shapes.rotateHorizontal(shape);
                var snapshots = new JsonArray();
                for (Direction direction : Direction.values()) {
                    if (directions.containsKey(direction)) {
                        var entry = new JsonObject();
                        entry.addProperty("direction", direction.getName());
                        entry.add("shape", snapshot(directions.get(direction)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_horizontal_initial")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                OctahedralGroup initial = OctahedralGroup.valueOf(request.get("initial").getAsString());
                var directions = Shapes.rotateHorizontal(shape, initial);
                var snapshots = new JsonArray();
                for (Direction direction : Direction.values()) {
                    if (directions.containsKey(direction)) {
                        var entry = new JsonObject();
                        entry.addProperty("direction", direction.getName());
                        entry.add("shape", snapshot(directions.get(direction)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_all")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var directions = Shapes.rotateAll(shape);
                var snapshots = new JsonArray();
                for (Direction direction : Direction.values()) {
                    if (directions.containsKey(direction)) {
                        var entry = new JsonObject();
                        entry.addProperty("direction", direction.getName());
                        entry.add("shape", snapshot(directions.get(direction)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_all_center")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var directions = Shapes.rotateAll(shape, vec3(request.getAsJsonObject("center")));
                var snapshots = new JsonArray();
                for (Direction direction : Direction.values()) {
                    if (directions.containsKey(direction)) {
                        var entry = new JsonObject();
                        entry.addProperty("direction", direction.getName());
                        entry.add("shape", snapshot(directions.get(direction)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_all_initial")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                OctahedralGroup initial = OctahedralGroup.valueOf(request.get("initial").getAsString());
                var directions = Shapes.rotateAll(shape, initial, vec3(request.getAsJsonObject("center")));
                var snapshots = new JsonArray();
                for (Direction direction : Direction.values()) {
                    if (directions.containsKey(direction)) {
                        var entry = new JsonObject();
                        entry.addProperty("direction", direction.getName());
                        entry.add("shape", snapshot(directions.get(direction)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_attach_face")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var faces = Shapes.rotateAttachFace(shape);
                var faceSnapshots = new JsonArray();
                for (AttachFace face : AttachFace.values()) {
                    var directions = faces.get(face);
                    var directionSnapshots = new JsonArray();
                    for (Direction direction : Direction.values()) {
                        if (directions.containsKey(direction)) {
                            var directionEntry = new JsonObject();
                            directionEntry.addProperty("direction", direction.getName());
                            directionEntry.add("shape", snapshot(directions.get(direction)));
                            directionSnapshots.add(directionEntry);
                        }
                    }
                    var faceEntry = new JsonObject();
                    faceEntry.addProperty("face", face.getSerializedName());
                    faceEntry.add("shapes", directionSnapshots);
                    faceSnapshots.add(faceEntry);
                }
                result.add("faces", faceSnapshots);
            } else if (kind.equals("rotate_attach_face_initial")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                OctahedralGroup initial = OctahedralGroup.valueOf(request.get("initial").getAsString());
                var faces = Shapes.rotateAttachFace(shape, initial);
                var faceSnapshots = new JsonArray();
                for (AttachFace face : AttachFace.values()) {
                    var directions = faces.get(face);
                    var directionSnapshots = new JsonArray();
                    for (Direction direction : Direction.values()) {
                        if (directions.containsKey(direction)) {
                            var directionEntry = new JsonObject();
                            directionEntry.addProperty("direction", direction.getName());
                            directionEntry.add("shape", snapshot(directions.get(direction)));
                            directionSnapshots.add(directionEntry);
                        }
                    }
                    var faceEntry = new JsonObject();
                    faceEntry.addProperty("face", face.getSerializedName());
                    faceEntry.add("shapes", directionSnapshots);
                    faceSnapshots.add(faceEntry);
                }
                result.add("faces", faceSnapshots);
            } else if (kind.equals("collide")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                AABB moving = box(request.getAsJsonObject("moving"));
                Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
                if (axis == null) {
                    throw new IllegalArgumentException("Unknown collision axis");
                }
                result.addProperty("distance", Oracle.bits(shape.collide(axis, moving, f64(request, "distance"))));
            } else if (kind.equals("collide_many")) {
                AABB moving = box(request.getAsJsonObject("moving"));
                Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
                if (axis == null) {
                    throw new IllegalArgumentException("Unknown collision axis");
                }
                var shapes = new java.util.ArrayList<VoxelShape>();
                for (var entry : request.getAsJsonArray("shapes")) {
                    shapes.add(voxelShape(entry.getAsJsonObject()));
                }
                result.addProperty("distance", Oracle.bits(Shapes.collide(
                    axis, moving, shapes, f64(request, "distance"))));
            } else if (kind.equals("collide_movement")) {
                AABB moving = box(request.getAsJsonObject("moving"));
                var shapes = new java.util.ArrayList<VoxelShape>();
                for (var entry : request.getAsJsonArray("shapes")) {
                    shapes.add(voxelShape(entry.getAsJsonObject()));
                }
                try {
                    var method = Entity.class.getDeclaredMethod(
                        "collideWithShapes", Vec3.class, AABB.class, java.util.List.class);
                    method.setAccessible(true);
                    var resolved = (Vec3) method.invoke(
                        null, vec3(request.getAsJsonObject("movement")), moving, shapes);
                    result.add("movement", snapshot(resolved));
                } catch (ReflectiveOperationException error) {
                    throw new IllegalStateException("Entity.collideWithShapes reflection failed", error);
                }
            } else if (kind.equals("entity_movement_state")) {
                var states = new JsonArray();
                for (var entry : request.getAsJsonArray("inputs")) {
                    JsonObject input = entry.getAsJsonObject();
                    Vec3 movement = vec3(input.getAsJsonObject("movement"));
                    AABB moving = box(input.getAsJsonObject("moving"));
                    var shapes = new java.util.ArrayList<VoxelShape>();
                    for (var shapeEntry : input.getAsJsonArray("shapes")) {
                        shapes.add(voxelShape(shapeEntry.getAsJsonObject()));
                    }
                    try {
                        var method = Entity.class.getDeclaredMethod(
                            "collideWithShapes", Vec3.class, AABB.class, java.util.List.class);
                        method.setAccessible(true);
                        var resolved = (Vec3) method.invoke(null, movement, moving, shapes);
                        boolean xCollision = !Mth.equal(movement.x, resolved.x);
                        boolean yCollision = movement.y != resolved.y;
                        boolean zCollision = !Mth.equal(movement.z, resolved.z);
                        boolean horizontalCollision = xCollision || zCollision;
                        boolean verticalCollisionBelow = yCollision && movement.y < 0.0;
                        var state = new JsonObject();
                        state.addProperty("label", input.get("label").getAsString());
                        state.add("movement", snapshot(resolved));
                        state.addProperty("x_collision", xCollision);
                        state.addProperty("y_collision", yCollision);
                        state.addProperty("z_collision", zCollision);
                        state.addProperty("horizontal_collision", horizontalCollision);
                        state.addProperty("vertical_collision", yCollision);
                        state.addProperty("vertical_collision_below", verticalCollisionBelow);
                        state.addProperty("on_ground", verticalCollisionBelow);
                        states.add(state);
                    } catch (ReflectiveOperationException error) {
                        throw new IllegalStateException("Entity.collideWithShapes reflection failed", error);
                    }
                }
                result.add("states", states);
            } else if (kind.equals("entity_movement_state_retention")) {
                var states = new JsonArray();
                for (var entry : request.getAsJsonArray("inputs")) {
                    JsonObject input = entry.getAsJsonObject();
                    Vec3 delta = vec3(input.getAsJsonObject("delta"));
                    AABB moving = box(input.getAsJsonObject("moving"));
                    var shapes = new java.util.ArrayList<VoxelShape>();
                    for (var shapeEntry : input.getAsJsonArray("shapes")) {
                        shapes.add(voxelShape(shapeEntry.getAsJsonObject()));
                    }
                    try {
                        var method = Entity.class.getDeclaredMethod(
                            "collideWithShapes", Vec3.class, AABB.class, java.util.List.class);
                        method.setAccessible(true);
                        var resolved = (Vec3)method.invoke(null, delta, moving, shapes);
                        boolean xCollision = !Mth.equal(delta.x, resolved.x);
                        boolean yCollision = delta.y != resolved.y;
                        boolean zCollision = !Mth.equal(delta.z, resolved.z);
                        boolean horizontalCollision = xCollision || zCollision;
                        boolean verticalCollision = input.getAsJsonObject("previous")
                            .get("vertical_collision").getAsBoolean();
                        boolean verticalCollisionBelow = input.getAsJsonObject("previous")
                            .get("vertical_collision_below").getAsBoolean();
                        boolean onGround = input.getAsJsonObject("previous")
                            .get("on_ground").getAsBoolean();
                        boolean movedVertically = Math.abs(delta.y) > 0.0;
                        boolean authoritative = input.get("authoritative").getAsBoolean();
                        if (movedVertically || authoritative) {
                            verticalCollision = yCollision;
                            verticalCollisionBelow = verticalCollision && delta.y < 0.0;
                            onGround = verticalCollisionBelow;
                        }
                        var state = new JsonObject();
                        state.addProperty("label", input.get("label").getAsString());
                        state.add("movement", snapshot(resolved));
                        state.addProperty("x_collision", xCollision);
                        state.addProperty("y_collision", yCollision);
                        state.addProperty("z_collision", zCollision);
                        state.addProperty("horizontal_collision", horizontalCollision);
                        state.addProperty("vertical_collision", verticalCollision);
                        state.addProperty("vertical_collision_below", verticalCollisionBelow);
                        state.addProperty("on_ground", onGround);
                        states.add(state);
                    } catch (ReflectiveOperationException error) {
                        throw new IllegalStateException(
                            "Entity movement state retention reflection failed", error);
                    }
                }
                result.add("states", states);
            } else if (kind.equals("entity_collision_shapes")) {
                var output = System.out;
                var errorOutput = System.err;
                try {
                    SharedConstants.setVersion(TEST_VERSION);
                    Bootstrap.bootStrap();
                } finally {
                    System.setOut(output);
                    System.setErr(errorOutput);
                }
                AABB testArea = box(request.getAsJsonObject("test_area"));
                boolean sourcePresent = request.has("source_present")
                    && request.get("source_present").getAsBoolean();
                CollisionFixtureEntity source = sourcePresent
                    ? CollisionFixtureEntity.create(
                        box(request.getAsJsonObject("source_box")), false, true, false)
                    : null;
                var entities = new java.util.ArrayList<Entity>();
                for (var entry : request.getAsJsonArray("candidates")) {
                    JsonObject candidate = entry.getAsJsonObject();
                    boolean isSource = candidate.has("is_source")
                        && candidate.get("is_source").getAsBoolean();
                    entities.add(sourcePresent && isSource
                        ? source
                        : CollisionFixtureEntity.create(
                            box(candidate.getAsJsonObject("box")),
                            candidate.has("spectator") && candidate.get("spectator").getAsBoolean(),
                            candidate.has("can_be_collided_with")
                                && candidate.get("can_be_collided_with").getAsBoolean(),
                            candidate.has("same_vehicle") && candidate.get("same_vehicle").getAsBoolean()));
                }
                EntityCollisionFixture collisionFixture = new EntityCollisionFixture(entities);
                var shapes = collisionFixture.getEntityCollisions(source, testArea);
                var shapeSnapshots = new JsonArray();
                for (VoxelShape shape : shapes) {
                    shapeSnapshots.add(snapshot(shape));
                }
                result.add("shapes", shapeSnapshots);
            } else if (kind.equals("world_border_collision_shape")) {
                SharedConstants.setVersion(TEST_VERSION);
                var states = new JsonArray();
                for (var entry : request.getAsJsonArray("inputs")) {
                    JsonObject input = entry.getAsJsonObject();
                    WorldBorder border = new WorldBorder();
                    border.setCenter(f64(input, "center_x"), f64(input, "center_z"));
                    border.setAbsoluteMaxSize(input.get("absolute_max_size").getAsInt());
                    border.setSize(f64(input, "size"));
                    states.add(snapshot(border.getCollisionShape()));
                }
                result.add("states", states);
            } else if (kind.equals("world_border_collision_shape_moving")) {
                SharedConstants.setVersion(TEST_VERSION);
                var states = new JsonArray();
                for (var entry : request.getAsJsonArray("inputs")) {
                    JsonObject input = entry.getAsJsonObject();
                    WorldBorder border = new WorldBorder();
                    border.setCenter(f64(input, "center_x"), f64(input, "center_z"));
                    border.setAbsoluteMaxSize(input.get("absolute_max_size").getAsInt());
                    border.lerpSizeBetween(
                        f64(input, "from"),
                        f64(input, "to"),
                        input.get("duration").getAsLong(),
                        0L
                    );
                    var partialTicks = input.getAsJsonArray("partial_ticks");
                    int tickCount = input.get("tick_count").getAsInt();
                    var track = new JsonArray();
                    for (int tick = 0; tick <= tickCount; tick++) {
                        var samples = new JsonArray();
                        for (var partialEntry : partialTicks) {
                            float partialTick = Float.intBitsToFloat(
                                Integer.parseUnsignedInt(partialEntry.getAsString(), 16));
                            var sample = new JsonObject();
                            sample.addProperty("partial_tick", Oracle.bits(partialTick));
                            sample.addProperty("min_x", Oracle.bits(border.getMinX(partialTick)));
                            sample.addProperty("max_x", Oracle.bits(border.getMaxX(partialTick)));
                            sample.addProperty("min_z", Oracle.bits(border.getMinZ(partialTick)));
                            sample.addProperty("max_z", Oracle.bits(border.getMaxZ(partialTick)));
                            samples.add(sample);
                        }
                        var state = new JsonObject();
                        state.addProperty("size", Oracle.bits(border.getSize()));
                        state.addProperty("lerp_time", border.getLerpTime());
                        state.addProperty("lerp_target", Oracle.bits(border.getLerpTarget()));
                        state.addProperty("status", border.getStatus().name());
                        state.add("samples", samples);
                        state.add("collision_shape", snapshot(border.getCollisionShape()));
                        track.add(state);
                        if (tick < tickCount) {
                            border.tick();
                        }
                    }
                    states.add(track);
                }
                result.add("states", states);
            } else if (kind.equals("client_world_border_state")) {
                SharedConstants.setVersion(TEST_VERSION);
                var tracks = new JsonArray();
                for (var trackEntry : request.getAsJsonArray("inputs")) {
                    WorldBorder border = new WorldBorder();
                    long gameTime = 0L;
                    var states = new JsonArray();
                    for (var eventEntry : trackEntry.getAsJsonObject().getAsJsonArray("events")) {
                        JsonObject event = eventEntry.getAsJsonObject();
                        String eventKind = event.get("op").getAsString();
                        switch (eventKind) {
                            case "initialize" -> {
                                border.setCenter(f64(event, "center_x"), f64(event, "center_z"));
                                long lerpTime = event.get("lerp_time").getAsLong();
                                if (lerpTime > 0L) {
                                    border.lerpSizeBetween(
                                        f64(event, "old_size"),
                                        f64(event, "new_size"),
                                        lerpTime,
                                        gameTime
                                    );
                                } else {
                                    border.setSize(f64(event, "new_size"));
                                }
                                border.setAbsoluteMaxSize(event.get("absolute_max_size").getAsInt());
                                border.setWarningBlocks(event.get("warning_blocks").getAsInt());
                                border.setWarningTime(event.get("warning_time").getAsInt());
                            }
                            case "center" -> border.setCenter(
                                f64(event, "center_x"), f64(event, "center_z"));
                            case "lerp" -> border.lerpSizeBetween(
                                f64(event, "old_size"),
                                f64(event, "new_size"),
                                event.get("lerp_time").getAsLong(),
                                gameTime
                            );
                            case "size" -> border.setSize(f64(event, "size"));
                            case "warning_blocks" -> border.setWarningBlocks(event.get("value").getAsInt());
                            case "warning_time" -> border.setWarningTime(event.get("value").getAsInt());
                            case "tick" -> {
                                border.tick();
                                gameTime++;
                            }
                            default -> throw new IllegalArgumentException("Unknown world border event");
                        }
                        var samples = new JsonArray();
                        for (float partialTick : new float[] {0.0F, 0.5F, 1.0F}) {
                            var sample = new JsonObject();
                            sample.addProperty("partial_tick", Oracle.bits(partialTick));
                            sample.addProperty("min_x", Oracle.bits(border.getMinX(partialTick)));
                            sample.addProperty("max_x", Oracle.bits(border.getMaxX(partialTick)));
                            sample.addProperty("min_z", Oracle.bits(border.getMinZ(partialTick)));
                            sample.addProperty("max_z", Oracle.bits(border.getMaxZ(partialTick)));
                            samples.add(sample);
                        }
                        var state = new JsonObject();
                        state.addProperty("event", eventKind);
                        state.addProperty("game_time", gameTime);
                        state.addProperty("center_x", Oracle.bits(border.getCenterX()));
                        state.addProperty("center_z", Oracle.bits(border.getCenterZ()));
                        state.addProperty("absolute_max_size", border.getAbsoluteMaxSize());
                        state.addProperty("warning_blocks", border.getWarningBlocks());
                        state.addProperty("warning_time", border.getWarningTime());
                        state.addProperty("size", Oracle.bits(border.getSize()));
                        state.addProperty("lerp_time", border.getLerpTime());
                        state.addProperty("lerp_target", Oracle.bits(border.getLerpTarget()));
                        state.addProperty("status", border.getStatus().name());
                        state.add("samples", samples);
                        state.add("collision_shape", snapshot(border.getCollisionShape()));
                        states.add(state);
                    }
                    tracks.add(states);
                }
                result.add("tracks", tracks);
            } else if (kind.equals("client_world_collision")) {
                var output = System.out;
                var errorOutput = System.err;
                try {
                    SharedConstants.setVersion(TEST_VERSION);
                    Bootstrap.bootStrap();
                } finally {
                    System.setOut(output);
                    System.setErr(errorOutput);
                }
                var states = new JsonArray();
                for (var entry : request.getAsJsonArray("inputs")) {
                    JsonObject input = entry.getAsJsonObject();
                    AABB testArea = box(input.getAsJsonObject("test_area"));
                    boolean sourcePresent = input.has("source_present")
                        && input.get("source_present").getAsBoolean();
                    CollisionFixtureEntity source = null;
                    if (sourcePresent) {
                        AABB sourceBox = box(input.getAsJsonObject("source_box"));
                        JsonObject sourcePosition = input.getAsJsonObject("source_position");
                        source = CollisionFixtureEntity.create(
                            sourceBox,
                            false,
                            true,
                            false,
                            new Vec3(
                            f64(sourcePosition, "x"),
                            f64(sourcePosition, "y"),
                            f64(sourcePosition, "z")
                            )
                        );
                    }
                    var entities = new java.util.ArrayList<Entity>();
                    for (var candidateEntry : input.getAsJsonArray("candidates")) {
                        JsonObject candidate = candidateEntry.getAsJsonObject();
                        boolean isSource = candidate.has("is_source")
                            && candidate.get("is_source").getAsBoolean();
                        entities.add(sourcePresent && isSource
                            ? source
                            : CollisionFixtureEntity.create(
                                box(candidate.getAsJsonObject("box")),
                                candidate.has("spectator")
                                    && candidate.get("spectator").getAsBoolean(),
                                candidate.has("can_be_collided_with")
                                    && candidate.get("can_be_collided_with").getAsBoolean(),
                                candidate.has("same_vehicle")
                                    && candidate.get("same_vehicle").getAsBoolean()));
                    }
                    JsonObject borderInput = input.getAsJsonObject("border");
                    WorldBorder border = new WorldBorder();
                    border.setCenter(f64(borderInput, "center_x"), f64(borderInput, "center_z"));
                    border.setAbsoluteMaxSize(borderInput.get("absolute_max_size").getAsInt());
                    border.setSize(f64(borderInput, "size"));
                    CollisionGetter collision = collisionFixture(
                        input.getAsJsonArray("placements"), entities, border);
                    var state = new JsonObject();
                    state.addProperty("label", input.get("label").getAsString());
                    state.addProperty("no_block_collision", collision.noBlockCollision(source, testArea));
                    state.addProperty("no_entity_collision", collision.noEntityCollision(source, testArea));
                    state.addProperty("no_border_collision", collision.noBorderCollision(source, testArea));
                    state.addProperty("no_collision", collision.noCollision(source, testArea));
                    states.add(state);
                }
                result.add("states", states);
            } else if (kind.equals("client_world_indexed_collision")) {
                var output = System.out;
                var errorOutput = System.err;
                try {
                    SharedConstants.setVersion(TEST_VERSION);
                    Bootstrap.bootStrap();
                } finally {
                    System.setOut(output);
                    System.setErr(errorOutput);
                }
                var states = new JsonArray();
                for (var entry : request.getAsJsonArray("inputs")) {
                    JsonObject input = entry.getAsJsonObject();
                    AABB testArea = box(input.getAsJsonObject("test_area"));
                    var entityStorage = new TransientEntitySectionManager<Entity>(
                        Entity.class, new NoopEntityCallbacks());
                    var entitiesById = new java.util.HashMap<Integer, Entity>();
                    for (var entityEntry : input.getAsJsonArray("entities")) {
                        JsonObject entityInput = entityEntry.getAsJsonObject();
                        int entityId = entityInput.get("entity_id").getAsInt();
                        int typeId = entityInput.get("type_id").getAsInt();
                        JsonObject positionInput = entityInput.getAsJsonObject("position");
                        Vec3 position = new Vec3(
                            f64(positionInput, "x"),
                            f64(positionInput, "y"),
                            f64(positionInput, "z"));
                        boolean spectator = entityInput.has("spectator")
                            && entityInput.get("spectator").getAsBoolean();
                        boolean alive = !entityInput.has("alive")
                            || entityInput.get("alive").getAsBoolean();
                        boolean onClimbable = entityInput.has("on_climbable")
                            && entityInput.get("on_climbable").getAsBoolean();
                        boolean flying = entityInput.has("flying")
                            && entityInput.get("flying").getAsBoolean();
                        JsonObject climbableContext = entityInput.has("climbable_context")
                            ? entityInput.getAsJsonObject("climbable_context")
                            : null;
                        boolean collidable = entityInput.has("can_be_collided_with")
                            && entityInput.get("can_be_collided_with").getAsBoolean();
                        boolean sameVehicle = entityInput.has("same_vehicle")
                            && entityInput.get("same_vehicle").getAsBoolean();
                        Entity entity = CollisionFixtureEntity.createIndexed(
                            entityId,
                            typeId,
                            position,
                            spectator,
                            alive,
                            onClimbable,
                            flying,
                            collidable,
                            sameVehicle,
                            climbableContext);
                        entityStorage.addEntity(entity);
                        entitiesById.put(entityId, entity);
                    }
                    Integer sourceId = input.has("source_entity_id")
                        && !input.get("source_entity_id").isJsonNull()
                        ? input.get("source_entity_id").getAsInt()
                        : null;
                    Entity source = sourceId == null ? null : entitiesById.get(sourceId);
                    JsonObject borderInput = input.getAsJsonObject("border");
                    WorldBorder border = new WorldBorder();
                    border.setCenter(f64(borderInput, "center_x"), f64(borderInput, "center_z"));
                    border.setAbsoluteMaxSize(borderInput.get("absolute_max_size").getAsInt());
                    border.setSize(f64(borderInput, "size"));
                    CollisionGetter collision = collisionFixture(
                        input.getAsJsonArray("placements"),
                        new IndexedEntityCollisionFixture(entityStorage.getEntityGetter()),
                        border);
                    var state = new JsonObject();
                    state.addProperty("label", input.get("label").getAsString());
                    state.addProperty("entity_count", entityStorage.count());
                    state.addProperty("no_entity_collision", collision.noEntityCollision(source, testArea));
                    state.addProperty("no_collision", collision.noCollision(source, testArea));
                    states.add(state);
                }
                result.add("states", states);
            } else if (kind.equals("client_world_entity_section_index")) {
                var output = System.out;
                var errorOutput = System.err;
                try {
                    SharedConstants.setVersion(TEST_VERSION);
                    Bootstrap.bootStrap();
                } finally {
                    System.setOut(output);
                    System.setErr(errorOutput);
                }
                var tracks = new JsonArray();
                for (var entry : request.getAsJsonArray("inputs")) {
                    JsonObject input = entry.getAsJsonObject();
                    var entityStorage = new TransientEntitySectionManager<Entity>(
                        Entity.class, new NoopEntityCallbacks());
                    var entitiesById = new java.util.HashMap<Integer, Entity>();
                    var entityTypesById = new java.util.HashMap<Integer, Integer>();
                    for (var entityEntry : input.getAsJsonArray("entities")) {
                        JsonObject entityInput = entityEntry.getAsJsonObject();
                        int entityId = entityInput.get("entity_id").getAsInt();
                        int typeId = entityInput.get("type_id").getAsInt();
                        Vec3 position = vec3(entityInput.getAsJsonObject("position"));
                        boolean spectator = entityInput.has("spectator")
                            && entityInput.get("spectator").getAsBoolean();
                        boolean alive = !entityInput.has("alive")
                            || entityInput.get("alive").getAsBoolean();
                        boolean collidable = entityInput.has("can_be_collided_with")
                            && entityInput.get("can_be_collided_with").getAsBoolean();
                        boolean sameVehicle = entityInput.has("same_vehicle")
                            && entityInput.get("same_vehicle").getAsBoolean();
                        Entity entity = CollisionFixtureEntity.createIndexed(
                            entityId,
                            typeId,
                            position,
                            spectator,
                            alive,
                            false,
                            false,
                            collidable,
                            sameVehicle,
                            null);
                        entityStorage.addEntity(entity);
                        entitiesById.put(entityId, entity);
                        entityTypesById.put(entityId, typeId);
                    }
                    Integer sourceId = input.has("source_entity_id")
                        && !input.get("source_entity_id").isJsonNull()
                        ? input.get("source_entity_id").getAsInt()
                        : null;
                    JsonObject borderInput = input.getAsJsonObject("border");
                    WorldBorder border = new WorldBorder();
                    border.setCenter(f64(borderInput, "center_x"), f64(borderInput, "center_z"));
                    border.setAbsoluteMaxSize(borderInput.get("absolute_max_size").getAsInt());
                    border.setSize(f64(borderInput, "size"));
                    CollisionGetter collision = collisionFixture(
                        input.getAsJsonArray("placements"),
                        new IndexedEntityCollisionFixture(entityStorage.getEntityGetter()),
                        border);
                    var states = new JsonArray();
                    for (var eventEntry : input.getAsJsonArray("events")) {
                        JsonObject event = eventEntry.getAsJsonObject();
                        String eventKind = event.get("op").getAsString();
                        switch (eventKind) {
                            case "move" -> {
                                int entityId = event.get("entity_id").getAsInt();
                                Entity entity = entitiesById.get(entityId);
                                if (entity == null) {
                                    throw new IllegalArgumentException("Cannot move missing entity " + entityId);
                                }
                                Vec3 position = vec3(event.getAsJsonObject("position"));
                                entity.setPosRaw(position.x, position.y, position.z);
                                int typeId = entityTypesById.get(entityId);
                                var entityType = BuiltInRegistries.ENTITY_TYPE.byId(typeId);
                                entity.setBoundingBox(entityType.getDimensions().makeBoundingBox(position));
                            }
                            case "remove" -> {
                                int entityId = event.get("entity_id").getAsInt();
                                Entity entity = entitiesById.remove(entityId);
                                entityTypesById.remove(entityId);
                                if (entity != null) {
                                    entity.setRemoved(Entity.RemovalReason.DISCARDED);
                                }
                            }
                            case "add" -> {
                                JsonObject entityInput = event.getAsJsonObject("entity");
                                int entityId = entityInput.get("entity_id").getAsInt();
                                Entity previous = entitiesById.remove(entityId);
                                entityTypesById.remove(entityId);
                                if (previous != null) {
                                    previous.setRemoved(Entity.RemovalReason.DISCARDED);
                                }
                                int typeId = entityInput.get("type_id").getAsInt();
                                Vec3 position = vec3(entityInput.getAsJsonObject("position"));
                                boolean spectator = entityInput.has("spectator")
                                    && entityInput.get("spectator").getAsBoolean();
                                boolean alive = !entityInput.has("alive")
                                    || entityInput.get("alive").getAsBoolean();
                                boolean collidable = entityInput.has("can_be_collided_with")
                                    && entityInput.get("can_be_collided_with").getAsBoolean();
                                boolean sameVehicle = entityInput.has("same_vehicle")
                                    && entityInput.get("same_vehicle").getAsBoolean();
                                Entity entity = CollisionFixtureEntity.createIndexed(
                                    entityId,
                                    typeId,
                                    position,
                                    spectator,
                                    alive,
                                    false,
                                    false,
                                    collidable,
                                    sameVehicle,
                                    null);
                                entityStorage.addEntity(entity);
                                entitiesById.put(entityId, entity);
                                entityTypesById.put(entityId, typeId);
                            }
                            case "query" -> {
                                AABB testArea = box(event.getAsJsonObject("test_area"));
                                Entity source = sourceId == null ? null : entitiesById.get(sourceId);
                                String[] stats = entityStorage.gatherStats().split(",");
                                var state = new JsonObject();
                                state.addProperty("label", event.get("label").getAsString());
                                state.addProperty("entity_count", entityStorage.count());
                                state.addProperty("section_count", Integer.parseInt(stats[1]));
                                state.addProperty("no_entity_collision", collision.noEntityCollision(source, testArea));
                                state.addProperty("no_collision", collision.noCollision(source, testArea));
                                states.add(state);
                            }
                            default -> throw new IllegalArgumentException("Unknown entity section event " + eventKind);
                        }
                    }
                    var track = new JsonObject();
                    track.addProperty("label", input.get("label").getAsString());
                    track.add("states", states);
                    tracks.add(track);
                }
                result.add("tracks", tracks);
            } else if (kind.equals("client_world_happy_ghast_collision")) {
                var output = System.out;
                var errorOutput = System.err;
                try {
                    SharedConstants.setVersion(TEST_VERSION);
                    Bootstrap.bootStrap();
                } finally {
                    System.setOut(output);
                    System.setErr(errorOutput);
                }
                var states = new JsonArray();
                for (var entry : request.getAsJsonArray("inputs")) {
                    JsonObject input = entry.getAsJsonObject();
                    var entityStorage = new TransientEntitySectionManager<Entity>(
                        Entity.class, new NoopEntityCallbacks());
                    for (var entityEntry : input.getAsJsonArray("entities")) {
                        Entity entity = CollisionFixtureEntity.createHappyGhastIndexed(
                            entityEntry.getAsJsonObject());
                        entityStorage.addEntity(entity);
                    }
                    Integer sourceId = input.has("source_entity_id")
                        && !input.get("source_entity_id").isJsonNull()
                        ? input.get("source_entity_id").getAsInt()
                        : null;
                    Entity source = null;
                    if (sourceId != null) {
                        source = entityStorage.getEntityGetter().get(sourceId);
                        if (source == null) {
                            throw new IllegalArgumentException("Unknown collision source " + sourceId);
                        }
                    }
                    JsonObject borderInput = input.getAsJsonObject("border");
                    WorldBorder border = new WorldBorder();
                    border.setCenter(f64(borderInput, "center_x"), f64(borderInput, "center_z"));
                    border.setAbsoluteMaxSize(borderInput.get("absolute_max_size").getAsInt());
                    border.setSize(f64(borderInput, "size"));
                    CollisionGetter collision = collisionFixture(
                        input.getAsJsonArray("placements"),
                        new IndexedEntityCollisionFixture(entityStorage.getEntityGetter()),
                        border);
                    AABB testArea = box(input.getAsJsonObject("test_area"));
                    var state = new JsonObject();
                    state.addProperty("label", input.get("label").getAsString());
                    state.addProperty("entity_count", entityStorage.count());
                    state.addProperty("no_entity_collision", collision.noEntityCollision(source, testArea));
                    state.addProperty("no_collision", collision.noCollision(source, testArea));
                    states.add(state);
                }
                result.add("states", states);
            } else if (kind.equals("client_world_registry_collision")) {
                var output = System.out;
                var errorOutput = System.err;
                try {
                    SharedConstants.setVersion(TEST_VERSION);
                    Bootstrap.bootStrap();
                } finally {
                    System.setOut(output);
                    System.setErr(errorOutput);
                }
                var resolvedStates = new JsonArray();
                for (var stateEntry : request.getAsJsonArray("state_ids")) {
                    int requestedStateId = stateEntry.getAsInt();
                    BlockState state = Block.stateById(requestedStateId);
                    var stateSnapshot = new JsonObject();
                    stateSnapshot.addProperty("requested_state_id", requestedStateId);
                    stateSnapshot.addProperty("resolved_state_id", Block.getId(state));
                    stateSnapshot.add("collision_shape", snapshot(
                        state.getCollisionShape(
                            net.minecraft.world.level.EmptyBlockGetter.INSTANCE,
                            BlockPos.ZERO)));
                    resolvedStates.add(stateSnapshot);
                }
                AABB testArea = box(request.getAsJsonObject("test_area"));
                CollisionGetter collision = registryCollisionFixture(
                    request.getAsJsonArray("placements"));
                var collisions = new JsonArray();
                for (VoxelShape shape : collision.getBlockCollisions(null, testArea)) {
                    collisions.add(snapshot(shape));
                }
                result.add("states", resolvedStates);
                result.add("collisions", collisions);
                result.addProperty("no_block_collision", collision.noBlockCollision(null, testArea));
            } else if (kind.equals("client_world_stair_neighbor_collision")) {
                var output = System.out;
                var errorOutput = System.err;
                try {
                    SharedConstants.setVersion(TEST_VERSION);
                    Bootstrap.bootStrap();
                } finally {
                    System.setOut(output);
                    System.setErr(errorOutput);
                }
                var states = new JsonArray();
                for (var inputElement : request.getAsJsonArray("inputs")) {
                    JsonObject input = inputElement.getAsJsonObject();
                    BlockPos targetPos = blockPos(input.getAsJsonObject("target"));
                    var stateByPosition = registryStates(input.getAsJsonArray("placements"));
                    String targetKey = registryKey(targetPos);
                    BlockState before = stateByPosition.getOrDefault(
                        targetKey, Blocks.AIR.defaultBlockState());
                    if (!(before.getBlock() instanceof StairBlock)) {
                        throw new IllegalArgumentException("stair target must resolve to a stair block");
                    }
                    StairsShape updatedShape = stairShape(
                        before,
                        registryCollisionFixture(stateByPosition),
                        targetPos);
                    BlockState updated = before.setValue(StairBlock.SHAPE, updatedShape);
                    stateByPosition.put(targetKey, updated);
                    CollisionGetter collision = registryCollisionFixture(stateByPosition);
                    AABB testArea = box(input.getAsJsonObject("test_area"));
                    var collisions = new JsonArray();
                    for (VoxelShape shape : collision.getBlockCollisions(null, testArea)) {
                        collisions.add(snapshot(shape));
                    }
                    var state = new JsonObject();
                    state.addProperty("label", input.get("label").getAsString());
                    state.addProperty("before_state_id", Block.getId(before));
                    state.addProperty("updated_state_id", Block.getId(updated));
                    state.addProperty("updated_shape", updatedShape.name().toLowerCase(Locale.ROOT));
                    state.add("updated_collision_shape", snapshot(
                        updated.getCollisionShape(net.minecraft.world.level.EmptyBlockGetter.INSTANCE, targetPos)));
                    state.add("collisions", collisions);
                    state.addProperty("no_block_collision", collision.noBlockCollision(null, testArea));
                    states.add(state);
                }
                result.add("states", states);
            } else if (kind.equals("local_player_minor_horizontal_collision")) {
                try {
                    Unsafe unsafe = unsafeInstance();
                    var method = LocalPlayer.class.getDeclaredMethod(
                        "isHorizontalCollisionMinor", Vec3.class);
                    method.setAccessible(true);
                    var yRotField = Entity.class.getDeclaredField("yRot");
                    var xxaField = LivingEntity.class.getDeclaredField("xxa");
                    var zzaField = LivingEntity.class.getDeclaredField("zza");
                    var states = new JsonArray();
                    for (var inputEntry : request.getAsJsonArray("inputs")) {
                        JsonObject input = inputEntry.getAsJsonObject();
                        LocalPlayer player = (LocalPlayer)unsafe.allocateInstance(LocalPlayer.class);
                        unsafe.putFloat(player, unsafe.objectFieldOffset(yRotField),
                            Oracle.f32(input, "y_rot"));
                        unsafe.putFloat(player, unsafe.objectFieldOffset(xxaField),
                            Oracle.f32(input, "xxa"));
                        unsafe.putFloat(player, unsafe.objectFieldOffset(zzaField),
                            Oracle.f32(input, "zza"));
                        Vec3 movement = vec3(input.getAsJsonObject("movement"));
                        var state = new JsonObject();
                        state.addProperty("label", input.get("label").getAsString());
                        state.addProperty("minor_horizontal_collision",
                            (Boolean)method.invoke(player, movement));
                        states.add(state);
                    }
                    result.add("states", states);
                } catch (ReflectiveOperationException error) {
                    throw new IllegalStateException(
                        "LocalPlayer.isHorizontalCollisionMinor reflection failed", error);
                }
            } else if (kind.equals("collect_step_up_heights")) {
                AABB moving = box(request.getAsJsonObject("moving"));
                var shapes = new java.util.ArrayList<VoxelShape>();
                for (var entry : request.getAsJsonArray("shapes")) {
                    shapes.add(voxelShape(entry.getAsJsonObject()));
                }
                try {
                    var method = Entity.class.getDeclaredMethod(
                        "collectCandidateStepUpHeights", AABB.class, java.util.List.class, float.class, float.class);
                    method.setAccessible(true);
                    var candidates = (float[]) method.invoke(
                        null,
                        moving,
                        shapes,
                        Oracle.f32(request, "max_step_height"),
                        Oracle.f32(request, "step_height_to_skip"));
                    var values = new JsonArray();
                    for (float candidate : candidates) {
                        values.add(Oracle.bits(candidate));
                    }
                    result.add("candidates", values);
                } catch (ReflectiveOperationException error) {
                    throw new IllegalStateException("Entity.collectCandidateStepUpHeights reflection failed", error);
                }
            } else if (kind.equals("select_step_up_movement")) {
                AABB boundingBox = box(request.getAsJsonObject("bounding_box"));
                AABB groundedBox = box(request.getAsJsonObject("grounded_box"));
                Vec3 movement = vec3(request.getAsJsonObject("movement"));
                Vec3 movementStep = vec3(request.getAsJsonObject("movement_step"));
                var shapes = new java.util.ArrayList<VoxelShape>();
                for (var entry : request.getAsJsonArray("shapes")) {
                    shapes.add(voxelShape(entry.getAsJsonObject()));
                }
                try {
                    var candidatesMethod = Entity.class.getDeclaredMethod(
                        "collectCandidateStepUpHeights", AABB.class, java.util.List.class, float.class, float.class);
                    var collideMethod = Entity.class.getDeclaredMethod(
                        "collideWithShapes", Vec3.class, AABB.class, java.util.List.class);
                    candidatesMethod.setAccessible(true);
                    collideMethod.setAccessible(true);
                    var candidates = (float[]) candidatesMethod.invoke(
                        null,
                        groundedBox,
                        shapes,
                        Oracle.f32(request, "max_step_height"),
                        Oracle.f32(request, "step_height_to_skip"));
                    Vec3 selected = movementStep;
                    for (float candidate : candidates) {
                        Vec3 stepFromGround = (Vec3)collideMethod.invoke(
                            null,
                            new Vec3(movement.x, candidate, movement.z),
                            groundedBox,
                            shapes);
                        if (stepFromGround.horizontalDistanceSqr() > movementStep.horizontalDistanceSqr()) {
                            double distanceToGround = boundingBox.minY - groundedBox.minY;
                            selected = stepFromGround.subtract(0.0, distanceToGround, 0.0);
                            break;
                        }
                    }
                    result.add("movement", snapshot(selected));
                } catch (ReflectiveOperationException error) {
                    throw new IllegalStateException("Entity step-up movement selection reflection failed", error);
                }
            } else if (kind.equals("collect_block_collisions")) {
                AABB query = box(request.getAsJsonObject("box"));
                CollisionGetter collisionGetter = collisionFixture(request.getAsJsonArray("placements"));
                var collisions = new JsonArray();
                var iterator = new BlockCollisions<JsonObject>(
                    collisionGetter,
                    (Entity)null,
                    query,
                    false,
                    (pos, shape) -> {
                        var collision = new JsonObject();
                        collision.add("position", snapshot(pos));
                        collision.add("shape", snapshot(shape));
                        return collision;
                    });
                while (iterator.hasNext()) {
                    collisions.add(iterator.next());
                }
                result.add("collisions", collisions);
            } else if (kind.equals("entity_fit")) {
                JsonObject dimensionInput = request.getAsJsonObject("dimensions");
                float width = Oracle.f32(dimensionInput, "width");
                float height = Oracle.f32(dimensionInput, "height");
                boolean fixed = dimensionInput.has("fixed") && dimensionInput.get("fixed").getAsBoolean();
                EntityDimensions dimensions = fixed
                    ? EntityDimensions.fixed(width, height)
                    : EntityDimensions.scalable(width, height);
                if (dimensionInput.has("width_scale") || dimensionInput.has("height_scale")) {
                    float widthScale = dimensionInput.has("width_scale")
                        ? Oracle.f32(dimensionInput, "width_scale") : 1.0F;
                    float heightScale = dimensionInput.has("height_scale")
                        ? Oracle.f32(dimensionInput, "height_scale") : 1.0F;
                    dimensions = dimensions.scale(widthScale, heightScale);
                }
                if (dimensionInput.has("eye_height")) {
                    dimensions = dimensions.withEyeHeight(Oracle.f32(dimensionInput, "eye_height"));
                }
                Vec3 position = vec3(request.getAsJsonObject("position"));
                AABB boundingBox = dimensions.makeBoundingBox(position);
                AABB query = boundingBox.deflate(1.0E-7);
                CollisionGetter collisionGetter = collisionFixture(request.getAsJsonArray("placements"));
                var dimensionSnapshot = new JsonObject();
                dimensionSnapshot.addProperty("width", Oracle.bits(dimensions.width()));
                dimensionSnapshot.addProperty("height", Oracle.bits(dimensions.height()));
                dimensionSnapshot.addProperty("eye_height", Oracle.bits(dimensions.eyeHeight()));
                dimensionSnapshot.addProperty("fixed", dimensions.fixed());
                result.add("dimensions", dimensionSnapshot);
                result.add("box", snapshot(boundingBox));
                result.add("query", snapshot(query));
                result.addProperty("fits", collisionGetter.noCollision(null, query));
            } else if (kind.equals("living_entity_dimensions_scale")) {
                var states = new JsonArray();
                for (var entry : request.getAsJsonArray("inputs")) {
                    JsonObject input = entry.getAsJsonObject();
                    int requestedPoseId = input.get("pose_id").getAsInt();
                    Pose pose = Pose.BY_ID.apply(requestedPoseId);
                    JsonObject dimensionInput = input.getAsJsonObject("default_dimensions");
                    float width = Oracle.f32(dimensionInput, "width");
                    float height = Oracle.f32(dimensionInput, "height");
                    boolean fixed = dimensionInput.has("fixed") && dimensionInput.get("fixed").getAsBoolean();
                    EntityDimensions defaultDimensions = fixed
                        ? EntityDimensions.fixed(width, height)
                        : EntityDimensions.scalable(width, height);
                    if (dimensionInput.has("eye_height")) {
                        defaultDimensions = defaultDimensions.withEyeHeight(
                            Oracle.f32(dimensionInput, "eye_height"));
                    }
                    float ageScale = Oracle.f32(input, "age_scale");
                    float entityScale = Oracle.f32(input, "entity_scale");
                    EntityDimensions dimensions = pose == Pose.SLEEPING
                        ? EntityDimensions.fixed(0.2F, 0.2F).withEyeHeight(0.2F)
                        : defaultDimensions.scale(ageScale).scale(entityScale);
                    var state = new JsonObject();
                    state.addProperty("label", input.get("label").getAsString());
                    state.addProperty("requested_pose_id", requestedPoseId);
                    state.addProperty("resolved_pose_id", pose.id());
                    state.add("dimensions", snapshot(dimensions));
                    state.add("box", snapshot(dimensions.makeBoundingBox(
                        vec3(input.getAsJsonObject("position")))));
                    states.add(state);
                }
                result.add("states", states);
            } else if (kind.equals("player_pose_dimensions")) {
                var avatarDimensions = avatarPoseDimensions();
                Vec3 position = vec3(request.getAsJsonObject("position"));
                var poses = new JsonArray();
                for (var entry : request.getAsJsonArray("pose_ids")) {
                    int requestedId = entry.getAsInt();
                    Pose pose = Pose.BY_ID.apply(requestedId);
                    EntityDimensions dimensions = avatarDimensions.getOrDefault(
                        pose, avatarDimensions.get(Pose.STANDING));
                    var poseResult = new JsonObject();
                    poseResult.addProperty("requested_id", requestedId);
                    poseResult.addProperty("resolved_id", pose.id());
                    poseResult.add("dimensions", snapshot(dimensions));
                    poseResult.add("box", snapshot(dimensions.makeBoundingBox(position)));
                    poses.add(poseResult);
                }
                result.add("poses", poses);
            } else if (kind.equals("player_pose_selection")) {
                var selections = new JsonArray();
                for (var entry : request.getAsJsonArray("inputs")) {
                    var input = entry.getAsJsonObject();
                    boolean sleeping = input.get("sleeping").getAsBoolean();
                    boolean swimming = input.get("swimming").getAsBoolean();
                    boolean fallFlying = input.get("fall_flying").getAsBoolean();
                    boolean autoSpinAttack = input.get("auto_spin_attack").getAsBoolean();
                    boolean shiftKeyDown = input.get("shift_key_down").getAsBoolean();
                    boolean flying = input.get("flying").getAsBoolean();
                    boolean spectator = input.get("spectator").getAsBoolean();
                    boolean passenger = input.get("passenger").getAsBoolean();
                    boolean swimmingFits = input.get("swimming_fits").getAsBoolean();
                    boolean desiredFits = input.get("desired_fits").getAsBoolean();
                    boolean crouchingFits = input.get("crouching_fits").getAsBoolean();

                    Pose desiredPose;
                    if (sleeping) {
                        desiredPose = Pose.SLEEPING;
                    } else if (swimming) {
                        desiredPose = Pose.SWIMMING;
                    } else if (fallFlying) {
                        desiredPose = Pose.FALL_FLYING;
                    } else if (autoSpinAttack) {
                        desiredPose = Pose.SPIN_ATTACK;
                    } else {
                        desiredPose = shiftKeyDown && !flying ? Pose.CROUCHING : Pose.STANDING;
                    }

                    Pose actualPose = null;
                    if (swimmingFits) {
                        if (spectator || passenger || desiredFits) {
                            actualPose = desiredPose;
                        } else if (crouchingFits) {
                            actualPose = Pose.CROUCHING;
                        } else {
                            actualPose = Pose.SWIMMING;
                        }
                    }

                    var selection = new JsonObject();
                    selection.addProperty("label", input.get("label").getAsString());
                    selection.addProperty("desired_id", desiredPose.id());
                    selection.addProperty("actual_id", actualPose == null ? null : actualPose.id());
                    selections.add(selection);
                }
                result.add("selections", selections);
            } else if (kind.equals("clip")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                var events = new JsonArray();
                for (var entry : request.getAsJsonArray("queries")) {
                    JsonObject query = entry.getAsJsonObject();
                    events.add(snapshot(shape.clip(
                        vec3(query.getAsJsonObject("from")),
                        vec3(query.getAsJsonObject("to")),
                        blockPos(query.getAsJsonObject("pos")))));
                }
                result.add("events", events);
            } else if (kind.equals("clip_join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                VoxelShape shape = Shapes.join(first, second, BooleanOp.OR);
                var events = new JsonArray();
                for (var entry : request.getAsJsonArray("queries")) {
                    JsonObject query = entry.getAsJsonObject();
                    events.add(snapshot(shape.clip(
                        vec3(query.getAsJsonObject("from")),
                        vec3(query.getAsJsonObject("to")),
                        blockPos(query.getAsJsonObject("pos")))));
                }
                result.add("events", events);
            } else if (kind.equals("closest")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                var closest = shape.closestPointTo(vec3(request.getAsJsonObject("point")));
                result.add("point", closest.isPresent()
                    ? snapshot(closest.get()) : com.google.gson.JsonNull.INSTANCE);
            } else if (kind.equals("closest_join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                VoxelShape shape = Shapes.join(first, second, BooleanOp.OR);
                var closest = shape.closestPointTo(vec3(request.getAsJsonObject("point")));
                result.add("point", closest.isPresent()
                    ? snapshot(closest.get()) : com.google.gson.JsonNull.INSTANCE);
            } else if (kind.equals("face")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                Direction direction = Direction.byName(request.get("direction").getAsString());
                if (direction == null) {
                    throw new IllegalArgumentException("Unknown face direction");
                }
                result.add("shape", snapshot(shape.getFaceShape(direction)));
            } else if (kind.equals("face_join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                Direction direction = Direction.byName(request.get("direction").getAsString());
                if (direction == null) {
                    throw new IllegalArgumentException("Unknown face direction");
                }
                result.add("shape", snapshot(Shapes.or(first, second).getFaceShape(direction)));
            } else if (kind.equals("block_occludes")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
                Direction direction = Direction.byName(request.get("direction").getAsString());
                if (direction == null) {
                    throw new IllegalArgumentException("Unknown occlusion direction");
                }
                result.addProperty("occludes", Shapes.blockOccludes(shape, occluder, direction));
            } else if (kind.equals("block_occludes_join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
                Direction direction = Direction.byName(request.get("direction").getAsString());
                if (direction == null) {
                    throw new IllegalArgumentException("Unknown occlusion direction");
                }
                result.addProperty("occludes", Shapes.blockOccludes(Shapes.or(first, second), occluder, direction));
            } else if (kind.equals("merged_occludes")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
                Direction direction = Direction.byName(request.get("direction").getAsString());
                if (direction == null) {
                    throw new IllegalArgumentException("Unknown merged occlusion direction");
                }
                result.addProperty("occludes", Shapes.mergedFaceOccludes(shape, occluder, direction));
            } else if (kind.equals("face_occludes")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
                result.addProperty("occludes", Shapes.faceShapeOccludes(shape, occluder));
            } else {
                throw new IllegalArgumentException("Unknown VoxelShape kind");
            }
        } else if (operation.equals("random")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            var events = new JsonArray();
            events.add(new JsonObject());
            for (var entry : request.getAsJsonArray("steps")) {
                events.add(randomState(random, entry.getAsJsonObject()));
            }
            result.add("events", events);
        } else if (operation.equals("random_fork")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            result.add("prefix_events", randomSteps(random, request.getAsJsonArray("prefix_steps")));
            RandomSource child = random.fork();
            result.add("child_events", randomSteps(child, request.getAsJsonArray("child_steps")));
            result.add("parent_events", randomSteps(random, request.getAsJsonArray("parent_steps")));
        } else if (operation.equals("random_positional")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            PositionalRandomFactory factory = random.forkPositional();
            var factoryEvents = new JsonArray();
            for (var entry : request.getAsJsonArray("factory_steps")) {
                JsonObject step = entry.getAsJsonObject();
                RandomSource child = switch (step.get("op").getAsString()) {
                    case "at" -> factory.at(step.get("x").getAsInt(), step.get("y").getAsInt(),
                        step.get("z").getAsInt());
                    case "fromHashOf" -> factory.fromHashOf(step.get("name").getAsString());
                    case "fromSeed" -> factory.fromSeed(Long.parseLong(step.get("seed").getAsString()));
                    default -> throw new IllegalArgumentException("Unknown positional random op");
                };
                var state = new JsonObject();
                state.addProperty("op", step.get("op").getAsString());
                state.add("events", randomSteps(child, step.getAsJsonArray("steps")));
                factoryEvents.add(state);
            }
            result.add("factory_events", factoryEvents);
            result.add("parent_events", randomSteps(random, request.getAsJsonArray("parent_steps")));
        } else if (operation.equals("aabb_builder")) {
            AABB.Builder builder = new AABB.Builder();
            var events = new JsonArray();
            var initial = new JsonObject();
            initial.addProperty("defined", builder.isDefined());
            events.add(initial);
            for (var entry : request.getAsJsonArray("points")) {
                JsonObject point = entry.getAsJsonObject();
                builder.include(new Vector3f(
                    Oracle.f32(point, "x"),
                    Oracle.f32(point, "y"),
                    Oracle.f32(point, "z")
                ));
                var state = new JsonObject();
                state.addProperty("defined", builder.isDefined());
                events.add(state);
            }
            result.add("events", events);
            var build = new JsonObject();
            try {
                build.addProperty("outcome", "value");
                build.add("box", snapshot(builder.build()));
            } catch (IllegalStateException error) {
                build.addProperty("outcome", "error");
                build.addProperty("error", "undefined");
                result.addProperty("java_exception", error.getClass().getName());
            }
            result.add("build", build);
        } else if (operation.equals("aabb")) {
            AABB box = box(request);
            var events = new JsonArray();
            events.add(snapshot(box));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                switch (step.get("op").getAsString()) {
                    case "contains" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.contains(
                            f64(step, "x"), f64(step, "y"), f64(step, "z")));
                        events.add(state);
                    }
                    case "intersects" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(box(step.getAsJsonObject("box"))));
                        events.add(state);
                    }
                    case "contains_vec3" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.contains(vec3(step.getAsJsonObject("position"))));
                        events.add(state);
                    }
                    case "intersects_coords" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(
                            f64(step, "min_x"), f64(step, "min_y"), f64(step, "min_z"),
                            f64(step, "max_x"), f64(step, "max_y"), f64(step, "max_z")));
                        events.add(state);
                    }
                    case "intersects_vec3" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(
                            vec3(step.getAsJsonObject("min")),
                            vec3(step.getAsJsonObject("max"))));
                        events.add(state);
                    }
                    case "intersects_block_pos" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(
                            blockPos(step.getAsJsonObject("position"))));
                        events.add(state);
                    }
                    case "collided_along_vector" -> {
                        var boxes = new java.util.ArrayList<AABB>();
                        for (var boxEntry : step.getAsJsonArray("boxes")) {
                            boxes.add(box(boxEntry.getAsJsonObject()));
                        }
                        var state = new JsonObject();
                        state.addProperty("hit", box.collidedAlongVector(
                            vec3(step.getAsJsonObject("vector")), boxes));
                        events.add(state);
                    }
                    case "from_block_pos" -> {
                        box = new AABB(blockPos(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "from_vec3" -> {
                        box = new AABB(
                            vec3(step.getAsJsonObject("begin")),
                            vec3(step.getAsJsonObject("end")));
                        events.add(snapshot(box));
                    }
                    case "unit_cube_from_lower_corner" -> {
                        box = AABB.unitCubeFromLowerCorner(
                            vec3(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "encapsulating_full_blocks" -> {
                        box = AABB.encapsulatingFullBlocks(
                            blockPos(step.getAsJsonObject("first")),
                            blockPos(step.getAsJsonObject("second")));
                        events.add(snapshot(box));
                    }
                    case "of_bounding_box" -> {
                        JsonObject boundingBox = step.getAsJsonObject("box");
                        box = AABB.of(new BoundingBox(
                            Integer.parseInt(boundingBox.get("min_x").getAsString()),
                            Integer.parseInt(boundingBox.get("min_y").getAsString()),
                            Integer.parseInt(boundingBox.get("min_z").getAsString()),
                            Integer.parseInt(boundingBox.get("max_x").getAsString()),
                            Integer.parseInt(boundingBox.get("max_y").getAsString()),
                            Integer.parseInt(boundingBox.get("max_z").getAsString())));
                        events.add(snapshot(box));
                    }
                    case "expand_vec3" -> {
                        box = box.expandTowards(vec3(step.getAsJsonObject("delta")));
                        events.add(snapshot(box));
                    }
                    case "equals" -> {
                        var state = new JsonObject();
                        state.addProperty("equal", box.equals(box(step.getAsJsonObject("box"))));
                        events.add(state);
                    }
                    case "hash_code" -> {
                        var state = new JsonObject();
                        state.addProperty("hash_code", box.hashCode());
                        events.add(state);
                    }
                    case "to_string" -> {
                        var state = new JsonObject();
                        state.addProperty("value", box.toString());
                        events.add(state);
                    }
                    case "intersect" -> {
                        box = box.intersect(box(step.getAsJsonObject("box")));
                        events.add(snapshot(box));
                    }
                    case "minmax" -> {
                        box = box.minmax(box(step.getAsJsonObject("box")));
                        events.add(snapshot(box));
                    }
                    case "move" -> {
                        box = box.move(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "move_block_pos" -> {
                        box = box.move(blockPos(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "move_vec3" -> {
                        box = box.move(vec3(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "move_vector3f" -> {
                        JsonObject position = step.getAsJsonObject("position");
                        box = box.move(new Vector3f(
                            Oracle.f32(position, "x"),
                            Oracle.f32(position, "y"),
                            Oracle.f32(position, "z")));
                        events.add(snapshot(box));
                    }
                    case "sizes" -> {
                        var state = new JsonObject();
                        state.addProperty("x_size", Oracle.bits(box.getXsize()));
                        state.addProperty("y_size", Oracle.bits(box.getYsize()));
                        state.addProperty("z_size", Oracle.bits(box.getZsize()));
                        state.addProperty("size", Oracle.bits(box.getSize()));
                        events.add(state);
                    }
                    case "has_nan" -> {
                        var state = new JsonObject();
                        state.addProperty("has_nan", box.hasNaN());
                        events.add(state);
                    }
                    case "positions" -> {
                        var state = new JsonObject();
                        state.add("center", snapshot(box.getCenter()));
                        state.add("bottom_center", snapshot(box.getBottomCenter()));
                        state.add("min_position", snapshot(box.getMinPosition()));
                        state.add("max_position", snapshot(box.getMaxPosition()));
                        events.add(state);
                    }
                    case "min", "max" -> {
                        Direction.Axis axis = Direction.Axis.byName(step.get("axis").getAsString());
                        if (axis == null) {
                            throw new IllegalArgumentException("Unknown axis name");
                        }
                        var state = new JsonObject();
                        state.addProperty("value", Oracle.bits(step.get("op").getAsString().equals("min")
                            ? box.min(axis) : box.max(axis)));
                        events.add(state);
                    }
                    case "distance_point" -> {
                        var state = new JsonObject();
                        state.addProperty("value", Oracle.bits(box.distanceToSqr(
                            vec3(step.getAsJsonObject("point")))));
                        events.add(state);
                    }
                    case "distance_box" -> {
                        var state = new JsonObject();
                        state.addProperty("value", Oracle.bits(box.distanceToSqr(
                            box(step.getAsJsonObject("box")))));
                        events.add(state);
                    }
                    case "of_size" -> {
                        box = AABB.ofSize(
                            vec3(step.getAsJsonObject("center")),
                            f64(step, "size_x"),
                            f64(step, "size_y"),
                            f64(step, "size_z"));
                        events.add(snapshot(box));
                    }
                    case "set_min_x" -> {
                        box = box.setMinX(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_min_y" -> {
                        box = box.setMinY(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_min_z" -> {
                        box = box.setMinZ(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_max_x" -> {
                        box = box.setMaxX(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_max_y" -> {
                        box = box.setMaxY(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_max_z" -> {
                        box = box.setMaxZ(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "clip" -> {
                        var hit = box.clip(
                            vec3(step.getAsJsonObject("from")),
                            vec3(step.getAsJsonObject("to")));
                        var state = new JsonObject();
                        if (hit.isPresent()) {
                            state.addProperty("hit", true);
                            state.add("point", snapshotClip(hit.get()));
                        } else {
                            state.addProperty("hit", false);
                        }
                        events.add(state);
                    }
                    case "clip_coords" -> {
                        var hit = AABB.clip(
                            f64(step, "min_x"), f64(step, "min_y"), f64(step, "min_z"),
                            f64(step, "max_x"), f64(step, "max_y"), f64(step, "max_z"),
                            vec3(step.getAsJsonObject("from")),
                            vec3(step.getAsJsonObject("to")));
                        var state = new JsonObject();
                        if (hit.isPresent()) {
                            state.addProperty("hit", true);
                            state.add("point", snapshotClip(hit.get()));
                        } else {
                            state.addProperty("hit", false);
                        }
                        events.add(state);
                    }
                    case "clip_many" -> {
                        var boxes = new ArrayList<AABB>();
                        for (var boxEntry : step.getAsJsonArray("boxes")) {
                            boxes.add(box(boxEntry.getAsJsonObject()));
                        }
                        var hit = AABB.clip(
                            boxes,
                            vec3(step.getAsJsonObject("from")),
                            vec3(step.getAsJsonObject("to")),
                            blockPos(step.getAsJsonObject("pos")));
                        events.add(snapshot(hit));
                    }
                    case "expand" -> {
                        box = box.expandTowards(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "contract" -> {
                        box = box.contract(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "inflate" -> {
                        box = box.inflate(f64(step, "amount"));
                        events.add(snapshot(box));
                    }
                    case "inflate_xyz" -> {
                        box = box.inflate(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "deflate" -> {
                        box = box.deflate(f64(step, "amount"));
                        events.add(snapshot(box));
                    }
                    case "deflate_xyz" -> {
                        box = box.deflate(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    default -> throw new IllegalArgumentException("Unknown AABB op");
                }
            }
            result.add("events", events);
        } else {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                float value = Oracle.f32(request, "value");
                switch (operation) {
                    case "floor" -> result.addProperty("value", Integer.toString(Mth.floor(value)));
                    case "wrap_degrees" -> result.addProperty("bits", Oracle.bits(Mth.wrapDegrees(value)));
                    case "clamp" -> result.addProperty("bits", Oracle.bits(Mth.clamp(value,
                        Oracle.f32(request, "min"), Oracle.f32(request, "max"))));
                    default -> throw new IllegalArgumentException("Unknown math operation");
                }
            } else if (width.equals("f64")) {
                double value = f64(request, "value");
                switch (operation) {
                    case "floor" -> result.addProperty("value", Integer.toString(Mth.floor(value)));
                    case "wrap_degrees" -> result.addProperty("bits", Oracle.bits(Mth.wrapDegrees(value)));
                    case "clamp" -> result.addProperty("bits", Oracle.bits(Mth.clamp(value,
                        f64(request, "min"), f64(request, "max"))));
                    default -> throw new IllegalArgumentException("Unknown math operation");
                }
            } else {
                throw new IllegalArgumentException("Unknown floating-point type");
            }
        }
        return result;
    }

    private static double f64(JsonObject object, String name) {
        return f64(object.get(name).getAsString());
    }

    private static double f64(String bits) {
        return Double.longBitsToDouble(Long.parseUnsignedLong(bits, 16));
    }

    private static final class CollisionFixtureEntity extends Entity {
        private boolean spectator;
        private boolean collidable;
        private boolean sameVehicle;
        private boolean pushable;

        private CollisionFixtureEntity(
            AABB boundingBox, boolean spectator, boolean collidable, boolean sameVehicle
        ) {
            super(net.minecraft.world.entity.EntityTypes.ARMOR_STAND, null);
            this.spectator = spectator;
            this.collidable = collidable;
            this.sameVehicle = sameVehicle;
            this.setBoundingBox(boundingBox);
        }

        private static CollisionFixtureEntity create(
            AABB boundingBox, boolean spectator, boolean collidable, boolean sameVehicle
        ) {
            return create(boundingBox, spectator, collidable, sameVehicle, null, false);
        }

        private static CollisionFixtureEntity create(
            AABB boundingBox, boolean spectator, boolean collidable, boolean sameVehicle,
            Vec3 position
        ) {
            return create(boundingBox, spectator, collidable, sameVehicle, position, false);
        }

        private static CollisionFixtureEntity create(
            AABB boundingBox, boolean spectator, boolean collidable, boolean sameVehicle,
            Vec3 position, boolean pushable
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                CollisionFixtureEntity entity =
                    (CollisionFixtureEntity)unsafe.allocateInstance(CollisionFixtureEntity.class);
                entity.spectator = spectator;
                entity.collidable = collidable;
                entity.sameVehicle = sameVehicle;
                entity.pushable = pushable;
                entity.setBoundingBox(boundingBox);
                if (position != null) {
                    var field = Entity.class.getDeclaredField("position");
                    unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                }
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate collision fixture entity", error);
            }
        }

        private static Entity createIndexed(
            int entityId,
            int typeId,
            Vec3 position,
            boolean spectator,
            boolean alive,
            boolean onClimbable,
            boolean flying,
            boolean collidable,
            boolean sameVehicle,
            JsonObject climbableContext
        ) {
            var entityType = BuiltInRegistries.ENTITY_TYPE.byId(typeId);
            if (entityType == null) {
                throw new IllegalArgumentException("Unknown entity type id " + typeId);
            }
            AABB boundingBox = entityType.getDimensions().makeBoundingBox(position);
            Entity entity = switch (typeId) {
                case 0 -> BoatCollisionFixtureEntity.create(boundingBox, position);
                case 25, 29, 56, 65, 85, 122, 134
                    -> MinecartCollisionFixtureEntity.create(boundingBox, position);
                case 112 -> ShulkerCollisionFixtureEntity.create(boundingBox, sameVehicle, position);
                case 156 -> PlayerCollisionFixtureEntity.create(
                    boundingBox, position, spectator, alive, onClimbable, flying,
                    climbableContext);
                default -> create(boundingBox, spectator, collidable, sameVehicle, position, false);
            };
            indexEntity(entity, entityId, position);
            return entity;
        }

        private static Entity createHappyGhastIndexed(JsonObject input) {
            int entityId = input.get("entity_id").getAsInt();
            int typeId = input.get("type_id").getAsInt();
            Vec3 position = vec3(input.getAsJsonObject("position"));
            boolean baby = input.has("baby") && input.get("baby").getAsBoolean();
            boolean alive = !input.has("alive") || input.get("alive").getAsBoolean();
            boolean stillTimeout = input.has("still_timeout")
                && input.get("still_timeout").getAsBoolean();
            boolean vehicle = input.has("is_vehicle")
                && input.get("is_vehicle").getAsBoolean();
            boolean spectator = input.has("spectator")
                && input.get("spectator").getAsBoolean();
            boolean collidable = input.has("can_be_collided_with")
                && input.get("can_be_collided_with").getAsBoolean();
            boolean sameVehicle = input.has("same_vehicle")
                && input.get("same_vehicle").getAsBoolean();
            var entityType = BuiltInRegistries.ENTITY_TYPE.byId(typeId);
            if (entityType == null) {
                throw new IllegalArgumentException("Unknown entity type id " + typeId);
            }
            Entity entity;
            if (typeId == 58) {
                EntityDimensions dimensions = entityType.getDimensions();
                if (baby) {
                    dimensions = dimensions.scale(0.2375F).withEyeHeight(0.46875F);
                }
                entity = HappyGhastCollisionFixtureEntity.create(
                    dimensions.makeBoundingBox(position), position, baby, alive,
                    stillTimeout, vehicle);
            } else if (typeId == 156) {
                entity = PlayerCollisionFixtureEntity.create(
                    entityType.getDimensions().makeBoundingBox(position), position);
            } else if (typeId == 0) {
                entity = BoatCollisionFixtureEntity.create(
                    entityType.getDimensions().makeBoundingBox(position), position);
            } else if (typeId == 112) {
                entity = ShulkerCollisionFixtureEntity.create(
                    entityType.getDimensions().makeBoundingBox(position), sameVehicle, position);
            } else {
                entity = create(
                    entityType.getDimensions().makeBoundingBox(position), spectator,
                    collidable, sameVehicle, position, false);
            }
            indexEntity(entity, entityId, position);
            return entity;
        }

        private static void indexEntity(Entity entity, int entityId, Vec3 position) {
            try {
                Unsafe unsafe = unsafeInstance();
                var idField = Entity.class.getDeclaredField("id");
                unsafe.putInt(entity, unsafe.objectFieldOffset(idField), entityId);
                var uuidField = Entity.class.getDeclaredField("uuid");
                unsafe.putObject(
                    entity, unsafe.objectFieldOffset(uuidField),
                    new UUID(0L, Integer.toUnsignedLong(entityId)));
                var passengersField = Entity.class.getDeclaredField("passengers");
                unsafe.putObject(
                    entity, unsafe.objectFieldOffset(passengersField),
                    com.google.common.collect.ImmutableList.of());
                BlockPos blockPosition = BlockPos.containing(position.x, position.y, position.z);
                var blockPositionField = Entity.class.getDeclaredField("blockPosition");
                unsafe.putObject(
                    entity, unsafe.objectFieldOffset(blockPositionField), blockPosition);
                var chunkPositionField = Entity.class.getDeclaredField("chunkPosition");
                unsafe.putObject(
                    entity, unsafe.objectFieldOffset(chunkPositionField),
                    ChunkPos.containing(blockPosition));
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to index collision fixture entity", error);
            }
        }

        @Override
        protected void defineSynchedData(net.minecraft.network.syncher.SynchedEntityData.Builder builder) {
        }

        @Override
        public boolean isSpectator() {
            return this.spectator;
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }

        @Override
        public boolean canBeCollidedWith(Entity other) {
            return this.collidable;
        }

        @Override
        public boolean isPushable() {
            return this.pushable;
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return other instanceof CollisionFixtureEntity fixture && fixture.sameVehicle;
        }

        @Override
        public boolean hurtServer(
            net.minecraft.server.level.ServerLevel level,
            net.minecraft.world.damagesource.DamageSource source,
            float damage
        ) {
            return false;
        }

        @Override
        protected void readAdditionalSaveData(net.minecraft.world.level.storage.ValueInput input) {
        }

        @Override
        protected void addAdditionalSaveData(net.minecraft.world.level.storage.ValueOutput output) {
        }
    }

    private static final class BoatCollisionFixtureEntity extends Boat {
        private BoatCollisionFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.OAK_BOAT, null, null);
        }

        private static BoatCollisionFixtureEntity create(AABB boundingBox, Vec3 position) {
            try {
                Unsafe unsafe = unsafeInstance();
                BoatCollisionFixtureEntity entity =
                    (BoatCollisionFixtureEntity)unsafe.allocateInstance(BoatCollisionFixtureEntity.class);
                entity.setBoundingBox(boundingBox);
                var field = Entity.class.getDeclaredField("position");
                unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate boat collision fixture entity", error);
            }
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return false;
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }
    }

    private static final class MinecartCollisionFixtureEntity extends Minecart {
        private static final net.minecraft.world.level.Level CLIENT_LEVEL = createClientLevel();

        private MinecartCollisionFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.MINECART, null);
        }

        private static net.minecraft.world.level.Level createClientLevel() {
            try {
                return (net.minecraft.world.level.Level)unsafeInstance()
                    .allocateInstance(HappyGhastClientLevelFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException("Unable to allocate minecart client level fixture", error);
            }
        }

        private static MinecartCollisionFixtureEntity create(AABB boundingBox, Vec3 position) {
            try {
                Unsafe unsafe = unsafeInstance();
                MinecartCollisionFixtureEntity entity =
                    (MinecartCollisionFixtureEntity)unsafe.allocateInstance(MinecartCollisionFixtureEntity.class);
                var levelField = Entity.class.getDeclaredField("level");
                unsafe.putObject(entity, unsafe.objectFieldOffset(levelField), CLIENT_LEVEL);
                entity.setBoundingBox(boundingBox);
                var field = Entity.class.getDeclaredField("position");
                unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate minecart collision fixture entity", error);
            }
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return false;
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }
    }

    private static final class ShulkerCollisionFixtureEntity extends Shulker {
        private boolean sameVehicle;

        private ShulkerCollisionFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.SHULKER, null);
        }

        private static ShulkerCollisionFixtureEntity create(
            AABB boundingBox, boolean sameVehicle, Vec3 position
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                ShulkerCollisionFixtureEntity entity =
                    (ShulkerCollisionFixtureEntity)unsafe.allocateInstance(ShulkerCollisionFixtureEntity.class);
                entity.sameVehicle = sameVehicle;
                entity.setBoundingBox(boundingBox);
                var field = Entity.class.getDeclaredField("position");
                unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate shulker collision fixture entity", error);
            }
        }

        @Override
        public boolean isAlive() {
            return true;
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return other instanceof ShulkerCollisionFixtureEntity fixture && fixture.sameVehicle;
        }

        @Override
        protected void defineSynchedData(net.minecraft.network.syncher.SynchedEntityData.Builder builder) {
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }

        @Override
        public boolean hurtServer(
            net.minecraft.server.level.ServerLevel level,
            net.minecraft.world.damagesource.DamageSource source,
            float damage
        ) {
            return false;
        }

        @Override
        protected void readAdditionalSaveData(net.minecraft.world.level.storage.ValueInput input) {
        }

        @Override
        protected void addAdditionalSaveData(net.minecraft.world.level.storage.ValueOutput output) {
        }
    }

    private static final class HappyGhastClientLevelFixture extends net.minecraft.client.multiplayer.ClientLevel {
        private HappyGhastClientLevelFixture() {
            super(null, null, null, null, 0, 0, null, false, 0L, 0);
        }

        @Override
        public boolean isClientSide() {
            return true;
        }

        @Override
        public net.minecraft.world.flag.FeatureFlagSet enabledFeatures() {
            return net.minecraft.world.flag.FeatureFlagSet.of();
        }
    }

    private static final class HappyGhastCollisionFixtureEntity
        extends net.minecraft.world.entity.animal.happyghast.HappyGhast {
        private static final net.minecraft.world.level.Level CLIENT_LEVEL = createClientLevel();
        private boolean baby;
        private boolean alive;
        private boolean stillTimeout;
        private boolean vehicle;

        private HappyGhastCollisionFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.HAPPY_GHAST, null);
            this.baby = false;
            this.alive = true;
            this.stillTimeout = false;
            this.vehicle = false;
        }

        private static net.minecraft.world.level.Level createClientLevel() {
            try {
                return (net.minecraft.world.level.Level)unsafeInstance()
                    .allocateInstance(HappyGhastClientLevelFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException("Unable to allocate client level fixture", error);
            }
        }

        private static HappyGhastCollisionFixtureEntity create(
            AABB boundingBox, Vec3 position, boolean baby, boolean alive,
            boolean stillTimeout, boolean vehicle
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                HappyGhastCollisionFixtureEntity entity =
                    (HappyGhastCollisionFixtureEntity)unsafe.allocateInstance(
                        HappyGhastCollisionFixtureEntity.class);
                entity.baby = baby;
                entity.alive = alive;
                entity.stillTimeout = stillTimeout;
                entity.vehicle = vehicle;
                entity.setBoundingBox(boundingBox);
                var field = Entity.class.getDeclaredField("position");
                unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate happy ghast collision fixture entity", error);
            }
        }

        @Override
        public int getAge() {
            return this.baby ? -1 : 1;
        }

        @Override
        public boolean isAlive() {
            return this.alive;
        }

        @Override
        public boolean isOnStillTimeout() {
            return this.stillTimeout;
        }

        @Override
        public boolean isVehicle() {
            return this.vehicle;
        }

        @Override
        public boolean isPushable() {
            return this.alive;
        }

        @Override
        public net.minecraft.world.level.Level level() {
            return CLIENT_LEVEL;
        }

        @Override
        public net.minecraft.world.item.ItemStack getMainHandItem() {
            return net.minecraft.world.item.ItemStack.EMPTY;
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return false;
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }
    }

    private static final class PlayerCollisionFixtureEntity
        extends net.minecraft.server.level.ServerPlayer {
        private static final net.minecraft.server.level.ServerLevel CLIMBABLE_LEVEL =
            ClimbableClientLevelFixture.createClientLevel();
        private boolean spectator;
        private boolean alive;
        private boolean onClimbable;
        private boolean flying;
        private boolean fallFlying;
        private boolean useClimbableContext;
        private BlockState inBlockState;

        private PlayerCollisionFixtureEntity() {
            super(null, null, null, null);
        }

        private static PlayerCollisionFixtureEntity create(AABB boundingBox, Vec3 position) {
            return create(boundingBox, position, false, true, false, false, null);
        }

        private static PlayerCollisionFixtureEntity create(
            AABB boundingBox, Vec3 position, boolean spectator, boolean alive,
            boolean onClimbable, boolean flying
        ) {
            return create(boundingBox, position, spectator, alive, onClimbable, flying, null);
        }

        private static PlayerCollisionFixtureEntity create(
            AABB boundingBox, Vec3 position, boolean spectator, boolean alive,
            boolean onClimbable, boolean flying, JsonObject climbableContext
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                PlayerCollisionFixtureEntity entity =
                    (PlayerCollisionFixtureEntity)unsafe.allocateInstance(
                        PlayerCollisionFixtureEntity.class);
                entity.spectator = spectator;
                entity.alive = alive;
                entity.onClimbable = onClimbable;
                entity.flying = flying;
                entity.fallFlying = climbableContext != null
                    && climbableContext.has("fall_flying")
                    && climbableContext.get("fall_flying").getAsBoolean();
                entity.useClimbableContext = climbableContext != null;
                entity.setBoundingBox(boundingBox);
                var field = Entity.class.getDeclaredField("position");
                unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                if (climbableContext != null) {
                    entity.inBlockState = createClimbableBlockState(climbableContext);
                    var abilitiesField = net.minecraft.world.entity.player.Player.class
                        .getDeclaredField("abilities");
                    var abilities = new net.minecraft.world.entity.player.Abilities();
                    abilities.flying = flying;
                    unsafe.putObject(entity, unsafe.objectFieldOffset(abilitiesField), abilities);
                }
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate player collision fixture entity", error);
            }
        }

        private static BlockState createClimbableBlockState(JsonObject context) {
            String kind = context.get("kind").getAsString();
            if (kind.equals("trapdoor_ladder")) {
                boolean open = context.has("trapdoor_open")
                    && context.get("trapdoor_open").getAsBoolean();
                Direction trapdoorFacing = Direction.valueOf(
                    context.get("trapdoor_facing").getAsString().toUpperCase(Locale.ROOT));
                Direction ladderFacing = Direction.valueOf(
                    context.get("ladder_facing").getAsString().toUpperCase(Locale.ROOT));
                boolean belowIsLadder = !context.has("below_is_ladder")
                    || context.get("below_is_ladder").getAsBoolean();
                BlockState belowState = belowIsLadder
                    ? Blocks.LADDER.defaultBlockState().setValue(LadderBlock.FACING, ladderFacing)
                    : Blocks.AIR.defaultBlockState();
                ClimbableClientLevelFixture.setBelowState(belowState);
                return Blocks.OAK_TRAPDOOR.defaultBlockState()
                    .setValue(TrapDoorBlock.OPEN, open)
                    .setValue(TrapDoorBlock.FACING, trapdoorFacing);
            } else if (kind.equals("climbable_tag")) {
                int stateId = context.get("state_id").getAsInt();
                BlockState state = Block.stateById(stateId);
                bindClimbableTag(state.getBlock());
                ClimbableClientLevelFixture.setBelowState(Blocks.AIR.defaultBlockState());
                return state;
            } else if (kind.equals("glide_through_tag")) {
                int stateId = context.get("state_id").getAsInt();
                BlockState state = Block.stateById(stateId);
                bindClimbableAndGlideThroughTags(state.getBlock());
                ClimbableClientLevelFixture.setBelowState(Blocks.AIR.defaultBlockState());
                return state;
            } else if (kind.equals("update_tags_packet")) {
                int stateId = context.get("state_id").getAsInt();
                BlockState state = Block.stateById(stateId);
                applyUpdateTagsPacket(context);
                ClimbableClientLevelFixture.setBelowState(Blocks.AIR.defaultBlockState());
                return state;
            } else {
                throw new IllegalArgumentException("Unknown climbable context kind");
            }
        }

        private static void applyUpdateTagsPacket(JsonObject context) {
            java.util.Map<net.minecraft.resources.Identifier, it.unimi.dsi.fastutil.ints.IntList> tags =
                new java.util.HashMap<>();
            JsonObject blockTags = context.getAsJsonObject("block_tags");
            for (var entry : blockTags.entrySet()) {
                it.unimi.dsi.fastutil.ints.IntList values = new it.unimi.dsi.fastutil.ints.IntArrayList();
                for (JsonElement value : entry.getValue().getAsJsonArray()) {
                    values.add(value.getAsInt());
                }
                tags.put(net.minecraft.resources.Identifier.parse(entry.getKey()), values);
            }
            var networkPayload = new net.minecraft.tags.TagNetworkSerialization.NetworkPayload(tags);
            java.util.Map<net.minecraft.resources.ResourceKey<? extends net.minecraft.core.Registry<?>>,
                net.minecraft.tags.TagNetworkSerialization.NetworkPayload> registries =
                new java.util.HashMap<>();
            registries.put(BuiltInRegistries.BLOCK.key(), networkPayload);
            var packet = new net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket(registries);
            var blockPayload = packet.getTags().get(BuiltInRegistries.BLOCK.key());
            BuiltInRegistries.BLOCK.prepareTagReload(
                blockPayload.resolve(BuiltInRegistries.BLOCK)).apply();
        }

        private static void bindClimbableAndGlideThroughTags(Block block) {
            java.util.List<net.minecraft.core.Holder<Block>> climbableValues = block == Blocks.VINE
                ? java.util.List.of(BuiltInRegistries.BLOCK.wrapAsHolder(block))
                : java.util.List.of();
            java.util.List<net.minecraft.core.Holder<Block>> glideThroughValues = block == Blocks.VINE
                ? java.util.List.of(BuiltInRegistries.BLOCK.wrapAsHolder(block))
                : java.util.List.of();
            java.util.Map<net.minecraft.tags.TagKey<Block>, java.util.List<net.minecraft.core.Holder<Block>>> tags =
                java.util.Map.of(
                    net.minecraft.tags.BlockTags.CLIMBABLE, climbableValues,
                    net.minecraft.tags.BlockTags.CAN_GLIDE_THROUGH, glideThroughValues);
            BuiltInRegistries.BLOCK.prepareTagReload(
                new net.minecraft.tags.TagLoader.LoadResult<>(
                    net.minecraft.core.registries.Registries.BLOCK, tags)).apply();
        }

        private static void bindClimbableTag(Block block) {
            java.util.List<net.minecraft.core.Holder<Block>> values = block == Blocks.LADDER
                ? java.util.List.of(BuiltInRegistries.BLOCK.wrapAsHolder(block))
                : java.util.List.of();
            java.util.Map<net.minecraft.tags.TagKey<Block>, java.util.List<net.minecraft.core.Holder<Block>>> tags =
                java.util.Map.of(net.minecraft.tags.BlockTags.CLIMBABLE, values);
            BuiltInRegistries.BLOCK.prepareTagReload(
                new net.minecraft.tags.TagLoader.LoadResult<>(
                    net.minecraft.core.registries.Registries.BLOCK, tags)).apply();
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return false;
        }

        @Override
        public boolean isAlive() {
            return this.alive;
        }

        @Override
        public boolean isSpectator() {
            return this.spectator;
        }

        @Override
        public boolean onClimbable() {
            return this.useClimbableContext
                ? super.onClimbable()
                : this.flying ? false : this.onClimbable;
        }

        @Override
        public BlockState getInBlockState() {
            return this.useClimbableContext
                ? this.inBlockState
                : Blocks.AIR.defaultBlockState();
        }

        @Override
        public boolean isFallFlying() {
            return this.fallFlying;
        }

        @Override
        public net.minecraft.server.level.ServerLevel level() {
            return CLIMBABLE_LEVEL;
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }

        @Override
        public net.minecraft.world.item.ItemStack getMainHandItem() {
            return net.minecraft.world.item.ItemStack.EMPTY;
        }
    }

    private static final class ClimbableClientLevelFixture
        extends net.minecraft.server.level.ServerLevel {
        private static BlockState belowState;

        private ClimbableClientLevelFixture() {
            super(null, null, null, null, null, null, false, 0L, null, false);
        }

        private static net.minecraft.server.level.ServerLevel createClientLevel() {
            try {
                return (net.minecraft.server.level.ServerLevel)unsafeInstance()
                    .allocateInstance(ClimbableClientLevelFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException("Unable to allocate climbable level fixture", error);
            }
        }

        private static void setBelowState(BlockState state) {
            belowState = state;
        }

        @Override
        public BlockState getBlockState(BlockPos pos) {
            return pos.getY() == 0 && belowState != null
                ? belowState
                : Blocks.AIR.defaultBlockState();
        }
    }

    private static final class NoopEntityCallbacks implements LevelCallback<Entity> {
        @Override
        public void onCreated(Entity entity) {
        }

        @Override
        public void onDestroyed(Entity entity) {
        }

        @Override
        public void onTickingStart(Entity entity) {
        }

        @Override
        public void onTickingEnd(Entity entity) {
        }

        @Override
        public void onTrackingStart(Entity entity) {
        }

        @Override
        public void onTrackingEnd(Entity entity) {
        }

        @Override
        public void onSectionChange(Entity entity) {
        }
    }

    private static final class EntityCollisionFixture implements EntityGetter {
        private final java.util.List<Entity> entities;

        private EntityCollisionFixture(java.util.List<Entity> entities) {
            this.entities = entities;
        }

        @Override
        public java.util.List<Entity> getEntities(
            Entity except, AABB box, java.util.function.Predicate<? super Entity> selector
        ) {
            var result = new java.util.ArrayList<Entity>();
            for (Entity entity : this.entities) {
                if (entity != except && entity.getBoundingBox().intersects(box) && selector.test(entity)) {
                    result.add(entity);
                }
            }
            return result;
        }

        @Override
        public <T extends Entity> java.util.List<T> getEntities(
            net.minecraft.world.level.entity.EntityTypeTest<Entity, T> type,
            AABB box,
            java.util.function.Predicate<? super T> selector
        ) {
            return java.util.List.of();
        }

        @Override
        public java.util.List<? extends net.minecraft.world.entity.player.Player> players() {
            return java.util.List.of();
        }
    }

    private static final class IndexedEntityCollisionFixture implements EntityGetter {
        private final LevelEntityGetter<Entity> entityGetter;

        private IndexedEntityCollisionFixture(LevelEntityGetter<Entity> entityGetter) {
            this.entityGetter = entityGetter;
        }

        @Override
        public java.util.List<Entity> getEntities(
            Entity except, AABB box, java.util.function.Predicate<? super Entity> selector
        ) {
            var result = new java.util.ArrayList<Entity>();
            this.entityGetter.get(box, entity -> {
                if (entity != except && selector.test(entity)) {
                    result.add(entity);
                }
            });
            return result;
        }

        @Override
        public <T extends Entity> java.util.List<T> getEntities(
            net.minecraft.world.level.entity.EntityTypeTest<Entity, T> type,
            AABB box,
            java.util.function.Predicate<? super T> selector
        ) {
            return java.util.List.of();
        }

        @Override
        public java.util.List<? extends net.minecraft.world.entity.player.Player> players() {
            return java.util.List.of();
        }
    }

    private static CollisionGetter collisionFixture(JsonArray placements) {
        return collisionFixture(placements, java.util.List.of(), null);
    }

    private static CollisionGetter collisionFixture(
        JsonArray placements, java.util.List<Entity> entities, WorldBorder worldBorder
    ) {
        return collisionFixture(
            placements, new EntityCollisionFixture(entities), worldBorder);
    }

    private static CollisionGetter collisionFixture(
        JsonArray placements, EntityGetter entityGetter, WorldBorder worldBorder
    ) {
        var states = new java.util.HashMap<String, BlockState>();
        for (var entry : placements) {
            JsonObject placement = entry.getAsJsonObject();
            int x = placement.get("x").getAsInt();
            int y = placement.get("y").getAsInt();
            int z = placement.get("z").getAsInt();
            BlockState state = switch (placement.get("kind").getAsString()) {
                case "air" -> Blocks.AIR.defaultBlockState();
                case "stone" -> Blocks.STONE.defaultBlockState();
                case "slab_bottom" -> Blocks.OAK_SLAB.defaultBlockState()
                    .setValue(SlabBlock.TYPE, SlabType.BOTTOM);
                case "slab_top" -> Blocks.OAK_SLAB.defaultBlockState()
                    .setValue(SlabBlock.TYPE, SlabType.TOP);
                case "slab_double" -> Blocks.OAK_SLAB.defaultBlockState()
                    .setValue(SlabBlock.TYPE, SlabType.DOUBLE);
                default -> throw new IllegalArgumentException("Unknown collision fixture block");
            };
            states.put(x + "," + y + "," + z, state);
        }
        return new CollisionGetter() {
            private String key(BlockPos pos) {
                return pos.getX() + "," + pos.getY() + "," + pos.getZ();
            }

            @Override
            public WorldBorder getWorldBorder() {
                return worldBorder;
            }

            @Override
            public BlockGetter getChunkForCollisions(int chunkX, int chunkZ) {
                return this;
            }

            @Override
            public java.util.List<VoxelShape> getEntityCollisions(Entity source, AABB testArea) {
                return entityGetter.getEntityCollisions(source, testArea);
            }

            @Override
            public BlockEntity getBlockEntity(BlockPos pos) {
                return null;
            }

            @Override
            public BlockState getBlockState(BlockPos pos) {
                return states.getOrDefault(key(pos), Blocks.AIR.defaultBlockState());
            }

            @Override
            public FluidState getFluidState(BlockPos pos) {
                return getBlockState(pos).getFluidState();
            }

            @Override
            public int getHeight() {
                return 384;
            }

            @Override
            public int getMinY() {
                return -64;
            }
        };
    }

    private static CollisionGetter registryCollisionFixture(JsonArray placements) {
        return registryCollisionFixture(registryStates(placements));
    }

    private static java.util.Map<String, BlockState> registryStates(JsonArray placements) {
        var states = new java.util.HashMap<String, BlockState>();
        for (var entry : placements) {
            JsonObject placement = entry.getAsJsonObject();
            int x = placement.get("x").getAsInt();
            int y = placement.get("y").getAsInt();
            int z = placement.get("z").getAsInt();
            states.put(
                x + "," + y + "," + z,
                Block.stateById(placement.get("state_id").getAsInt()));
        }
        return states;
    }

    private static String registryKey(BlockPos pos) {
        return pos.getX() + "," + pos.getY() + "," + pos.getZ();
    }

    private static CollisionGetter registryCollisionFixture(
        java.util.Map<String, BlockState> states
    ) {
        return new CollisionGetter() {
            private String key(BlockPos pos) {
                return registryKey(pos);
            }

            @Override
            public WorldBorder getWorldBorder() {
                return null;
            }

            @Override
            public BlockGetter getChunkForCollisions(int chunkX, int chunkZ) {
                return this;
            }

            @Override
            public java.util.List<VoxelShape> getEntityCollisions(Entity source, AABB testArea) {
                return java.util.List.of();
            }

            @Override
            public BlockEntity getBlockEntity(BlockPos pos) {
                return null;
            }

            @Override
            public BlockState getBlockState(BlockPos pos) {
                return states.getOrDefault(key(pos), Blocks.AIR.defaultBlockState());
            }

            @Override
            public FluidState getFluidState(BlockPos pos) {
                return getBlockState(pos).getFluidState();
            }

            @Override
            public int getHeight() {
                return 384;
            }

            @Override
            public int getMinY() {
                return -64;
            }
        };
    }

    private static StairsShape stairShape(
        BlockState state, BlockGetter level, BlockPos pos
    ) {
        try {
            var method = StairBlock.class.getDeclaredMethod(
                "getStairsShape", BlockState.class, BlockGetter.class, BlockPos.class);
            method.setAccessible(true);
            return (StairsShape) method.invoke(null, state, level, pos);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("StairBlock.getStairsShape reflection failed", error);
        }
    }

    @SuppressWarnings("unchecked")
    private static java.util.Map<Pose, EntityDimensions> avatarPoseDimensions() {
        try {
            var field = Avatar.class.getDeclaredField("POSES");
            field.setAccessible(true);
            return (java.util.Map<Pose, EntityDimensions>) field.get(null);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("Avatar.POSES reflection failed", error);
        }
    }

    private static AABB box(JsonObject object) {
        return new AABB(f64(object, "min_x"), f64(object, "min_y"), f64(object, "min_z"),
            f64(object, "max_x"), f64(object, "max_y"), f64(object, "max_z"));
    }

    private static VoxelShape voxelShape(JsonObject object) {
        return Shapes.box(f64(object, "min_x"), f64(object, "min_y"), f64(object, "min_z"),
            f64(object, "max_x"), f64(object, "max_y"), f64(object, "max_z"));
    }

    private static BlockPos blockPos(JsonObject object) {
        return new BlockPos(
            Integer.parseInt(object.get("x").getAsString()),
            Integer.parseInt(object.get("y").getAsString()),
            Integer.parseInt(object.get("z").getAsString())
        );
    }

    private static Direction directionOrElse(JsonObject object) {
        String name = object.get("or_else").getAsString();
        if (name.equals("none")) {
            return null;
        }
        Direction direction = Direction.byName(name);
        if (direction == null) {
            throw new IllegalArgumentException("Unknown fallback direction");
        }
        return direction;
    }

    private static Vec3 vec3(JsonObject object) {
        return new Vec3(f64(object, "x"), f64(object, "y"), f64(object, "z"));
    }

    private static ChunkPos chunkPos(JsonObject object) {
        return new ChunkPos(
            Integer.parseInt(object.get("x").getAsString()),
            Integer.parseInt(object.get("z").getAsString())
        );
    }

    private static JsonObject chunkPosBlockCoordinates(ChunkPos pos, int offset) {
        var state = new JsonObject();
        state.addProperty("min_block_x", Integer.toString(pos.getMinBlockX()));
        state.addProperty("min_block_z", Integer.toString(pos.getMinBlockZ()));
        state.addProperty("middle_block_x", Integer.toString(pos.getMiddleBlockX()));
        state.addProperty("middle_block_z", Integer.toString(pos.getMiddleBlockZ()));
        state.addProperty("max_block_x", Integer.toString(pos.getMaxBlockX()));
        state.addProperty("max_block_z", Integer.toString(pos.getMaxBlockZ()));
        state.addProperty("block_x", Integer.toString(pos.getBlockX(offset)));
        state.addProperty("block_z", Integer.toString(pos.getBlockZ(offset)));
        return state;
    }

    private static JsonObject chunkPosRegionCoordinates(ChunkPos pos, int regionX, int regionZ) {
        var state = new JsonObject();
        state.addProperty("region_x", Integer.toString(pos.getRegionX()));
        state.addProperty("region_z", Integer.toString(pos.getRegionZ()));
        state.addProperty("packed_region_x", Integer.toString(ChunkPos.getRegionX(pos.pack())));
        state.addProperty("packed_region_z", Integer.toString(ChunkPos.getRegionZ(pos.pack())));
        state.addProperty("region_local_x", Integer.toString(pos.getRegionLocalX()));
        state.addProperty("region_local_z", Integer.toString(pos.getRegionLocalZ()));
        state.add("min_from_region", snapshot(ChunkPos.minFromRegion(regionX, regionZ)));
        state.add("max_from_region", snapshot(ChunkPos.maxFromRegion(regionX, regionZ)));
        return state;
    }

    private static JsonObject chunkPosBlockPositions(ChunkPos pos, int blockX, int y, int blockZ, JsonArray queries) {
        var state = new JsonObject();
        state.add("block_at", snapshot(pos.getBlockAt(blockX, y, blockZ)));
        state.add("middle_block_position", snapshot(pos.getMiddleBlockPosition(y)));
        state.add("world_position", snapshot(pos.getWorldPosition()));
        var contains = new JsonArray();
        for (var entry : queries) {
            contains.add(pos.contains(blockPos(entry.getAsJsonObject())));
        }
        state.add("contains", contains);
        return state;
    }

    private static JsonObject chunkPosRepresentation(ChunkPos pos, int hashX, int hashZ) {
        var state = new JsonObject();
        state.addProperty("to_string", pos.toString());
        state.addProperty("hash_code", Integer.toString(pos.hashCode()));
        state.addProperty("static_hash", Integer.toString(ChunkPos.hash(hashX, hashZ)));
        return state;
    }

    private static JsonObject chunkPosFromBlockPos(BlockPos pos) {
        var state = new JsonObject();
        state.add("containing", snapshot(ChunkPos.containing(pos)));
        state.addProperty("packed", Long.toUnsignedString(ChunkPos.pack(pos), 16));
        return state;
    }

    private static JsonObject chunkPosDistances(ChunkPos pos, int otherX, int otherZ) {
        ChunkPos other = new ChunkPos(otherX, otherZ);
        var state = new JsonObject();
        state.addProperty("chessboard_chunk", Integer.toString(pos.getChessboardDistance(other)));
        state.addProperty("chessboard_coordinates", Integer.toString(pos.getChessboardDistance(otherX, otherZ)));
        state.addProperty("distance_squared_chunk", Integer.toString(pos.distanceSquared(other)));
        state.addProperty("distance_squared_packed", Integer.toString(pos.distanceSquared(other.pack())));
        return state;
    }

    private static JsonObject chunkPosRangeClosed(JsonObject request) {
        Stream<ChunkPos> positions;
        if (request.get("mode").getAsString().equals("center-radius")) {
            ChunkPos center = chunkPos(request);
            int radius = Integer.parseInt(request.get("radius").getAsString());
            positions = ChunkPos.rangeClosed(center, radius);
        } else if (request.get("mode").getAsString().equals("from-to")) {
            ChunkPos from = new ChunkPos(
                Integer.parseInt(request.get("from_x").getAsString()),
                Integer.parseInt(request.get("from_z").getAsString())
            );
            ChunkPos to = new ChunkPos(
                Integer.parseInt(request.get("to_x").getAsString()),
                Integer.parseInt(request.get("to_z").getAsString())
            );
            positions = ChunkPos.rangeClosed(from, to);
        } else {
            throw new IllegalArgumentException("Unknown ChunkPos range mode");
        }
        var values = new JsonArray();
        positions.forEach(pos -> values.add(snapshot(pos)));
        var state = new JsonObject();
        state.add("positions", values);
        return state;
    }

    private static JsonObject sectionPosRanges(JsonObject request) {
        Stream<SectionPos> positions;
        switch (request.get("mode").getAsString()) {
            case "between" -> positions = SectionPos.betweenClosedStream(
                Integer.parseInt(request.get("min_x").getAsString()),
                Integer.parseInt(request.get("min_y").getAsString()),
                Integer.parseInt(request.get("min_z").getAsString()),
                Integer.parseInt(request.get("max_x").getAsString()),
                Integer.parseInt(request.get("max_y").getAsString()),
                Integer.parseInt(request.get("max_z").getAsString()));
            case "cube" -> positions = SectionPos.cube(
                SectionPos.of(
                    Integer.parseInt(request.get("x").getAsString()),
                    Integer.parseInt(request.get("y").getAsString()),
                    Integer.parseInt(request.get("z").getAsString())),
                Integer.parseInt(request.get("radius").getAsString()));
            case "around-chunk" -> positions = SectionPos.aroundChunk(
                new ChunkPos(
                    Integer.parseInt(request.get("chunk_x").getAsString()),
                    Integer.parseInt(request.get("chunk_z").getAsString())),
                Integer.parseInt(request.get("radius").getAsString()),
                Integer.parseInt(request.get("min_section").getAsString()),
                Integer.parseInt(request.get("max_section").getAsString()));
            default -> throw new IllegalArgumentException("Unknown SectionPos range mode");
        }
        var values = new JsonArray();
        positions.forEach(pos -> values.add(snapshot(pos)));
        var state = new JsonObject();
        state.add("positions", values);
        return state;
    }

    private static JsonObject sectionPosBlocksInside(JsonObject request) {
        SectionPos pos = SectionPos.of(
            Integer.parseInt(request.get("x").getAsString()),
            Integer.parseInt(request.get("y").getAsString()),
            Integer.parseInt(request.get("z").getAsString()));
        var values = new JsonArray();
        pos.blocksInside().forEach(block -> values.add(snapshot(block)));
        var state = new JsonObject();
        state.add("positions", values);
        return state;
    }

    private static JsonObject sectionPosAroundAndAtBlockPos(JsonObject request) {
        var values = new JsonArray();
        LongConsumer sectionConsumer = sectionNode -> values.add(Long.toUnsignedString(sectionNode, 16));
        switch (request.get("mode").getAsString()) {
            case "coordinates" -> SectionPos.aroundAndAtBlockPos(
                Integer.parseInt(request.get("x").getAsString()),
                Integer.parseInt(request.get("y").getAsString()),
                Integer.parseInt(request.get("z").getAsString()),
                sectionConsumer);
            case "blockpos" -> SectionPos.aroundAndAtBlockPos(blockPos(request), sectionConsumer);
            case "packed" -> SectionPos.aroundAndAtBlockPos(
                Long.parseUnsignedLong(request.get("block_node").getAsString(), 16),
                sectionConsumer);
            default -> throw new IllegalArgumentException("Unknown SectionPos around-and-at mode");
        }
        var state = new JsonObject();
        state.add("section_nodes", values);
        return state;
    }

    private static JsonObject chunkPosConstant(JsonObject request) {
        var state = new JsonObject();
        switch (request.get("name").getAsString()) {
            case "zero" -> state.add("chunk_pos", snapshot(ChunkPos.ZERO));
            case "invalid_chunk_pos" -> state.addProperty("packed", Long.toUnsignedString(ChunkPos.INVALID_CHUNK_POS, 16));
            case "region_size" -> state.addProperty("value", Integer.toString(ChunkPos.REGION_SIZE));
            case "region_max_index" -> state.addProperty("value", Integer.toString(ChunkPos.REGION_MAX_INDEX));
            default -> throw new IllegalArgumentException("Unknown ChunkPos constant");
        }
        return state;
    }

    private static JsonObject chunkPosValidity(ChunkPos pos) {
        var output = System.out;
        SharedConstants.setVersion(TEST_VERSION);
        Bootstrap.bootStrap();
        System.setOut(output);
        var state = new JsonObject();
        state.addProperty("instance", pos.isValid());
        state.addProperty("static", ChunkPos.isValid(pos.x(), pos.z()));
        return state;
    }

    private static JsonObject codecDecodeResult(DataResult<Vec3> decoded) {
        var result = new JsonObject();
        decoded.result().ifPresent(value -> {
            result.addProperty("outcome", "value");
            result.add("value", snapshot(value));
        });
        decoded.error().ifPresent(error -> {
            result.addProperty("outcome", "error");
            result.addProperty("error", error.message());
            error.partialValue().ifPresent(value -> result.add("partial", snapshot(value)));
        });
        return result;
    }

    private static JsonObject codecEncodeResult(DataResult<JsonElement> encoded) {
        var result = new JsonObject();
        encoded.result().ifPresent(value -> {
            result.addProperty("outcome", "value");
            var values = new JsonArray();
            for (var entry : value.getAsJsonArray()) {
                values.add(Oracle.bits(entry.getAsDouble()));
            }
            result.add("value", values);
        });
        encoded.error().ifPresent(error -> {
            result.addProperty("outcome", "error");
            result.addProperty("error", error.message());
        });
        return result;
    }

    private static JsonObject chunkPosCodecDecodeResult(DataResult<ChunkPos> decoded) {
        var result = new JsonObject();
        decoded.result().ifPresent(value -> {
            result.addProperty("outcome", "value");
            result.add("value", snapshot(value));
        });
        decoded.error().ifPresent(error -> {
            result.addProperty("outcome", "error");
            result.addProperty("error", error.message());
            error.partialValue().ifPresent(value -> result.add("partial", snapshot(value)));
        });
        return result;
    }

    private static JsonObject chunkPosCodecEncodeResult(DataResult<JsonElement> encoded) {
        var result = new JsonObject();
        encoded.result().ifPresent(value -> {
            result.addProperty("outcome", "value");
            var values = new JsonArray();
            for (var entry : value.getAsJsonArray()) {
                values.add(entry.getAsInt());
            }
            result.add("value", values);
        });
        encoded.error().ifPresent(error -> {
            result.addProperty("outcome", "error");
            result.addProperty("error", error.message());
        });
        return result;
    }

    private static JsonArray randomSteps(RandomSource random, JsonArray entries) {
        var events = new JsonArray();
        for (var entry : entries) {
            events.add(randomState(random, entry.getAsJsonObject()));
        }
        return events;
    }

    private static JsonObject randomState(RandomSource random, JsonObject step) {
        var state = new JsonObject();
        switch (step.get("op").getAsString()) {
            case "nextInt" -> state.addProperty("value", Integer.toString(random.nextInt()));
            case "nextIntBound" -> state.addProperty("value",
                Integer.toString(random.nextInt(step.get("bound").getAsInt())));
            case "nextLong" -> state.addProperty("value", Long.toString(random.nextLong()));
            case "nextBoolean" -> state.addProperty("value", Boolean.toString(random.nextBoolean()));
            case "nextFloat" -> state.addProperty("bits", Oracle.bits(random.nextFloat()));
            case "nextDouble" -> state.addProperty("bits", Oracle.bits(random.nextDouble()));
            case "nextGaussian" -> state.addProperty("bits", Oracle.bits(random.nextGaussian()));
            case "setSeed" -> {
                random.setSeed(Long.parseLong(step.get("seed").getAsString()));
                state.addProperty("op", "setSeed");
            }
            case "nextIntBetweenInclusive" -> state.addProperty("value", Integer.toString(
                random.nextIntBetweenInclusive(step.get("min").getAsInt(), step.get("max").getAsInt())));
            case "nextIntRange" -> state.addProperty("value", Integer.toString(
                random.nextInt(step.get("origin").getAsInt(), step.get("bound").getAsInt())));
            case "triangleDouble" -> state.addProperty("bits", Oracle.bits(
                random.triangle(f64(step, "mean"), f64(step, "spread"))));
            case "triangleFloat" -> state.addProperty("bits", Oracle.bits(
                random.triangle(Oracle.f32(step, "mean"), Oracle.f32(step, "spread"))));
            case "consumeCount" -> {
                random.consumeCount(step.get("rounds").getAsInt());
                state.addProperty("op", "consumeCount");
            }
            default -> throw new IllegalArgumentException("Unknown random op");
        }
        return state;
    }

    private static JsonObject snapshot(AABB box) {
        var state = new JsonObject();
        state.addProperty("min_x", Oracle.bits(box.minX));
        state.addProperty("min_y", Oracle.bits(box.minY));
        state.addProperty("min_z", Oracle.bits(box.minZ));
        state.addProperty("max_x", Oracle.bits(box.maxX));
        state.addProperty("max_y", Oracle.bits(box.maxY));
        state.addProperty("max_z", Oracle.bits(box.maxZ));
        return state;
    }

    private static JsonObject snapshot(EntityDimensions dimensions) {
        var state = new JsonObject();
        state.addProperty("width", Oracle.bits(dimensions.width()));
        state.addProperty("height", Oracle.bits(dimensions.height()));
        state.addProperty("eye_height", Oracle.bits(dimensions.eyeHeight()));
        state.addProperty("fixed", dimensions.fixed());
        return state;
    }

    private static JsonObject snapshot(VoxelShape shape) {
        var state = new JsonObject();
        state.addProperty("empty", shape.isEmpty());
        var boxes = new JsonArray();
        for (AABB box : shape.toAabbs()) {
            boxes.add(snapshot(box));
        }
        state.add("boxes", boxes);
        if (!shape.isEmpty()) {
            state.add("bounds", snapshot(shape.bounds()));
        }
        return state;
    }

    private static JsonObject snapshotWithCoords(VoxelShape shape) {
        var state = snapshot(shape);
        var coordinates = new JsonObject();
        for (Direction.Axis axis : Direction.Axis.values()) {
            var values = new JsonArray();
            var coords = shape.getCoords(axis);
            for (int i = 0; i < coords.size(); i++) {
                values.add(Oracle.bits(coords.getDouble(i)));
            }
            coordinates.add(axis.getName(), values);
        }
        state.add("coords", coordinates);
        return state;
    }

    private static JsonObject snapshot(BlockPos pos) {
        var state = new JsonObject();
        state.addProperty("x", Integer.toString(pos.getX()));
        state.addProperty("y", Integer.toString(pos.getY()));
        state.addProperty("z", Integer.toString(pos.getZ()));
        return state;
    }

    private static JsonObject snapshot(ChunkPos pos) {
        var state = new JsonObject();
        state.addProperty("x", Integer.toString(pos.x()));
        state.addProperty("z", Integer.toString(pos.z()));
        return state;
    }

    private static JsonObject snapshot(SectionPos pos) {
        var state = new JsonObject();
        state.addProperty("x", Integer.toString(pos.x()));
        state.addProperty("y", Integer.toString(pos.y()));
        state.addProperty("z", Integer.toString(pos.z()));
        state.addProperty("packed", Long.toUnsignedString(pos.asLong(), 16));
        return state;
    }

    private static JsonObject snapshot(Direction facing) {
        var state = new JsonObject();
        state.addProperty("name", facing.getName());
        state.addProperty("data3d", facing.get3DDataValue());
        state.addProperty("data2d", facing.get2DDataValue());
        state.addProperty("opposite", facing.getOpposite().getName());
        state.addProperty("axis", facing.getAxis().getName());
        state.addProperty("axis_direction", facing.getAxisDirection().getName());
        state.addProperty("step_x", facing.getStepX());
        state.addProperty("step_y", facing.getStepY());
        state.addProperty("step_z", facing.getStepZ());
        return state;
    }

    private static JsonObject snapshot(BlockHitResult hit) {
        var state = new JsonObject();
        if (hit == null) {
            state.addProperty("hit", false);
            return state;
        }
        state.addProperty("hit", true);
        state.addProperty("type", hit.getType().name());
        state.add("location", snapshotClip(hit.getLocation()));
        state.addProperty("direction", hit.getDirection().getName());
        state.add("block_pos", snapshot(hit.getBlockPos()));
        state.addProperty("inside", hit.isInside());
        state.addProperty("world_border_hit", hit.isWorldBorderHit());
        return state;
    }

    private static JsonObject snapshot(Vec3 vector) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(vector.x));
        state.addProperty("y", Oracle.bits(vector.y));
        state.addProperty("z", Oracle.bits(vector.z));
        state.addProperty("length", Oracle.bits(vector.length()));
        state.addProperty("length_squared", Oracle.bits(vector.lengthSqr()));
        return state;
    }

    private static JsonObject snapshot(Vector3f vector) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(vector.x));
        state.addProperty("y", Oracle.bits(vector.y));
        state.addProperty("z", Oracle.bits(vector.z));
        return state;
    }

    private static JsonObject snapshot(Quaternionf quaternion) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(quaternion.x));
        state.addProperty("y", Oracle.bits(quaternion.y));
        state.addProperty("z", Oracle.bits(quaternion.z));
        state.addProperty("w", Oracle.bits(quaternion.w));
        return state;
    }

    private static JsonObject snapshotClip(Vec3 point) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(point.x));
        state.addProperty("y", Oracle.bits(point.y));
        state.addProperty("z", Oracle.bits(point.z));
        return state;
    }

    private static JsonObject snapshot(Vec2 vector) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(vector.x));
        state.addProperty("y", Oracle.bits(vector.y));
        state.addProperty("length", Oracle.bits(vector.length()));
        state.addProperty("length_squared", Oracle.bits(vector.lengthSquared()));
        return state;
    }

    private static JsonObject snapshot(Input input) {
        var state = new JsonObject();
        state.addProperty("forward", input.forward());
        state.addProperty("backward", input.backward());
        state.addProperty("left", input.left());
        state.addProperty("right", input.right());
        state.addProperty("jump", input.jump());
        state.addProperty("shift", input.shift());
        state.addProperty("sprint", input.sprint());
        return state;
    }

    private static JsonArray snapshotUtf16(String value) {
        var units = new JsonArray();
        for (int i = 0; i < value.length(); i++) {
            units.add(String.format(Locale.ROOT, "%04x", (int)value.charAt(i)));
        }
        return units;
    }

    private static JsonObject snapshotPreedit(PreeditEvent event) {
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

    private static JsonObject snapshotInputWithModifiers(InputWithModifiers input) {
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

    private static InputConstants.Key inputKey(JsonObject value) {
        InputConstants.Type type = switch (value.get("type").getAsString()) {
            case "keysym" -> InputConstants.Type.KEYSYM;
            case "scancode" -> InputConstants.Type.SCANCODE;
            case "mouse" -> InputConstants.Type.MOUSE;
            default -> throw new IllegalArgumentException("Unknown input key type");
        };
        return type.getOrCreate(value.get("value").getAsInt());
    }

    private static KeyMapping newKeyMapping(String name, InputConstants.Key key) {
        return new KeyMapping(name, key.getType(), key.getValue(), KeyMapping.Category.MOVEMENT);
    }

    private static KeyMapping mapping(String target, KeyMapping primary, KeyMapping secondary, KeyMapping other) {
        return switch (target) {
            case "primary" -> primary;
            case "secondary" -> secondary;
            case "other" -> other;
            default -> throw new IllegalArgumentException("Unknown key mapping target");
        };
    }

    private static InputConstants.Key actionKey(String target, InputConstants.Key primary,
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

    private static JsonObject keyMappingState(KeyMapping primary, KeyMapping secondary, KeyMapping other) {
        var state = new JsonObject();
        state.addProperty("primary_down", primary.isDown());
        state.addProperty("secondary_down", secondary.isDown());
        state.addProperty("other_down", other.isDown());
        return state;
    }

    private static JsonObject keyMappingProperties(
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

    private static JsonObject keyMappingProperty(
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

    private static Options optionsFor(Input keys) {
        try {
            Unsafe unsafe = unsafeInstance();
            Options options = (Options)unsafe.allocateInstance(Options.class);
            putKey(unsafe, options, "keyUp", "behavior.keyboard.up", 100, keys.forward());
            putKey(unsafe, options, "keyDown", "behavior.keyboard.down", 101, keys.backward());
            putKey(unsafe, options, "keyLeft", "behavior.keyboard.left", 102, keys.left());
            putKey(unsafe, options, "keyRight", "behavior.keyboard.right", 103, keys.right());
            putKey(unsafe, options, "keyJump", "behavior.keyboard.jump", 104, keys.jump());
            putKey(unsafe, options, "keyShift", "behavior.keyboard.shift", 105, keys.shift());
            putKey(unsafe, options, "keySprint", "behavior.keyboard.sprint", 106, keys.sprint());
            return options;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("Unable to construct controlled keyboard options", error);
        }
    }

    private static Unsafe unsafeInstance() {
        try {
            var unsafeField = Unsafe.class.getDeclaredField("theUnsafe");
            unsafeField.setAccessible(true);
            return (Unsafe)unsafeField.get(null);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("Unable to access controlled native memory", error);
        }
    }

    private static void putKey(Unsafe unsafe, Options options, String fieldName, String name,
                               int keyCode, boolean down) throws ReflectiveOperationException {
        KeyMapping key = new KeyMapping(name, keyCode, KeyMapping.Category.MOVEMENT);
        key.setDown(down);
        var field = Options.class.getField(fieldName);
        unsafe.putObject(options, unsafe.objectFieldOffset(field), key);
    }
}
