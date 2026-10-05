# T38c — Minecart'ın yeni (deneysel) davranışı

**Amaç:** `minecart_improvements` bayrağı sunucudan açıksa `NewMinecartBehavior`: `ClientboundMoveMinecartAlongTrack` paketi ve `lerpSteps`, `cartLerpPosition/XRot/YRot`, `getRenderOffset`, yeni çizim (`newRender`). Bayrak `ClientboundUpdateEnabledFeatures` ile gelir.
**Bağımlılık:** T38b. **Boy:** M.

## Oku
- Java: `world/entity/vehicle/minecart/NewMinecartBehavior.java` (`cartLerp*`, `lerpSteps`, `tick` istemci kolu), `AbstractMinecartRenderer.newExtractState/newRender/getRenderOffset`, `ClientPacketListener.handleMinecartAlongTrack`, `network/protocol/game/ClientboundMoveMinecartPacket.java`.
- Rust: `crates/voxel-world/src/minecart.rs`, `crates/voxel/src/minecarts.rs`, `crates/voxel-render/src/minecart.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
