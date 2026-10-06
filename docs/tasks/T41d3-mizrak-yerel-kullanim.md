# T41d3 — Mızrak pozları ve yerel oyuncunun eşya kullanımı

**Amaç:** `SPEAR` kol pozu (`SpearAnimations.thirdPersonHandUse/thirdPersonUseItem/thirdPersonAttackHand/thirdPersonAttackItem`, `KineticWeapon` bileşeni, `Ease`, vuruş geri bildirimi, mızrak etiketli eşyada hep `SPEAR`), yerel oyuncunun kendi eşya kullanımı (üçüncü şahıs modeli: `startUsingItem`, `ticksUsingItem`).
**Bağımlılık:** T41d. **Boy:** M.

## Oku
- Java: `client/model/effects/SpearAnimations.java`, `client/renderer/entity/layers/ItemInHandLayer.java`, `world/item/component/KineticWeapon.java`, `client/player/LocalPlayer.java`.
- Rust: `voxel-render/src/player_model.rs`, `crates/voxel/src/arm_poses.rs`, `voxel-world/src/local_player.rs`.

## Oracle
`entity/held-item-pose` satırları (poz `spear`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-held-item-parity-conformance.md`.

## Devir notu
(boş)
