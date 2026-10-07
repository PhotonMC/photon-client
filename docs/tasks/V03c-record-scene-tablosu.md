# V03c — `RenderType` tablosu ve `record_scene`'in tek döngüye inmesi

**Amaç:** V03b'nin kalanı: çizim türleri veri tablosunda, kare tek döngüyle kaydedilir.
**Bağımlılık:** V03b. **Boy:** M.

## Oku
- [V03b devir notu](V03b-render-type-tablosu.md)
- `crates/voxel-render/src/vulkan/frame/scene.rs` (`record_scene`), `draw_calls.rs`, `crates/voxel-render/src/render_types.rs`

## Yapılacaklar
- `RenderType { pipeline, layout, descriptor, blend, depth }` (Java `RenderTypes` adlarıyla) ve çizim sırasını veren dizi.
- `record_scene` her tür için `bind pipeline → descriptor → push → vertex buffer → draw` döngüsünü tek yerde yapar; `record_dissolve`/`record_maps`/`record_flat_shapes`/`draw_triangles` kalkar.
- Mevcut çizim sırası (gökyüzü, arazi, entity, yüzeyler, yarı saydam, bulut/hava, el, GUI) bir birim testiyle (komut kaydı yapan sahte backend ya da sıra listesi) sabitlenir.

## Kabul
`python3 tools/voxel.py check` → `CHECK OK`; `bench --validation` 0 hata; bir `shot` karesi öncekiyle görsel eşit; kare sırası testi.

## Kapsam dışı
Senkronizasyon, yeni çizim türü.

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
(boş)
