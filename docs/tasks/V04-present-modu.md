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
(boş)
