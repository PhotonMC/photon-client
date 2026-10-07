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
(boş)
