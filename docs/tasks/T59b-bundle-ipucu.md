# T59b — Bundle ipucu ve seçili eşya

**Amaç:** `ClientBundleTooltip` (ızgara, seçili eşya çerçevesi, doluluk çubuğu, "boş" metni) ve
`BundleMouseActions`: ipucu üstünde kaydırma seçili eşyayı değiştirir, `SelectBundleItem` gönderilir;
`BundleContents.Mutable.removeOne` seçiliyi alır (`menus/container/bundle.rs` şimdi hep ilkini alıyor).
**Bağımlılık:** T59. **Boy:** M.

## Oku
Java: `client/gui/screens/inventory/tooltip/ClientBundleTooltip.java`, `client/gui/BundleMouseActions.java`,
`world/item/component/BundleContents.java`. Rust: `crates/photon-game/src/menus/container/bundle.rs`,
`crates/photon-render/src/gui/tooltip.rs`.

## Oracle
G tarifi (ipucu çizimi) ve `movement/container-clicks` (seçimden sonra çıkarma, paket baytı).

## Kabul
`tools/photon.py check --capture` → yalnız bilinen Q04 farkı dışında yeşil.

## Belgeler
`docs/records/generic-container-conformance.md`, `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3.

## Devir notu
(boş)
