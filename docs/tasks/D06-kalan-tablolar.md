# D06 — Sesler, kalan tablolar ve çalışma anı veri okuyucu

**Amaç:** Kalan küçük tabloları çevirmek; (b) sınıfı veriyi çalışma anında okumak.
**Bağımlılık:** D01. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §6
- D01 envanteri; `generated/{sound_events,entity_types,entity_attributes,painting_variants,wolf_variants,sheep_colors,map_data,…}.rs`

## Yapılacaklar
- (a) sınıfı: `SoundEvents`, `EntityType`, `Attributes` karşılıkları elle.
- (b) sınıfı: kullanıcının yerel oyun dosyasından (assetlerle aynı seçili klasör) veya `RegistryData`'dan okunan JSON; derlemeye gömülmez.
- Parity karşılaştırmaları korunur.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Yeni oyun özelliği.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
