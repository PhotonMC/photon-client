# T37c — Shulker

**Amaç:** `ShulkerModel` (kapak, bükülme, baş dönüşü), `ShulkerRenderer.setupRotations` (tutunma yüzü), renk dokusu, `Shulker.tick` kapak durumu ve veri alanları.
**Bağımlılık:** T37. **Boy:** M.

## Oku
- Java: `world/entity/monster/Shulker.java`, `client/model/monster/shulker/ShulkerModel.java`, `client/renderer/entity/{ShulkerRenderer,state/ShulkerRenderState}.java`.
- Rust: `crates/voxel-world/src/{shulker,entity}.rs`, `crates/voxel/src/{shulkers,cube_mobs}.rs` (kalıp).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `voxel-world/shulker.rs` (kapak tiki), veriler 16-18, gövde kontrolü boş ve yaratılınca gövde güney, `MobModel::setup_shulker`, `MobPlacement::attach_face` (`Direction.getRotation`, `Pose::rotate_around` JOML sırasına çekildi), `crates/voxel/src/shulkers.rs`. Kayıt: `docs/records/client-shulker-parity-conformance.md`.
- Oracle: `entity/shulker-model` 21, `entity/shulker-rotations` 13 (üretimdeki feet/scale önsözüyle: sıfır işareti yoluna bağlı), `living-entity-animations`'a 6 satır; `shulker_data` olayı.
- Sapma ve kalan: konum yakalama, blok değişince açıklık sıfırı, ışınlanma kaydırması, vuruş kutusu T37c2'de; derinlik kaydırmalı render türü, ses ve mermi yok. Canlı gözlem yapılmadı.
