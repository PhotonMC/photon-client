# A04 — Tipli paket enum'u, listener, izlemeyi üretimden ayırma

**Amaç:** Ayrıştırmayı saf ve tek yerde, uygulamayı listener'da toplamak; test izleme yapılarını üretim durumundan ayırmak.
**Bağımlılık:** A01. **Boy:** L.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §3.4, §4.2
- `crates/voxel-net/src/connection/play.rs` (`PlaySession :71`, `handle :1563`), `protocol.rs:598+`

## Yapılacaklar
- `ClientboundPlay` enum + `Decode` trait; okuyucu iş parçacığı çözülmüş paketleri gönderir.
- `PlayListener` (Java `ClientPacketListener`) `ClientState`'i günceller; dünya `trace` içinden çıkar.
- `*Observation` ve `PlayConnectionTrace` bir `Recorder` arayüzüne; üretimde boş uygulama.
- Ayrıştırma hataları kaydedilir (Q03).

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Protokol davranışı.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
