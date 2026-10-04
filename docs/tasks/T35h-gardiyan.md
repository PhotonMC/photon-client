# T35h — Gardiyan ve elder gardiyan

**Amaç:** `GuardianModel` (12 diken, göz, üç parçalı kuyruk), `GuardianRenderer` (ışın çizimi) ve `ElderGuardianRenderer` (ölçek 2,35, doku) ile istemci durumları: `spikesAnimation`, `tailAnimation`, `clientSideAttackTime`, `getAttackAnimationScale`, `DATA_ID_ATTACK_TARGET`.
**Bağımlılık:** T35b. **Boy:** L (sığmazsa ışını ayır).

## Oku
- Java: `client/model/monster/guardian/GuardianModel.java` (diken tabloları, `setupSpikes`, göz bakışı), `client/renderer/entity/{Guardian,ElderGuardian}Renderer.java` (`renderBeam`, `shouldRender`), `client/renderer/entity/state/GuardianRenderState.java`, `world/entity/monster/Guardian.java` (`aiStep` istemci yarısı, `getSpikesAnimation`, `getTailAnimation`, `getAttackAnimationScale`).
- Rust: `crates/voxel-render/src/mob_model.rs` (`setup_squid`: ölçekli katman kalıbı), `crates/voxel-world/src/squid.rs`, `crates/voxel/src/squids.rs`.

## Oracle
`squid_model` kalıbı; ışın için yakalayıcı toplayıcı (`submitCustomGeometry`); gerçek `Guardian` tiki için `living-entity-animations`. Elder gardiyan katmanı `createElderGuardianLayer` (ölçek 2,35) oracle'dan yakalanır.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar (`ElderGuardianParticle`), saldırı davranışı.

## Devir notu
(boş)
