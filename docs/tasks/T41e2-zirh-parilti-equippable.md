# T41e2 — Zırh büyü parıltısı ve `equippable` yaması

**Amaç:** `armorEntityGlint` (büyülü parçanın kayan parıltısı: `hasFoil`, `enchanted_glint_armor` dokusu, doku matrisi, ışıksız karışım; yeni boru hattı) ve eşya yamasındaki `equippable` bileşeninin özel varlık kimliği.
**Bağımlılık:** T41e. **Boy:** M.

## Oku
- Java: `client/renderer/entity/layers/EquipmentLayerRenderer.java`, `client/renderer/rendertype/RenderTypes.java` (`armorEntityGlint`), `world/item/ItemStack.java` (`hasFoil`), `world/item/component/Equippable.java`.
- Rust: `crates/voxel/src/armor.rs`, `voxel-render/src/vulkan.rs` (entity boru hattı), `voxel-game/src/item_values.rs`.

## Oracle
`entity/equipment-layers` satırları (`hasFoil` kararı), parıltı dokusu matrisi için sayısal oracle.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-armor-parity-conformance.md`.

## Devir notu
(boş)
