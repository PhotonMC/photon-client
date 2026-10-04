# T35i — Gardiyanın ışını ve hedefe bakışı

**Amaç:** `GuardianRenderer.renderBeam` (özel geometri: dönen dört yüz ve uç, dokulu, tam aydınlık), hedef alınınca `Guardian.aiStep` dalları: `LookControl` ile başın hedefe dönmesi, `setYRot(yHeadRot)`, kabarcık parçacıklarının rastgele tüketimi, karada çırpınış (`onGround` dalı).
**Bağımlılık:** T35h. **Boy:** M.

## Oku
- Java: `client/renderer/entity/GuardianRenderer.java` (`submit`, `renderBeam`, `shouldRender`), `world/entity/monster/Guardian.java` (`aiStep`), `world/entity/ai/control/LookControl.java`.
- Rust: `crates/voxel/src/guardians.rs`, `crates/voxel-world/src/guardian.rs` (tüketilmeyen çekilişler başlıkta), `crates/voxel-render/src/mob_model.rs` (`setup_guardian`).

## Oracle
Işın için yakalayıcı toplayıcı (`submitCustomGeometry`; bkz. `GuiOracle`/`EntityOracle` yakalayıcıları); gerçek `Guardian` tiki için `living-entity-animations` (hedef varlığı oracle sahnesinde bulunan satırlar).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacık çizimi (yalnız rastgele tüketimi), saldırı davranışı.

## Devir notu
(boş)
