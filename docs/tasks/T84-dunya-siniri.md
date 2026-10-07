# T84 — Dünya sınırı

**Amaç:** `InitializeBorder`, `SetBorderCenter/Size/LerpSize/WarningDelay/WarningDistance` paketleri, `WorldBorder` durumu ve hareketli sınır, `WorldBorderRenderer` (çizim), oyuncunun sınırla çarpışması.
**Bağımlılık:** –. **Boy:** M.

## Oku
Java: `world/level/border/WorldBorder.java`, `client/renderer/WorldBorderRenderer.java`, `network/protocol/game/ClientboundInitializeBorderPacket.java` ve `ClientboundSetBorder*`. Rust: `crates/voxel-world/src/lib.rs`, `crates/voxel-world/src/local_player.rs`.

## Oracle
`border_state` operation'ı: `lerp` durumu, çarpışma kutusu, çizim köşeleri. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
- Yapıldı: altı sınır paketi canlı oturumda işleniyor; sınırın çarpışma şekli yerel oyuncunun, binekler ve yerdeki eşyanın hareketine giriyor (`Level::{colliders, no_collision}` kaynak konumu alır); oracle'a `border` alanı ve 5 satır, gerçek `LocalPlayer` ile eşit. Kayıt: [`client-world-border-live-parity-conformance.md`](../client-world-border-live-parity-conformance.md).
- Kalan: sınırın çizimi [T84b](T84b-sinir-cizimi.md)'ye ayrıldı.
