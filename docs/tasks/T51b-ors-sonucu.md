# T51b — Örs sonucunun istemci tahmini

**Amaç:** `AnvilMenu.createResult` istemcide: onarım malzemesi (`repairItemCountCost`), iki aynı eşya,
büyülü kitap, büyü uyumu/üst sınırı/maliyeti, yeniden adlandırma (`setItemName`, `RenameItem`), maliyet
(`DataSlot`) ve 40 sınırı; `onTake`'in ek eşyayı tüketmesi (`transform_on_take` şimdi hepsini siler).
**Bağımlılık:** T51. **Boy:** L.

## Oku
Java: `world/inventory/AnvilMenu.java`, `world/item/enchantment/{Enchantment,EnchantmentHelper}.java`
(`canEnchant`, `areCompatible`, `getAnvilCost`), `world/item/ItemStack.java` (`isValidRepairItem`,
`getHoverName`). Rust: `crates/photon-game/src/menus/container/{components,grindstone,transform}.rs`,
`crates/photon-game/src/registries.rs` (`EnchantmentEntry`: `anvil_cost`, `supported_items`,
`exclusive_set` eklenecek), `menus/container.rs` (`EnchantmentTraits`).

## Veri
`repairable` bileşeni (onarım eşyaları/etiketi) üretilmiş eşya tablosunda yok: oracle'dan tablo olarak
alınır (AGENTS: formül değil tablo) ya da D05 ile birlikte elle; karar burada verilir.
`getHoverName` karşılaştırması istemci diliyle yapılır (çeviri: `photon-game` dil tablosu).

## Oracle
`movement/container-clicks` örs satırları (`menu="anvil"`, `xp_level`): onarım malzemesi, iki aynı eşya,
kitap, uyumsuz büyü, ad değişimi (`RenameItem` için ekran yolu veya `setItemName` adımı), maliyet ≥ 40.

## Kabul
`tools/photon.py check --capture` → yalnız bilinen Q04 farkı dışında yeşil.

## Belgeler
`docs/records/generic-container-conformance.md`, `docs/remaining-work.md` §3.

## Devir notu
(boş)
