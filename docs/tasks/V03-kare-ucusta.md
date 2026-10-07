# V03 — İki kare uçuşta, halka tampon, `RenderType` tablosu

**Amaç:** CPU ve GPU'nun örtüşmesini sağlamak: kare başına kaynaklar, `set_*` içindeki fence beklemelerinin kaldırılması.
**Bağımlılık:** V01. **Boy:** L.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §5 (5.1–5.4)
- `vulkan.rs` `render_frame :4954`, `fill_bytes :4695`, `record_scene :2078`

## Yapılacaklar
- `FrameResources { cmd, fence, image_available, ring }` × 2; `render_finished` swapchain görüntüsü başına kalır.
- Dinamik köşeler kalıcı map'li, kare başına halka tampona yazılır; `set_*` fence beklemez.
- Çizim türleri veri tablosu: `RenderType { pipeline, layout, descriptor, blend }` (Java `RenderTypes` adlarıyla); kare `(RenderType, first, count)` listesiyle tek döngüde kaydedilir; ~20 `Option<SkyBuffer>`+count alanı kalkar.
- Bölüm yüklemesi (`staging`/`retired`) kare indeksine göre güvenli serbest bırakılır.
- Validation layer (debug) temiz.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit. Bench: p50 kare süresi V00'a göre düşük (beklenti CPU+GPU toplamı yerine en büyüğü).

## Kapsam dışı
Present modu (V04).

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
