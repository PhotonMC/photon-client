# T58 — At, eşek, katır ve nautilus envanteri

**Amaç:** `HorseInventoryMenu`/`NautilusInventoryMenu` (eyer, zırh, sandık yuvaları: `AbstractMountInventoryMenu`), `HorseInventoryScreen`/`NautilusInventoryScreen` (varlık önizlemesi, `InventoryScreen.extractEntityInInventory`), `OpenHorseScreen` paketi; binek yolcusu açar (T44b1).
**Bağımlılık:** T50. **Boy:** L.

## Oku
Java: `client/gui/screens/inventory/{AbstractMountInventoryScreen,HorseInventoryScreen,NautilusInventoryScreen}.java`, `world/inventory/{AbstractMountInventoryMenu,HorseInventoryMenu,NautilusInventoryMenu}.java`, `network/protocol/game/ClientboundMountScreenOpenPacket.java`. Rust: `crates/voxel/src/screens.rs`, `crates/voxel-render/src/gui/screen/inventory.rs` (varlık önizlemesi).

## Oracle
`container-clicks` (eyer/zırh yuvası kuralları); `container_screen` horse (yuva sayısı, önizleme konumu). G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/records/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
(boş)
