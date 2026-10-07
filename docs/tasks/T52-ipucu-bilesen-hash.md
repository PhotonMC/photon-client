# T52 — Bileşenli eşyada tıklama paketi hash'i ve bundle tıklaması

**Amaç:** Tıklama paketi bileşeni olan yığını bileşensiz hash'liyor; `HashedStack` bileşen hash'i (CRC32C), bundle tıklama geçersiz kılmaları (`SelectBundleItem`, bundle üstüne tıklama), zırh kilidi büyüsü ve hasarlı eşya takası tahmini.
**Bağımlılık:** –. **Boy:** M.

## Oku
Java: `network/HashedStack.java`, `network/HashedPatchMap.java`, `world/inventory/AbstractContainerMenu.java` (`doClick`), `world/item/BundleItem.java`, `world/item/component/BundleContents.java`. Rust: `crates/voxel-game/src/container_menu.rs`, `crates/voxel-net/src/connection/protocol.rs` (tıklama kodlayıcıları).

## Oracle
`movement/container-clicks` satırları: bileşenli eşya (özel ad, büyü), bundle alma/koyma, `SelectBundleItem` baytı; paket baytı Java ile birebir. G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
(boş)
