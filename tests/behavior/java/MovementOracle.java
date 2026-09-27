import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.dimension.DimensionType;
import net.minecraft.world.phys.Vec3;

/**
 * Test glue only: runs the original movement code in a world the test builds.
 *
 * <p>Everything the reference decides stays the reference's. What this file
 * supplies is a place to decide it in: a {@link Level} whose blocks come from
 * the scenario rather than from a save, and whose collision walk hands back
 * that same storage. The movement code, the collision gathering, the shape
 * lookup and the resolution are all the original.
 */
public final class MovementOracle {
    private static boolean bootstrapped;

    /**
     * Bring up the registries once, keeping the streams the oracle writes to.
     *
     * The bootstrap redirects them into the game's logger at the end, which
     * would swallow every record written afterwards.
     */
    private static synchronized void bootstrap() {
        if (bootstrapped) {
            return;
        }
        var out = System.out;
        var err = System.err;
        try {
            net.minecraft.SharedConstants.setVersion(Oracle.TEST_VERSION);
            net.minecraft.server.Bootstrap.bootStrap();
        } finally {
            System.setOut(out);
            System.setErr(err);
        }
        bootstrapped = true;
    }

    /** Copy one registry the vanilla lookup carries into a frozen one. */
    @SuppressWarnings({"unchecked", "rawtypes"})
    private static <T> net.minecraft.core.Registry<T> copyRegistry(
        net.minecraft.core.HolderLookup.Provider provider,
        net.minecraft.resources.ResourceKey<? extends net.minecraft.core.Registry<? extends T>> key) {
        var target = new net.minecraft.core.MappedRegistry<T>(
            (net.minecraft.resources.ResourceKey) key,
            com.mojang.serialization.Lifecycle.stable());
        provider.lookupOrThrow(key)
            .listElements()
            .forEach(holder -> net.minecraft.core.Registry.register(
                target, (net.minecraft.resources.ResourceKey) holder.key(), holder.value()));
        target.freeze();
        return target;
    }

    /** The level data a fixture world needs; none of it is read. */
    record FixtureData() implements net.minecraft.world.level.storage.WritableLevelData {
        public void setSpawn(net.minecraft.world.level.storage.LevelData.RespawnData data) {}

        public net.minecraft.world.level.storage.LevelData.RespawnData getRespawnData() {
            return net.minecraft.world.level.storage.LevelData.RespawnData.DEFAULT;
        }

        public long getGameTime() {
            return 0L;
        }

        public boolean isHardcore() {
            return false;
        }

        public net.minecraft.world.Difficulty getDifficulty() {
            return net.minecraft.world.Difficulty.PEACEFUL;
        }

        public boolean isDifficultyLocked() {
            return false;
        }
    }

    /** Build an empty fixture world. */
    private static FixtureLevel newLevel(boolean clientSide) {
        var lookup = net.minecraft.data.registries.VanillaRegistries.createLookup();
        var registries = new java.util.LinkedHashMap<
            net.minecraft.resources.ResourceKey<?>, net.minecraft.core.Registry<?>>();
        net.minecraft.core.registries.BuiltInRegistries.REGISTRY
            .forEach(registry -> registries.put(registry.key(), registry));
        // A datapack registry the lookup carries wins over a built-in one of
        // the same name, because that is the one holding the vanilla content.
        lookup.listRegistryKeys().forEach(key -> {
            var copied = copyRegistry(lookup, key);
            registries.put(copied.key(), copied);
        });
        var access = new RegistryAccess.ImmutableRegistryAccess(
            new java.util.ArrayList<>(registries.values()));
        var dimensionType = lookup.lookupOrThrow(Registries.DIMENSION_TYPE)
            .getOrThrow(net.minecraft.world.level.dimension.BuiltinDimensionTypes.OVERWORLD);
        return new FixtureLevel(new FixtureData(), access, dimensionType, clientSide);
    }

    /** A world backed by a flat block map, so a scenario can describe one. */
    static final class FixtureLevel extends Level {

        private final java.util.Map<Long, BlockState> blocks = new java.util.HashMap<>();
        private final BlockState air;

        public FixtureLevel(
            net.minecraft.world.level.storage.WritableLevelData data,
            RegistryAccess access,
            Holder<DimensionType> dimensionType,
            boolean clientSide) {
            super(data, Level.OVERWORLD, access, dimensionType, clientSide, false, 0L, 1000000);
            this.air = net.minecraft.world.level.block.Blocks.AIR.defaultBlockState();
        }

        public void put(BlockPos pos, BlockState state) {
            this.blocks.put(pos.asLong(), state);
        }

        @Override
        public BlockState getBlockState(BlockPos pos) {
            return this.blocks.getOrDefault(pos.asLong(), this.air);
        }

        /// The fixture stores blocks flat rather than in chunks, so anything
        /// that asks for a chunk is told there is none. The one caller that
        /// matters here is the fluid interaction check, which reads that as
        /// "no fluid" — true for these scenarios, and the reason a water or
        /// lava scenario cannot use this fixture as it stands.
        @Override
        public net.minecraft.world.level.chunk.@org.jspecify.annotations.Nullable ChunkAccess
            getChunk(int x, int z, net.minecraft.world.level.chunk.status.ChunkStatus status,
                boolean requireChunk) {
            return null;
        }

        /// The collision walk asks for a chunk to read blocks from. The fixture
        /// stores its blocks flat, so it hands back itself: the walk, the shape
        /// lookup and the resolution all stay the reference's.
        @Override
        public net.minecraft.world.level.BlockGetter getChunkForCollisions(int chunkX, int chunkZ) {
            return this;
        }

        @Override
        public net.minecraft.world.level.material.FluidState getFluidState(BlockPos pos) {
            return getBlockState(pos).getFluidState();
        }

        @Override
        public void sendBlockUpdated(BlockPos pos, BlockState old, BlockState current, int flags) {}

        @Override
        public String gatherChunkSourceStats() {
            return "fixture";
        }

        // Everything below is a stub: the movement paths under test never reach
        // any of it, and a fixture that pretended to implement them would be
        // reimplementing the game rather than hosting it.
        private static <T> T unused() {
            throw new UnsupportedOperationException("not part of the movement fixture");
        }

        @Override
        public void playSeededSound(
            net.minecraft.world.entity.@org.jspecify.annotations.Nullable Entity entity,
            double x, double y, double z,
            Holder<net.minecraft.sounds.SoundEvent> sound,
            net.minecraft.sounds.SoundSource source,
            float volume, float pitch, long seed) {}

        @Override
        public void playSeededSound(
            net.minecraft.world.entity.@org.jspecify.annotations.Nullable Entity entity,
            net.minecraft.world.entity.Entity target,
            Holder<net.minecraft.sounds.SoundEvent> sound,
            net.minecraft.sounds.SoundSource source,
            float volume, float pitch, long seed) {}

        @Override
        public void explode(
            net.minecraft.world.entity.@org.jspecify.annotations.Nullable Entity source,
            net.minecraft.world.damagesource.@org.jspecify.annotations.Nullable DamageSource damage,
            net.minecraft.world.level.@org.jspecify.annotations.Nullable ExplosionDamageCalculator calculator,
            double x, double y, double z, float radius, boolean fire,
            Level.ExplosionInteraction interaction,
            net.minecraft.core.particles.ParticleOptions small,
            net.minecraft.core.particles.ParticleOptions large,
            net.minecraft.util.random.WeightedList<
                net.minecraft.core.particles.ExplosionParticleInfo> blockParticles,
            Holder<net.minecraft.sounds.SoundEvent> sound) {}

        @Override
        public void setRespawnData(net.minecraft.world.level.storage.LevelData.RespawnData data) {}

        @Override
        public net.minecraft.world.level.storage.LevelData.RespawnData getRespawnData() {
            return unused();
        }

        @Override
        public net.minecraft.world.entity.@org.jspecify.annotations.Nullable Entity getEntity(int id) {
            return null;
        }

        @Override
        public java.util.Collection<net.minecraft.world.entity.boss.enderdragon.EnderDragonPart> dragonParts() {
            return java.util.List.of();
        }

        @Override
        public net.minecraft.world.TickRateManager tickRateManager() {
            return unused();
        }

        @Override
        public net.minecraft.world.level.saveddata.maps.@org.jspecify.annotations.Nullable MapItemSavedData
            getMapData(net.minecraft.world.level.saveddata.maps.MapId id) {
            return null;
        }

        @Override
        public void destroyBlockProgress(int id, BlockPos pos, int progress) {}

        @Override
        public net.minecraft.world.scores.Scoreboard getScoreboard() {
            return unused();
        }

        @Override
        public net.minecraft.world.item.crafting.RecipeAccess recipeAccess() {
            return unused();
        }

        /// No entities: the movement fixture is one entity against blocks, so the
        /// entity collision pass finds nothing and the block pass is what is
        /// being exercised.
        private final net.minecraft.world.level.entity.LevelEntityGetter<
            net.minecraft.world.entity.Entity> noEntities =
            new net.minecraft.world.level.entity.LevelEntityGetter<>() {
                @Override
                public net.minecraft.world.entity.@org.jspecify.annotations.Nullable Entity get(int id) {
                    return null;
                }

                @Override
                public net.minecraft.world.entity.@org.jspecify.annotations.Nullable Entity get(
                    java.util.UUID id) {
                    return null;
                }

                @Override
                public Iterable<net.minecraft.world.entity.Entity> getAll() {
                    return java.util.List.of();
                }

                @Override
                public <U extends net.minecraft.world.entity.Entity> void get(
                    net.minecraft.world.level.entity.EntityTypeTest<
                        net.minecraft.world.entity.Entity, U> type,
                    net.minecraft.util.AbortableIterationConsumer<U> consumer) {}

                @Override
                public void get(
                    net.minecraft.world.phys.AABB box,
                    java.util.function.Consumer<net.minecraft.world.entity.Entity> output) {}

                @Override
                public <U extends net.minecraft.world.entity.Entity> void get(
                    net.minecraft.world.level.entity.EntityTypeTest<
                        net.minecraft.world.entity.Entity, U> type,
                    net.minecraft.world.phys.AABB box,
                    net.minecraft.util.AbortableIterationConsumer<U> consumer) {}
            };

        @Override
        protected net.minecraft.world.level.entity.LevelEntityGetter<net.minecraft.world.entity.Entity>
            getEntities() {
            return this.noEntities;
        }

        @Override
        public net.minecraft.world.clock.ClockManager clockManager() {
            return unused();
        }

        @Override
        public net.minecraft.world.attribute.EnvironmentAttributeSystem environmentAttributes() {
            return unused();
        }

        @Override
        public net.minecraft.world.item.alchemy.PotionBrewing potionBrewing() {
            return unused();
        }

        @Override
        public net.minecraft.world.level.block.entity.FuelValues fuelValues() {
            return unused();
        }

        @Override
        public net.minecraft.world.level.chunk.ChunkSource getChunkSource() {
            return unused();
        }

        @Override
        public java.util.List<? extends net.minecraft.world.entity.player.Player> players() {
            return java.util.List.of();
        }

        @Override
        public net.minecraft.world.ticks.LevelTickAccess<
            net.minecraft.world.level.block.Block> getBlockTicks() {
            return net.minecraft.world.ticks.BlackholeTickAccess.emptyLevelList();
        }

        @Override
        public net.minecraft.world.ticks.LevelTickAccess<
            net.minecraft.world.level.material.Fluid> getFluidTicks() {
            return net.minecraft.world.ticks.BlackholeTickAccess.emptyLevelList();
        }

        private final net.minecraft.world.level.border.WorldBorder border =
            new net.minecraft.world.level.border.WorldBorder();

        @Override
        public net.minecraft.world.level.border.WorldBorder getWorldBorder() {
            return this.border;
        }

        @Override
        public Holder<net.minecraft.world.level.biome.Biome> getUncachedNoiseBiome(int x, int y, int z) {
            return this.registryAccess()
                .lookupOrThrow(Registries.BIOME)
                .getOrThrow(net.minecraft.world.level.biome.Biomes.PLAINS);
        }

        @Override
        public int getSeaLevel() {
            return 63;
        }

        @Override
        public net.minecraft.world.flag.FeatureFlagSet enabledFeatures() {
            return net.minecraft.world.flag.FeatureFlags.VANILLA_SET;
        }

        @Override
        public void levelEvent(
            net.minecraft.world.entity.@org.jspecify.annotations.Nullable Entity entity,
            int type, BlockPos pos, int data) {}

        @Override
        public void gameEvent(
            Holder<net.minecraft.world.level.gameevent.GameEvent> event,
            net.minecraft.world.phys.Vec3 position,
            net.minecraft.world.level.gameevent.GameEvent.Context context) {}
    }

    public static JsonObject run(JsonObject request) {
        bootstrap();
        String operation = request.get("operation").getAsString();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            switch (operation) {
                case "entity_move" -> rows.add(entityMove(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException(
                    "Unknown movement operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Step one entity through a scene with `Entity.move`, tick by tick.
     *
     * The whole composition runs: the edge back-off, the collision gathering
     * and resolution, the four collision flags, the ground test, the bounce
     * and the block speed factor.
     */
    private static JsonObject entityMove(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        // Whether the world is a client's changes `move` itself: on a client
        // only the locally authoritative entity — the player — runs the fall
        // damage check and the bounce, so a scenario says which side it is.
        var level = newLevel(row.get("client_side").getAsBoolean());
        for (var element : row.get("blocks").getAsJsonArray()) {
            var described = element.getAsJsonObject();
            var block = net.minecraft.core.registries.BuiltInRegistries.BLOCK.getValue(
                net.minecraft.resources.Identifier.parse(described.get("block").getAsString()));
            level.put(
                new BlockPos(
                    described.get("x").getAsInt(),
                    described.get("y").getAsInt(),
                    described.get("z").getAsInt()),
                block.defaultBlockState());
        }

        var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
            net.minecraft.resources.Identifier.parse(row.get("entity").getAsString()));
        var entity = type.create(level, net.minecraft.world.entity.EntitySpawnReason.LOAD);
        if (entity == null) {
            entry.addProperty("outcome", "no_entity");
            return entry;
        }
        entity.setPos(
            Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"));
        entity.setDeltaMovement(Vec3.ZERO);
        entity.setOnGround(row.get("on_ground").getAsBoolean());
        if (row.get("shift_key_down").getAsBoolean() && entity instanceof
            net.minecraft.world.entity.player.Player player) {
            player.setShiftKeyDown(true);
        }

        var ticks = new JsonArray();
        for (var element : row.get("ticks").getAsJsonArray()) {
            var step = element.getAsJsonObject();
            var delta = new Vec3(
                Oracle.f64(step, "dx"), Oracle.f64(step, "dy"), Oracle.f64(step, "dz"));
            if (step.get("accumulate").getAsBoolean()) {
                entity.setDeltaMovement(entity.getDeltaMovement().add(delta));
                delta = entity.getDeltaMovement();
            }
            entity.move(net.minecraft.world.entity.MoverType.SELF, delta);
            var described = new JsonObject();
            described.addProperty("x", Oracle.bits(entity.getX()));
            described.addProperty("y", Oracle.bits(entity.getY()));
            described.addProperty("z", Oracle.bits(entity.getZ()));
            described.addProperty("motion_x", Oracle.bits(entity.getDeltaMovement().x));
            described.addProperty("motion_y", Oracle.bits(entity.getDeltaMovement().y));
            described.addProperty("motion_z", Oracle.bits(entity.getDeltaMovement().z));
            described.addProperty("on_ground", entity.onGround());
            described.addProperty("horizontal_collision", entity.horizontalCollision);
            described.addProperty("vertical_collision", entity.verticalCollision);
            described.addProperty("vertical_collision_below", entity.verticalCollisionBelow);
            described.addProperty("minor_horizontal_collision", entity.minorHorizontalCollision);
            ticks.add(described);
        }
        entry.addProperty("outcome", "moved");
        entry.add("ticks", ticks);
        return entry;
    }
}
