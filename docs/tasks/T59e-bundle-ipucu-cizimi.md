# T59e — Bundle ipucunun çizimi

**Amaç:** `ClientBundleTooltip`: ipucunda görüntü bileşeni (ızgara, yuva arka planı, seçili yuva
vurgusu, gizlenen sayısı `+N`, seçili eşyanın adının üstte ayrı ipucu, doluluk çubuğu, boş bundle
açıklaması); `GuiGraphics.tooltip`'in metin + görüntü bileşenli yerleşimi; `showTooltipWithItemInHand`.
**Bağımlılık:** T59b. **Boy:** M.

## Oku
Java: `client/gui/screens/inventory/tooltip/ClientBundleTooltip.java`, `GuiGraphicsExtractor.tooltip`
(~1212), `AbstractContainerScreen.extractTooltip`. Rust: `photon-render/src/gui/frame/text.rs`
(`tooltip_with`), `gui/screen/container.rs` (`tooltip`), `menus/container/bundle.rs`.

## Oracle
G tarifi: ipucunu doğrudan çizen yeni bir gui operation'ı (konteyner oracle'ı eşya ipucu çizmiyor).

## Kabul
`tools/photon.py check --capture` → yalnız bilinen Q04 farkı dışında yeşil.

## Belgeler
`docs/records/generic-container-conformance.md`, `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3.

## Devir notu
(boş)
