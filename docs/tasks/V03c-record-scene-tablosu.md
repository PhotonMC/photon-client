# V03c — `RenderType` tablosu ve `record_scene`'in tek döngüye inmesi

**Amaç:** V03b'nin kalanı: çizim türleri veri tablosunda, kare tek döngüyle kaydedilir.
**Bağımlılık:** V03b. **Boy:** M.

## Oku
- [V03b devir notu](V03b-render-type-tablosu.md)
- `crates/photon-render/src/vulkan/frame/scene.rs` (`record_scene`), `draw_calls.rs`, `crates/photon-render/src/render_types.rs`

## Yapılacaklar
- `RenderType { pipeline, layout, descriptor, blend, depth }` (Java `RenderTypes` adlarıyla) ve çizim sırasını veren dizi.
- `record_scene` her tür için `bind pipeline → descriptor → push → vertex buffer → draw` döngüsünü tek yerde yapar; `record_dissolve`/`record_maps`/`record_flat_shapes`/`draw_triangles` kalkar.
- Mevcut çizim sırası (gökyüzü, arazi, entity, yüzeyler, yarı saydam, bulut/hava, el, GUI) bir birim testiyle (komut kaydı yapan sahte backend ya da sıra listesi) sabitlenir.

## Kabul
`python3 tools/photon.py check` → `CHECK OK`; `bench --validation` 0 hata; bir `shot` karesi öncekiyle görsel eşit; kare sırası testi.

## Kapsam dışı
Senkronizasyon, yeni çizim türü.

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
- `vulkan/frame/passes.rs`: `Pass` ve `PASSES` (çizim sırası, her geçişten sonra yazılan zaman damgası `stamp_after`), `Run` ve `ENTITY_RUNS` (entity/yüzey çizimlerinin sırası) ile her çalıştırmanın boru hattı ve örnekleyeceği set (`Run::pipeline`, `Run::sampled`). Üç birim testi sırayı sabitler (gökyüzü ilk, GUI son, arazi < entity < yarı saydam < bulut < hava < el; damgalar 1…7).
- `record_scene` 575 satırdan ~70'e indi: görüntüleme kurulumu `begin_rendering`, sonra `for pass in PASSES` ve geçiş başına küçük işlev (`record_sky/terrain/translucent/clouds/weather/hand/gui`). Tek tür çizimleri `draw_run` (Run → bind pipeline, set, push, vertex tamponu, draw) yapar; `record_dissolve`, `record_maps`, `record_flat_shapes`, `draw_triangles` kalktı, çıplak çağrılar `draw_vertices`/`bind_state`'e indi.
- Görev metninden sapma: tür yapısı `RenderType { pipeline, layout, descriptor, blend, depth }` yerine iki sıralı tablo (`Pass`, `Run`); boru hattı bağlamı (`Pipeline` = boru hattı + yerleşim) V01b'de oluştu, harmanlama/derinlik durumu boru hattı oluşturmada (`PipelineKind`) kaldı. Java `RenderTypes` adlarıyla eşleme yapılmadı.
- Doğrulama: `bench --validation` 0 hata 0 uyarı; makine sessiz değildi (ekran 60/120 Hz arasında oynadı, koşular arası fark %29), bu yüzden taban ve yeni ikili aynı oturumda arka arkaya koşuldu: taban 8.63/8.33 ms, yeni 4.17/8.35 ms (p50) — gerileme yok. `shot` karelerinde gökyüzü, arazi, bayraklar, bulut, ok, el + eşya ve GUI doğru.
