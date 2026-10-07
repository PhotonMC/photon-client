# V00 — Performans ölçümü ve bench

**Amaç:** Optimizasyonlardan önce tekrarlanabilir kare süresi ve GPU pass süreleri elde etmek.
**Bağımlılık:** –. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §5 (5.6)
- `crates/voxel/src/{play,script,profile}.rs`, `crates/voxel-render/src/vulkan.rs` (`render_frame`)

## Yapılacaklar
- Kare süresi istatistiği: p50/p95/p99, ortalama FPS; CPU tarafında tik, mesh, extract, kayıt, submit süreleri.
- GPU zaman damgası sorguları (`vkCmdWriteTimestamp`) pass başına: gökyüzü, bölümler, entity, şeffaf, GUI.
- `python3 tools/voxel.py bench`: sabit betik, sabit görüş mesafesi, sabit konum; JSON rapor yerelde.
- Mevcut değerler (temel çizgi) bu görevin devir notuna yazılır; sonraki V görevleri karşılaştırır.

## Kabul
Aynı makinede ardışık iki koşu arasında p50 farkı < %5; temel çizgi kayıtlı.

## Kapsam dışı
Herhangi bir optimizasyon.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- Eklendi: `crates/voxel-render/src/frame_timing.rs` (CPU aralıkları, GPU pass süreleri), `vulkan.rs`'te `vkCmdWriteTimestamp` sorguları (8 pass, 9 damga) ve tek `wait_in_flight` (her `set_*` beklemesi artık ölçülüyor), `crates/voxel/src/bench.rs` (`VOXEL_BENCH`, JSON rapor), `python3 tools/voxel.py bench [adres] [--repeat N]`.
- **Temel çizgi** (Apple M1, MoltenVK, 2560x1440 (1280x720 istenir, HiDPI ×2), görüş 12, 637 chunk, ~65 entity, 3 koşunun medyanı, warm-up 10 sn, ~1780 kare): kare p50/p95/p99 = 16.69/19.67/27.83 ms. CPU p50: tik 0.15, mesh 10.43, extract 0.07, backend 4.63 (kayıt 0.30, submit+present 4.25), fence beklemesi 7.92. GPU sahne p50/p95 = 7.49/10.21 ms.
- Ardışık koşular arası p50 farkı: work_ms %0.0–0.1, fence_wait %0.9–1.4, mesh %0.8–1.7, GPU sahne %0.5–0.7 (<%5, kabul sağlandı).
- **Okuma:** kare süresi ekranın 16.67 ms'sine kilitli (FIFO), bu yüzden kare p50 tek başına iyileşmeyi göstermez; V03 için `fence_wait`, V04 sonrası kare süresi, V05/V06 için GPU sahne ve `mesh`/`backend` karşılaştırılır. `mesh` aralığındaki ~8 ms'nin büyük kısmı `set_*` yüklemelerinin fence beklemesidir (rapor 5.1/5.2: CPU ile GPU sırayla çalışıyor); bekleme sayılınca asıl CPU işi ~2.5 ms.
- **Açık/sınır:** bu sürücüde (MoltenVK, Apple GPU) tek render pass içindeki damgalar pass sınırında çözülüyor: `sky` sahnenin ~%99'unu taşıyor, diğer yedi pass ~0.013 ms. Pass dökümü yok; yalnız sahne toplamı güvenilir. Rapor bunu `pass_breakdown_resolved: false` ile söyler. Döküm için masaüstü GPU gerekir; pass'leri ayrı render pass'e bölmek ölçülen performansı değiştirirdi, yapılmadı.
- Bench sabit görüş mesafesini sunucudan alır (`view-distance=12`); rapor `render_distance`'ı yazar. Sunucu başka görüş veya entity sayısıyla çalışırsa koşular karşılaştırılamaz (rapora `loaded_chunks` ve `entities` yazılır).
- Koşu 25565'teki çalışan sunucuya karşı yapıldı (`Bench` kullanıcısı, dünya spawn'ında durur).
