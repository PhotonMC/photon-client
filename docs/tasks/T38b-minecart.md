# T38b — Minecart ailesi

**Amaç:** Minecart ve türevleri (sandık, ocak, huni, TNT, spawner, komut bloğu): `MinecartModel`, `AbstractMinecartRenderer`/`MinecartRenderer`, yönelme ve ray eğimi (`NewMinecartBehavior` istemci iç-konumlaması), içerik bloğu (`getDisplayBlockState`) ve vuruş sallanması.
**Bağımlılık:** T38. **Boy:** M.

## Oku
- Java: `client/model/object/cart/MinecartModel.java`, `client/renderer/entity/{AbstractMinecart,Minecart}Renderer.java`, `client/renderer/entity/state/MinecartRenderState.java`, `world/entity/vehicle/minecart/AbstractMinecart.java`, `NewMinecartBehavior.java`, `OldMinecartBehavior.java`.
- Rust: `crates/voxel-render/src/objects/boat.rs` (`boat_pose`: araç duruşu kalıbı), `crates/voxel-world/src/objects/boat.rs` (vuruş sönmesi: `hurt_time`, `damage`), `crates/voxel/src/objects/boats.rs`, `crates/voxel/src/entity_block.rs`.

## Oracle
`boat_model` / `boat_pose` kalıbı (gerçek model + gerçek renderer'ın `submit` duruşu); gerçek varlık tiki için `living-entity-animations` (`boat` ekstrası kalıbı).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sürüş (T44), ray fiziği sunucu tarafında, sesler.

## Devir notu
- Yapıldı: eski davranışla minecart ailesinin çizimi (`voxel-render/minecart.rs`: pose, `voxel-world/minecart.rs`: vuruş, blok, kayma, ray sorguları), konum-dönüş yaklaşması (minecart de hareket/ışınlanma paketlerinde aradeğerlenir), `minecarts.rs` çizici; `x_rot_at`. Kayıt: `docs/records/client-minecart-parity-conformance.md`.
- Bölündü: yeni (deneysel) davranış [T38c](T38c-minecart-yeni-davranis.md), TNT fitili ve spawner [T38d](T38d-minecart-tnt-spawner.md).
- Sapma: sandık minecart'ın sandığı (blok varlık modeli) görünmez; oracle'da blok ışığı yok.
