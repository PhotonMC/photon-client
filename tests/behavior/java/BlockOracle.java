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
                case "state_definitions" -> rows.add(stateDefinitions(element.getAsJsonObject()));
                case "block_tints" -> rows.add(blockTints(element.getAsJsonObject()));
                case "face_occlusion" -> rows.add(faceOcclusion(element.getAsJsonObject()));
                case "collision_shapes" -> rows.add(collisionShapes(element.getAsJsonObject()));
                case "outline_shapes" -> rows.add(outlineShapes(element.getAsJsonObject()));
                case "movement_properties" -> rows.add(movementProperties(element.getAsJsonObject()));
                case "light_properties" -> rows.add(lightProperties(element.getAsJsonObject()));
                case "offset_properties" -> rows.add(offsetProperties(element.getAsJsonObject()));
                case "fluid_overlay_blocks" -> rows.add(fluidOverlayBlocks(element.getAsJsonObject()));
                case "motion_blocking" -> rows.add(motionBlocking(element.getAsJsonObject()));
                case "fluid_flow_properties" -> rows.add(fluidFlowProperties(element.getAsJsonObject()));
                case "destroy_properties" -> rows.add(destroyProperties(element.getAsJsonObject()));
                case "item_tools" -> rows.add(itemTools(element.getAsJsonObject()));
                case "item_stacking" -> rows.add(itemStacking(element.getAsJsonObject()));
                case "item_components" -> rows.add(itemComponents(element.getAsJsonObject()));
                case "tooltip_registries" -> rows.add(tooltipRegistries(element.getAsJsonObject()));
                case "potions" -> rows.add(potions(element.getAsJsonObject()));
                case "creative_tabs" -> rows.add(creativeTabs(element.getAsJsonObject()));
                case "block_behaviours" -> rows.add(blockBehaviours(element.getAsJsonObject()));
                case "block_declarations" -> rows.add(blockDeclarations(element.getAsJsonObject()));
                case "item_declarations" -> rows.add(itemDeclarations(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException("Unknown block operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
