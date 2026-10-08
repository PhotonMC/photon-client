# T83 — Ekran örtüleri ve kamera sallanması

**Amaç:** Hasar alırken kamera eğilmesi (`GameRenderer.bobHurt`, `hurtDir`), hasar kırmızısı (`ScreenEffectRenderer`), gece görüşü, `Darkness`, körlük (`fog`), yanma ve su içi örtüleri.
**Bağımlılık:** –. **Boy:** M.

## Oku
Java: `client/renderer/GameRenderer.java` (`bobHurt`), `client/renderer/ScreenEffectRenderer.java`, `client/renderer/fog/*`, `client/Camera.java`. Rust: `crates/photon-render/src/camera.rs`, `crates/photon-render/src/vulkan.rs`.

## Oracle
Sayısal `camera_hurt` oracle'ı; örtü satırları `gui` ailesinde. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/photon.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
- Yapıldı: `GameRenderer.bobHurt` (hasarda ve ölürken kamera eğimi, el pozu dahil) gerçek Java'ya bit bit eşit; `HurtAnimation` yaw'ı yerel oyuncuya. Kayıt: [`client-camera-hurt-parity-conformance.md`](../records/client-camera-hurt-parity-conformance.md).
- Kalan: ekran örtüleri ve etki sisleri [T83b](T83b-ekran-orturleri.md)'ye ayrıldı.
