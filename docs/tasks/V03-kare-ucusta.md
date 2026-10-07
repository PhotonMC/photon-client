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
- İki kare uçuşta: `FrameSlot` (komut tamponu, fence, `image_available`) × `FRAMES_IN_FLIGHT = 2`; her kare numaralanır (`frames_submitted`), kaynaklar "hangi kareye kadar kullanılıyor" damgası taşır, `complete_through(n)` yalnız o kareye kadar bekler. `render_finished` swapchain görüntüsü başına kaldı; zaman damgası havuzu kare dilimi başına.
- `set_*` yüklemeleri artık fence beklemez: 20 dinamik vertex tamponu `DynamicBuffer` (iki tampon, dönüşümlü yazılır; yalnız o tamponu çizebilecek kareler beklenir; veri verilmeyen kare son veriyi çizer, eski davranış korunur). Bulut yüzleri (nadir) ile atlas animasyonu ve `clear_sections` hâlâ tüm karelerin bitmesini bekler.
- Bölüm/staging güvenliği damgayla: emekli bölüm ve staging parçası, kullanan son kare bitince serbest kalır. **V00'da kaydedilen açık sorun düzeltildi:** emekli edilen bölümün bekleyen kopyaları serbest bırakılmadan önce `staged` listesinden atılır (`free_retired`), böylece serbest/yeniden atanmış aralığa kopya kaydedilmez.
- Doğrulama: `VOXEL_VALIDATION=1` (Khronos katmanı, `bench --validation`) **0 hata, 0 uyarı** (önce, V03 öncesi kodda iki hata vardı: derinlik görüntüsünün ilk düzen geçişi ve `weather.frag`'ın kullanılmayan girdisi; ayrı commit'te düzeltildi).
- **Bench (Apple M1, 2560x1440, görüş 12, 2 koşu, V00 temel çizgisiyle):** kare p50/p95/p99 = 8.37/10.03/12.47 ms (temel 16.69/19.67/27.83); `mesh` aralığı 1.88 (10.43), fence beklemesi 0.00 (7.92), backend 6.14 (4.63), GPU sahne 5.66 (7.49); 30 sn'de 3458 kare (1779). Kare artık 16.67 ms'ye kilitli değil: CPU ile GPU örtüşüyor (süre ≈ ikisinin büyüğü). Ardışık koşu p50 farkı work_ms %0.4, fence %0.0, mesh %4.3, GPU %0.3.
- Bir `shot` karesi okundu: gökyüzü, bulut, arazi, bayrak, HUD, el doğru. Piksel eşitliği ölçülmedi (canlı dünya).
- **Kalan → V03b:** `RenderType` tablosu ve `*_buffer`+`*_count` alan çiftlerinin `DrawList`e inmesi (V01b'deki aynı madde buraya taşındı).
