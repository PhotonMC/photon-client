# T40g2 — Çerçevedeki haritanın içeriği

**Amaç:** `ItemFrameRenderer`'in `mapId` kolu: `ClientboundMapItemData` paketi (renkler, süsler), istemci harita deposu (`level.getMapData`), `MapRenderer` (128×128 doku, süsler), harita çerçevesi modeli (`map=true`) ve harita ışığı (`15728850`, parlak çerçevede). Şimdi harita taşıyan çerçeve haritanın verisi gelmemiş gibi çizilir.
**Bağımlılık:** T40g. **Boy:** L.

## Oku
- Java: `client/renderer/MapRenderer.java`, `client/renderer/state/MapRenderState.java`, `client/multiplayer/ClientPacketListener.handleMapItemData`, `network/protocol/game/ClientboundMapItemDataPacket.java`, `world/level/saveddata/maps/MapItemSavedData.java`, `ItemFrameRenderer.java` (map kolu).
- Rust: `crates/voxel/src/objects/item_frames.rs`, `crates/voxel-render/src/objects/item_frame.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `MapItemData` paketi ve istemci harita deposu (`voxel-world/map.rs`), `MapColor` ve süs türü tabloları (oracle'dan, `map_data.rs`), `MapRenderer` geometrisi (`voxel-render/map.rs`; gerçek `MapRenderer.render`'a karşı) ve çerçevenin harita dalı (poz, ışık, model `map=true`), Vulkan'da harita görüntüsü ve iki yeni arabellek, oyunda `item_frames::MapTextures`. Kayıt: `docs/records/client-item-frame-parity-conformance.md`. Canlıda görüldü.
- Açık: süs adları, 16 harita yuvası, elde taşınan harita.
