import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.BlockPos;

/**
 * Test glue only: runs the original client player in a world the test builds.
 *
 * <p>The player here is the reference's own {@code LocalPlayer}, in the
 * reference's own {@code ClientLevel}, with the reference's own keyboard
 * input. Registries and tags come from the vanilla data pack through the same
 * loading path the server uses, and chunks arrive through the client chunk
 * cache's packet entry point. What this file supplies is only what a headless
 * run has no other way to get: the game singleton, an options object, a
 * connection that records what it is asked to send rather than sending it,
 * and a sound manager that plays nothing.
 */
public final class MovementOracle {
    private static boolean bootstrapped;
    private static net.minecraft.core.RegistryAccess.Frozen registries;
    private static sun.misc.Unsafe unsafe;

    /** Records what the player sends instead of putting it on a wire. */
    static final class CapturingConnection extends net.minecraft.network.Connection {
        final java.util.List<net.minecraft.network.protocol.Packet<?>> sent =
            new java.util.ArrayList<>();

        CapturingConnection() {
            super(net.minecraft.network.protocol.PacketFlow.CLIENTBOUND);
        }

        @Override
        public void send(net.minecraft.network.protocol.Packet<?> packet) {
            this.sent.add(packet);
        }
    }

    /** Swallows sounds: movement plays step sounds, and there is no speaker. */
    static final class SilentSoundManager extends net.minecraft.client.sounds.SoundManager {
        private SilentSoundManager() {
            super(null);
        }

        @Override
        public net.minecraft.client.sounds.SoundEngine.PlayResult play(
            net.minecraft.client.resources.sounds.SoundInstance instance) {
            return net.minecraft.client.sounds.SoundEngine.PlayResult.NOT_STARTED;
        }

        @Override
        public void playDelayed(
            net.minecraft.client.resources.sounds.SoundInstance instance, int delay) {}
    }

    private static void setField(Object target, String name, Object value) throws Exception {
        Class<?> type = target.getClass();
        while (type != null) {
            try {
                var field = type.getDeclaredField(name);
                field.setAccessible(true);
                field.set(target, value);
                return;
            } catch (NoSuchFieldException missing) {
                type = type.getSuperclass();
            }
        }
        throw new NoSuchFieldException(name);
    }

    /** Bring the game up once, with the vanilla data pack's registries and tags. */
    private static synchronized void bootstrap() throws Exception {
        if (bootstrapped) {
            return;
        }
        registries = Oracle.vanillaRegistries();
        var unsafeField = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
        unsafeField.setAccessible(true);
        unsafe = (sun.misc.Unsafe) unsafeField.get(null);
        bootstrapped = true;
    }

    public static JsonObject run(JsonObject request) throws Exception {
        bootstrap();
        String operation = request.get("operation").getAsString();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            switch (operation) {
                case "local_player_ticks" -> rows.add(localPlayerTicks(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException(
                    "Unknown movement operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static net.minecraft.world.level.block.state.BlockState state(JsonObject described) {
        var block = net.minecraft.core.registries.BuiltInRegistries.BLOCK.getValue(
            net.minecraft.resources.Identifier.parse(described.get("block").getAsString()));
        var state = block.defaultBlockState();
        if (described.has("properties")) {
            for (var entry : described.getAsJsonObject("properties").entrySet()) {
                state = withProperty(state, entry.getKey(), entry.getValue().getAsString());
            }
        }
        return state;
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    private static net.minecraft.world.level.block.state.BlockState withProperty(
        net.minecraft.world.level.block.state.BlockState state, String name, String value) {
        var property = state.getBlock().getStateDefinition().getProperty(name);
        if (property == null) {
            throw new IllegalArgumentException("No property " + name);
        }
        var parsed = property.getValue(value)
            .orElseThrow(() -> new IllegalArgumentException("Bad value " + value));
        return state.setValue((net.minecraft.world.level.block.state.properties.Property) property,
            (Comparable) parsed);
    }

    /**
     * Step the reference's own player through a scene, tick by tick.
     *
     * The keys held and the look direction are set before each tick, exactly
     * where the game's frame loop would have left them; then the level ticks
     * the player the way it ticks every entity. After every tick the position, the motion, the flags
     * and every packet the player asked to send are reported, the packets as
     * the bytes their own codec writes.
     */
    private static JsonObject localPlayerTicks(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());

        var listener = (net.minecraft.client.multiplayer.ClientPacketListener)
            unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
        setField(listener, "registryAccess", registries);
        setField(listener, "clockManager", new net.minecraft.client.ClientClockManager());
        setField(listener, "localGameProfile", new com.mojang.authlib.GameProfile(
            java.util.UUID.nameUUIDFromBytes(new byte[] {1}), "Parity"));
        var wire = new CapturingConnection();
        setField(listener, "connection", wire);
        setField(listener, "clientLoaded", true);

        var dimensionType = registries.lookupOrThrow(
            net.minecraft.core.registries.Registries.DIMENSION_TYPE).getOrThrow(
            net.minecraft.world.level.dimension.BuiltinDimensionTypes.OVERWORLD);
        var level = new net.minecraft.client.multiplayer.ClientLevel(
            listener,
            new net.minecraft.client.multiplayer.ClientLevel.ClientLevelData(
                net.minecraft.world.Difficulty.PEACEFUL, false, false),
            net.minecraft.world.level.Level.OVERWORLD, dimensionType, 8, 8, null, false, 0L, 63);

        var minecraft = (net.minecraft.client.Minecraft)
            unsafe.allocateInstance(net.minecraft.client.Minecraft.class);
        var instanceField = net.minecraft.client.Minecraft.class.getDeclaredField("instance");
        instanceField.setAccessible(true);
        instanceField.set(null, minecraft);
        setField(level, "minecraft", minecraft);
        var workDir = java.nio.file.Files.createTempDirectory("oracle-options").toFile();
        var options = new net.minecraft.client.Options(minecraft, workDir);
        options.autoJump().set(row.get("auto_jump").getAsBoolean());
        setField(minecraft, "options", options);
        setField(minecraft, "gui", unsafe.allocateInstance(net.minecraft.client.gui.Gui.class));
        setField(minecraft, "tutorial", new net.minecraft.client.tutorial.Tutorial(minecraft, options));
        setField(minecraft, "soundManager", unsafe.allocateInstance(SilentSoundManager.class));

        // Chunks go in through the client cache's packet entry point, as the
        // sections the reference's own section code builds and writes.
        var blocks = new java.util.HashMap<Long, net.minecraft.world.level.block.state.BlockState>();
        for (var element : row.get("blocks").getAsJsonArray()) {
            var described = element.getAsJsonObject();
            blocks.put(BlockPos.asLong(described.get("x").getAsInt(), described.get("y").getAsInt(),
                described.get("z").getAsInt()), state(described));
        }
        int radius = row.get("chunk_radius").getAsInt();
        for (int chunkX = -radius; chunkX <= radius; chunkX++) {
            for (int chunkZ = -radius; chunkZ <= radius; chunkZ++) {
                var buffer = new net.minecraft.network.FriendlyByteBuf(io.netty.buffer.Unpooled.buffer());
                for (int index = 0; index < level.getSectionsCount(); index++) {
                    var section = new net.minecraft.world.level.chunk.LevelChunkSection(
                        level.palettedContainerFactory());
                    int sectionMinY = level.getMinY() + index * 16;
                    for (int x = 0; x < 16; x++) {
                        for (int y = 0; y < 16; y++) {
                            for (int z = 0; z < 16; z++) {
                                var state = blocks.get(BlockPos.asLong(
                                    chunkX * 16 + x, sectionMinY + y, chunkZ * 16 + z));
                                if (state != null) {
                                    section.setBlockState(x, y, z, state);
                                }
                            }
                        }
                    }
                    section.write(buffer);
                }
                level.getChunkSource().replaceWithPacketData(
                    chunkX, chunkZ, buffer, java.util.Map.of(), output -> {});
            }
        }

        var player = new net.minecraft.client.player.LocalPlayer(
            minecraft, level, listener, null, null,
            net.minecraft.world.entity.player.Input.EMPTY, false, null);
        // The server hands out the entity id at login; the scenario names it,
        // because the sprint command packet carries it.
        player.setId(row.get("entity_id").getAsInt());
        setField(minecraft, "player", player);
        var info = new net.minecraft.client.multiplayer.PlayerInfo(listener.getLocalGameProfile(), false);
        var gameType = net.minecraft.world.level.GameType.byName(row.get("game_mode").getAsString());
        var setMode = net.minecraft.client.multiplayer.PlayerInfo.class.getDeclaredMethod(
            "setGameMode", net.minecraft.world.level.GameType.class);
        setMode.setAccessible(true);
        setMode.invoke(info, gameType);
        setField(player, "playerInfo", info);
        var gameMode = new net.minecraft.client.multiplayer.MultiPlayerGameMode(minecraft, listener);
        gameMode.setLocalMode(gameType);
        gameMode.adjustPlayer(player);
        setField(minecraft, "gameMode", gameMode);
        var renderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        var camera = new net.minecraft.client.Camera();
        setField(renderer, "mainCamera", camera);
        setField(minecraft, "gameRenderer", renderer);
        camera.setEntity(player);
        player.input = new net.minecraft.client.player.KeyboardInput(options);
        player.snapTo(
            Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"),
            Oracle.f32(row, "y_rot"), Oracle.f32(row, "x_rot"));

        var ticks = new JsonArray();
        for (var element : row.get("ticks").getAsJsonArray()) {
            var input = element.getAsJsonObject();
            options.keyUp.setDown(input.get("forward").getAsBoolean());
            options.keyDown.setDown(input.get("backward").getAsBoolean());
            options.keyLeft.setDown(input.get("left").getAsBoolean());
            options.keyRight.setDown(input.get("right").getAsBoolean());
            options.keyJump.setDown(input.get("jump").getAsBoolean());
            options.keyShift.setDown(input.get("sneak").getAsBoolean());
            options.keySprint.setDown(input.get("sprint").getAsBoolean());
            if (input.has("y_rot")) {
                player.setYRot(Oracle.f32(input, "y_rot"));
            }
            if (input.has("x_rot")) {
                player.setXRot(Oracle.f32(input, "x_rot"));
            }
            wire.sent.clear();
            // The level ticks its entities, not the entity itself: it saves
            // the old position and rotation and counts the tick first, and
            // both are read during the tick.
            level.tickNonPassenger(player);
            ticks.add(describe(player, wire.sent));
        }
        entry.addProperty("outcome", "ticked");
        entry.add("ticks", ticks);
        return entry;
    }

    private static JsonObject describe(
        net.minecraft.client.player.LocalPlayer player,
        java.util.List<net.minecraft.network.protocol.Packet<?>> sent) throws Exception {
        var tick = new JsonObject();
        tick.addProperty("x", Oracle.bits(player.getX()));
        tick.addProperty("y", Oracle.bits(player.getY()));
        tick.addProperty("z", Oracle.bits(player.getZ()));
        tick.addProperty("motion_x", Oracle.bits(player.getDeltaMovement().x));
        tick.addProperty("motion_y", Oracle.bits(player.getDeltaMovement().y));
        tick.addProperty("motion_z", Oracle.bits(player.getDeltaMovement().z));
        tick.addProperty("y_rot", Oracle.bits(player.getYRot()));
        tick.addProperty("x_rot", Oracle.bits(player.getXRot()));
        tick.addProperty("on_ground", player.onGround());
        tick.addProperty("horizontal_collision", player.horizontalCollision);
        tick.addProperty("vertical_collision", player.verticalCollision);
        tick.addProperty("minor_horizontal_collision", player.minorHorizontalCollision);
        tick.addProperty("sprinting", player.isSprinting());
        tick.addProperty("shift_key_down", player.isShiftKeyDown());
        tick.addProperty("pose", player.getPose().name());
        tick.addProperty("fall_distance", Oracle.bits(player.fallDistance));
        tick.addProperty("flying", player.getAbilities().flying);
        var packets = new JsonArray();
        for (var packet : sent) {
            var described = new JsonObject();
            described.addProperty("type", packet.type().id().toString());
            described.addProperty("bytes", encode(packet));
            packets.add(described);
        }
        tick.add("sent", packets);
        return tick;
    }

    /** A packet as the bytes its own codec writes, without the packet id. */
    @SuppressWarnings({"unchecked", "rawtypes"})
    private static String encode(net.minecraft.network.protocol.Packet<?> packet) throws Exception {
        Class<?> type = packet.getClass();
        java.lang.reflect.Field codecField = null;
        while (type != null && codecField == null) {
            try {
                codecField = type.getDeclaredField("STREAM_CODEC");
            } catch (NoSuchFieldException missing) {
                type = type.getSuperclass();
            }
        }
        if (codecField == null) {
            throw new IllegalStateException("No codec on " + packet.getClass());
        }
        codecField.setAccessible(true);
        var codec = (net.minecraft.network.codec.StreamCodec) codecField.get(null);
        var buffer = new net.minecraft.network.RegistryFriendlyByteBuf(
            io.netty.buffer.Unpooled.buffer(), registries);
        codec.encode(buffer, packet);
        var bytes = new byte[buffer.readableBytes()];
        buffer.readBytes(bytes);
        return java.util.HexFormat.of().formatHex(bytes);
    }
}
