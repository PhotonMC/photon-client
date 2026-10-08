# T41d3 — Mızrak pozları

**Amaç:** `SPEAR` kol pozu (`SpearAnimations.thirdPersonHandUse/thirdPersonUseItem/thirdPersonAttackHand/thirdPersonAttackItem`, `KineticWeapon` bileşeni, `Ease`, vuruş geri bildirimi, mızrak etiketli eşyada hep `SPEAR`) uzaktaki oyuncu ve insansı mobda; yerel oyuncunun kendi kullanımı [T41d4](T41d4-yerel-kullanim.md)'e ayrıldı.
**Bağımlılık:** T41d. **Boy:** M.

## Oku
- Java: `client/model/effects/SpearAnimations.java`, `client/renderer/entity/layers/ItemInHandLayer.java`, `world/item/component/KineticWeapon.java`, `client/player/LocalPlayer.java`.
- Rust: `photon-render/src/player_model.rs`, `crates/photon/src/equipment/arm_poses.rs`, `photon-world/src/local_player.rs`.

## Oracle
`entity/held-item-pose` satırları (poz `spear`).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Belgeler
`docs/records/client-held-item-parity-conformance.md`.

## Devir notu
- `photon-render::spear_animation` (`Ease` dokuz eğri, `UseParams`, el/eşya dönüşleri), `photon-data::spear` (yedi mızrağın süreleri, gerçek kayıt defteriyle `entity/spear-table`), `HandMotion` ile `hand_pose`; `entity/held-item-pose` +138 satır gerçek model ve katmana karşı bit bit eşit.
- Java'ya uyum düzeltmeleri: `Matrix4f.rotateAround` ötelemesi (yazılmış üçüncü sütun, sıfır olmayan `oz` ilk kez kullanıldı), `animateZombieArms` (dürtmede kollar oynamaz, bebek elinde eşyayla indirik).
- Oyunda: `arm_poses::{spear_view, mob_spear_pose}`, `EntityEvent` 2 → `ticks_since_kinetic_hit`, `ClientEntity::swing_durations` (sallama süresi elde tutulan mızrakla); `spears` etiketi yedi mızrak sayıldı; çıkarım Java'ya karşı karşılaştırılmadı.
- Sapma/açık: yerel oyuncunun kullanımı ve sallama süresi, etkilerin sallamaya katkısı, vuruş sesi, `NONE` sallama türü, yamayla değişen bileşen → [T41d4](T41d4-yerel-kullanim.md).
