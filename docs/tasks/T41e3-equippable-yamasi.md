# T41e3 — Eşya yamasındaki `equippable` bileşeni

**Amaç:** Bir yığının yamasındaki `equippable` bileşeninin (yuva, ekipman sesi, `asset_id`, kamera katmanı, izin verilen varlıklar, bayraklar) okunması; zırh/eyer katmanlarının ekipman varlığını eşyanın tablodaki varsayılanından değil bu yamadan seçmesi (özel `asset_id`'li eşya).
**Bağımlılık:** T41e2. **Boy:** S.

## Oku
- Java: `world/item/equipment/../component/Equippable.java` (`STREAM_CODEC`), `client/renderer/entity/layers/HumanoidArmorLayer.java` (`shouldRender`), `world/item/ItemStack.java`.
- Rust: `crates/voxel/src/equipment/armor.rs` (`asset_for`), `crates/voxel-game/src/item/values.rs`, `crates/voxel-game/src/item/default_components.rs`.

## Oracle
`equippable` yamalı yığınların gerçek `ItemStack.get(EQUIPPABLE)` ile çözülmüş yuva ve `asset_id`'si (yama telin biçiminde, Java gerçek codec'le çözer; `entity/item-foil` kalıbı).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `item_values::equippable_of` (yama ekler/kaldırırsa yama, yoksa eşya tablosu; yuva ve varlık kimliği), `Armor::asset_for` onu kullanır. Kayıt: `docs/records/client-armor-parity-conformance.md`. Canlıda görüldü.
- Oracle: `entity/item-equippable` 8 satır (gerçek `ItemStack.get(EQUIPPABLE)`).
- Sapma: yalnız yuva ve varlık kimliği okunur (ses, kamera katmanı, izin verilen varlıklar, bayraklar katmanlarca okunmaz); eşya bölümlerindeki öteki okuyucular (`container_menu.rs`) hâlâ tablodan okur, yamaya bakmaz.
