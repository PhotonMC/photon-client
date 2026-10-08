# T12 — `ClickEvent`/`HoverEvent` stil modeli + sohbet ipucu

**Amaç:** `Style`'a `click_event`, `hover_event`, `insertion` ekle (JSON/NBT bileşenlerinden okunur); sohbet panelinde fare yazının üstüne gelince ipucu çiz (`show_text`, `show_item`, `show_entity`). Tıklama eylemleri T13'te.
**Bağımlılık:** yok. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/network/chat/Style.java` (`grep -n "clickEvent\|hoverEvent\|insertion"`), `.data/minecraft/source/net/minecraft/network/chat/ClickEvent.java` (169; `Action` listesi ve alanlar), `.data/minecraft/source/net/minecraft/network/chat/HoverEvent.java` (145; `ShowText/ShowItem/ShowEntity`), `.data/minecraft/source/net/minecraft/client/gui/components/ChatComponent.java` (`getClickedComponentStyleAt`, `extractRenderState` içinde hover: `grep -n "hover\|Style"`), `.data/minecraft/source/net/minecraft/client/gui/screens/Screen.java` (`renderComponentHoverEffect`: `grep -n "HoverEvent"`), `.data/minecraft/source/net/minecraft/client/gui/GuiGraphicsExtractor.java` (`setComponentTooltipForNextFrame`).
- Rust: `crates/photon-game/src/text.rs` (`Style` ~41, `Component` ~119, `Run` ~395, JSON ayrıştırma — `grep -n "fn parse\|color\|bold"`), `crates/photon-render/src/gui/chat.rs` (satır/run düzeni: `Run`, `split`, `plain_runs`), `crates/photon-render/src/gui/frame.rs` ipucu çizimi.

## Yapılacaklar
1. `Style`'a `click: Option<ClickEvent>`, `hover: Option<HoverEvent>`, `insertion: Option<String>`; `ClickEvent::{OpenUrl, RunCommand, SuggestCommand, ChangePage, CopyToClipboard, OpenFile, ShowDialog, Custom}` Java'daki alan adlarıyla (26.2 biçimi: `click_event: {"action":..,"command":..}`); `HoverEvent::{ShowText(Component), ShowItem(stack), ShowEntity{type, uuid, name}}`. `Style` birleştirme/miras (`applyTo`) davranışı Java gibi (`grep -n "applyTo"`).
2. Bileşen ayrıştırma: JSON metin bileşenlerinde ve NBT bileşenlerinde (hangisi mevcutsa) bu alanlar okunur; bilinmeyen eylem sessizce atlanmaz, `None` olur.
3. `ChatComponent` hit-test: `style_at(mouse)` (satır/run → `Style`), hover ipucu: `show_text` (bileşen sarmalı, `ClientTextTooltip` kutusu), `show_item` (mevcut eşya ipucu yolu `tooltip.rs`), `show_entity` (ad + tür + uuid satırları).
4. Satır çizim `Run` yapısına `style` referansı taşınır (renk/kalın zaten var; çizimi değiştirme).

## Oracle (G)
`chat` operation'ı hover satırı alsın: `mouse` ile birlikte `components` içinde `hoverEvent` (show_text, show_item bayt-yok: yalnız text); beklenen: ertelenmiş ipucu çizimi (`extractDeferredElements`). `gui_cases` chat bölümüne satırlar, **yeni capture**. Bileşen JSON ayrıştırma için `json`/`component` ailesinde Java `ComponentSerialization.CODEC` çözümü yoksa Java kaynağından birim test.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Tıklama sonucu (T13), `show_item` bileşenli ipucu zenginleştirmesi.

## Belgeler
`docs/gui-oracle.md`; `docs/remaining-work.md` "tıklama/üzerine gelme olayları" satırı.

## Devir notu
- `photon-game/src/text.rs`: `Style.click/hover/insertion`, `ClickEvent`, `HoverEvent` (JSON ve NBT; `Eq` kalktı); bilinmeyen eylem olayı `None` yapar (Java bileşeni hiç ayrıştırmaz — sapma, oracle'da yok), `show_dialog` okunmaz, `show_item` bileşensiz. `gui/text_hit.rs`: `style_under` (glif etkin alanları: `BakedSheetGlyph.left/top/activeRight/bottom`, boşluk `EmptyArea`) ve `bounds` (`PreparedText.bounds`; kırpmayla elenen çizim artık gerçek kutuya göre). Sohbet `draw_with` fare üstündeki `show_text` ipucunu döndürür (etiket ipucu önce, genişlik max(ekran/2, 200)); `show_item` ipucu çizilmez, `show_entity` yalnız ileri ipuçlarında (kapalı).
- Oracle: `chat` satırlarına `mouse` + `json` (gerçek `ComponentSerialization.CODEC`) + ertelenmiş ipucu; yazı tipi sahtesi artık gerçek `BakedSheetGlyph` kutuları veriyor (bitmap), boşluk `EmptyGlyph` gibi (+36 hover satırı). `merged_runs` tam stile göre birleştirir (Java gibi).
- Açık: `show_item` ipucu, tıklama sonucu (T13), canlıda hover. Canlı denenmedi.
