# T38d — TNT ve spawner minecart

**Amaç:** `TntMinecartRenderer` (fitil yanarken blok beyaz yanıp söner: `submitWhiteSolidBlock`, `MinecartTNT` fitil verisi) ve `SpawnerMinecartRenderer` (içinde dönen yaratık, `SpawnerRenderer.renderEntityInSpawner`), sandık minecart'ın sandığı için blok varlık modeli gelince onu da bağlamak.
**Bağımlılık:** T38b. **Boy:** M.

## Oku
- Java: `client/renderer/entity/{TntMinecartRenderer,SpawnerMinecartRenderer}.java`, `world/entity/vehicle/minecart/{MinecartTNT,MinecartSpawner}.java`.
- Rust: `crates/voxel/src/minecarts.rs`, T37b3 (blok kaplaması: fitil beyazı) ve T40'taki TNT/düşen blok çizimi kalıbı.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
