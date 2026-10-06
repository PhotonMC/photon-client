# T40i2 — Metin display'i

**Amaç:** `text_display` (`DisplayRenderer.TextDisplayRenderer`): metin verisi (bileşen, satır genişliği, arka plan rengi, opaklık, bayraklar), satır bölme (`Font.split`) ve önbellek (`cacheDisplay`), hizalama, arka plan dörtgeni (`textBackground`, görünür/içinden), gölge, opaklık ve arka plan renginin aradeğerlemesi (`LinearIntInterpolator`, `ColorInterpolator`).
**Bağımlılık:** T40i. **Boy:** L.

## Oku
- Java: `client/renderer/entity/DisplayRenderer.java` (`TextDisplayRenderer`), `world/entity/Display.java` (`TextDisplay`), `client/gui/Font.java` (`split`, `width`).
- Rust: `crates/voxel-world/src/display.rs`, `crates/voxel/src/displays.rs`, `crates/voxel-render/src/name_tag.rs` (yazı çizimi).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon text_display`.

## Devir notu
- Yapıldı: metin verisi, render durumu ve opaklık/arka plan aradeğerlemesi (`voxel-world/display.rs`; gerçek `TextDisplay` varlığı), `submitInner` yerleşimi (`voxel-render::display::{text_submit, text_vertices}`; gerçek `TextDisplayRenderer.submit`), `Matrix4::rotate_y`, oyunda `displays::display_texts`. Canlıda görüldü. Kayıt: `docs/client-display-parity-conformance.md`.
- Oracle: `entity/text-display-submit`; `entity` ailesine `icu4j` eklendi (gerçek `Font` sınıfı için).
- Sapma/açık: satır bölme oracle'sız, arka plan opaklığı seçeneği sabit, satır önbelleği yok, `POLYGON_OFFSET` derinlik kayması yok.
