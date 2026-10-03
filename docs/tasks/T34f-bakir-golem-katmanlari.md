# T34f — Bakır golemin elinde, antende ve başında taşıdığı

**Amaç:** `CopperGolemRenderer`'ın `ItemInHandLayer`, `BlockDecorationLayer` (antendeki blok: `applyBlockOnAntennaTransform`, `blockOnAntenna`) ve `CustomHeadLayer` (`translateToHead`) katmanlarını çizmek; `CopperGolemModel.translateToHand` (`IDLE` iken el `∓90°` döner ve `0,125` öne gider, aksi hâlde `0,55` ölçeklenir ve `(-0,125; 0,3125; -0,1875)` kayar).
**Bağımlılık:** T34e, T41c, T34d. **Boy:** M.

## Oku
- Java: `client/model/animal/golem/CopperGolemModel.java` (`translateToHand`, `translateToHead`, `applyBlockOnAntennaTransform`), `client/renderer/entity/CopperGolemRenderer.java`, `client/renderer/entity/layers/{ItemInHandLayer,BlockDecorationLayer,CustomHeadLayer}.java`, `world/entity/animal/golem/CopperGolem.java` (`EQUIPMENT_SLOT_ANTENNA` = `SADDLE`).
- Rust: `crates/voxel-render/src/{held_item,entity_block}.rs`, `crates/voxel/src/{held_items,golems,copper_golems}.rs`.

## Yapılacaklar
1. Üç katmanın pozları (oracle ile): elde eşya (iki durumda), antendeki blok durumu (`BlockItemStateProperties` uygulanmış varsayılan durum), kafa eşyası.
2. `copper_golems.rs` içinde `held` listesine toplamak (T34d'nin blok yolu, T41'in eşya yolu).

## Oracle
`villager_held_layers` ve `entity_block_layers` kalıbı: gerçek katmanlar, yakalayıcı toplayıcı; golemin durumu `IDLE` ve öteki.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon copper_golem` ile eşyalı ve antende çiçek/blok taşıyan golem.

## Kapsam dışı
Gölge, sesler, heykel dönüşümü.

## Belgeler
`docs/client-copper-golem-parity-conformance.md` içindeki sapma satırı güncellenir.

## Devir notu
- Yapıldı: `copper_golem_{hand,head,antenna}_pose` (voxel-render), joml'a `Matrix4::{translation, rotate_around}`, `Matrix3::mul`, `Pose::mul_matrix`; `copper_golems.rs` elleri, kafa eşyasını ve antendeki bloğu (`BlockItem` + `block_state` bileşeni) `held` listesine toplar; `item_components::block_state_properties`. Kayıt: `docs/client-copper-golem-parity-conformance.md`.
- Oracle: `entity/copper-golem-layers` 27 satır (gerçek `ItemInHandLayer`, `CustomHeadLayer`, `BlockDecorationLayer`); anten matrisi `rotateAround` dâhil bit bit eşit.
- Sapma: parlayan golemin blok dış hattı yok; canlı gözlem yapılmadı.
