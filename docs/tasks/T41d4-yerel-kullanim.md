# T41d4 — Yerel oyuncunun eşya kullanımı ve sallama süresi

**Amaç:** yerel oyuncunun kendi eşya kullanımını istemcide izlemek (`startUsingItem`, `ticksUsingItem`, kullanılan el) ve üçüncü şahıs modelinin kullanım pozlarını (yay, kalkan, mızrak…) buna bağlamak; yerel oyuncunun sallama süresini elde tutulan eşyadan almak (mızrak 13–23 tick); hız/yorgunluk etkisinin sallama süresine katkısı; vuruş sesi (`KineticWeapon.makeLocalHitSound`).
**Bağımlılık:** T41d3. **Boy:** M.

## Oku
- Java: `client/player/LocalPlayer.java` (`startUsingItem`, `updateUsingItem`), `world/entity/LivingEntity.java` (`getCurrentSwingDuration`, `startUsingItem`), `world/effect/MobEffectUtil.java`.
- Rust: `crates/voxel/src/overlays.rs` (`player_model`), `voxel-world/src/local_player.rs` (`SWING_DURATION`), `crates/voxel/src/arm_poses.rs`.

## Oracle
Gerçek `LocalPlayer` sahnesi (`MovementOracle.buildScene`) ile sallama süresi ve kullanım sayacı; kol pozları `entity/held-item-pose` ile zaten kapalı.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-held-item-parity-conformance.md`.

## Devir notu
(boş)
