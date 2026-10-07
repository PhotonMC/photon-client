# A05 — `ClientWorld` alt sistemlere bölme

**Amaç:** 184 public fonksiyonlu tek `impl`'i alt sistem tiplerine bölmek.
**Bağımlılık:** A02. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §3.4, §4.5
- `crates/voxel-world/src/lib.rs:120-4760`

## Yapılacaklar
- `Chunks`, `Lighting`, `Entities`, `Weather`, `Recipes`, `WorldBorder`, `BlockPredictions` alanları; fonksiyonlar sahibine taşınır.
- Çağıranlar `world.entities().…` biçimine mekanik geçer.
- Chunk/entity haritası için hash/slab değişikliği yalnız V00 ölçümüyle ve sıra bağımlılığı denetlenerek.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Davranış değişikliği.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- `world/entities/store.rs`: `Entities { by_id, sections }` (entity haritası ve bölüm indeksi), `ClientWorld::entities()`/`entities_mut()`; okuma `Deref` ile haritaya geçer, yazma `by_id`/`at_mut` ile. Yalnız entity alanlarına dokunan 26 yöntem taşındı ve adları kısaldı (`entity_at`→`entities().at`, `apply_entity_metadata`… hariç; `apply_entity_attributes`→`apply_attributes`, `apply_entity_motion`→`apply_motion`, `loaded_entity_count`→`count`, `fix_*_seed` aynı ad). Eski `entities()` yineleyicisi `entities().all()`; çağıranlar betikle geçirildi (≈650 yer), davranış aynı (542 test, 1109 parity).
- Gözlem: entity yöntemlerinin çoğu yerel oyuncuyu da "entity" sayar (`local_player` yedeği: `entity_exists`, `apply_entity_metadata`, `set_entity_vehicle_id`, `apply_entity_head_rotation`, `clamp_entity_passenger_rotation_after_turn`) ya da zemin/akışkan için `ClientWorld`'ün tamamını ister (`mob_in_water`, tik): bunlar `ClientWorld`'de kaldı. Bölme ancak yerel oyuncunun entity deposuna alınmasıyla (ya da `Entities` yöntemlerinin `&LocalPlayerSlot` almasıyla) ilerler.
- Kalan → [A05b](A05b-chunk-isik-hava.md): `Chunks` (chunk haritası, önbellek, ışık motoru ve görevleri), `Weather`/`Level` bilgisi, yerel oyuncu–entity bağı.
