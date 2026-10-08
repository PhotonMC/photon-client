# T59 — Kap içeriği ipucu, `#etiket` araması ve eşya süsleri

**Amaç:** Shulker/bundle satırları ipucunda, harita/simya/kitap ipucu satırları, creative arama `#etiket`, dayanıklılık çubuğu ve bekleme örtüsü; F3+H gelişmiş ipucu.
**Bağımlılık:** –. **Boy:** M.

## Oku
Java: `client/gui/screens/inventory/tooltip/*`, `world/item/Item.java` (`appendHoverText`), `client/gui/GuiGraphicsExtractor.java` (`itemDecorations`). Rust: `crates/photon-render/src/gui/tooltip.rs`, `crates/photon-render/src/gui/items.rs`.

## Oracle
`tooltip` oracle satırları (kap içeriği satırları, harita), `item_decorations` (çubuk rengi/uzunluğu, bekleme). G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/photon.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/records/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
(boş)
