# T05 — Kaydırıcı ve döngü düğmesi widget'ları

**Amaç:** Ayar ekranlarının iki temel widget'ı: `AbstractSliderButton` (OptionInstance kaydırıcıları) ve `CycleButton` (boolean/enum düğmeleri), etiket biçimiyle ("FOV: 70", "Normal", "Ters Fare: Kapalı").
**Bağımlılık:** T04. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/gui/components/AbstractSliderButton.java` (161), `.data/minecraft/source/net/minecraft/client/gui/components/CycleButton.java` (349; `Builder`, `onOffBuilder`, `displayOnlyValue`), `.data/minecraft/source/net/minecraft/client/gui/components/AbstractWidget.java` (`renderWidget`, `getTextureY`), `.data/minecraft/source/net/minecraft/client/OptionInstance.java` (`createButton`, `toString` caption: `grep -n "createButton\|CaptionBasedToString\|OptionInstanceSliderButton"`), `.data/minecraft/source/net/minecraft/client/gui/components/OptionsList.java` yalnız `addBig/addSmall` düzeni için T07'de.
- Sprite adları: `widget/slider`, `widget/slider_highlighted`, `widget/slider_handle`, `widget/slider_handle_highlighted` (`grep -n "SLIDER" AbstractSliderButton.java`). `crates/voxel-render/src/gui/atlas.rs` sprite klasörleri (`widget` zaten var mı bak).
- Rust: `crates/voxel-render/src/gui/widgets.rs` (`Widget`, `Kind`, `draw`; nine-slice düğme kalıbı), `layout.rs`.

## Yapılacaklar
1. `widgets.rs`'e `Kind::Slider{value, ...}` ve `Kind::Cycle{...}` (veya ayrı `slider.rs`); çizim Java'daki gibi (sprite, tutamak konumu `value*(w-8)`, etiket ortalama, devre dışı rengi).
2. Girdi: tıkla/sürükle → değer (`setValueFromMouse`), ok tuşları (±1 adım, `keyPressed` 263/262), `CycleButton`: tıklama/Enter sonraki değer, kaydırma ters yön, Shift ile geri.
3. Etiket üretimi: `Lang` anahtarlarıyla (`options.fov`, `options.fov.min`/`max` özel metinleri, `options.generic_value`, `options.on/off`); `Options` alanına bağlı değer dönüşü.

## Oracle (G)
Operation `widgets`: Java'da gerçek `OptionInstance`'tan `createButton` ile üretilen düğme (`options` fake'i; `OptionInstance` yapılandırıcı örneği `Options.java`'da: FOV `IntRange` kaydırıcı, `chatVisibility` enum döngü, `invertYMouse` boolean). Satırlar: değer 0/0.5/1, vurgulu/normal, devre dışı, Fov min/max özel etiketleri. `AbstractWidget.extractRenderState` ile çizip `fill/sprite/text` kayıtlarını kıyasla.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
`ValueSet` türlerinin hepsi (yalnız IntRange, UnitDouble, boolean, basit enum); ipuçları (T11).

## Belgeler
`docs/gui-oracle.md` satırı.

## Devir notu
(boş)
