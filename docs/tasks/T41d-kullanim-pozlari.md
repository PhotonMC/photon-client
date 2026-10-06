# T41d — Eşya kullanım pozları ve silah kol pozları

**Amaç:** Uzaktaki canlıların eşya kullanırken kolları: `isUsingItem` (canlı bayrakları 8, kullanılan el), blok, yay çekme, arbalet (doldurma/tutma), mızrak, dürbün, boru, fırça pozları; `ItemInHandLayer`'ın `animateUseItem`/`SpearAnimations` dönüşleri; piglinin `PiglinArmPose` kolları.
**Bağımlılık:** T41. **Boy:** M–L (poz ailesi başına bölünebilir).

## Oku
- Java: `client/model/HumanoidModel.java` (`poseRightArm`/`poseLeftArm`, `ArmPose`), `client/model/AnimationUtils.java` (`animateCrossbow*`), `client/model/effects/SpearAnimations.java`, `client/renderer/entity/player/AvatarRenderer.java` (`getArmPose`), `client/model/monster/piglin/PiglinModel.java`, `world/entity/LivingEntity.java` (`getTicksUsingItem`, kullanım bayrakları).
- Rust: `voxel-render/src/player_model.rs` (`ArmPose`, `pose_arm`), `crates/voxel/src/{overlays,mobs}.rs`, `crates/voxel-world/src/entity.rs` (`is_using_item`, `used_item_is_off_hand`).

## Yapılacaklar
1. `ArmPose` kalan değerleri ve `pose_arm` kolları; `ticksUsingItem` (`useItemRemainingTicks` istemci tarafı) ve eşyanın kullanım animasyonu (`ItemUseAnimation`: eşya bileşeni).
2. `ItemInHandLayer`'da `ArmPose.animateUseItem` ve mızrak saldırı dönüşü.
3. Piglinin silah/arbalet/hayranlık pozları.

## Oracle
`entity/held-item-pose` satırlarına poz ve `ticksUsingItem` alanları (aynı operation).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Saldırı sallaması (T43).

## Belgeler
`docs/client-held-item-parity-conformance.md` sapma listesi, `docs/index.md`.

## Devir notu
- `ArmPose` on poz (mızrak hariç) + `HumanoidModel.setupAnim` kullanım dalı, yolcu/süzülme/`speedValue`, dürbünde soluk almama: `entity/held-item-pose` ~110 yeni satır, bit bit eşit.
- Oyunda: `arm_poses.rs` uzaktaki oyuncunun pozunu eşya sınıfından/bileşenden çıkarır (`getArmPose`), `ClientEntity::ticks_using_item`, yolcu ve süzülme bayrakları modele geçer.
- Açık → T41d2: mızrak, piglin, yerel oyuncunun kendi kullanımı; poz çıkarımı Java'ya karşı karşılaştırılmadı.
