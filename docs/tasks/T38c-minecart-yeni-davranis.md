# T38c — Minecart'ın yeni (deneysel) davranışı

**Amaç:** `minecart_improvements` bayrağı sunucudan açıksa `NewMinecartBehavior`: `ClientboundMoveMinecartAlongTrack` paketi ve `lerpSteps`, `cartLerpPosition/XRot/YRot`, `getRenderOffset`, yeni çizim (`newRender`). Bayrak `ClientboundUpdateEnabledFeatures` ile gelir.
**Bağımlılık:** T38b. **Boy:** M.

## Oku
- Java: `world/entity/vehicle/minecart/NewMinecartBehavior.java` (`cartLerp*`, `lerpSteps`, `tick` istemci kolu), `AbstractMinecartRenderer.newExtractState/newRender/getRenderOffset`, `ClientPacketListener.handleMinecartAlongTrack`, `network/protocol/game/ClientboundMoveMinecartPacket.java`.
- Rust: `crates/voxel-world/src/objects/minecart.rs`, `crates/voxel/src/objects/minecarts.rs`, `crates/voxel-render/src/objects/minecart.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `minecart_improvements` bayrağı yapılandırmada okunur (`ClientWorld::set_minecart_improvements`); bundan sonra eklenen minecart'lar `TrackLerp` taşır (`voxel-world/minecart_track.rs`: `lerpSteps`, `currentLerpSteps`, `getCurrentLerpStep` ve tek girişlik belleği); `MoveMinecartAlongTrack` (kimlik 55) ayrıştırılıp adımlar eklenir; konum/dönüş paketleri yeni davranışta yaklaşım olmadan hemen koyar; çizim `newExtractState`/`getRenderOffset`/`newRender`.
- Kanıt: `entity/minecart-pose` +4 satır, `living-entity-animations` +6 satır (gerçek `Minecart` + gerçek `handleMinecartAlongTrack`; ağırlıksız adımlarda bellek eskiliği dahil). Kayıt: `docs/records/client-minecart-parity-conformance.md`.
- Sapma/açık: istemcinin `onRails` bayrağı tutulmaz (yalnız ses, T64b); yeni davranışın sunucu kolu (`moveAlongTrack`, itme/toplama) sunucunundur, yazılmadı. Oracle'da çizim konumu `getRenderOffset` + dağıtıcı toplamı değil, bileşenleri ayrı ayrı (oynatma yeri) karşılaştırılır.
