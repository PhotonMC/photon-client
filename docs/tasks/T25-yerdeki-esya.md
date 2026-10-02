# T25 — Yerdeki eşya (XP küresi T25b'ye ayrıldı)

**Amaç:** `ItemEntity`: eşya modelini yerde dönen/sallanan olarak çizme, yığın boyutuna göre kopya sayısı. `ExperienceOrb` [T25b](T25b-xp-kuresi.md)'de.
**Bağımlılık:** T24. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/renderer/entity/ItemEntityRenderer.java` (155), `.data/minecraft/source/net/minecraft/client/renderer/entity/ExperienceOrbRenderer.java`, `.data/minecraft/source/net/minecraft/client/renderer/entity/state/ItemEntityRenderState.java`, `.data/minecraft/source/net/minecraft/world/entity/item/ItemEntity.java` (`getSpin`, `bobOffs`, `age`; `grep -n "bobOffs\|getSpin\|DATA_ITEM"`), `.data/minecraft/source/net/minecraft/client/renderer/item/ItemModelResolver.java`.
- Rust: `crates/voxel-render/src/item_model.rs` + `gui/items.rs` + `crates/voxel/src/item_layers.rs` (eldeki/GUI eşya çizimi: **yer** bağlamı `ItemDisplayContext::GROUND` var mı bak), T24 çizim yolu, `SetEntityData` meta verisi (eşya yığını okuma: `crates/voxel-world/src/metadata.rs`).

## Yapılacaklar
1. `ItemEntity` meta verisinden yığın; `ItemDisplayContext::GROUND` dönüşümü, bob/spin formülü (`partial_tick` + `age` + `bobOffs`), yığın sayısına göre kopya sayısı (`getRenderedAmount`) ve rasgele ofset (entity kimliğinden seed'li; **Java'nın `RandomSource.create(seed)` akışı** `voxel_core::random`).
2. ~~`ExperienceOrb`~~ → T25b.
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
- Yapıldı: `voxel-render/src/item_entity.rs` (`rendered_amount`, `item_entity_poses` bob/spin/kopya ofseti `RandomSource` akışıyla, `model_extent`, `item_entity_vertices`), `crates/voxel/src/dropped_items.rs` (`minecraft:item` entity'leri `ItemDisplayContext::GROUND` katmanlarıyla), `VulkanBackend::set_ground_items` (blok atlası, `held_pipeline`). Oracle: `item_entity_pose` (gerçek `ItemEntityRenderer` matrisleri, bit bit; 1–5 kopya, kalın/ince model, eşik, yaş/bob/seed) geçti. Canlı: elmas, kütük yığını, çubuk doğru görünür.
- Sapma: hasar `seed_for_stack`'e 0 gidiyor (bileşenlerden okunmuyor), gölge ve toplama animasyonu yok, eşya ışığı entity'nin 0,2125 yukarısından tek örnek.
- Açık: T25b (XP küresi); [T25c](T25c-esya-fizigi.md) (istemci fiziği: `ItemEntity` tiki yok, bu yüzden blok kırınca eşya düşmeden yere inmiş görünür; canlı denemede kullanıcı fark etti). Oracle sahte `ItemEntityRenderState` ile sınırlı; model kutusu `item_model` çıktısından hesaplanır, onun Java ile karşılaştırması yalnız canlı görüntü.
