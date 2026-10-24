/** Bringing the 26.2 game up: its version stamp, its registries and its data pack. */
final class Game26_2 {
    private Game26_2() {}

    /**
     * The version every bootstrapping family installs.
     *
     * `SharedConstants.setVersion` refuses a second, different instance, so
     * the families that bootstrap have to agree on one object rather than each
     * building its own.
     */
    static final net.minecraft.WorldVersion TEST_VERSION =
        net.minecraft.DetectedVersion.createBuiltIn("26.2", "26.2");

    private static net.minecraft.core.RegistryAccess.Frozen vanillaRegistries;

    /**
     * Bring the game up once and load the vanilla data pack the way the
     * server does: the static layer's tags, then the worldgen registries with
     * theirs. Families that read tags go through this, so a tag is never read
     * unbound. The bootstrap redirects the standard streams into the game's
     * logger, which would swallow every record, so they are put back.
     */
    /**
     * Bootstraps the game's built-in registries once. A case that touches game
     * classes calls this itself instead of relying on an earlier case having
     * done it. The bootstrap redirects the standard streams into the game's
     * logger, which would swallow every record, so they are put back.
     */
    static synchronized void bootstrap() {
        var out = System.out;
        var err = System.err;
        // Another family may already have installed the shared version.
        net.minecraft.SharedConstants.setVersion(TEST_VERSION);
        try {
            net.minecraft.server.Bootstrap.bootStrap();
        } finally {
            System.setOut(out);
            System.setErr(err);
        }
    }

    static synchronized net.minecraft.core.RegistryAccess.Frozen vanillaRegistries() {
        if (vanillaRegistries != null) {
            return vanillaRegistries;
        }
        bootstrap();
        var pack = net.minecraft.server.packs.repository.ServerPacksSource.createVanillaPackSource();
        var resources = new net.minecraft.server.packs.resources.MultiPackResourceManager(
            net.minecraft.server.packs.PackType.SERVER_DATA, java.util.List.of(pack));
        var layers = net.minecraft.server.RegistryLayer.createRegistryAccess();
        var staticTags = net.minecraft.tags.TagLoader.loadTagsForExistingRegistries(
            resources, layers.getLayer(net.minecraft.server.RegistryLayer.STATIC));
        var worldgenContext = layers.getAccessForLoading(net.minecraft.server.RegistryLayer.WORLDGEN);
        var worldgenLookups = net.minecraft.tags.TagLoader.buildUpdatedLookups(worldgenContext, staticTags);
        var worldgen = net.minecraft.resources.RegistryDataLoader.load(
            resources, worldgenLookups, net.minecraft.resources.RegistryDataLoader.WORLDGEN_REGISTRIES,
            Runnable::run).join();
        var loaded = layers.replaceFrom(net.minecraft.server.RegistryLayer.WORLDGEN, worldgen);
        staticTags.forEach(net.minecraft.core.Registry.PendingTags::apply);
        vanillaRegistries = loaded.compositeAccess().freeze();
        // Items and other built-in entries get their default components the
        // way the server's resource reload gives them, once the lookups exist.
        net.minecraft.core.registries.BuiltInRegistries.DATA_COMPONENT_INITIALIZERS
            .build(vanillaRegistries)
            .forEach(net.minecraft.core.component.DataComponentInitializers.PendingComponents::apply);
        return vanillaRegistries;
    }
}
