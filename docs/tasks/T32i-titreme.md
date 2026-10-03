# T32i — Titreme (`isShaking`)

**Amaç:** `LivingEntityRenderer.isShaking`: donmuş (`isFullyFrozen`) her mob ve zombileşen hoglin gövde açısı titremesi (`bodyRot += cos(floor(ageInTicks)·3,25)·π·0,4`).
**Bağımlılık:** T32d. **Boy:** S.

## Oku
- Java: `client/renderer/entity/LivingEntityRenderer.java` (`isShaking`, `setupRotations`), `HoglinRenderer.java`, `Hoglin.isConverting`, `EnvironmentAttributes.PIGLINS_ZOMBIFY`, dimension türü `attributes` alanı (kayıt verisi).
- Rust: `crates/voxel-world/src/dimension.rs` (`DimensionEnvironment`), `ClientEntity::is_fully_frozen`, `crates/voxel/src/mobs.rs` (`placement`), `crates/voxel/src/hoglins.rs`.

## Yapılacaklar
1. `placement` titremeyi `body_rot`a ekler (tüm mobların donmuş hâli).
2. Boyut özniteliği `gameplay/piglins_zombify` (kayıt verisinden; varsayılan doğru) ve `Hoglin.isConverting` (`!immune && !noAI && öznitelik`); veri `DATA_IMMUNE_TO_ZOMBIFICATION`.

## Oracle
Gerçek `LivingEntityRenderer.setupRotations` ile (`entity/…-render` kalıbı); dönüşen/ bağışık/ nether hoglin ve donmuş mob satırları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Boğulan zombi (`isUnderWaterConverting`) vb. diğer dönüşüm titremeleri ayrı görev olabilir; sesler.

## Belgeler
`docs/index.md` yeni satır gerekmez; `docs/client-hoglin-parity-conformance.md` sapma satırı güncellenir.

## Devir notu
(boş)
