# T41d2 — Mızrak pozları, piglin kolları ve yerel kullanım

**Amaç:** T41d'den kalanlar: `SPEAR` kol pozu (`SpearAnimations.thirdPersonHandUse/thirdPersonUseItem/thirdPersonAttackItem`, kinetik silah bileşeni, saldırı sallaması), piglinin `PiglinArmPose` kolları (hayranlık, arbalet, silah, dans), yerel oyuncunun kendi eşya kullanımı (üçüncü şahıs modeli ve `ticksUsingItem`).
**Bağımlılık:** T41d. **Boy:** M.

## Oku
- Java: `client/model/effects/SpearAnimations.java`, `client/model/monster/piglin/PiglinModel.java`, `client/renderer/entity/layers/ItemInHandLayer.java`, `client/player/LocalPlayer.java` (`startUsingItem`).
- Rust: `voxel-render/src/player_model.rs` (`ArmPose`, `pose_arm`), `crates/voxel/src/arm_poses.rs`, `voxel-world/src/local_player.rs`.

## Oracle
`entity/held-item-pose` satırları (poz adı `spear`), `piglin_model` satırları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-held-item-parity-conformance.md`.

## Devir notu
(boş)
