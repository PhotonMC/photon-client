# T51 — Örs ve taş çarkı sonucunun istemci tahmini

**Amaç:** `AnvilMenu.createResult` ve `GrindstoneMenu.createResult` istemcide de çalışır: onarım maliyeti, büyü birleştirme, yeniden adlandırma; sonuç yuvası ve maliyet etiketi şimdilik yalnız sunucudan gelir ([kalan işler](../remaining-work.md) §3).
**Bağımlılık:** –. **Boy:** L.

## Oku
Java: `world/inventory/{AnvilMenu,GrindstoneMenu,ItemCombinerMenu}.java`, `world/item/enchantment/EnchantmentHelper.java`, `world/item/enchantment/Repairable.java`. Rust: `crates/voxel-game/src/container_menu/transform.rs`, `crates/voxel-game/src/item_values.rs`.

## Oracle
`movement/container-clicks` ailesine anvil/grindstone satırları: onarım malzemesi, iki aynı eşya, büyülü kitap, ad değişimi, maliyet ve `RenameItem` paketi; `container_screen` anvil maliyet etiketi. G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
(boş)
