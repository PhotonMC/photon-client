# T40g2 — Çerçevedeki haritanın içeriği

**Amaç:** `ItemFrameRenderer`'in `mapId` kolu: `ClientboundMapItemData` paketi (renkler, süsler), istemci harita deposu (`level.getMapData`), `MapRenderer` (128×128 doku, süsler), harita çerçevesi modeli (`map=true`) ve harita ışığı (`15728850`, parlak çerçevede). Şimdi harita taşıyan çerçeve haritanın verisi gelmemiş gibi çizilir.
**Bağımlılık:** T40g. **Boy:** L.

## Oku
- Java: `client/renderer/MapRenderer.java`, `client/renderer/state/MapRenderState.java`, `client/multiplayer/ClientPacketListener.handleMapItemData`, `network/protocol/game/ClientboundMapItemDataPacket.java`, `world/level/saveddata/maps/MapItemSavedData.java`, `ItemFrameRenderer.java` (map kolu).
- Rust: `crates/voxel/src/item_frames.rs`, `crates/voxel-render/src/item_frame.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
