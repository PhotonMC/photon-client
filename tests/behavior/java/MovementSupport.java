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
            // A piston's sound is asked of the level for everyone but the player, who hears the
            // server's: the real client plays none, and its player is not in this scene.
            if (instance.getIdentifier().toString().startsWith("minecraft:block.piston.")) {
                return net.minecraft.client.sounds.SoundEngine.PlayResult.NOT_STARTED;
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
            // (The portal's trigger sound is certain when the effect starts; its pitch is the player's own random.)
            boolean portalTrigger = instance.getIdentifier().toString().equals("minecraft:block.portal.trigger");
            if (instance.getSource() == net.minecraft.sounds.SoundSource.AMBIENT && !fireworkSound && !waterChange && !portalTravel && !portalTrigger || chanceSlide) {
                return net.minecraft.client.sounds.SoundEngine.PlayResult.NOT_STARTED;
            }
            // The swim, splash, hurt and death sounds' pitch is drawn from the entity's own random source, which the scene
            // does not seed: it is compared by volume and place only.
            String played = instance.getIdentifier().toString();
            if (played.equals("minecraft:entity.player.swim") || played.startsWith("minecraft:entity.player.splash")
                || played.startsWith("minecraft:entity.player.hurt") || played.equals("minecraft:entity.player.death")
                || played.equals("minecraft:block.portal.trigger")) {
                pitch = 0.0F;
            }
            PLAYED.add(instance.getIdentifier() + "," + instance.getSource().getName() + ","
                + Oracle.bits(volume) + "," + Oracle.bits(pitch) + ","
                + Oracle.bits(instance.getX()) + "," + Oracle.bits(instance.getY()) + "," + Oracle.bits(instance.getZ())
                + (this.recordedDelay > 0 ? "," + this.recordedDelay : ""));
            return net.minecraft.client.sounds.SoundEngine.PlayResult.NOT_STARTED;
        }

        /** A song that ends: the sound and where it played from. */
        @Override
        public void stop(net.minecraft.client.resources.sounds.SoundInstance instance) {
            PLAYED.add("stop," + instance.getIdentifier() + ","
                + Oracle.bits(instance.getX()) + "," + Oracle.bits(instance.getY()) + "," + Oracle.bits(instance.getZ()));
        }

        /** The delay of the sound being recorded, in ticks; 0 when it is played at once. */
        private int recordedDelay;

        @Override
        public void playDelayed(
            net.minecraft.client.resources.sounds.SoundInstance instance, int delay) {
            // A sound heard from afar is delayed by the distance: kept as asked for, with its delay.
            this.recordedDelay = delay;
            try {
                this.play(instance);
            } finally {
                this.recordedDelay = 0;
            }
        }
    }

    /** A hud that notes which record's name it is asked to show. */
    static final class RecordingHud extends net.minecraft.client.gui.Hud {
        RecordingHud() {
            super(null);
        }

        @Override
        public void setNowPlaying(net.minecraft.network.chat.Component description) {
            var contents = description.getContents();
            SilentSoundManager.PLAYED.add("now_playing," + (contents instanceof
                net.minecraft.network.chat.contents.TranslatableContents translatable
                ? translatable.getKey() : description.getString()));
        }
    }

    /** Says nothing: a record's name is shown on the hud and spoken by the narrator. */
    static final class SilentNarrator extends net.minecraft.client.GameNarrator {
        SilentNarrator() {
            super(null);
        }

        @Override
        public void saySystemNow(net.minecraft.network.chat.Component message) {}
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
        // What a block entity asks the registries for (`Level.enabledFeatures`).
        setField(listener, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
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

        // A block that changes tells the renderer which section to rebuild; nothing draws here.
        var rebuilds = unsafe.allocateInstance(NoRebuilds.class);
        setField(level, "levelExtractor", rebuilds);

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
        var gui = unsafe.allocateInstance(net.minecraft.client.gui.Gui.class);
        var hud = unsafe.allocateInstance(RecordingHud.class);
        setField(hud, "minecraft", minecraft);
        setField(gui, "hud", hud);
        setField(minecraft, "gui", gui);
        setField(minecraft, "narrator", unsafe.allocateInstance(SilentNarrator.class));
        // `hasControlDown` asks the window's keys: a window with no keys down.
        var window = unsafe.allocateInstance(com.mojang.blaze3d.platform.Window.class);
        setField(window, "handle", 1L);
        setField(minecraft, "window", window);
        setField(minecraft, "tutorial", new net.minecraft.client.tutorial.Tutorial(minecraft, options));
        setField(minecraft, "levelExtractor", rebuilds);
        setField(minecraft, "soundManager", unsafe.allocateInstance(SilentSoundManager.class));
        // A block entity the level makes asks the dispatcher for its renderer: none draws.
        var dispatcher = unsafe.allocateInstance(
            net.minecraft.client.renderer.blockentity.BlockEntityRenderDispatcher.class);
        setField(dispatcher, "renderers", java.util.Map.of());
        setField(minecraft, "blockEntityRenderDispatcher", dispatcher);

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
                    if (row.has("biome")) {
                        // The whole scene is one biome (it is the plains where a row says nothing).
                        var biome = registries.lookupOrThrow(net.minecraft.core.registries.Registries.BIOME)
                            .getOrThrow(net.minecraft.resources.ResourceKey.create(
                                net.minecraft.core.registries.Registries.BIOME,
                                net.minecraft.resources.Identifier.parse(row.get("biome").getAsString())));
                        @SuppressWarnings("unchecked")
                        var biomes = (net.minecraft.world.level.chunk.PalettedContainer<
                            net.minecraft.core.Holder<net.minecraft.world.level.biome.Biome>>) section.getBiomes();
                        for (int x = 0; x < 4; x++) {
                            for (int y = 0; y < 4; y++) {
                                for (int z = 0; z < 4; z++) {
                                    biomes.set(x, y, z, biome);
                                }
                            }
                        }
                    }
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
                // The heightmaps a row sends as the server's packet carries them (raw longs).
                var sent = new java.util.EnumMap<net.minecraft.world.level.levelgen.Heightmap.Types, long[]>(
                    net.minecraft.world.level.levelgen.Heightmap.Types.class);
                if (row.has("heightmaps")) {
                    for (var element : row.getAsJsonArray("heightmaps")) {
                        var described = element.getAsJsonObject();
                        var at = described.getAsJsonArray("chunk");
                        if (at.get(0).getAsInt() != chunkX || at.get(1).getAsInt() != chunkZ) {
                            continue;
                        }
                        var raw = described.getAsJsonArray("raw");
                        var longs = new long[raw.size()];
                        for (int index = 0; index < longs.length; index++) {
                            longs[index] = Long.parseUnsignedLong(raw.get(index).getAsString(), 16);
                        }
                        sent.put(net.minecraft.world.level.levelgen.Heightmap.Types.valueOf(
                            described.get("type").getAsString()), longs);
                    }
                }
                level.getChunkSource().replaceWithPacketData(chunkX, chunkZ, buffer, sent, output -> {});
                // The server sends the heightmaps the client reads (`WORLD_SURFACE` and the like):
                // those a row does not send are worked out from the blocks the chunk holds.
                var unsent = java.util.EnumSet.of(
                    net.minecraft.world.level.levelgen.Heightmap.Types.WORLD_SURFACE,
                    net.minecraft.world.level.levelgen.Heightmap.Types.MOTION_BLOCKING,
                    net.minecraft.world.level.levelgen.Heightmap.Types.MOTION_BLOCKING_NO_LEAVES);
                unsent.removeAll(sent.keySet());
                net.minecraft.world.level.levelgen.Heightmap.primeHeightmaps(
                    level.getChunk(chunkX, chunkZ), unsent);
            }
        }

        placeVaultConnections(row, level);
        return new Scene(listener, wire, level, minecraft, options);
    }

    /**
     * The players a row puts in the level, and the players each vault block entity shares as
     * connected to it (its `shared_data`, as the server's update tag carries it).
     */
    static void placeVaultConnections(JsonObject row, net.minecraft.client.multiplayer.ClientLevel level) throws Exception {
        if (row.has("light") && row.get("light").getAsBoolean()) {
            lightTheChunks(row, level);
        }
        if (row.has("weather")) {
            var weather = row.getAsJsonArray("weather");
            level.setRainLevel(Float.intBitsToFloat(Integer.parseUnsignedInt(weather.get(0).getAsString(), 16)));
            level.setThunderLevel(Float.intBitsToFloat(Integer.parseUnsignedInt(weather.get(1).getAsString(), 16)));
        }
        if (row.has("game_time")) {
            level.setTimeFromServer(row.get("game_time").getAsLong());
        }
        if (row.has("players")) {
            int id = 100;
            for (var element : row.getAsJsonArray("players")) {
                var described = element.getAsJsonObject();
                var uuid = vaultUuid(described.getAsJsonArray("uuid"));
                var remote = new net.minecraft.client.player.RemotePlayer(
                    level, new com.mojang.authlib.GameProfile(uuid, "Player" + id));
                // What `isSpectator` asks the player list of: the player is listed, in the default mode.
                setField(remote, "playerInfo", new net.minecraft.client.multiplayer.PlayerInfo(
                    new com.mojang.authlib.GameProfile(uuid, "Player" + id), false));
                remote.setId(id++);
                remote.snapTo(Oracle.f64(described, "x"), Oracle.f64(described, "y"),
                    Oracle.f64(described, "z"), 0.0F, 0.0F);
                level.addEntity(remote);
            }
        }
        if (row.has("parrots")) {
            int id = 200;
            for (var element : row.getAsJsonArray("parrots")) {
                var described = element.getAsJsonObject();
                var parrot = new net.minecraft.world.entity.animal.parrot.Parrot(
                    net.minecraft.world.entity.EntityTypes.PARROT, level);
                parrot.setSilent(described.has("silent") && described.get("silent").getAsBoolean());
                parrot.setId(id++);
                parrot.setPos(Oracle.f64(described, "x"), Oracle.f64(described, "y"), Oracle.f64(described, "z"));
                level.addEntity(parrot);
            }
        }
        if (row.has("mobs")) {
            // Other mobs, by their type's name, numbered from 300.
            int id = 300;
            for (var element : row.getAsJsonArray("mobs")) {
                var described = element.getAsJsonObject();
                // Built directly, as the parrots are: the level has no features for a type to need.
                net.minecraft.world.entity.Entity mob = switch (described.get("type").getAsString()) {
                    case "minecraft:creeper" -> new net.minecraft.world.entity.monster.Creeper(
                        net.minecraft.world.entity.EntityTypes.CREEPER, level);
                    case "minecraft:zombie" -> new net.minecraft.world.entity.monster.zombie.Zombie(
                        net.minecraft.world.entity.EntityTypes.ZOMBIE, level);
                    default -> throw new IllegalArgumentException("no such mob in a row");
                };
                mob.setSilent(described.has("silent") && described.get("silent").getAsBoolean());
                mob.setId(id++);
                mob.setPos(Oracle.f64(described, "x"), Oracle.f64(described, "y"), Oracle.f64(described, "z"));
                level.addEntity(mob);
            }
        }
        if (row.has("gateways")) {
            // End gateways with the age the server sent, set up in the order of the row.
            for (var element : row.getAsJsonArray("gateways")) {
                var described = element.getAsJsonObject();
                var pos = new BlockPos(described.get("x").getAsInt(), described.get("y").getAsInt(),
                    described.get("z").getAsInt());
                var tag = new net.minecraft.nbt.CompoundTag();
                tag.putLong("Age", described.get("age").getAsLong());
                level.getBlockEntity(pos).loadWithComponents(
                    net.minecraft.world.level.storage.TagValueInput.create(
                        net.minecraft.util.ProblemReporter.DISCARDING, level.registryAccess(), tag));
            }
        }
        if (row.has("book_seed")) {
            // The enchanting tables' class-wide random source, seeded for the row.
            Class.forName("net.minecraft.world.level.block.entity.EnchantingTableBlockEntity", true,
                net.minecraft.world.level.block.entity.EnchantingTableBlockEntity.class.getClassLoader());
            var random = net.minecraft.world.level.block.entity.EnchantingTableBlockEntity.class.getDeclaredField("RANDOM");
            random.setAccessible(true);
            ((net.minecraft.util.RandomSource) random.get(null)).setSeed(row.get("book_seed").getAsLong());
        }
        if (row.has("block_entities")) {
            // The block entities of blocks that tick, made (and set to tick) in the order of the row.
            for (var element : row.getAsJsonArray("block_entities")) {
                var described = element.getAsJsonObject();
                level.getBlockEntity(new BlockPos(described.get("x").getAsInt(), described.get("y").getAsInt(),
                    described.get("z").getAsInt()));
            }
        }
        if (row.has("conduit_targets")) {
            // The mob a conduit's update tag names, by its UUID: one of the row's players.
            for (var element : row.getAsJsonArray("conduit_targets")) {
                var described = element.getAsJsonObject();
                var pos = new BlockPos(described.get("x").getAsInt(), described.get("y").getAsInt(),
                    described.get("z").getAsInt());
                var tag = new net.minecraft.nbt.CompoundTag();
                tag.put("Target", new net.minecraft.nbt.IntArrayTag(net.minecraft.core.UUIDUtil.uuidToIntArray(
                    vaultUuid(described.getAsJsonArray("uuid")))));
                level.getBlockEntity(pos).loadWithComponents(
                    net.minecraft.world.level.storage.TagValueInput.create(
                        net.minecraft.util.ProblemReporter.DISCARDING, level.registryAccess(), tag));
            }
        }
        if (row.has("spawners")) {
            // Spawners with what the update tag carries; the display entity is made once, as the
            // block entity renderer does on its first frame.
            for (var element : row.getAsJsonArray("spawners")) {
                var described = element.getAsJsonObject();
                var pos = new BlockPos(described.get("x").getAsInt(), described.get("y").getAsInt(),
                    described.get("z").getAsInt());
                var tag = new net.minecraft.nbt.CompoundTag();
                if (described.has("entity")) {
                    var entity = new net.minecraft.nbt.CompoundTag();
                    entity.putString("id", described.get("entity").getAsString());
                    var data = new net.minecraft.nbt.CompoundTag();
                    data.put("entity", entity);
                    tag.put("SpawnData", data);
                }
                if (described.has("range")) {
                    tag.putShort("RequiredPlayerRange", described.get("range").getAsShort());
                }
                if (described.has("delay")) {
                    tag.putShort("Delay", described.get("delay").getAsShort());
                }
                var spawner = (net.minecraft.world.level.block.entity.SpawnerBlockEntity) level.getBlockEntity(pos);
                spawner.loadWithComponents(net.minecraft.world.level.storage.TagValueInput.create(
                    net.minecraft.util.ProblemReporter.DISCARDING, level.registryAccess(), tag));
                spawner.getSpawner().getOrCreateDisplayEntity(level, pos);
            }
        }
        if (row.has("vaults")) {
            for (var element : row.getAsJsonArray("vaults")) {
                var described = element.getAsJsonObject();
                var pos = new BlockPos(described.get("x").getAsInt(), described.get("y").getAsInt(),
                    described.get("z").getAsInt());
                var shared = new net.minecraft.nbt.CompoundTag();
                var connected = new net.minecraft.nbt.ListTag();
                for (var player : described.getAsJsonArray("players")) {
                    connected.add(new net.minecraft.nbt.IntArrayTag(
                        net.minecraft.core.UUIDUtil.uuidToIntArray(vaultUuid(player.getAsJsonArray()))));
                }
                shared.put("connected_players", connected);
                if (described.has("range")) {
                    shared.putDouble("connected_particles_range", Oracle.f64(described, "range"));
                }
                if (described.has("item")) {
                    var item = new net.minecraft.nbt.CompoundTag();
                    item.putString("id", described.get("item").getAsString());
                    item.putInt("count", 1);
                    shared.put("display_item", item);
                }
                var tag = new net.minecraft.nbt.CompoundTag();
                tag.put("shared_data", shared);
                level.getBlockEntity(pos).loadWithComponents(
                    net.minecraft.world.level.storage.TagValueInput.create(
                        net.minecraft.util.ProblemReporter.DISCARDING, level.registryAccess(), tag));
            }
        }
    }

    /** Which faces an end portal block entity draws, a bit to each direction in the enum's order. */
    static int faceMask(final net.minecraft.world.level.block.entity.TheEndPortalBlockEntity entity) {
        int mask = 0;
        for (var direction : net.minecraft.core.Direction.values()) {
            if (entity.shouldRenderFace(direction)) {
                mask |= 1 << direction.ordinal();
            }
        }
        return mask;
    }

    /**
     * `be,x,y,z,...` for each place a row watches: what its block entity holds of what block
     * events set. A place with no block entity reads as `none`.
     */
    static java.util.List<String> watchLines(
        JsonObject row, net.minecraft.client.multiplayer.ClientLevel level) throws Exception {
        var lines = new java.util.ArrayList<String>();
        if (!row.has("watch")) {
            return lines;
        }
        for (var element : row.getAsJsonArray("watch")) {
            var watched = element.getAsJsonObject();
            var pos = new BlockPos(watched.get("x").getAsInt(), watched.get("y").getAsInt(),
                watched.get("z").getAsInt());
            var entity = level.getBlockEntity(pos);
            String held;
            if (entity == null) {
                held = "none";
            } else if (entity instanceof net.minecraft.world.level.block.entity.DecoratedPotBlockEntity pot) {
                held = "pot," + pot.wobbleStartedAtTick + ","
                    + (pot.lastWobbleStyle == null ? -1 : pot.lastWobbleStyle.ordinal());
            } else if (entity instanceof net.minecraft.world.level.block.entity.TheEndGatewayBlockEntity gateway) {
                held = "gateway," + getField(gateway, "teleportCooldown") + "," + getField(gateway, "age") + ","
                    + faceMask(gateway);
            } else if (entity instanceof net.minecraft.world.level.block.entity.TheEndPortalBlockEntity portal) {
                held = "portal," + faceMask(portal);
            } else if (entity instanceof net.minecraft.world.level.block.entity.SpawnerBlockEntity spawner) {
                held = "spawner," + getField(spawner.getSpawner(), "spawnDelay") + ","
                    + Oracle.bits(spawner.getSpawner().getOSpin()) + "," + Oracle.bits(spawner.getSpawner().getSpin());
            } else if (entity instanceof net.minecraft.world.level.block.entity.BeaconBlockEntity beacon) {
                var sections = new java.util.ArrayList<String>();
                @SuppressWarnings("unchecked")
                var raw = (java.util.List<net.minecraft.world.level.block.entity.BeaconBeamOwner.Section>)
                    getField(beacon, "beamSections");
                for (var section : raw) {
                    sections.add(String.format(java.util.Locale.ROOT, "%08x:%d", section.getColor(), section.getHeight()));
                }
                held = "beacon," + getField(beacon, "levels") + "," + getField(beacon, "lastCheckY") + ","
                    + String.join(";", sections);
            } else if (entity instanceof net.minecraft.world.level.block.entity.vault.VaultBlockEntity vault) {
                held = "vault," + (int) vault.getClientData().previousSpin() + "," + (int) vault.getClientData().currentSpin();
            } else if (entity instanceof net.minecraft.world.level.block.entity.ConduitBlockEntity conduit) {
                held = "conduit," + conduit.tickCount + "," + Oracle.bits((float) getField(conduit, "activeRotation"))
                    + "," + conduit.isActive() + "," + conduit.isHunting();
            } else if (entity instanceof net.minecraft.world.level.block.entity.EnchantingTableBlockEntity book) {
                held = "book," + book.time + "," + String.join(",",
                    Oracle.bits(book.flip), Oracle.bits(book.oFlip), Oracle.bits(book.flipT),
                    Oracle.bits(book.flipA), Oracle.bits(book.open), Oracle.bits(book.oOpen),
                    Oracle.bits(book.rot), Oracle.bits(book.oRot), Oracle.bits(book.tRot));
            } else if (entity instanceof net.minecraft.world.level.block.entity.PotentSulfurBlockEntity sulfur) {
                held = "sulfur," + sulfur.eruptionTick;
            } else {
                held = "other";
            }
            lines.add("be," + pos.getX() + "," + pos.getY() + "," + pos.getZ() + "," + held);
        }
        return lines;
    }

    /**
     * The level's light the way the server first lights a chunk: every section with blocks is
     * known, light is switched on in each chunk, and the sources spread.
     */
    static void lightTheChunks(JsonObject row, net.minecraft.client.multiplayer.ClientLevel level) {
        var engine = level.getChunkSource().getLightEngine();
        int radius = row.get("chunk_radius").getAsInt();
        var chunks = new java.util.ArrayList<net.minecraft.world.level.ChunkPos>();
        for (int chunkX = -radius; chunkX <= radius; chunkX++) {
            for (int chunkZ = -radius; chunkZ <= radius; chunkZ++) {
                chunks.add(new net.minecraft.world.level.ChunkPos(chunkX, chunkZ));
            }
        }
        for (var chunkPos : chunks) {
            var sections = level.getChunk(chunkPos.x(), chunkPos.z()).getSections();
            for (int index = 0; index < sections.length; index++) {
                if (!sections[index].hasOnlyAir()) {
                    engine.updateSectionStatus(net.minecraft.core.SectionPos.of(
                        chunkPos, level.getSectionYFromSectionIndex(index)), false);
                }
            }
        }
        chunks.forEach(chunkPos -> engine.setLightEnabled(chunkPos, true));
        chunks.forEach(engine::propagateLightSources);
        engine.runLightUpdates();
    }

    /**
     * The blocks of the box a row watches (`blk,x,y,z,id` for each that is not air) and the moves of
     * the moving pistons in it (`mov,x,y,z,moved,direction,extending,source,progress,previous`).
     */
    static java.util.List<String> regionLines(
        JsonObject row, net.minecraft.client.multiplayer.ClientLevel level) throws Exception {
        var lines = new java.util.ArrayList<String>();
        if (!row.has("region")) {
            return lines;
        }
        var box = row.getAsJsonArray("region");
        for (int x = box.get(0).getAsInt(); x <= box.get(3).getAsInt(); x++) {
            for (int y = box.get(1).getAsInt(); y <= box.get(4).getAsInt(); y++) {
                for (int z = box.get(2).getAsInt(); z <= box.get(5).getAsInt(); z++) {
                    var pos = new BlockPos(x, y, z);
                    var state = level.getBlockState(pos);
                    if (state.isAir()) {
                        continue;
                    }
                    lines.add("blk," + x + "," + y + "," + z + "," + net.minecraft.world.level.block.Block.getId(state));
                    if (state.is(net.minecraft.world.level.block.Blocks.MOVING_PISTON)
                        && level.getBlockEntity(pos) instanceof
                            net.minecraft.world.level.block.piston.PistonMovingBlockEntity moving) {
                        lines.add("mov," + x + "," + y + "," + z + ","
                            + net.minecraft.world.level.block.Block.getId(moving.getMovedState()) + ","
                            + moving.getDirection().ordinal() + "," + moving.isExtending() + ","
                            + moving.isSourcePiston() + ","
                            + Oracle.bits((float) getField(moving, "progress")) + ","
                            + Oracle.bits((float) getField(moving, "progressO")));
                        // What the block fills: the boxes of its collision shape.
                        var boxes = new StringBuilder("shape," + x + "," + y + "," + z);
                        for (var part : state.getCollisionShape(level, pos).toAabbs()) {
                            for (double edge : new double[] {part.minX, part.minY, part.minZ, part.maxX, part.maxY, part.maxZ}) {
                                boxes.append(',').append(Oracle.bits(edge));
                            }
                        }
                        lines.add(boxes.toString());
                    }
                }
            }
        }
        return lines;
    }

    /** The parrots a row put in the level, by their numbers. */
    static java.util.List<net.minecraft.world.entity.animal.parrot.Parrot> parrots(
        net.minecraft.client.multiplayer.ClientLevel level) {
        var found = new java.util.ArrayList<net.minecraft.world.entity.animal.parrot.Parrot>();
        for (var entity : level.entitiesForRendering()) {
            if (entity instanceof net.minecraft.world.entity.animal.parrot.Parrot parrot) {
                found.add(parrot);
            }
        }
        found.sort(java.util.Comparator.comparingInt(net.minecraft.world.entity.Entity::getId));
        return found;
    }

    static java.util.UUID vaultUuid(com.google.gson.JsonArray ints) {
        return net.minecraft.core.UUIDUtil.uuidFromIntArray(new int[] {
            ints.get(0).getAsInt(), ints.get(1).getAsInt(), ints.get(2).getAsInt(), ints.get(3).getAsInt()});
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
