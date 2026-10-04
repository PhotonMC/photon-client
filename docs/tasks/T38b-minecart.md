# T38b — Minecart ailesi

**Amaç:** Minecart ve türevleri (sandık, ocak, huni, TNT, spawner, komut bloğu): `MinecartModel`, `AbstractMinecartRenderer`/`MinecartRenderer`, yönelme ve ray eğimi (`NewMinecartBehavior` istemci iç-konumlaması), içerik bloğu (`getDisplayBlockState`) ve vuruş sallanması.
**Bağımlılık:** T38. **Boy:** M.

## Oku
- Java: `client/model/object/cart/MinecartModel.java`, `client/renderer/entity/{AbstractMinecart,Minecart}Renderer.java`, `client/renderer/entity/state/MinecartRenderState.java`, `world/entity/vehicle/minecart/AbstractMinecart.java`, `NewMinecartBehavior.java`, `OldMinecartBehavior.java`.
- Rust: `crates/voxel-render/src/boat.rs` (`boat_pose`: araç duruşu kalıbı), `crates/voxel-world/src/boat.rs` (vuruş sönmesi: `hurt_time`, `damage`), `crates/voxel/src/boats.rs`, `crates/voxel/src/entity_block.rs`.

## Oracle
`boat_model` / `boat_pose` kalıbı (gerçek model + gerçek renderer'ın `submit` duruşu); gerçek varlık tiki için `living-entity-animations` (`boat` ekstrası kalıbı).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sürüş (T44), ray fiziği sunucu tarafında, sesler.

## Devir notu
(boş)
