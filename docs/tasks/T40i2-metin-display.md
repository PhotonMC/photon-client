# T40i2 — Metin display'i

**Amaç:** `text_display` (`DisplayRenderer.TextDisplayRenderer`): metin verisi (bileşen, satır genişliği, arka plan rengi, opaklık, bayraklar), satır bölme (`Font.split`) ve önbellek (`cacheDisplay`), hizalama, arka plan dörtgeni (`textBackground`, görünür/içinden), gölge, opaklık ve arka plan renginin aradeğerlemesi (`LinearIntInterpolator`, `ColorInterpolator`).
**Bağımlılık:** T40i. **Boy:** L.

## Oku
- Java: `client/renderer/entity/DisplayRenderer.java` (`TextDisplayRenderer`), `world/entity/Display.java` (`TextDisplay`), `client/gui/Font.java` (`split`, `width`).
- Rust: `crates/voxel-world/src/display.rs`, `crates/voxel/src/displays.rs`, `crates/voxel-render/src/name_tag.rs` (yazı çizimi).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon text_display`.

## Devir notu
(boş)
