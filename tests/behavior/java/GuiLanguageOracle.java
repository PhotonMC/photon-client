import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The client language and text lookups. */
class GuiLanguageOracle extends GuiRecorder {
    /** `.data/minecraft/assets`, found from the game's jar in `.data/minecraft/runtime`. */
    static java.nio.file.Path assetFolder() throws Exception {
        var jar = java.nio.file.Path.of(net.minecraft.client.Minecraft.class
            .getProtectionDomain().getCodeSource().getLocation().toURI());
        return jar.getParent().getParent().resolve("assets");
    }

    /** The pack's names of the languages, and the file of a language of the asset index. */
    static java.util.Map<String, com.google.gson.JsonObject> languageIndex() throws Exception {
        var index = com.google.gson.JsonParser.parseReader(java.nio.file.Files.newBufferedReader(
            assetFolder().resolve("indexes/32.json"))).getAsJsonObject().getAsJsonObject("objects");
        var result = new java.util.HashMap<String, com.google.gson.JsonObject>();
        for (var entry : index.entrySet()) {
            result.put(entry.getKey(), entry.getValue().getAsJsonObject());
        }
        return result;
    }

    static java.nio.file.Path object(java.util.Map<String, com.google.gson.JsonObject> index, String name)
            throws Exception {
        String hash = index.get(name).get("hash").getAsString();
        return assetFolder().resolve("objects").resolve(hash.substring(0, 2)).resolve(hash);
    }

    /** `LanguageManager.getLanguages` as the pack names them. */
    static net.minecraft.client.resources.language.LanguageManager languageManager(String code, JsonArray codes)
            throws Exception {
        var allowed = new java.util.HashSet<String>();
        for (var listed : codes) allowed.add(listed.getAsString());
        var index = languageIndex();
        var meta = com.google.gson.JsonParser.parseReader(java.nio.file.Files.newBufferedReader(object(index, "pack.mcmeta")))
            .getAsJsonObject().getAsJsonObject("language");
        var languages = new java.util.HashMap<String, net.minecraft.client.resources.language.LanguageInfo>();
        for (var entry : meta.entrySet()) {
            var info = entry.getValue().getAsJsonObject();
            if (!allowed.isEmpty() && !allowed.contains(entry.getKey())) continue;
            languages.put(entry.getKey(), new net.minecraft.client.resources.language.LanguageInfo(
                info.get("region").getAsString(), info.get("name").getAsString(),
                info.has("bidirectional") && info.get("bidirectional").getAsBoolean()));
        }
        var manager = new net.minecraft.client.resources.language.LanguageManager(code, language -> {});
        set(manager, "languages", languages);
        return manager;
    }

    /** `ClientLanguage.loadFrom` for `en_us` and a code, from the jar and the asset index. */
    static net.minecraft.client.resources.language.ClientLanguage clientLanguage(String code) throws Exception {
        return clientLanguage(code, java.util.Map.of());
    }

    /**
     * The same with some translations added: the names GLFW gives printable keys on a keyboard layout,
     * which a headless run has no window to ask (`InputConstants.Type.KEYSYM`'s display names fall back
     * to their language key without it).
     */
    static net.minecraft.client.resources.language.ClientLanguage clientLanguage(
            String code, java.util.Map<String, String> added) throws Exception {
        var index = languageIndex();
        var translations = new java.util.HashMap<String, String>();
        try (var jar = GuiOracle.class.getResourceAsStream("/assets/minecraft/lang/en_us.json")) {
            net.minecraft.locale.Language.loadFromJson(jar, translations::put);
        }
        if (!code.equals("en_us") && index.containsKey("minecraft/lang/" + code + ".json")) {
            try (var stream = java.nio.file.Files.newInputStream(object(index, "minecraft/lang/" + code + ".json"))) {
                net.minecraft.locale.Language.loadFromJson(stream, translations::put);
            }
        }
        translations.putAll(added);
        var constructor = net.minecraft.client.resources.language.ClientLanguage.class
            .getDeclaredConstructor(java.util.Map.class, boolean.class);
        constructor.setAccessible(true);
        return constructor.newInstance(java.util.Map.copyOf(translations), false);
    }

    /** What the game's language says of some keys, in a language. */
    static JsonObject langText(JsonObject request) throws Exception {
        var rows = new JsonArray();
        var before = net.minecraft.locale.Language.getInstance();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            net.minecraft.locale.Language.inject(clientLanguage(row.get("language").getAsString()));
            var texts = new JsonArray();
            for (var key : row.getAsJsonArray("keys")) {
                texts.add(net.minecraft.locale.Language.getInstance().getOrDefault(key.getAsString()));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("texts", texts);
            rows.add(out);
        }
        net.minecraft.locale.Language.inject(before);
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
