# T41f — Elytra, kafatası modelleri, gözde dürbün

**Amaç:** `WingsLayer` (elytra: `ElytraModel` + `elytraAnimationState` tik durumu, `wings` katman türü), `SkullBlockRenderer` kafatası modelleri (iskelet, zombi, creeper, ejderha, piglin, oyuncu kafası; oyuncu profili dokusu PLAN-M9) ve `PlayerItemInHandLayer.renderItemHeldToEye` (dürbün).
**Bağımlılık:** T41c. **Boy:** M.

## Oku
- Java: `client/renderer/entity/layers/{WingsLayer,CustomHeadLayer,PlayerItemInHandLayer}.java`, `client/model/object/equipment/ElytraModel.java`, `client/model/object/skull/*.java`, `client/renderer/blockentity/SkullBlockRenderer.java`, `world/entity/{LivingEntity.elytraAnimationState,ElytraAnimationState}.java`.
- Rust: `crates/voxel/src/{armor,held_items}.rs`, `voxel-render/src/held_item.rs`, `voxel-world/src/animal.rs` (tik durumu kalıbı).

## Yapılacaklar
1. Elytra: model, `ElytraAnimationState` tiki (gerçek `ClientPacketListener` + gerçek tikle oracle), `wings` katman dokuları (`Armor`).
2. Kafatası: model katmanları tabloda (`*_head#main`), dokular, kafatası pozu (`submitSkull`).
3. Dürbün: `renderItemHeldToEye` pozu.

## Oracle
`held_item_pose` kalıbı; elytra durumu `living_entity_ticks` kalıbı.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Pelerin dokusu indirme (PLAN-M9).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
