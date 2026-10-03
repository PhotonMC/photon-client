# T32i — Titreme (`isShaking`)

**Amaç:** `LivingEntityRenderer.isShaking`: donmuş (`isFullyFrozen`) her mob, zombileşen hoglin ve boğulan strider (`StriderRenderer.isShaking`: `Strider.isSuffocating`, T32h'de okunur) gövde açısı titremesi (`bodyRot += cos(floor(ageInTicks)·3,25)·π·0,4`).
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
- Yapıldı: `EntitySnapshot.{age_in_ticks, shaking}` (`ClientEntity::is_shaking`: donmuş | hoglin dönüşüyor | strider boğuluyor), `MobPlacement.shaking`, `DimensionEnvironment.piglins_zombify` (kayıt `attributes` alanı: düz değer ve `BooleanModifier`, yoksa varsayılan doğru), hoglin `DATA_IMMUNE_TO_ZOMBIFICATION` (kimlik 18).
- Oracle: `entity/shaking-rotations` (gerçek `setupRotations` + `HoglinRenderer`/`StriderRenderer`/`CowRenderer`); `Hoglin.isConverting` formülü ve `isFullyFrozen` (≥140) oracle'da değerlerden kurulur (varlık örneği yok), boyut özniteliği kayıt defterinden okunur.
- Sapma/açık: biyom/hava katmanları `piglins_zombify`ı etkilemez (okunmaz); zombi (`isUnderWaterConverting`), piglin, zombi köylü, iskelet/bogged dönüşüm titremeleri ayrı (ilgili render durumları yok).
- `tools/voxel.py` yerel port değişikliği kullanıcıya ait, commit'e dahil edilmedi.
