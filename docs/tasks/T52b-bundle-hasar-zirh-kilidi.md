# T52b — Bundle tıklaması, hasarlı eşya ve zırh kilidi tahmini

**Amaç:** `doClick`'in kalan dalları: bundle tıklama geçersiz kılmaları (bundle taşınırken veya
üstüne tıklanınca `overrideStackedOnOther`/`overrideOtherStackedOnMe`, `SelectBundleItem` baytı),
hasarlı yığının istiflenmemesi (`isStackable`: `damage` bileşeni), zırh kilidi büyüsü
(`EnchantmentHelper.has(..., PREVENT_ARMOR_CHANGE)`, yaratıcı hariç `mayPickup`).
**Bağımlılık:** T52. **Boy:** M.

## Oku
Java: `world/inventory/AbstractContainerMenu.java` (`doClick`), `world/item/BundleItem.java`,
`world/item/component/BundleContents.java`, `world/inventory/ArmorSlot.java`, `world/item/ItemStack.java`
(`isStackable`, `isDamaged`). Rust: `crates/photon-game/src/menus/container/{clicks,stacks}.rs`,
`crates/photon-world/src/item_components/hash.rs` (bundle içeriğinin hash'i: `bundle_contents`).

## Oracle
`movement/container-clicks` satırları (yamalar `containers.py` `patch(...)` yardımcısıyla): bundle
alma/koyma, hasarlı kılıç istifi, kilitli zırh; `SelectBundleItem` paket baytı.

## Kabul
`tools/photon.py check --capture` → yalnız bilinen Q04 farkı dışında `CHECK OK`.

## Kapsam dışı
Bundle ipucu çizimi (T59); kalan bileşenlerin hash'i (gerektikçe tablo satırı).

## Belgeler
`docs/records/generic-container-conformance.md` (bileşen hash'leri bölümü), `docs/remaining-work.md` §3.

## Devir notu
(boş)
