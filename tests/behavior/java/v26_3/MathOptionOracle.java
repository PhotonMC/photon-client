import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.GraphicsPreset;
import net.minecraft.client.OptionInstance;
import net.minecraft.util.Mth;

class MathOptionOracle extends MathNumberOracle {
    static JsonObject optionValuesCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject sliderableEnumCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject cycleableEnumCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject altEnumCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject lazyEnumCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject clampingLazyMaxIntRangeCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject mappedIntRangeCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }

    static JsonObject mappedUnitDoubleCase(JsonObject request) {
        var result = new JsonObject();
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
        return result;
    }
}
