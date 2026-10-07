# T41d4 — Yerel oyuncunun eşya kullanımı ve sallama süresi

**Amaç:** yerel oyuncunun kendi eşya kullanımını istemcide izlemek (`startUsingItem`, `ticksUsingItem`, kullanılan el) ve üçüncü şahıs modelinin kullanım pozlarını (yay, kalkan, mızrak…) buna bağlamak; yerel oyuncunun sallama süresini elde tutulan eşyadan almak (mızrak 13–23 tick); hız/yorgunluk etkisinin sallama süresine katkısı; vuruş sesi (`KineticWeapon.makeLocalHitSound`).
**Bağımlılık:** T41d3. **Boy:** M.

## Oku
- Java: `client/player/LocalPlayer.java` (`startUsingItem`, `updateUsingItem`), `world/entity/LivingEntity.java` (`getCurrentSwingDuration`, `startUsingItem`), `world/effect/MobEffectUtil.java`.
- Rust: `crates/voxel/src/overlays.rs` (`player_model`), `voxel-world/src/local_player.rs` (`SWING_DURATION`), `crates/voxel/src/equipment/arm_poses.rs`.

## Oracle
Gerçek `LocalPlayer` sahnesi (`MovementOracle.buildScene`) ile sallama süresi ve kullanım sayacı; kol pozları `entity/held-item-pose` ile zaten kapalı.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/records/client-held-item-parity-conformance.md`.

## Devir notu
- Yapıldı: yerel sallama süresi (`LocalPlayer::current_swing_duration`: mızrak, hız/kanal gücü, yorgunluk), el seçen sallama (ikinci elin kolu), kullanım bayrağı ve sayacı (`HandState`, `ClientWorld::local_hand_state`), üçüncü şahıs kolları uzak oyuncularla ortak koddan (`arm_poses::arm_state_of`). Kayıt: `docs/records/client-held-item-parity-conformance.md`.
- Oracle: yerel oyuncu satırlarına sallama ve kullanım alanları (gerçek `LocalPlayer`); satırlar: hız, kanal gücü, yorgunluk, ikinci el koyma/kullanma.
- Açık: kendi kullanımını başlatma (ön tahmin), kullanırken yavaşlama ve koşma kesilmesi, yerel mızrağın saldırısı ve vuruş sesi, uzaktaki varlıkların etkileri. Üçüncü şahıs canlıda bakılmadı.
