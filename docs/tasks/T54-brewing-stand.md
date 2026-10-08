# T54 — Simya standı menüsü ve ekranı

**Amaç:** `BrewingStandMenu` (şişe yuvaları, bileşen, yakıt; `PotionSlot`/`IngredientsSlot`/`FuelSlot` kuralları, `quickMoveStack`), `BrewingStandScreen` (kabarcık ve ok ilerlemesi, yakıt çubuğu), `ContainerSetData` (kaynama süresi, yakıt); menü türü 11.
**Bağımlılık:** –. **Boy:** M.

## Oku
Java: `client/gui/screens/inventory/BrewingStandScreen.java`, `world/inventory/BrewingStandMenu.java`, `world/item/alchemy/PotionBrewing.java` (`isIngredient`, `isPotionIngredient`). Rust: `crates/photon-game/src/menus/container.rs` (iksir tarifleri: `grep -rn potion crates/photon-data/src -l`).

## Oracle
Menü: `movement/container-clicks` simya satırları; ekran: `container_screen` brewing (ilerleme değerleri). G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/photon.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/records/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
- Yapıldı: menü (`MenuKind::Brewing`, yuva kuralları, `quickMoveStack`), eşya tanımına `brewing_ingredient` (oracle'dan, `items.py`), ekran (yakıt çubuğu, ok, kabarcıklar, simgeler), `Frame::sprite_cut` (kısmi sprite'ı çizim günlüğüne kaydeden). Kayıt: `docs/records/generic-container-conformance.md` "Simya standı".
- Ek düzeltme: açık kabın `ContainerSetData` değerleri menüye 4 yerine 10 tane taşınıyor (büyü masasının 4–9. değerleri).
- Açık: kaynama ve yakıtı sunucu yürütür (istemci tahmini yok); canlı gözlem yapılmadı.
