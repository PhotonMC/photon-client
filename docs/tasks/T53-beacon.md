# T53 — İşaret ışığı menüsü ve ekranı (`SetBeacon`)

**Amaç:** `BeaconMenu` (ödeme yuvası), `BeaconScreen` (güç düğmeleri, onay/iptal, efekt simgeleri), `ContainerSetData` ile seviye ve efektler, `ServerboundSetBeaconPacket`; menü türü 9 şimdilik kapatılıyor.
**Bağımlılık:** T50. **Boy:** M.

## Oku
Java: `client/gui/screens/inventory/BeaconScreen.java`, `world/inventory/BeaconMenu.java`, `network/protocol/game/ServerboundSetBeaconPacket.java`, `world/level/block/entity/BeaconBlockEntity.java` (yalnız efekt listesi). Rust: `crates/voxel-game/src/container_menu.rs` (`MenuKind`, `from_menu_type`), `crates/voxel-render/src/gui/screen/container.rs`.

## Oracle
Menü: `movement/container-clicks` (ödeme yuvası kuralı, hızlı taşıma); ekran: `container_screen` beacon (seviye 0–4, seçili efekt, devre dışı düğmeler). G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
(boş)
