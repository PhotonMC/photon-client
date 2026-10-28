import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/**
 * Test glue only: reads the original block registry, never rebuilds it.
 *
 * <p>Bootstrapping the registry replaces {@code System.out} with a stream that
 * feeds the game's logger, which would swallow every record written after it.
 * The original stream is put back as soon as the bootstrap returns, so the
 * other families in the same run are unaffected.
 */
public final class BlockOracle extends BlockDeclarationOracle {
    public static JsonObject run(JsonObject request) {
        bootstrap();
        String operation = request.get("operation").getAsString();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            switch (operation) {
                case "registry_names" -> rows.add(registryNames(element.getAsJsonObject()));
                case "state_definitions" -> rows.add(stateDefinitions(element.getAsJsonObject()));
                case "block_tints" -> rows.add(blockTints(element.getAsJsonObject()));
                case "face_occlusion" -> rows.add(faceOcclusion(element.getAsJsonObject()));
                case "collision_shapes" -> rows.add(collisionShapes(element.getAsJsonObject()));
                case "outline_shapes" -> rows.add(outlineShapes(element.getAsJsonObject()));
                case "movement_properties" -> rows.add(movementProperties(element.getAsJsonObject()));
                case "light_properties" -> rows.add(lightProperties(element.getAsJsonObject()));
                case "offset_properties" -> rows.add(offsetProperties(element.getAsJsonObject()));
                case "fluid_overlay_blocks" -> rows.add(fluidOverlayBlocks(element.getAsJsonObject()));
                case "menu_providers" -> rows.add(menuProviders(element.getAsJsonObject()));
                case "motion_blocking" -> rows.add(motionBlocking(element.getAsJsonObject()));
                case "fluid_flow_properties" -> rows.add(fluidFlowProperties(element.getAsJsonObject()));
                case "destroy_properties" -> rows.add(destroyProperties(element.getAsJsonObject()));
                case "item_tools" -> rows.add(itemTools(element.getAsJsonObject()));
                case "item_stacking" -> rows.add(itemStacking(element.getAsJsonObject()));
                case "item_components" -> rows.add(itemComponents(element.getAsJsonObject()));
                case "tooltip_registries" -> rows.add(tooltipRegistries(element.getAsJsonObject()));
                case "potions" -> rows.add(potions(element.getAsJsonObject()));
                case "creative_tabs" -> rows.add(creativeTabs(element.getAsJsonObject()));
                case "creative_tag_search" -> rows.add(creativeTagSearch(element.getAsJsonObject()));
                case "block_behaviours" -> rows.add(blockBehaviours(element.getAsJsonObject()));
                case "block_declarations" -> rows.add(blockDeclarations(element.getAsJsonObject()));
                case "placement_rules" -> rows.add(placementRules(element.getAsJsonObject()));
                case "item_declarations" -> rows.add(itemDeclarations(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException("Unknown block operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The names of the registries whose wire ids the client reads, in id order. */
    static JsonObject registryNames(JsonObject row) {
        Game26_2.vanillaRegistries();
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.add("block_entity_types",
            names(net.minecraft.core.registries.BuiltInRegistries.BLOCK_ENTITY_TYPE));
        entry.add("menu_types", names(net.minecraft.core.registries.BuiltInRegistries.MENU));
        entry.add("stat_types", names(net.minecraft.core.registries.BuiltInRegistries.STAT_TYPE));
        entry.add("custom_stats", names(net.minecraft.core.registries.BuiltInRegistries.CUSTOM_STAT));
        entry.add("custom_stat_formatters", customStatFormatters());
        return entry;
    }

    /** What formats each custom statistic's value, by the formatter the game registered it with. */
    private static JsonArray customStatFormatters() {
        var formatters = new JsonArray();
        var registry = net.minecraft.core.registries.BuiltInRegistries.CUSTOM_STAT;
        try {
            var field = Reflect.field(net.minecraft.stats.Stat.class, "formatter");
            var known = java.util.Map.of(
                "default", net.minecraft.stats.StatFormatter.DEFAULT,
                "divide_by_ten", net.minecraft.stats.StatFormatter.DIVIDE_BY_TEN,
                "distance", net.minecraft.stats.StatFormatter.DISTANCE,
                "time", net.minecraft.stats.StatFormatter.TIME);
            for (int id = 0; id < registry.size(); id++) {
                var formatter = field.get(net.minecraft.stats.Stats.CUSTOM.get(registry.byId(id)));
                var name = known.entrySet().stream().filter(entry -> entry.getValue() == formatter)
                    .map(java.util.Map.Entry::getKey).findFirst().orElseThrow();
                formatters.add(name);
            }
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(error);
        }
        return formatters;
    }

    private static <T> JsonArray names(net.minecraft.core.Registry<T> registry) {
        var names = new JsonArray();
        for (int id = 0; id < registry.size(); id++) {
            var key = registry.getKey(registry.byId(id));
            names.add(key.toString());
        }
        return names;
    }
}
