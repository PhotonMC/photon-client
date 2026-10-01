# T07 — `OptionsList` + `VideoSettingsScreen` (FOV, GUI ölçeği)

**Amaç:** Kaydırmalı ayar listesi ve video ayarları ekranı; yalnız uygulanabilenler (GUI ölçeği, FOV, görüş mesafesi hariç) etkin.
**Bağımlılık:** T06. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/gui/screens/options/VideoSettingsScreen.java` (299), `.data/minecraft/source/net/minecraft/client/gui/screens/options/OptionsSubScreen.java`, `.data/minecraft/source/net/minecraft/client/gui/components/OptionsList.java` (226), `.data/minecraft/source/net/minecraft/client/gui/components/{AbstractSelectionList,AbstractScrollArea,ContainerObjectSelectionList}.java` (`grep -n "scrollbar\|SCROLLER\|renderScrollbar"`), `.data/minecraft/source/net/minecraft/client/Options.java` ilgili alanlar (T04'te yazıldı).
- Rust: T05 widget'ları, T06 `options.rs`, `crates/voxel-render/src/gui/layout.rs`, `crates/voxel-render/src/window/scale.rs` (GUI ölçeği `0..=maxScale` ve Java `getMaxGuiScale`).

## Yapılacaklar
1. `OptionsList` (kaydırma çubuğu sprite'ları `widget/scroller`, `scroller_background`; çark, tıklama sürükleme, Java ölçüleri).
2. `VideoSettingsScreen`: bütün Java satırları **çizilir**; yalnız GUI ölçeği (`guiScale`, `CycleButton`: Auto/1..max) ve FOV çalışır, görüş mesafesi/FPS/V-Sync/tam ekran vb. etkisiz (Java'da etkin çizilir; etkisiz bırakılanları listele).
3. GUI ölçeği değişince `scale.rs` yeniden hesaplar, tüm açık ekranlar yeni boyutla yeniden kurulur (`resized` zaten var).

## Oracle (G)
`video_settings_screen`: gerçek `VideoSettingsScreen` (alt kümesi `OptionInstance[]` ile kurulu); boyutlar, kaydırma konumu 0/orta/son, GUI ölçeği etiketleri.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (isteğe bağlı): GUI ölçeğini 3→2 yapınca HUD küçülür.

## Kapsam dışı
Grafik seçenekleri gerçek uygulaması (görüş mesafesi, bulanıklık, mipmap…).

## Belgeler
`docs/gui-oracle.md`, `docs/remaining-work.md`.

## Devir notu
(boş)
