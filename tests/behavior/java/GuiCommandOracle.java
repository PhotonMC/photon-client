import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** Command suggestions and parse results. */
class GuiCommandOracle extends GuiChatOracle {
    /**
     * What the tree and the types suggest for text at a cursor, by the game's
     * own `getCompletionSuggestions`, and whether the server would be asked.
     */
    static JsonObject commandSuggest(JsonObject request) throws Exception {
        var dispatcher = commandDispatcher(request.get("tree").getAsString());
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            String text = row.get("text").getAsString();
            int cursor = row.has("cursor") ? row.get("cursor").getAsInt() : text.length();
            var players = new java.util.ArrayList<String>();
            for (var player : row.getAsJsonArray("players")) {
                players.add(player.getAsString());
            }
            var teams = new java.util.ArrayList<String>();
            for (var team : row.getAsJsonArray("teams")) {
                teams.add(team.getAsString());
            }
            var source = new OracleSource(players, teams, !row.has("selectors") || row.get("selectors").getAsBoolean());
            if (row.has("selected_entities")) {
                var selected = new java.util.ArrayList<String>();
                for (var entity : row.getAsJsonArray("selected_entities")) {
                    selected.add(entity.getAsString());
                }
                source.selected = selected;
            }
            if (row.has("registry_backed")) {
                // The game lists these from its registries; this port asks the server.
                var out = new JsonObject();
                out.addProperty("label", row.get("label").getAsString());
                out.add("matches", new JsonArray());
                out.addProperty("start", -1);
                out.addProperty("asked", true);
                rows.add(out);
                continue;
            }
            var reader = new com.mojang.brigadier.StringReader(text);
            if (reader.canRead() && reader.peek() == '/') {
                reader.skip();
            }
            var parse = dispatcher.parse(reader, source);
            var suggestions = dispatcher.getCompletionSuggestions(parse, cursor).get();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var matches = new JsonArray();
            for (var suggestion : suggestions.getList()) {
                matches.add(suggestion.getText());
            }
            out.add("matches", matches);
            out.addProperty("start", suggestions.isEmpty() ? -1 : suggestions.getRange().getStart());
            out.addProperty("asked", source.asked);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A command tree the way the client reads one off the wire, and what the
     * chat box makes of texts typed against it: where each node read, what
     * could not be read, the colours of the text and the lines under it.
     */
    @SuppressWarnings("unchecked")
    static JsonObject commandParse(JsonObject request) throws Exception {
        var dispatcher = commandDispatcher(request.get("tree").getAsString());
        var formatText = net.minecraft.client.gui.components.CommandSuggestions.class.getDeclaredMethod("formatText",
            com.mojang.brigadier.ParseResults.class, String.class, int.class);
        formatText.setAccessible(true);
        var exceptionMessage = net.minecraft.client.gui.components.CommandSuggestions.class
            .getDeclaredMethod("getExceptionMessage", com.mojang.brigadier.exceptions.CommandSyntaxException.class);
        exceptionMessage.setAccessible(true);
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            String text = row.get("text").getAsString();
            var reader = new com.mojang.brigadier.StringReader(text);
            if (reader.canRead() && reader.peek() == '/') {
                reader.skip();
            }
            boolean selectors = !row.has("selectors") || row.get("selectors").getAsBoolean();
            net.minecraft.client.multiplayer.ClientSuggestionProvider source =
                new net.minecraft.client.multiplayer.ClientSuggestionProvider(null, null, permission -> selectors
                    || !permission.equals(net.minecraft.server.permissions.Permissions.COMMANDS_ENTITY_SELECTORS));
            var parse = dispatcher.parse(reader, source);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var nodes = new JsonArray();
            for (var context = parse.getContext(); context != null; context = context.getChild()) {
                for (var node : context.getNodes()) {
                    var described = new JsonArray();
                    described.add(node.getNode().getUsageText());
                    described.add(node.getRange().getStart());
                    described.add(node.getRange().getEnd());
                    nodes.add(described);
                }
            }
            out.add("nodes", nodes);
            var arguments = new JsonArray();
            for (var entry : parse.getContext().getLastChild().getArguments().entrySet()) {
                var described = new JsonArray();
                described.add(entry.getKey());
                described.add(entry.getValue().getRange().getStart());
                described.add(entry.getValue().getRange().getEnd());
                arguments.add(described);
            }
            out.add("arguments", arguments);
            out.addProperty("cursor", parse.getReader().getCursor());
            out.addProperty("executable", parse.getContext().getLastChild().getCommand() != null);
            var exceptions = new JsonArray();
            for (var entry : parse.getExceptions().entrySet()) {
                var exception = entry.getValue();
                var described = new JsonArray();
                described.add(net.minecraft.network.chat.ComponentUtils.fromMessage(exception.getRawMessage()).getString());
                described.add(exception.getCursor());
                exceptions.add(described);
            }
            out.add("exceptions", exceptions);
            var runs = new JsonArray();
            var sequence = (net.minecraft.util.FormattedCharSequence) formatText.invoke(null, parse, text, 0);
            var current = new StringBuilder();
            int[] colour = {-2};
            sequence.accept((index, style, codepoint) -> {
                int rgb = style.getColor() == null ? -1 : style.getColor().getValue();
                if (rgb != colour[0] && current.length() > 0) {
                    var described = new JsonArray();
                    described.add(current.toString());
                    described.add(colour[0]);
                    runs.add(described);
                    current.setLength(0);
                }
                colour[0] = rgb;
                current.appendCodePoint(codepoint);
                return true;
            });
            if (current.length() > 0) {
                var described = new JsonArray();
                described.add(current.toString());
                described.add(colour[0]);
                runs.add(described);
            }
            out.add("runs", runs);
            // `updateUsageInfo` with the cursor at the end of the text.
            var usage = new JsonArray();
            boolean trailing = false;
            if (row.get("suggestions_empty").getAsBoolean() && !parse.getExceptions().isEmpty()) {
                int literals = 0;
                for (var entry : parse.getExceptions().entrySet()) {
                    var exception = entry.getValue();
                    if (exception.getType() == com.mojang.brigadier.exceptions.CommandSyntaxException.BUILT_IN_EXCEPTIONS.literalIncorrect()) {
                        literals++;
                    } else {
                        usage.add(plain((net.minecraft.util.FormattedCharSequence) exceptionMessage.invoke(null, exception)));
                    }
                }
                if (literals > 0) {
                    usage.add(plain((net.minecraft.util.FormattedCharSequence) exceptionMessage.invoke(null,
                        com.mojang.brigadier.exceptions.CommandSyntaxException.BUILT_IN_EXCEPTIONS
                            .dispatcherUnknownArgument().createWithContext(parse.getReader()))));
                }
            } else if (parse.getReader().canRead()) {
                trailing = true;
            }
            var suggestionContext = parse.getContext().findSuggestionContext(text.length());
            if (usage.isEmpty()) {
                var entries = new JsonArray();
                for (var entry : dispatcher.getSmartUsage(suggestionContext.parent, source).entrySet()) {
                    if (!(entry.getKey() instanceof com.mojang.brigadier.tree.LiteralCommandNode)) {
                        entries.add(entry.getValue());
                    }
                }
                if (entries.isEmpty() && trailing) {
                    usage.add(plain((net.minecraft.util.FormattedCharSequence) exceptionMessage.invoke(null,
                        net.minecraft.commands.Commands.getParseException(parse))));
                }
                usage.addAll(entries);
            }
            out.add("usage", usage);
            out.addProperty("usage_start", suggestionContext.startPos);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
