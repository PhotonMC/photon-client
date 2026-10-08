# T40j — Yıldırımın gökyüzü çakması

**Amaç:** `ClientLevel.skyFlashTime`: yıldırım varken (`LightningBolt.tick`, `life >= 0`) `setSkyFlashTime(2)`; her tikte bir azalır; `> 0` iken gökyüzü rengi `srgbLerp(0.22, renk, flashColor)` olur ve gökyüzü ışık çarpanı `1.0`'dır. `hideLightningFlash` seçeneği süreyi sıfır gösterir.
**Bağımlılık:** T40e. **Boy:** S.

## Oku
- Java: `client/multiplayer/ClientLevel.java` (satır ~260–310, ~990), `world/entity/LightningBolt.java`, `client/Options.java` (`hideLightningFlash`).
- Rust: `crates/photon/src/world/sky.rs`, `crates/photon/src/world/atmosphere.rs` (gökyüzü rengi), `crates/photon-world` (ışık çarpanı).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler.

## Belgeler
`docs/records/client-lightning-bolt-parity-conformance.md` sapma satırını güncelle.

## Devir notu
- `ClientWorld::sky_flash_time` (ışıklı bolt tikinde 2, `tick_sky_flash` ile azalır; `Minecraft.tick` sırası: önce entity'ler, sonra seviye tiki); `Environment::evaluate(…, sky_flash)` son katman olarak gökyüzü rengini ve ışık çarpanını değiştirir; seçenek `hideLightningFlashes` (`options.txt`).
- Oracle: `movement/sky-flash` (gerçek `ClientLevel` öznitelik sistemi) 9 satır, bit bit eşit. Kayıt: `client-lightning-bolt-parity-conformance.md`.
- Açık: yağmur/fırtına ve saat ilerlemesiyle çakma karşılaştırılmadı; canlı gözlem yapılmadı.
