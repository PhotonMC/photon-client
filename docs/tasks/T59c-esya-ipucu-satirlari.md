# T59c — Kap, harita, iksir ve kitap ipucu satırları; F3+H

**Amaç:** `appendHoverText` satırları: shulker kutusu içeriği (`ItemContainerContents`), harita kimliği/ölçeği,
iksir etkileri (simya şablonu), yazılı kitap yazarı/kuşağı; gelişmiş ipucu (F3+H: eşya kimliği, dayanıklılık,
bileşen sayısı).
**Bağımlılık:** T59. **Boy:** M.

## Oku
Java: `world/item/Item.java` ve alt sınıfların `appendHoverText`i, `world/item/ItemStack.java`
(`getTooltipLines`, `addDetailsToTooltip`). Rust: `crates/photon-game/src/tooltip/lines.rs`.

## Oracle
`tooltip` oracle satırları.

## Kabul
`tools/photon.py check --capture` → yalnız bilinen Q04 farkı dışında yeşil.

## Belgeler
`docs/records/generic-container-conformance.md`, `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3.

## Devir notu
(boş)
