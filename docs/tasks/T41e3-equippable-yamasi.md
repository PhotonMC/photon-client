# T41e3 — Eşya yamasındaki `equippable` bileşeni

**Amaç:** Bir yığının yamasındaki `equippable` bileşeninin (yuva, ekipman sesi, `asset_id`, kamera katmanı, izin verilen varlıklar, bayraklar) okunması; zırh/eyer katmanlarının ekipman varlığını eşyanın tablodaki varsayılanından değil bu yamadan seçmesi (özel `asset_id`'li eşya).
**Bağımlılık:** T41e2. **Boy:** S.

## Oku
- Java: `world/item/equipment/../component/Equippable.java` (`STREAM_CODEC`), `client/renderer/entity/layers/HumanoidArmorLayer.java` (`shouldRender`), `world/item/ItemStack.java`.
- Rust: `crates/voxel/src/armor.rs` (`asset_for`), `crates/voxel-game/src/item_values.rs`, `crates/voxel-game/src/default_components.rs`.

## Oracle
`equippable` yamalı yığınların gerçek `ItemStack.get(EQUIPPABLE)` ile çözülmüş yuva ve `asset_id`'si (yama telin biçiminde, Java gerçek codec'le çözer; `entity/item-foil` kalıbı).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
