# T41f — Elytra, kafatası modelleri, gözde dürbün

**Amaç:** `WingsLayer` (elytra: `ElytraModel` + `elytraAnimationState` tik durumu, `wings` katman türü), `SkullBlockRenderer` kafatası modelleri (iskelet, zombi, creeper, ejderha, piglin, oyuncu kafası; oyuncu profili dokusu PLAN-M9) ve `PlayerItemInHandLayer.renderItemHeldToEye` (dürbün).
**Bağımlılık:** T41c. **Boy:** M.

## Oku
- Java: `client/renderer/entity/layers/{WingsLayer,CustomHeadLayer,PlayerItemInHandLayer}.java`, `client/model/object/equipment/ElytraModel.java`, `client/model/object/skull/*.java`, `client/renderer/blockentity/SkullBlockRenderer.java`, `world/entity/{LivingEntity.elytraAnimationState,ElytraAnimationState}.java`.
- Rust: `crates/photon/src/{armor,held_items}.rs`, `photon-render/src/held_item.rs`, `photon-world/src/animal.rs` (tik durumu kalıbı).

## Yapılacaklar
1. Elytra: model, `ElytraAnimationState` tiki (gerçek `ClientPacketListener` + gerçek tikle oracle), `wings` katman dokuları (`Armor`).
2. Kafatası: model katmanları tabloda (`*_head#main`), dokular, kafatası pozu (`submitSkull`).
3. Dürbün: `renderItemHeldToEye` pozu.

## Oracle
`held_item_pose` kalıbı; elytra durumu `living_entity_ticks` kalıbı.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Pelerin dokusu indirme (PLAN-M9).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yalnız uzaktaki oyuncunun elytrası: `ElytraState` (`photon-world::elytra`, her yaşayan varlığın tikinde), `MobModel::{setup_elytra, shift_root}`, `Armor::wings` (katman türü `wings`, doku atlasa eklendi).
- Oracle: `entity/elytra-model` (8) ve `movement/elytra-state` (13 senaryo, gerçek `ElytraAnimationState`), bit bit eşit; canlı gözlem yok.
- Açık → [T41f2](T41f2-kafatasi-yerel-elytra.md): kafatası, yerel/mob elytrası, süzülme eğimi, dürbün.
