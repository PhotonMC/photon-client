import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.BlockPos;

/** What a headless run lacks: the game singleton, options, a recording connection, a silent sound manager, and the scene every row builds. */
class MovementSupport {
    static boolean bootstrapped;
    static net.minecraft.core.RegistryAccess.Frozen registries;
    static sun.misc.Unsafe unsafe;

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
        SilentSoundManager() {
            super(null);
        }

        /** The sounds asked for since the last tick was described: the sound, source, volume, pitch and place. */
        static final java.util.List<String> PLAYED = new java.util.ArrayList<>();

        @Override
        public net.minecraft.client.sounds.SoundEngine.PlayResult play(
            net.minecraft.client.resources.sounds.SoundInstance instance) {
            // The volume and pitch asked for: the instance has not been resolved against a sound.
            float volume;
            float pitch;
            try {
                var volumeField = net.minecraft.client.resources.sounds.AbstractSoundInstance.class.getDeclaredField("volume");
                var pitchField = net.minecraft.client.resources.sounds.AbstractSoundInstance.class.getDeclaredField("pitch");
                volumeField.setAccessible(true);
                pitchField.setAccessible(true);
                volume = (float) volumeField.get(instance);
                pitch = (float) pitchField.get(instance);
            } catch (ReflectiveOperationException failure) {
                throw new IllegalStateException(failure);
            }
            // The player's ambient sounds (underwater additions, the cave's) are drawn by chance from a
            // random source the scene does not seed: they are left out.
            // The honey block's slide sound while sliding down it is a one-in-five chance each tick, from
            // the level's random; the one of landing on it is certain and is kept.
            boolean chanceSlide = false;
            for (StackTraceElement frame : Thread.currentThread().getStackTrace()) {
                chanceSlide |= frame.getMethodName().equals("maybeDoSlideEffects");
            }
            // (A firework's blast is ambient too, and is drawn from a seeded starter: it is kept.)
            boolean fireworkSound = instance.getIdentifier().toString().startsWith("minecraft:entity.firework_rocket.");
            // (Getting into and out of water is certain, and is kept; the loop and the additions are not.)
            boolean waterChange = instance.getIdentifier().toString().equals("minecraft:ambient.underwater.enter")
                || instance.getIdentifier().toString().equals("minecraft:ambient.underwater.exit");
            // (The portal travel sound of a level event is drawn from the level's seeded random: kept.)
            boolean portalTravel = instance.getIdentifier().toString().equals("minecraft:block.portal.travel");
            if (instance.getSource() == net.minecraft.sounds.SoundSource.AMBIENT && !fireworkSound && !waterChange && !portalTravel || chanceSlide) {
                return net.minecraft.client.sounds.SoundEngine.PlayResult.NOT_STARTED;
            }
            // The swim, splash, hurt and death sounds' pitch is drawn from the entity's own random source, which the scene
            // does not seed: it is compared by volume and place only.
            String played = instance.getIdentifier().toString();
            if (played.equals("minecraft:entity.player.swim") || played.startsWith("minecraft:entity.player.splash")
                || played.startsWith("minecraft:entity.player.hurt") || played.equals("minecraft:entity.player.death")) {
                pitch = 0.0F;
            }
            PLAYED.add(instance.getIdentifier() + "," + instance.getSource().getName() + ","
                + Oracle.bits(volume) + "," + Oracle.bits(pitch) + ","
                + Oracle.bits(instance.getX()) + "," + Oracle.bits(instance.getY()) + "," + Oracle.bits(instance.getZ()));
            return net.minecraft.client.sounds.SoundEngine.PlayResult.NOT_STARTED;
        }

        @Override
        public void playDelayed(
            net.minecraft.client.resources.sounds.SoundInstance instance, int delay) {
            // A sound heard from afar is delayed by the distance; here it is kept as asked for, the
            // delay being the sound engine's.
            this.play(instance);
        }
    }

    /**
     * Breaking a block spawns terrain particles, which look up the block's
     * sprite and tint. A headless run has no atlas, so these stand in: every
     * block's particle sprite is one blank terrain sprite, and particles are
     * dropped when added. Nothing breaking decides reads them back.
     */
    static final class BlankSprite extends net.minecraft.client.renderer.texture.TextureAtlasSprite {
        BlankSprite() {
            super(null, null, 1, 1, 0, 0, 0);
        }

        @Override
        public com.mojang.blaze3d.platform.Transparency transparency() {
            return new com.mojang.blaze3d.platform.Transparency(false, false);
        }

        @Override
        public net.minecraft.resources.Identifier atlasLocation() {
            return net.minecraft.client.renderer.texture.TextureAtlas.LOCATION_BLOCKS;
        }
    }

    static final class BlankModels extends net.minecraft.client.renderer.block.BlockStateModelSet {
        static net.minecraft.client.renderer.texture.TextureAtlasSprite sprite;

        BlankModels() {
            super(null, null);
        }

        @Override
        public net.minecraft.client.resources.model.sprite.Material.Baked getParticleMaterial(
            net.minecraft.world.level.block.state.BlockState state) {
            return new net.minecraft.client.resources.model.sprite.Material.Baked(sprite, false);
        }
    }

    static final class BlankModelManager extends net.minecraft.client.resources.model.ModelManager {
        static net.minecraft.client.renderer.block.BlockStateModelSet models;

        BlankModelManager() {
            super(null, null, null);
        }

        @Override
        public net.minecraft.client.renderer.block.BlockStateModelSet getBlockStateModelSet() {
            return models;
        }
    }

    static final class DroppingParticleEngine extends net.minecraft.client.particle.ParticleEngine {
        DroppingParticleEngine() {
            super(null, null);
        }

        @Override
        public void add(net.minecraft.client.particle.Particle particle) {}

        @Override
        public net.minecraft.client.particle.Particle createParticle(
            net.minecraft.core.particles.ParticleOptions options,
            double x, double y, double z, double xd, double yd, double zd) {
            return null;
        }
    }

    /** The level tells the renderer which sections to rebuild; nothing draws here. */
    static final class NoRebuilds extends net.minecraft.client.renderer.extract.LevelExtractor {
        NoRebuilds() {
            super(null, null, null);
        }

        @Override
        public void blockChanged(BlockPos pos, int updateFlags) {}

        @Override
        public void setBlockDirty(BlockPos pos,
            net.minecraft.world.level.block.state.BlockState oldState,
            net.minecraft.world.level.block.state.BlockState newState) {}

        @Override
        public void setSectionDirtyWithNeighbors(int sectionX, int sectionY, int sectionZ) {}

        @Override
        public void setSectionRangeDirty(int minX, int minY, int minZ, int maxX, int maxY, int maxZ) {}

        @Override
        public void setSectionDirty(int sectionX, int sectionY, int sectionZ) {}
    }

    static Object getField(Object target, String name) throws Exception {
        Class<?> type = target.getClass();
        while (type != null) {
            try {
                var field = type.getDeclaredField(name);
                field.setAccessible(true);
                return field.get(target);
            } catch (NoSuchFieldException missing) {
                type = type.getSuperclass();
            }
        }
        throw new NoSuchFieldException(name);
    }

    static void setField(Object target, String name, Object value) throws Exception {
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
    static synchronized void bootstrap() throws Exception {
        if (bootstrapped) {
            return;
        }
        registries = Oracle.vanillaRegistries();
        var unsafeField = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
        unsafeField.setAccessible(true);
        unsafe = (sun.misc.Unsafe) unsafeField.get(null);
        bootstrapped = true;
    }

    static net.minecraft.world.level.block.state.BlockState state(JsonObject described) {
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
    static net.minecraft.world.level.block.state.BlockState withProperty(
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

    /** A client level with the scene's blocks, and the game around it. */
    record Scene(
        net.minecraft.client.multiplayer.ClientPacketListener listener,
        CapturingConnection wire,
        net.minecraft.client.multiplayer.ClientLevel level,
        net.minecraft.client.Minecraft minecraft,
        net.minecraft.client.Options options) {}

    static Scene buildScene(JsonObject row) throws Exception {
        var listener = (net.minecraft.client.multiplayer.ClientPacketListener)
            unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
        setField(listener, "registryAccess", registries);
        setField(listener, "clockManager", new net.minecraft.client.ClientClockManager());
        setField(listener, "localGameProfile", new com.mojang.authlib.GameProfile(
            java.util.UUID.nameUUIDFromBytes(new byte[] {1}), "Parity"));
        var wire = new CapturingConnection();
        setField(listener, "connection", wire);
        setField(listener, "clientLoaded", true);
        // What a brewing stand's ingredient slot asks the level.
        setField(listener, "potionBrewing", net.minecraft.world.item.alchemy.PotionBrewing.bootstrap(
            net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS));

        var dimensionType = registries.lookupOrThrow(
            net.minecraft.core.registries.Registries.DIMENSION_TYPE).getOrThrow(
            net.minecraft.world.level.dimension.BuiltinDimensionTypes.OVERWORLD);
        var level = new net.minecraft.client.multiplayer.ClientLevel(
            listener,
            new net.minecraft.client.multiplayer.ClientLevel.ClientLevelData(
                net.minecraft.world.Difficulty.PEACEFUL, false, false),
            net.minecraft.world.level.Level.OVERWORLD, dimensionType, 8, 8, null, false, 0L, 63);

        if (row.has("border")) {
            // The world border the server set, as the packets leave it.
            var border = row.getAsJsonObject("border");
            level.getWorldBorder().setCenter(Oracle.f64(border, "x"), Oracle.f64(border, "z"));
            level.getWorldBorder().setSize(Oracle.f64(border, "size"));
        }

        var minecraft = (net.minecraft.client.Minecraft)
            unsafe.allocateInstance(net.minecraft.client.Minecraft.class);
        var instanceField = net.minecraft.client.Minecraft.class.getDeclaredField("instance");
        instanceField.setAccessible(true);
        instanceField.set(null, minecraft);
        setField(level, "minecraft", minecraft);
        var workDir = java.nio.file.Files.createTempDirectory("oracle-options").toFile();
        var options = new net.minecraft.client.Options(minecraft, workDir);
        options.autoJump().set(row.has("auto_jump") && row.get("auto_jump").getAsBoolean());
        setField(minecraft, "options", options);
        setField(minecraft, "gui", unsafe.allocateInstance(net.minecraft.client.gui.Gui.class));
        // `hasControlDown` asks the window's keys: a window with no keys down.
        var window = unsafe.allocateInstance(com.mojang.blaze3d.platform.Window.class);
        setField(window, "handle", 1L);
        setField(minecraft, "window", window);
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

        return new Scene(listener, wire, level, minecraft, options);
    }

    static net.minecraft.world.item.ItemStack stack(JsonObject described) {
        var item = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
            net.minecraft.resources.Identifier.parse(described.get("item").getAsString()));
        var stack = new net.minecraft.world.item.ItemStack(item, described.get("count").getAsInt());
        if (described.has("patch")) {
            // The component patch as the wire carries it.
            var patch = described.get("patch").getAsString();
            var bytes = new byte[patch.length() / 2];
            for (int index = 0; index < bytes.length; index++) {
                bytes[index] = (byte) Integer.parseInt(patch.substring(index * 2, index * 2 + 2), 16);
            }
            var buffer = new net.minecraft.network.RegistryFriendlyByteBuf(
                io.netty.buffer.Unpooled.wrappedBuffer(bytes), Oracle.vanillaRegistries());
            stack.applyComponents(net.minecraft.core.component.DataComponentPatch.STREAM_CODEC.decode(buffer));
        }
        if (described.has("book_pages")) {
            // A written book of these pages (plain text).
            var pages = new java.util.ArrayList<net.minecraft.server.network.Filterable<net.minecraft.network.chat.Component>>();
            for (var page : described.getAsJsonArray("book_pages")) {
                pages.add(net.minecraft.server.network.Filterable.passThrough(
                    net.minecraft.network.chat.Component.literal(page.getAsString())));
            }
            stack.set(net.minecraft.core.component.DataComponents.WRITTEN_BOOK_CONTENT,
                new net.minecraft.world.item.component.WrittenBookContent(
                    net.minecraft.server.network.Filterable.passThrough("Title"), "Author", 0, pages, true));
        }
        return stack;
    }

    static com.google.gson.JsonElement describeStack(net.minecraft.world.item.ItemStack stack) {
        if (stack.isEmpty()) {
            return com.google.gson.JsonNull.INSTANCE;
        }
        var out = new JsonObject();
        out.addProperty("item", net.minecraft.core.registries.BuiltInRegistries.ITEM.getKey(stack.getItem()).toString());
        out.addProperty("count", stack.getCount());
        return out;
    }

    /** A packet as the bytes its own codec writes, without the packet id. */
    @SuppressWarnings({"unchecked", "rawtypes"})
    static String encode(net.minecraft.network.protocol.Packet<?> packet) throws Exception {
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

    /** One vector as three hex words of its doubles. */
    static JsonArray vectorBits(net.minecraft.world.phys.Vec3 vector) {
        var bits = new JsonArray();
        bits.add(String.format("%016x", Double.doubleToRawLongBits(vector.x)));
        bits.add(String.format("%016x", Double.doubleToRawLongBits(vector.y)));
        bits.add(String.format("%016x", Double.doubleToRawLongBits(vector.z)));
        return bits;
    }
}
