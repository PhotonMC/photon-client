# T33d — Enerji girdabı ve şarjlı creeper

**Amaç:** `RenderTypes.energySwirl` (doku kayması, toplamalı harman, yüz eleme yok) ve `EnergySwirlLayer` tüketicisi `CreeperPowerLayer`: şarjlı creeper (`creeper#armor` modeli, `-8355712` rengi, kayma `t * 0.01`); `DATA_IS_POWERED` verisi.
**Bağımlılık:** T33c. **Boy:** M.

## Oku
- Java: `client/renderer/entity/layers/{EnergySwirlLayer,CreeperPowerLayer}.java`, `client/renderer/rendertype/RenderTypes.java` (`energySwirl`), `RenderPipelines.ENERGY_SWIRL`, `world/entity/monster/Creeper.java` (`isPowered`, `DATA_IS_POWERED`).
- Rust: `crates/voxel-render/src/{render_types.rs,vulkan.rs,mob_model.rs}` (T33c'nin glow yolu), `crates/voxel/src/monsters.rs`, `voxel-world/src/metadata.rs`.

## Yapılacaklar
1. `ENERGY_SWIRL` hattı: kayma, `ALPHA_CUTOUT 0.1`, toplamalı harman, `cull false`. Dokuyu atlasta sarmak için kaymayı alt dikdörtgen içinde sarmalayan shader (`% 1`); referans `GL_REPEAT` kullanır.
2. Şarjlı creeper: `DATA_IS_POWERED` oku, `creeper#armor` modeli ve kayma.

## Oracle
`eyes_layers` kalıbı: gerçek `CreeperPowerLayer.submit`'in gönderdiği render türü (kayma ofsetleri, renk, hat durumu) satırları; model satırı `spider_model` kalıbıyla.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon creeper {powered:1b}`.

## Kapsam dışı
Patlama parçacığı (PLAN-M8).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: `ENERGY_SWIRL` hattı (`Swirl`, `swirl_buffer`, toplamalı harman, `fract` ile sarma), `mob_vertices_swirl`, `render_types::{ENERGY_SWIRL, POWER_*, swirl_offset}`, `Creeper.DATA_IS_POWERED` (17), şarjlı creeper (`creeper#armor`). Kayıt: `docs/records/client-energy-swirl-parity-conformance.md`.
- Oracle: `entity/swirl-layers` 10 satır (gerçek `CreeperPowerLayer`), `creeper#armor` model satırları, `variant-fields` bir satır. Canlı: şarjlı creeper mavi girdapla `shot` ile görüldü.
- Sapma: sarma shader'da `fract`; renk sabit.

