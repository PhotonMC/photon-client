# T40j — Yıldırımın gökyüzü çakması

**Amaç:** `ClientLevel.skyFlashTime`: yıldırım varken (`LightningBolt.tick`, `life >= 0`) `setSkyFlashTime(2)`; her tikte bir azalır; `> 0` iken gökyüzü rengi `srgbLerp(0.22, renk, flashColor)` olur ve gökyüzü ışık çarpanı `1.0`'dır. `hideLightningFlash` seçeneği süreyi sıfır gösterir.
**Bağımlılık:** T40e. **Boy:** S.

## Oku
- Java: `client/multiplayer/ClientLevel.java` (satır ~260–310, ~990), `world/entity/LightningBolt.java`, `client/Options.java` (`hideLightningFlash`).
- Rust: `crates/voxel/src/sky.rs`, `crates/voxel/src/atmosphere.rs` (gökyüzü rengi), `crates/voxel-world` (ışık çarpanı).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler.

## Belgeler
`docs/client-lightning-bolt-parity-conformance.md` sapma satırını güncelle.

## Devir notu
(boş)
