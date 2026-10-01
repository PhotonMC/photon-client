# T25 — Yerdeki eşya ve XP küresi

**Amaç:** `ItemEntity` (eşya modelini yerde dönen/sallanan olarak çizme, yığın boyutuna göre kopya sayısı) ve `ExperienceOrb`.
**Bağımlılık:** T24. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/renderer/entity/ItemEntityRenderer.java` (155), `.data/minecraft/source/net/minecraft/client/renderer/entity/ExperienceOrbRenderer.java`, `.data/minecraft/source/net/minecraft/client/renderer/entity/state/ItemEntityRenderState.java`, `.data/minecraft/source/net/minecraft/world/entity/item/ItemEntity.java` (`getSpin`, `bobOffs`, `age`; `grep -n "bobOffs\|getSpin\|DATA_ITEM"`), `.data/minecraft/source/net/minecraft/client/renderer/item/ItemModelResolver.java`.
- Rust: `crates/voxel-render/src/item_model.rs` + `gui/items.rs` + `crates/voxel/src/item_layers.rs` (eldeki/GUI eşya çizimi: **yer** bağlamı `ItemDisplayContext::GROUND` var mı bak), T24 çizim yolu, `SetEntityData` meta verisi (eşya yığını okuma: `crates/voxel-world/src/metadata.rs`).

## Yapılacaklar
1. `ItemEntity` meta verisinden yığın; `ItemDisplayContext::GROUND` dönüşümü, bob/spin formülü (`partial_tick` + `age` + `bobOffs`), yığın sayısına göre kopya sayısı (`getRenderedAmount`) ve rasgele ofset (entity kimliğinden seed'li; **Java'nın `RandomSource.create(seed)` akışı** `voxel_core::random`).
2. `ExperienceOrb`: küre sprite'ı (`textures/entity/experience_orb.png` 4×4 ızgara), renk salınımı, boyut (`value` → ikon indeksi).
3. Işık: entity konumu; gölge yok.

## Oracle
`item_entity_pose` operation'ı (entity ailesi): Java `ItemEntityRenderer`'ın hesapladığı dönüş/sallanma/kopya ofsetleri (sahte `ItemEntityRenderState` ile) — sayısal eşitlik.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot --rcon "summon item ~2 ~ ~ {Item:{id:diamond,count:3}}"`): görüntü bir kez.

## Kapsam dışı
Toplama animasyonu (`TakeItemEntity`), birleşme.

## Belgeler
`docs/remaining-work.md` M4 madde 3.

## Devir notu
(boş)
