# T50 — Loom, cartography ve smithing ekranlarının oracle'ı

**Amaç:** `LoomScreen` (afiş atlası desenleri, desen seçimi `ContainerButtonClick`), `CartographyTableScreen` (harita önizlemesi, kilit/büyüt/kopyala simgeleri) ve `SmithingScreen` (şablon/taban/ek yuva arka plan döngüsü `CyclingSlotBackground`, sonuç önizleme eşyası) Rust ekranları zaten çiziliyor ama kap ekranı oracle satırı yok ([gui-oracle sınırları](../gui-oracle.md)). `MovementOracle.openMenu` ile gerçek Java ekranına eşitle; fark çıkarsa Rust'ı düzelt.
**Bağımlılık:** –. **Boy:** M.

## Oku
Java: `client/gui/screens/inventory/{LoomScreen,CartographyTableScreen,SmithingScreen,ItemCombinerScreen,CyclingSlotBackground}.java`, `world/inventory/{LoomMenu,CartographyTableMenu,SmithingMenu}.java`. Rust: `crates/voxel-render/src/gui/screen/container.rs`, `crates/voxel-game/src/container_menu/transform.rs`. Oracle: `container_screen` operation'ı (`grep -n container_screen tests/behavior/java/GuiOracle.java tools/behavior/cases_gui.py`).

## Oracle
G tarifi; `container_screen` satırlarına loom (desen listesi, kaydırma, seçili), cartography (harita, kilit, kopya), smithing (boş/şablonlu yuvalar, sonuç önizleme). G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
(boş)
