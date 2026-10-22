import net.minecraft.network.chat.Component;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import java.util.HashMap;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** Name tags, the sidebar and the tab list. */
class GuiOverlayOracle extends GuiHudOracle {
    /** The game's own pause menu, over a game with nothing behind it. */
    /** What a name tag submit holds, as the reference's own phases receive it. */
    static JsonObject describeNameTag(String phase,
        net.minecraft.client.renderer.feature.NameTagFeatureRenderer.Submit submit) {
        var out = new JsonObject();
        out.addProperty("phase", phase);
        var pose = new float[16];
        submit.pose().get(pose);
        var matrix = new JsonArray();
        for (float value : pose) {
            matrix.add(Oracle.bits(value));
        }
        out.add("pose", matrix);
        out.addProperty("x", Oracle.bits(submit.x()));
        out.addProperty("y", Oracle.bits(submit.y()));
        out.addProperty("text", submit.text().getString());
        out.addProperty("light", submit.lightCoords());
        out.addProperty("color", submit.color());
        out.addProperty("background", submit.backgroundColor());
        out.addProperty("mode", submit.displayMode().name());
        return out;
    }

    /** A phase that keeps its name tags, as the reference's own would batch them. */
    static final class KeptNormal extends net.minecraft.client.renderer.feature.phase.SimpleFeatureRenderPhase {
        final JsonArray kept = new JsonArray();

        @Override
        public void submit(net.minecraft.client.renderer.feature.submit.SubmitNode submit) {
            kept.add(describeNameTag("normal",
                (net.minecraft.client.renderer.feature.NameTagFeatureRenderer.Submit) submit));
        }
    }

    static final class KeptSeeThrough extends net.minecraft.client.renderer.feature.phase.TranslucentFeatureRenderPhase {
        final JsonArray kept = new JsonArray();

        @Override
        public void submit(net.minecraft.client.renderer.feature.submit.TranslucentSubmit submit) {
            kept.add(describeNameTag("see_through",
                (net.minecraft.client.renderer.feature.NameTagFeatureRenderer.Submit) submit));
        }
    }

    /**
     * A remote player's name tag as the reference's own renderer decides and
     * submits it: whether it shows (the player, the camera, the options), then
     * the submits `AvatarRenderer.submitNameDisplay` makes at the place the
     * dispatcher puts the entity.
     */
    static JsonObject nameTag(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        MovementOracle.bootstrap();
        var scene = MovementOracle.buildScene(row);
        var level = scene.level();
        var minecraft = scene.minecraft();
        // The teams the row names; none, as on a server that has set none up, when it names none.
        MovementOracle.setField(scene.listener(), "scoreboard", scoreboard(row));
        MovementOracle.setField(minecraft, "font", loadedFont());

        // The local player: whether it watches from the spectator's seat.
        var local = new net.minecraft.client.player.LocalPlayer(
            minecraft, level, scene.listener(), null, null,
            net.minecraft.world.entity.player.Input.EMPTY, false, null);
        local.setId(1);
        var info = new net.minecraft.client.multiplayer.PlayerInfo(scene.listener().getLocalGameProfile(), false);
        var setMode = net.minecraft.client.multiplayer.PlayerInfo.class.getDeclaredMethod(
            "setGameMode", net.minecraft.world.level.GameType.class);
        setMode.setAccessible(true);
        setMode.invoke(info, row.get("local_spectator").getAsBoolean()
            ? net.minecraft.world.level.GameType.SPECTATOR : net.minecraft.world.level.GameType.SURVIVAL);
        MovementOracle.setField(local, "playerInfo", info);
        MovementOracle.setField(minecraft, "player", local);

        // The hud and the options the submit reads.
        var hud = unsafe.allocateInstance(net.minecraft.client.gui.Hud.class);
        MovementOracle.setField(hud, "isHidden", row.get("hud_hidden").getAsBoolean());
        var gui = unsafe.allocateInstance(net.minecraft.client.gui.Gui.class);
        MovementOracle.setField(gui, "hud", hud);
        MovementOracle.setField(minecraft, "gui", gui);
        var gameRenderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        var renderState = new net.minecraft.client.renderer.state.GameRenderState();
        renderState.optionsRenderState.backgroundForChatOnly = row.get("background_for_chat_only").getAsBoolean();
        renderState.optionsRenderState.textBackgroundOpacity = Oracle.f32(row, "background_opacity");
        MovementOracle.setField(gameRenderer, "gameRenderState", renderState);
        MovementOracle.setField(gameRenderer, "mainCamera", new net.minecraft.client.Camera());
        MovementOracle.setField(minecraft, "gameRenderer", gameRenderer);

        // The camera and the dispatcher that measures from it.
        var camera = new net.minecraft.client.Camera();
        var setPosition = net.minecraft.client.Camera.class.getDeclaredMethod(
            "setPosition", double.class, double.class, double.class);
        setPosition.setAccessible(true);
        setPosition.invoke(camera, Oracle.f64(row, "camera_x"), Oracle.f64(row, "camera_y"),
            Oracle.f64(row, "camera_z"));
        var setRotation = net.minecraft.client.Camera.class.getDeclaredMethod(
            "setRotation", float.class, float.class);
        setRotation.setAccessible(true);
        setRotation.invoke(camera, Oracle.f32(row, "camera_y_rot"), Oracle.f32(row, "camera_x_rot"));
        MovementOracle.setField(gameRenderer, "mainCamera", camera);
        camera.setEntity(local);
        var dispatcher = (net.minecraft.client.renderer.entity.EntityRenderDispatcher)
            unsafe.allocateInstance(net.minecraft.client.renderer.entity.EntityRenderDispatcher.class);
        MovementOracle.setField(dispatcher, "camera", camera);
        if (row.has("type")) {
            return mobNameTag(row, entry, level, camera, dispatcher);
        }
        var renderer = (net.minecraft.client.renderer.entity.player.AvatarRenderer<?>)
            unsafe.allocateInstance(net.minecraft.client.renderer.entity.player.AvatarRenderer.class);
        MovementOracle.setField(renderer, "entityRenderDispatcher", dispatcher);

        // The remote player.
        var profile = new com.mojang.authlib.GameProfile(java.util.UUID.nameUUIDFromBytes(new byte[] {2}),
            row.get("name").getAsString());
        var remote = new net.minecraft.client.player.RemotePlayer(level, profile);
        remote.setId(2);
        remote.snapTo(Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"),
            Oracle.f32(row, "y_rot"), 0.0F);
        if (row.get("crouching").getAsBoolean()) {
            remote.setPose(net.minecraft.world.entity.Pose.CROUCHING);
        }
        remote.setShiftKeyDown(row.get("shift_key_down").getAsBoolean());
        remote.setInvisible(row.get("invisible").getAsBoolean());
        if (row.get("vehicle").getAsBoolean()) {
            var rider = new net.minecraft.client.player.RemotePlayer(level, new com.mojang.authlib.GameProfile(
                java.util.UUID.nameUUIDFromBytes(new byte[] {3}), "Rider"));
            rider.startRiding(remote, true, false);
        }

        // `LevelRenderer`: the state is taken, and the entity put where it is.
        var state = renderer.createRenderState();
        state.isDiscrete = remote.isDiscrete();
        state.isCrouching = remote.isCrouching();
        state.scale = 1.0F;
        state.showExtraEars = remote.showExtraEars();
        state.lightCoords = row.get("light").getAsInt();
        var extract = net.minecraft.client.renderer.entity.LivingEntityRenderer.class.getDeclaredMethod(
            "extractNameTags", net.minecraft.world.entity.LivingEntity.class,
            net.minecraft.client.renderer.entity.state.LivingEntityRenderState.class, float.class);
        extract.setAccessible(true);
        extract.invoke(renderer, remote, state, 0.0F);
        entry.addProperty("shown", state.nameTag != null);
        var submits = new JsonArray();
        if (state.nameTag != null) {
            entry.addProperty("text", state.nameTag.getString());
            entry.addProperty("attachment_y", Oracle.bits(state.nameTagAttachment.y));
            var storage = new net.minecraft.client.renderer.SubmitNodeStorage();
            var collection = storage.order(0);
            var normal = new KeptNormal();
            var seeThrough = new KeptSeeThrough();
            MovementOracle.setField(collection, "nameTags", normal);
            MovementOracle.setField(collection, "seeThroughNameTags", seeThrough);
            var cameraState = new net.minecraft.client.renderer.state.level.CameraRenderState();
            cameraState.orientation.set(camera.rotation());
            var poseStack = new com.mojang.blaze3d.vertex.PoseStack();
            var offset = renderer.getRenderOffset(state);
            poseStack.pushPose();
            poseStack.translate(remote.getX() - camera.position().x + offset.x,
                remote.getY() - camera.position().y + offset.y,
                remote.getZ() - camera.position().z + offset.z);
            var display = net.minecraft.client.renderer.entity.player.AvatarRenderer.class.getDeclaredMethod(
                "submitNameDisplay", net.minecraft.client.renderer.entity.state.AvatarRenderState.class,
                com.mojang.blaze3d.vertex.PoseStack.class,
                net.minecraft.client.renderer.SubmitNodeCollector.class,
                net.minecraft.client.renderer.state.level.CameraRenderState.class);
            display.setAccessible(true);
            display.invoke(renderer, state, poseStack, storage, cameraState);
            poseStack.popPose();
            submits.addAll(normal.kept);
            submits.addAll(seeThrough.kept);
        }
        entry.add("submits", submits);
        return entry;
    }

    /**
     * The name tag of an entity that is not a player, as the renderer of its kind decides and
     * submits it: a `MobRenderer`, an `ArmorStandRenderer` or the plain `EntityRenderer`.
     */
    static JsonObject mobNameTag(JsonObject row, JsonObject entry, net.minecraft.client.multiplayer.ClientLevel level,
            net.minecraft.client.Camera camera,
            net.minecraft.client.renderer.entity.EntityRenderDispatcher dispatcher) throws Exception {
        var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
            net.minecraft.resources.Identifier.parse(row.get("type").getAsString()));
        var entity = type.create(level, new net.minecraft.world.entity.EntitySpawnRequest(
            net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
        entity.setId(2);
        entity.setUUID(new java.util.UUID(0L, 2L));
        entity.snapTo(Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"), Oracle.f32(row, "y_rot"), 0.0F);
        if (row.has("custom_name")) {
            entity.setCustomName(Component.literal(row.get("custom_name").getAsString()));
        }
        entity.setCustomNameVisible(row.get("custom_name_visible").getAsBoolean());
        entity.setInvisible(row.get("invisible").getAsBoolean());
        entity.setShiftKeyDown(row.get("shift_key_down").getAsBoolean());
        if (row.get("baby").getAsBoolean() && entity instanceof net.minecraft.world.entity.Mob mob) {
            mob.setBaby(true);
        }
        if (row.has("scale")) {
            ((net.minecraft.world.entity.LivingEntity) entity).getAttribute(
                net.minecraft.world.entity.ai.attributes.Attributes.SCALE).setBaseValue(Oracle.f64(row, "scale"));
            entity.refreshDimensions();
        }
        if (row.has("name_tag_distance")) {
            ((net.minecraft.world.entity.LivingEntity) entity).getAttribute(
                net.minecraft.world.entity.ai.attributes.Attributes.NAME_TAG_DISTANCE).setBaseValue(
                    Oracle.f64(row, "name_tag_distance"));
        }
        if (row.get("vehicle").getAsBoolean()) {
            var rider = new net.minecraft.client.player.RemotePlayer(level, new com.mojang.authlib.GameProfile(
                java.util.UUID.nameUUIDFromBytes(new byte[] {3}), "Rider"));
            rider.startRiding(entity, true, false);
        }
        if (row.get("picked").getAsBoolean()) {
            MovementOracle.setField(dispatcher, "crosshairPickEntity", entity);
        }
        // The renderer of the kind: the rules are the class's, so one of each stands for its kind.
        Class<?> rendererClass;
        Class<?> stateClass;
        switch (row.get("renderer").getAsString()) {
            case "mob" -> {
                rendererClass = net.minecraft.client.renderer.entity.ZombieRenderer.class;
                stateClass = net.minecraft.client.renderer.entity.state.LivingEntityRenderState.class;
            }
            case "armor_stand" -> {
                rendererClass = net.minecraft.client.renderer.entity.ArmorStandRenderer.class;
                stateClass = net.minecraft.client.renderer.entity.state.LivingEntityRenderState.class;
            }
            default -> {
                rendererClass = net.minecraft.client.renderer.entity.TntRenderer.class;
                stateClass = net.minecraft.client.renderer.entity.state.EntityRenderState.class;
            }
        }
        var renderer = unsafe.allocateInstance(rendererClass);
        MovementOracle.setField(renderer, "entityRenderDispatcher", dispatcher);
        var state = (net.minecraft.client.renderer.entity.state.EntityRenderState) stateClass.getDeclaredConstructor().newInstance();
        state.isDiscrete = entity.isDiscrete();
        state.lightCoords = row.get("light").getAsInt();
        boolean living = !row.get("renderer").getAsString().equals("entity");
        var extract = (living
            ? net.minecraft.client.renderer.entity.LivingEntityRenderer.class
            : net.minecraft.client.renderer.entity.EntityRenderer.class).getDeclaredMethod("extractNameTags",
                living ? net.minecraft.world.entity.LivingEntity.class : net.minecraft.world.entity.Entity.class,
                living ? net.minecraft.client.renderer.entity.state.LivingEntityRenderState.class
                    : net.minecraft.client.renderer.entity.state.EntityRenderState.class, float.class);
        extract.setAccessible(true);
        extract.invoke(renderer, entity, state, 0.0F);
        entry.addProperty("shown", state.nameTag != null);
        var submits = new JsonArray();
        if (state.nameTag != null) {
            entry.addProperty("text", state.nameTag.getString());
            entry.addProperty("attachment_y", Oracle.bits(state.nameTagAttachment.y));
            var storage = new net.minecraft.client.renderer.SubmitNodeStorage();
            var collection = storage.order(0);
            var normal = new KeptNormal();
            var seeThrough = new KeptSeeThrough();
            MovementOracle.setField(collection, "nameTags", normal);
            MovementOracle.setField(collection, "seeThroughNameTags", seeThrough);
            var cameraState = new net.minecraft.client.renderer.state.level.CameraRenderState();
            cameraState.orientation.set(camera.rotation());
            var poseStack = new com.mojang.blaze3d.vertex.PoseStack();
            poseStack.pushPose();
            poseStack.translate(entity.getX() - camera.position().x, entity.getY() - camera.position().y,
                entity.getZ() - camera.position().z);
            var display = net.minecraft.client.renderer.entity.EntityRenderer.class.getDeclaredMethod(
                "submitNameDisplay", net.minecraft.client.renderer.entity.state.EntityRenderState.class,
                com.mojang.blaze3d.vertex.PoseStack.class,
                net.minecraft.client.renderer.SubmitNodeCollector.class,
                net.minecraft.client.renderer.state.level.CameraRenderState.class);
            display.setAccessible(true);
            display.invoke(renderer, state, poseStack, storage, cameraState);
            poseStack.popPose();
            submits.addAll(normal.kept);
            submits.addAll(seeThrough.kept);
        }
        entry.add("submits", submits);
        return entry;
    }

    static JsonObject nameTags(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            rows.add(nameTag(element.getAsJsonObject()));
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A number format named in a request: `blank`, `style:<colour>` or `fixed:<text>`. */
    static net.minecraft.network.chat.numbers.NumberFormat numberFormat(String spec) {
        if (spec.equals("blank")) {
            return net.minecraft.network.chat.numbers.BlankFormat.INSTANCE;
        }
        if (spec.startsWith("style:")) {
            return new net.minecraft.network.chat.numbers.StyledFormat(net.minecraft.network.chat.Style.EMPTY
                .withColor(net.minecraft.ChatFormatting.valueOf(spec.substring("style:".length()).toUpperCase())));
        }
        return new net.minecraft.network.chat.numbers.FixedFormat(Component.literal(spec.substring("fixed:".length())));
    }

    /**
     * A scoreboard a request describes: objectives, scores, the slots they
     * show in and teams, built through the game's own `Scoreboard`.
     */
    static net.minecraft.world.scores.Scoreboard scoreboard(JsonObject row) {
        var scoreboard = new net.minecraft.world.scores.Scoreboard();
        var objectives = new HashMap<String, net.minecraft.world.scores.Objective>();
        for (var element : row.has("objectives") ? row.getAsJsonArray("objectives") : new JsonArray()) {
            var described = element.getAsJsonObject();
            var name = described.get("name").getAsString();
            objectives.put(name, scoreboard.addObjective(name, net.minecraft.world.scores.criteria.ObjectiveCriteria.DUMMY,
                Component.literal(described.get("title").getAsString()),
                described.has("hearts") && described.get("hearts").getAsBoolean()
                    ? net.minecraft.world.scores.criteria.ObjectiveCriteria.RenderType.HEARTS
                    : net.minecraft.world.scores.criteria.ObjectiveCriteria.RenderType.INTEGER,
                false, described.has("format") ? numberFormat(described.get("format").getAsString()) : null));
        }
        for (var element : row.has("scores") ? row.getAsJsonArray("scores") : new JsonArray()) {
            var described = element.getAsJsonObject();
            var access = scoreboard.getOrCreatePlayerScore(
                net.minecraft.world.scores.ScoreHolder.forNameOnly(described.get("owner").getAsString()),
                objectives.get(described.get("objective").getAsString()), true);
            access.set(described.get("value").getAsInt());
            if (described.has("display")) {
                access.display(Component.literal(described.get("display").getAsString()));
            }
            if (described.has("format")) {
                access.numberFormatOverride(numberFormat(described.get("format").getAsString()));
            }
        }
        if (row.has("slots")) {
            for (var entry : row.getAsJsonObject("slots").entrySet()) {
                scoreboard.setDisplayObjective(net.minecraft.world.scores.DisplaySlot.values()[Integer.parseInt(entry.getKey())],
                    objectives.get(entry.getValue().getAsString()));
            }
        }
        if (row.has("teams")) {
            for (var element : row.getAsJsonArray("teams")) {
                var described = element.getAsJsonObject();
                var team = scoreboard.addPlayerTeam(described.get("name").getAsString());
                if (described.has("prefix")) {
                    team.setPlayerPrefix(Component.literal(described.get("prefix").getAsString()));
                }
                if (described.has("suffix")) {
                    team.setPlayerSuffix(Component.literal(described.get("suffix").getAsString()));
                }
                if (described.has("color")) {
                    team.setColor(java.util.Optional.of(net.minecraft.world.scores.TeamColor.values()[described.get("color").getAsInt()]));
                }
                if (described.has("visibility")) {
                    team.setNameTagVisibility(
                        net.minecraft.world.scores.Team.Visibility.values()[described.get("visibility").getAsInt()]);
                }
                if (described.has("friendly")) {
                    team.setSeeFriendlyInvisibles(described.get("friendly").getAsBoolean());
                }
                for (var player : described.getAsJsonArray("players")) {
                    scoreboard.addPlayerToTeam(player.getAsString(), team);
                }
            }
        }
        return scoreboard;
    }

    /** The sidebar the scoreboard and the local player's team choose. */
    static JsonObject sidebar(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var screen = screen(row.get("width").getAsInt(), row.get("height").getAsInt(), 0, 0);
            var minecraft = screen.minecraft();
            var listener = (net.minecraft.client.multiplayer.ClientPacketListener)
                unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
            set(listener, "scoreboard", scoreboard(row));
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            set(player, "connection", listener);
            String local = row.get("local").getAsString();
            set(player, "gameProfile", new com.mojang.authlib.GameProfile(
                java.util.UUID.nameUUIDFromBytes(local.getBytes()), local));
            set(minecraft, "player", player);
            var level = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class);
            set(level, "connection", listener);
            set(minecraft, "level", level);
            var method = net.minecraft.client.gui.Hud.class.getDeclaredMethod("extractScoreboardSidebar",
                GuiGraphicsExtractor.class, net.minecraft.client.DeltaTracker.class);
            method.setAccessible(true);
            method.invoke(screen.hud(), screen.graphics(), null);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The player list over the game: who is listed, how, and the header and footer. */
    static JsonObject tabList(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            var screen = screen(width, row.get("height").getAsInt(), 0, 0);
            var minecraft = screen.minecraft();
            var listener = (net.minecraft.client.multiplayer.ClientPacketListener)
                unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
            var scoreboard = scoreboard(row);
            set(listener, "scoreboard", scoreboard);
            var listed = new it.unimi.dsi.fastutil.objects.ReferenceOpenHashSet<net.minecraft.client.multiplayer.PlayerInfo>();
            set(listener, "listedPlayers", listed);
            set(listener, "onlineMode", false);
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            set(player, "connection", listener);
            set(minecraft, "player", player);
            var level = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class);
            set(level, "connection", listener);
            set(minecraft, "level", level);
            for (var playerElement : row.getAsJsonArray("players")) {
                var described = playerElement.getAsJsonObject();
                String name = described.get("name").getAsString();
                var info = new net.minecraft.client.multiplayer.PlayerInfo(new com.mojang.authlib.GameProfile(
                    java.util.UUID.nameUUIDFromBytes(name.getBytes()), name), false);
                set(info, "gameMode", net.minecraft.world.level.GameType.byId(described.get("mode").getAsInt()));
                set(info, "latency", described.get("latency").getAsInt());
                set(info, "tabListOrder", described.get("order").getAsInt());
                if (described.has("display")) {
                    info.setTabListDisplayName(Component.literal(described.get("display").getAsString()));
                }
                listed.add(info);
            }
            var overlay = new net.minecraft.client.gui.components.PlayerTabOverlay(minecraft,
                (net.minecraft.client.gui.Hud) screen.hud());
            if (row.has("header")) {
                overlay.setHeader(Component.literal(row.get("header").getAsString()));
            }
            if (row.has("footer")) {
                overlay.setFooter(Component.literal(row.get("footer").getAsString()));
            }
            overlay.extractRenderState(screen.graphics(), width, scoreboard,
                scoreboard.getDisplayObjective(net.minecraft.world.scores.DisplaySlot.LIST));
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A sign face from a request: four lines as component JSON, the filtered ones, dye, glow. */
    static net.minecraft.world.level.block.entity.SignText signText(JsonObject face) {
        java.util.function.Function<com.google.gson.JsonArray, Component[]> lines = array -> {
            var out = new Component[4];
            for (int i = 0; i < 4; i++) {
                out[i] = net.minecraft.network.chat.ComponentSerialization.CODEC
                    .parse(com.mojang.serialization.JsonOps.INSTANCE, array.get(i)).getOrThrow();
            }
            return out;
        };
        var messages = lines.apply(face.getAsJsonArray("messages"));
        var filtered = face.get("filtered").isJsonNull() ? messages.clone() : lines.apply(face.getAsJsonArray("filtered"));
        return new net.minecraft.world.level.block.entity.SignText(messages, filtered,
            net.minecraft.world.item.DyeColor.valueOf(face.get("color").getAsString().toUpperCase()),
            face.get("glowing").getAsBoolean());
    }

    /**
     * What the real `StandingSignRenderer` or `HangingSignRenderer` submits for a sign: each
     * line's pose, place, text (as runs of one colour), light, colours and how it is drawn.
     */
    static JsonObject signTexts(JsonObject request) throws Exception {
        var font = loadedFont();
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            boolean hanging = row.get("kind").getAsString().equals("hanging");
            var wall = row.get("wall").isJsonNull() ? null
                : net.minecraft.core.Direction.valueOf(row.get("wall").getAsString().toUpperCase());
            int rotation = row.get("rotation").getAsInt();
            net.minecraft.client.renderer.blockentity.AbstractSignRenderer<?> renderer;
            net.minecraft.client.renderer.blockentity.state.SignRenderState state;
            if (hanging) {
                renderer = (net.minecraft.client.renderer.blockentity.AbstractSignRenderer<?>) unsafe.allocateInstance(
                    net.minecraft.client.renderer.blockentity.HangingSignRenderer.class);
                var t = net.minecraft.client.renderer.blockentity.HangingSignRenderer.TRANSFORMATIONS;
                state = new net.minecraft.client.renderer.blockentity.state.HangingSignRenderState();
                state.transformations = wall == null ? t.freeTransformations(rotation) : t.wallTransformation(wall);
                state.maxTextLineWidth = 60;
                state.textLineHeight = 9;
            } else {
                renderer = (net.minecraft.client.renderer.blockentity.AbstractSignRenderer<?>) unsafe.allocateInstance(
                    net.minecraft.client.renderer.blockentity.StandingSignRenderer.class);
                var t = net.minecraft.client.renderer.blockentity.StandingSignRenderer.TRANSFORMATIONS;
                state = new net.minecraft.client.renderer.blockentity.state.StandingSignRenderState();
                state.transformations = wall == null ? t.freeTransformations(rotation) : t.wallTransformation(wall);
                state.maxTextLineWidth = 90;
                state.textLineHeight = 10;
            }
            MovementOracle.setField(renderer, "font", font);
            state.frontText = signText(row.getAsJsonObject("front"));
            state.backText = signText(row.getAsJsonObject("back"));
            state.isTextFilteringEnabled = row.get("filtering").getAsBoolean();
            state.drawOutline = row.get("draw_outline").getAsBoolean();
            state.lightCoords = row.get("light").getAsInt();
            var submits = new JsonArray();
            var collector = (net.minecraft.client.renderer.SubmitNodeCollector) java.lang.reflect.Proxy.newProxyInstance(
                GuiOverlayOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.SubmitNodeCollector.class},
                (proxy, method, args) -> {
                    if (!method.getName().equals("submitText")) {
                        throw new IllegalStateException(method.getName());
                    }
                    var out = new JsonObject();
                    var pose = new float[16];
                    ((com.mojang.blaze3d.vertex.PoseStack) args[0]).last().pose().get(pose);
                    var matrix = new JsonArray();
                    for (float value : pose) {
                        matrix.add(Oracle.bits(value));
                    }
                    out.add("pose", matrix);
                    out.addProperty("x", Oracle.bits((Float) args[1]));
                    out.addProperty("y", Oracle.bits((Float) args[2]));
                    // The line as runs of one colour (-1 for none).
                    var runs = new JsonArray();
                    var text = new StringBuilder();
                    int[] colour = {Integer.MIN_VALUE};
                    ((net.minecraft.util.FormattedCharSequence) args[3]).accept((position, style, codepoint) -> {
                        int next = style.getColor() == null ? -1 : style.getColor().getValue();
                        if (next != colour[0] && text.length() > 0) {
                            var run = new JsonArray();
                            run.add(text.toString());
                            run.add(colour[0]);
                            runs.add(run);
                            text.setLength(0);
                        }
                        colour[0] = next;
                        text.appendCodePoint(codepoint);
                        return true;
                    });
                    if (text.length() > 0) {
                        var run = new JsonArray();
                        run.add(text.toString());
                        run.add(colour[0]);
                        runs.add(run);
                    }
                    out.add("runs", runs);
                    out.addProperty("shadow", (Boolean) args[4]);
                    out.addProperty("mode", args[5].toString());
                    out.addProperty("light", (Integer) args[6]);
                    out.addProperty("color", Integer.toHexString((Integer) args[7]));
                    out.addProperty("background", (Integer) args[8]);
                    out.addProperty("outline", Integer.toHexString((Integer) args[9]));
                    submits.add(out);
                    return null;
                });
            @SuppressWarnings("unchecked")
            var typed = (net.minecraft.client.renderer.blockentity.AbstractSignRenderer<
                net.minecraft.client.renderer.blockentity.state.SignRenderState>) renderer;
            typed.submit(state, new com.mojang.blaze3d.vertex.PoseStack(), collector, null);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("submits", submits);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
