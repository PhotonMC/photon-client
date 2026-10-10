# History

Photon started on 2026-09-07. Until 2026-10-10 the work was tracked as about 285 small tasks in a Markdown board; this page condenses what they delivered. The commit log has the details of every slice, and open work now lives in [GitHub issues](https://github.com/PhotonMC/photon-client/issues).

## Foundation (2026-09)

- **Reference.** The Java Edition 26.2 reference (protocol 776, Java 25) was pinned, together with a headless Java oracle, a capture/compare runner and a JSONL Rust adapter.
- **Java-semantics core.** VarInt/VarLong, UTF-8, UUID, BlockPos, BitSet, NBT, Vec2/Vec3, Direction, random sources, AABB and voxel shapes, all bit-identical to Java.
- **Connection.** Handshake, login, configuration, registries and the play session against vanilla servers.
- **World copy.** Paletted chunks, light (server data plus a client light engine), block entities and entity state.
- **Movement.** The whole `LocalPlayer` tick ported and compared tick by tick: collision, step-up, fluids, slime, honey, ladders, powder snow, flight, auto-jump.
- **World rendering.**
  - Blockstates and models with UV lock, tinting and biome colours.
  - Smooth light and AO, lightmap, sky, clouds, fog, weather, animated textures and fluids.
  - Section meshing on worker threads with occlusion graph and frustum culling.
- **Block actions.** Breaking with crack overlay and prediction, placing with class rules.
- **Items.** Item models, hotbar icons, held items, special models and component-driven tints.
- **Inventories.** Survival and creative inventories with click prediction.

## M3.d: containers opened by the server

- **Menus and screens.**
  - Chest, barrel, dispenser, hopper and shulker box.
  - Crafting table, furnace family, stonecutter, loom, cartography and smithing tables, anvil, grindstone and enchanting table.
  - Brewing stand, beacon, crafter, lectern and merchant.
  - Mount inventories with entity previews.
- **Recipe book** with ghost recipes.
- **Item details.** Component hashes in click packets, bundle tooltips and selection, container/map/book tooltip lines, and `#tag` search.

## M4: entities

- **Pipeline.** Entity packets, interpolation and ticking in the live session.
- **Models.** An entity drawing skeleton with model layers captured from the oracle, and a keyframe animation player.
- **Coverage.** Almost every mob family: humanoids, quadrupeds, aquatic and flying mobs, bosses (wither, ender dragon with death rays and crystal beams, warden), slimes and shulkers.
- **Layers.** Armor with trims and glint, elytra, skulls, held items and use poses.
- **Other entities.** Shadows, name tags, leash lines, item frames with map contents, paintings, displays (block/item/text), primed TNT, falling blocks and lightning.
- **Combat.** Hurt and death feedback, targeting with `Attack`/`Interact`, attributes.
- **Riding.** Boats, horses, pigs, striders, camels and minecarts with `PlayerInput`/`MoveVehicle`/`PaddleBoat`.
- **Culling.** Entity distance and visibility culling.

## M5: text and chat

- Chat with signatures, deletion and tags, and chat drafts.
- Command tree parsing, coloured syntax and suggestions, with click and hover events.
- Titles, the tab list, scoreboard sidebar, boss bars and the locator bar.

## M6: screens and settings

- **Menus.** Title screen with panorama and blurred background, server list with status pings and favicons, add/edit/direct connect, connecting and loading screens.
- **In game.** Pause, death and disconnected screens, credits, the game mode switcher, social interactions, and sign and book editing.
- **Settings.** `options.txt`, sliders and cycle buttons, keyboard focus and tooltips, and the video, chat, mouse, controls, sound, skin, key bind and language screens. Render distance is wired to the server.

## M7: sound

- **Selection.** `sounds.json` resolution and weighted selection.
- **Engine.** Sound packets and `SoundEngine` logic compared with Java; Ogg/Vorbis decoding with `lewton` checked against JOrbis.
- **Output.** `cpal` output with our own mixer.
- **Triggers.** Movement, hurt and death sounds; music; underwater, biome and cave ambience.

## M8: particles, effects and block entities

- **Particles.** The engine and the `LevelParticles` packet, with most particle families: terrain, smoke and fire, water and drips, dust and spells, portal, vibration and trail, explosions, item pickup, breaking items and fireworks.
- **Block animation.** Random display ticks.
- **Level events.** `LevelEvent` and `BlockEvent` handlers.
- **Block entities.** Chests, shulker boxes, bells, heads, banners and signs.
- **Camera and world.** Hurt camera tilt, screen overlays, and the world border with its rendering.

## M9: server compatibility

- **Packets.** Cookies, custom payloads and queries.
- **Session flow.** Reconfiguration and transfer.
- **Resource packs.** The resource pack prompt.
- **Dialogs.** Server dialog screens (notice, confirmation, multi-action).

## Code quality and performance

- **Repository.** Public-repository hygiene and CI.
- **Code.** Logging and `unsafe` justifications, file splitting, Java-bit-exact float newtypes, the `EntityKind` enum and `Entities` store.
- **Vulkan backend.** Modules with RAII ownership, build-time SPIR-V with a pipeline cache, two frames in flight, a `RenderType` table, present mode selection.
- **Frame time.** Entity culling and persistently mapped buffers took the still-route frame from about 2.8 ms to about 0.9 ms.

## 2026-10-10: restructuring

Feature work paused for an infrastructure phase:

- deterministic lint rules, git hooks and the xtask gate;
- English-only repository and history, and `main` as the default branch;
- issues on GitHub;
- the decisions recorded in [ADRs 0003-0008](adr/).
