# T51 — Örs ve taş çarkı sonucunun istemci tahmini

**Amaç:** `AnvilMenu.createResult` ve `GrindstoneMenu.createResult` istemcide de çalışır: onarım maliyeti, büyü birleştirme, yeniden adlandırma; sonuç yuvası ve maliyet etiketi şimdilik yalnız sunucudan gelir ([kalan işler](../remaining-work.md) §3).
**Bağımlılık:** –. **Boy:** L.

## Oku
Java: `world/inventory/{AnvilMenu,GrindstoneMenu,ItemCombinerMenu}.java`, `world/item/enchantment/EnchantmentHelper.java`, `world/item/enchantment/Repairable.java`. Rust: `crates/photon-game/src/menus/container/transform.rs`, `crates/photon-game/src/item/values.rs`.

## Oracle
`movement/container-clicks` ailesine anvil/grindstone satırları: onarım malzemesi, iki aynı eşya, büyülü kitap, ad değişimi, maliyet ve `RenameItem` paketi; `container_screen` anvil maliyet etiketi. G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/photon.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/records/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
Yapılan: taş çarkı sonucu (`menus/container/grindstone.rs`), bileşen düzenleme (`components.rs`:
`ItemStack.set/remove` normalleşmesi, hasar, `ItemEnchantments.Mutable`), `EnchantmentTraits` (lanet,
zırh kilidi; veri paketinden), `ContainerMenu::set_slot` (sunucudan gelen girdi sonucu yeniden yapar).
Oracle: `container-clicks`'e 5 taş çarkı satırı ve hasarlı yığın satırı (T52b birim testi kaldırıldı);
Java'da tıklama paketi bileşenleri tür kimliği sırasına çevrilir, Rust da öyle yazar.
Açık: örs → T51b (onarım eşyası verisi, büyü tanımları, yerelleştirilmiş ad gerekir).
Not: `particle-engine` Java yakalaması bir kez tekrarlanamaz çıktı (ikinci denemede geçti).
