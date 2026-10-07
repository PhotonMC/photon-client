# T40g — Eşya çerçevesi

**Amaç:** `ItemFrame` ve `GlowItemFrame` (`ItemFrameRenderer`): yön verisi, çerçeve bloğu modeli, içindeki eşya ve dönüşü, harita içeriği.
**Bağımlılık:** T40. **Boy:** L.

## Oku
- Java: `client/renderer/entity/ItemFrameRenderer.java`, `world/entity/decoration/{ItemFrame,HangingEntity}.java`.

## Oracle
Gerçek renderer/model sınıfı ve durumu (örnek: `docs/records/client-arrow-parity-conformance.md`, `docs/records/client-minecart-parity-conformance.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Sesler, partiküller, dış çizgi.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: çerçeve ve eşya yerleşimi (`voxel-render/item_frame.rs`, oracle'da gerçek `ItemFrameRenderer.submit`), sahte `item_frame[map]` blok durumları (`BlockRenderer::item_frame_state`), eşya ve dönüş verisi, oyunda `item_frames.rs`. Canlıda `summon` ile doğrulandı. Kayıt: `docs/records/client-item-frame-parity-conformance.md`.
- Sapma: harita içeriği yok (`mapData == null` kolu gibi), [T40g2](T40g2-cerceve-haritasi.md); konum/ışık kuralı ve çerçeve geometrisi oracle'sız; entity konumu/vuruş kutusu blok köşesi; z-ofset ve ad etiketi yok.
