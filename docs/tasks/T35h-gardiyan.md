# T35h — Gardiyan ve elder gardiyan

**Amaç:** `GuardianModel` (12 diken, göz, üç parçalı kuyruk), elder gardiyan katmanı ve doku, istemci durumları: `spikesAnimation`, `tailAnimation`, `clientSideAttackTime`, `DATA_ID_MOVING`, `DATA_ID_ATTACK_TARGET`. Işın ve hedefe bakış T35i'ye ayrıldı.
**Bağımlılık:** T35b. **Boy:** M.

## Oku
- Java: `client/model/monster/guardian/GuardianModel.java` (diken tabloları, `setupSpikes`, göz bakışı), `client/renderer/entity/{Guardian,ElderGuardian}Renderer.java` (`extractRenderState`), `client/renderer/entity/state/GuardianRenderState.java`, `world/entity/monster/Guardian.java` (`aiStep` istemci yarısı, `getSpikesAnimation`, `getTailAnimation`, `getAttackAnimationScale`).
- Rust: `crates/voxel-render/src/entity/mob_model.rs` (`setup_squid`: ölçekli katman kalıbı), `crates/voxel-world/src/mobs/squid.rs`, `crates/voxel/src/mobs/aquatic/squids.rs`.

## Oracle
`squid_model` kalıbı; gerçek `Guardian` tiki için `living-entity-animations`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar (`ElderGuardianParticle`), saldırı davranışı; ışın ve hedefe bakış (T35i).

## Devir notu
- Yapıldı: `setup_guardian` (diken nefesi, göz, kuyruk), `voxel-world/guardian.rs` (kuyruk/diken animasyonu, ışın sayacı), veriler 16/17, `crates/voxel/src/mobs/aquatic/guardians.rs`. Kayıt: `docs/records/client-guardian-parity-conformance.md`.
- Bulgu: `Mob.baseTick` her tik aynı rastgele kaynaktan bir değer çeker; karadaki dikenler bunsuz uyuşmuyordu.
- Bölündü: ışın çizimi, hedefe bakış denetimi ve karadaki çırpınış T35i'de; o zamana dek karadaki diken değerleri yalnız bu dallar çalışmıyorken Java ile eşleşir.
- Sapma: doku yolları oracle'sız; ses, parçacık, gölge yok. Canlı gözlem yapılmadı.
