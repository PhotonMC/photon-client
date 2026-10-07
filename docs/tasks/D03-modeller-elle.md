# D03 — Entity modelleri: model sınıfı başına `create_body_layer`

**Amaç:** `generated/entity_models.rs` katmanlarını Java model sınıflarına karşılık gelen fonksiyonlara çevirmek.
**Bağımlılık:** D01. **Boy:** L.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §6
- D01 envanteri, `crates/voxel-data/src/{generated/entity_models.rs,entity_model.rs}`

## Yapılacaklar
- `voxel-data/src/model/<model>.rs` içinde `create_body_layer() -> LayerDefinition`; `LayerDefinitions` karşılığı kayıt fonksiyonu.
- Statik dizi yerine ilk kullanımda kurulup önbelleğe alınabilir (`OnceLock`).
- Parity karşılaştırması bütün katmanları doğrular; sığmazsa aile aile bölünür.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Model çizimi.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
