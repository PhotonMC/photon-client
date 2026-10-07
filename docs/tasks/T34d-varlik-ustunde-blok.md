# T34d — Varlık üstünde blok durumu çizimi

**Amaç:** `BlockModelRenderState` ile `BlockDisplayContext`: kar golemin balkabağı (`SnowGolemHeadLayer`), demir golemin çiçeği (`IronGolemFlowerLayer`, `offerFlowerTick > 0` iken gelincik) ve endermanın taşıdığı blok (`CarriedBlockLayer`, `DATA_CARRY_STATE` blok durumu). Üçü de blok durumunu eşya değil blok modeli olarak, ekran dönüşümsüz çizer.
**Bağımlılık:** T34c. **Boy:** M.

## Oku
- Java: `client/renderer/block/{BlockModelRenderState,BlockModelResolver}.java`, `client/renderer/entity/layers/{SnowGolemHeadLayer,IronGolemFlowerLayer,CarriedBlockLayer}.java`, `client/renderer/entity/{SnowGolemRenderer,IronGolemRenderer,EndermanRenderer}.java`.
- Rust: `crates/voxel/src/mobs/humanoids/golems.rs`, `monsters.rs`, `voxel-render/src/{world_mesh,item_entity,held_item}.rs` (blok modeli ağı), `voxel-world/src/metadata.rs` (`enderman_flags_from_metadata`: taşıma durumunun blok kimliği de okunur).

## Yapılacaklar
1. Bir blok durumunu (kimlikle) blok modeli köşelerine çeviren ve katman dönüşümüyle çizen yol (`BlockDisplayContext`, ışık, saydamlık).
2. Üç katman: balkabağı (baş dönüşümü), çiçek (kol dönüşümü), taşınan blok (`CarriedBlockLayer` dönüşümü).

## Oracle
Katman dönüşümleri ve çözülen blok modeli köşeleri gerçek `BlockModelResolver.update` ile; `submitModel` yakalayıcısı (`villagerLayers` kalıbı).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon snow_golem`, enderman'e blok.

## Kapsam dışı
Eşya olarak çizilen bloklar (T25), ışıltı dış hatları.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: `BlockRenderer::entity_block` (tohum 42, sabit renkler, ışık), `voxel-render/src/entity_block.rs` (üç katmanın dönüşümü, `max_light`), `HeldItem::block` ve `Carried` (blok modelleri eşyalarla aynı çizim yolundan), enderman taşıma durumu (kimlik) ve titremesi, kar golemi balkabağı verisi, demir golem çiçeği. Kayıt: `docs/records/client-entity-block-parity-conformance.md`.
- Oracle: `entity/block-layers` 8 satır (gerçek katmanlar, yakalayıcı toplayıcı); `variant-fields` iki satır. Canlı: balkabağı ve taşınan blok `shot` ile görüldü (çiçek görülmedi).
- Sapma: blok modeli köşeleri uçtan uca karşılaştırılmadı; özel blok modelleri, parlayan görünmezde dış hat yok.
- Açık: blok çizen diğer tüketiciler (mantar ineği mantarları T30b, TNT/düşen blok/display T40b, minecart T38) aynı `entity_block` yolunu kullanır.

