# T41d2 — Piglinin kol pozları

**Amaç:** `PiglinArmPose` (hayranlık, arbalet doldurma/tutma, silah yukarıda/aşağı sallama, dans) ve `Piglin.getArmPose`/`PiglinBrute.getArmPose` çıkarımı.
**Bağımlılık:** T41d. **Boy:** S.

## Oku
- Java: `client/model/monster/piglin/PiglinModel.java`, `client/model/AnimationUtils.java`, `world/entity/monster/piglin/{Piglin,PiglinBrute,AbstractPiglin}.java`.
- Rust: `voxel-render/src/mob_model.rs` (`setup_piglin`), `crates/voxel/src/mobs.rs`.

## Oracle
`entity/piglin-model` satırları `arm_pose`, `main_arm`, `ticks_using_item`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-held-item-parity-conformance.md`.

## Devir notu
- `PiglinArmPose` + `setup_piglin` altı poz (silah yukarıda/`swingWeaponDown`, arbalet tutma/doldurma ortak `AnimationUtils` işlevleriyle, hayranlık, dans); `entity/piglin-model` ~60 yeni satır bit bit eşit.
- Oyunda: `piglin_arm_pose` (`getArmPose`: dans, sevilen eşya etiketi `piglin_loved` ile ikinci el, `TOOL` bileşenli saldırı, `DATA_IS_CHARGING_CROSSBOW` bayrağı, yüklü arbalet); çıkarım Java'ya karşı karşılaştırılmadı.
- Açık: [T41d3](T41d3-mizrak-yerel-kullanim.md).
