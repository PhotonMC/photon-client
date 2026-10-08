import com.google.gson.JsonObject;

/**
 * The game's own GUI drawing, recorded: the rectangles and text each screen
 * element asks for, in order, with the pose applied. The glyphs are not
 * drawn (no GPU); their advances come from the game's own font providers, so
 * the layout is the reference's.
 */
public final class GuiOracle extends GuiSignOracle {
    GuiOracle() {}

    public static JsonObject run(JsonObject request) {
        try {
            return switch (request.get("operation").getAsString()) {
                case "chat" -> chat(request);
                case "suggestions" -> suggestions(request);
                case "titles" -> titles(request);
                case "boss_bar" -> bossBar(request);
                case "locator_bar" -> locatorBar(request);
                case "chat_draft" -> chatDraft(request);
                case "sidebar" -> sidebar(request);
                case "command_parse" -> commandParse(request);
                case "command_suggest" -> commandSuggest(request);
                case "pause_screen" -> pauseScreen(request);
                case "title_screen" -> titleScreen(request);
                case "join_multiplayer_screen" -> joinMultiplayerScreen(request);
                case "server_form" -> serverForm(request);
                case "connect_screen" -> connectScreen(request);
                case "name_tags" -> nameTags(request);
                case "sign_texts" -> signTexts(request);
                case "sign_edit_screen" -> signEditScreen(request);
                case "book_edit_screen" -> bookEditScreen(request);
                case "hud_hotbar" -> hudHotbar(request);
                case "hud_status" -> hudStatus(request);
                case "confirm_screen" -> confirmScreen(request);
                case "widgets" -> widgets(request);
                case "options_screen" -> optionsScreen(request);
                case "video_settings_screen" -> settingsScreen(request, "video");
                case "chat_options_screen" -> settingsScreen(request, "chat");
                case "mouse_settings_screen" -> settingsScreen(request, "mouse");
                case "sound_options_screen" -> settingsScreen(request, "sound");
                case "controls_screen" -> settingsScreen(request, "controls");
                case "key_mappings" -> keyMappings(request);
                case "language_screen" -> settingsScreen(request, "language");
                case "lang_text" -> langText(request);
                case "death_screen" -> deathScreen(request);
                case "disconnected_screen" -> disconnectedScreen(request);
                case "tab_list" -> tabList(request);
                case "container_screen" -> containerScreen(request);
                default -> throw new IllegalArgumentException("Unknown gui operation");
            };
        } catch (Exception error) {
            throw new IllegalStateException(error);
        }
    }
}
