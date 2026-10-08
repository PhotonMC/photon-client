# T59d — Yaratıcı aramada `#etiket`

**Amaç:** `CreativeModeInventoryScreen.refreshSearchResults`'ın `#` ile başlayan araması: eşya etiketlerinin
adlarıyla eşleşen eşyalar (sunucunun gönderdiği etiket tablosu).
**Bağımlılık:** T59. **Boy:** S.

## Oku
Java: `client/gui/screens/inventory/CreativeModeInventoryScreen.java` (`updateVisibleTags`,
`refreshSearchResults`). Rust: `crates/photon-game/src/menus/creative/search.rs`.

## Oracle
Yaratıcı arama oracle satırları (G tarifi).

## Kabul
`tools/photon.py check --capture` → yalnız bilinen Q04 farkı dışında yeşil.

## Belgeler
`docs/records/generic-container-conformance.md`, `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3.

## Devir notu
(boş)
