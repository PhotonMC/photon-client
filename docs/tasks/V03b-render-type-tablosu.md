# V03b — `RenderType` tablosu ve `DrawList`

**Amaç:** V03'ün bölünen kalanı: çizim türlerini veri tablosuna indirmek, 20 `*_buffer` + `*_count` alan çiftini tek yapıya toplamak.
**Bağımlılık:** V03. **Boy:** M.

## Oku
- [V03 devir notu](V03-kare-ucusta.md)
- `crates/voxel-render/src/vulkan/{mod,draws,frame,scene}.rs`, `crates/voxel-render/src/render_types.rs`

## Yapılacaklar
- `RenderType { pipeline, layout, descriptor, blend }` tablosu (Java `RenderTypes` adlarıyla); kare `(RenderType, first, count)` listesiyle kaydedilir.
- `DynamicBuffer` + sayı çifti `DrawList` olur; `set_entities`/`set_shadows`/… çağrıları tek `set_draw(kind, vertices)` üzerinden (çağıranlarda mekanik değişiklik).
- `record_scene` (≈550 satır) bu listeyi tek döngüde çizer; `record_dissolve`/`record_maps`/`record_flat_shapes` kalkar.

## Kabul
`python3 tools/voxel.py check` → `CHECK OK`; `bench --validation` 0 hata; bench sayıları V03 devir notundaki aralıkta; bir `shot` karesi doğru.

## Kapsam dışı
Senkronizasyon ve present modu.

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
(boş)
