# V04 — Present modu ve kare sınırı

**Amaç:** FPS'in ekran hızına kilitlenmesini kaldırmak; Java'nın vsync ve kare sınırı seçeneklerini uygulamak.
**Bağımlılık:** V03. **Boy:** S.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §5 (5.5)
- `vulkan.rs:1546`, `:3242`; Java `Options` (`enableVsync`, `framerateLimit`), `FramerateLimitTracker`

## Yapılacaklar
- Vsync açık → `FIFO`; kapalı → `MAILBOX` (yoksa `IMMEDIATE`, o da yoksa `FIFO`).
- CPU tarafı kare sınırlayıcı Java'nın sınır kurallarıyla (menüde/odaksızken düşük sınır dahil).
- Seçenek değişince swapchain yeniden kurulur.

## Kabul
Vsync kapalıyken bench ortalama FPS'i ekran hızını aşar; `framerateLimit` ±%2 içinde tutulur.

## Kapsam dışı
Video ayarları ekranı düzeni.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- Seçenekler: `Options`'a `enableVsync` (varsayılan true), `maxFps` (`intRange(10, 260)`, varsayılan 120, 260 = sınırsız) ve `inactivityFpsLimit` (`"minimized"`/`"afk"`, varsayılan afk) eklendi; Java'nın `processOptions` sırasıyla okunur/yazılır (birim testli). Video ekranındaki üç widget artık seçeneğe bağlı (başlangıç değerini seçenekten alır, değişimi `controls.rs` geri okur); ekran düzeni değişmedi, GUI parity eşit.
- Present modu: `choose_present_mode` Java'nın `GpuSurface.PresentMode.getSupportedVsyncMode` tercih listeleriyle (vsync: FIFO_RELAXED, FIFO; kapalı: IMMEDIATE, MAILBOX, FIFO); seçenek değişince `set_vsync` swapchain'i bir sonraki `resize`'da yeniden kurdurur. Seçilen mod `BackendDescription::present_mode` ve bench raporunda.
- Kare sınırlayıcı: `crates/voxel/src/frame_limiter.rs`, `FramerateLimiter.limitDisplayFPS` (uyku + son kısımda dönme, aşım ortalaması) ve `FramerateLimitTracker` kurallarının portu: simge durumu 10, 60 sn girdisiz (afk seçeneğiyle) `min(sınır, 30)`, 10 dk 10, 260+ sınırsız. Java'nın beşinci nedeni (seviyesiz menü, 60) yoktur: istemci yalnız seviyede çalışır. Girdi = klavye/fare olayları. Davranış birim testli.
- Ortam: `VOXEL_VSYNC=0|1` ve `VOXEL_MAX_FPS=N` seçenekleri dosyaya dokunmadan geçersiz kılar; `VOXEL_BENCH` varsayılan olarak sınırsızdır (işi ölçmek için).
- **Ölçüm (Apple M1, 2560x1440, bench `--warmup 5`):** `--fps-limit 60` → 59.6 FPS (%0.7), `--fps-limit 30` → 30.0 FPS, vsync açık/FIFO 114.5, vsync kapalı/IMMEDIATE 114.4. Sınır ±%2 içinde tutuluyor.
- **Kabul'ün ilk yarısı bu makinede gösterilemedi:** "vsync kapalıyken ortalama FPS ekran hızını aşar" için FIFO ile IMMEDIATE aynı FPS'i verdi; MoltenVK/macOS bu yapılandırmada FIFO'yu ekran hızına kilitlemiyor (kare yine ~8.4 ms, 60 Hz ekranın üstünde), kısıtlayan CPU+GPU süresi. Mod seçimi (FIFO ↔ IMMEDIATE) rapordan doğrulandı; ekran hızını aşma gösterimi Linux/Windows gerektirir.
