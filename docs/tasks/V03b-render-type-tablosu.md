# V03b — `RenderType` tablosu ve `DrawList`

**Amaç:** V03'ün bölünen kalanı: çizim türlerini veri tablosuna indirmek, 20 `*_buffer` + `*_count` alan çiftini tek yapıya toplamak.
**Bağımlılık:** V03. **Boy:** M.

## Oku
- [V03 devir notu](V03-kare-ucusta.md)
- `crates/photon-render/src/vulkan/{mod,draws,frame,scene}.rs`, `crates/photon-render/src/render_types.rs`

## Yapılacaklar
- `RenderType { pipeline, layout, descriptor, blend }` tablosu (Java `RenderTypes` adlarıyla); kare `(RenderType, first, count)` listesiyle kaydedilir.
- `DynamicBuffer` + sayı çifti `DrawList` olur; `set_entities`/`set_shadows`/… çağrıları tek `set_draw(kind, vertices)` üzerinden (çağıranlarda mekanik değişiklik).
- `record_scene` (≈550 satır) bu listeyi tek döngüde çizer; `record_dissolve`/`record_maps`/`record_flat_shapes` kalkar.

## Kabul
`python3 tools/photon.py check` → `CHECK OK`; `bench --validation` 0 hata; bench sayıları V03 devir notundaki aralıkta; bir `shot` karesi doğru.

## Kapsam dışı
Senkronizasyon ve present modu.

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
- 16 `*_buffer` + `*_count` alan çifti tek `DrawList { buffer: DynamicBuffer, count }` alanına indi (`entity`, `ground`, `shadow`, `glow`, `swirl`, `glint`, `dissolve`, `rays`, `name_tag(_see)`, `orb`, `map(_decoration)`, `hand`, `outline`, `breaking`).
- Sekiz yalın `set_*` (`set_entities`, `set_entity_glow/swirl/glint`, `set_dragon_rays`, `set_ground_items`, `set_shadows`, `set_experience_orbs`) tek `set_draw(DrawKind, &[Vertex])` oldu; çağıranlar (`dropped_items`, `mobs/wild`, `play/play_draw`) mekanik güncellendi. Farklı imzalı olanlar (`set_entity_dissolve` ofsetli, `set_name_tags` iki liste, `set_hand`, `set_map_quads`, `set_outline`, `set_breaking`, `set_gui`, `set_sky`, `set_weather`, bulut) kendi setter'larında kaldı.
- Vulkan alt modülleri anlamlı adlandırıldı (`backend.rs` struct, `device/create.rs`, `frame/{sync,draw_calls,scene}.rs`).
- Doğrulama: 539 test, 1109 parity, `bench --validation` 0 hata/0 uyarı, bir `shot` karesi (arazi, domuzlar, bayraklar, HUD) doğru; bench kare p50 8.7 ms aralığında (V03 ile aynı).
- **Kalan → V03c:** `RenderType { pipeline, layout, descriptor, blend }` tablosu ve `record_scene`'in (≈550 satırlık tek işlev) tablo üzerinden tek döngüye indirilmesi. Çizim sırası ve blend durumu piksel eşitliğini belirlediği için ayrı bir görevde, kare başına çizim sırası birim testiyle sabitlenerek yapılmalı.
