# T54 — Simya standı menüsü ve ekranı

**Amaç:** `BrewingStandMenu` (şişe yuvaları, bileşen, yakıt; `PotionSlot`/`IngredientsSlot`/`FuelSlot` kuralları, `quickMoveStack`), `BrewingStandScreen` (kabarcık ve ok ilerlemesi, yakıt çubuğu), `ContainerSetData` (kaynama süresi, yakıt); menü türü 11.
**Bağımlılık:** –. **Boy:** M.

## Oku
Java: `client/gui/screens/inventory/BrewingStandScreen.java`, `world/inventory/BrewingStandMenu.java`, `world/item/alchemy/PotionBrewing.java` (`isIngredient`, `isPotionIngredient`). Rust: `crates/voxel-game/src/container_menu.rs` (iksir tarifleri: `grep -rn potion crates/voxel-data/src -l`).

## Oracle
Menü: `movement/container-clicks` simya satırları; ekran: `container_screen` brewing (ilerleme değerleri). G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
(boş)
