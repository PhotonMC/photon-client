# T90 — Komut bloğu, yapı bloğu ve jigsaw ekranları

**Amaç:** Operatör blok ekranları: `CommandBlockEditScreen` (+ minecart), `StructureBlockEditScreen`,
`JigsawBlockEditScreen`, test bloğu ekranları; ilgili serverbound paketler
(`SetCommandBlock`, `SetCommandMinecart`, `SetStructureBlock`, `SetJigsawBlock`, `JigsawGenerate`,
`TestInstanceBlockAction`, `SetTestBlock`). Büyükse komut bloğu + yapı bloğu ilk oturum, kalanı T90b.
**Bağımlılık:** T85. **Boy:** L.

## Oku
- Java: `client/gui/screens/inventory/{AbstractCommandBlockEditScreen,CommandBlockEditScreen,
  MinecartCommandBlockEditScreen,StructureBlockEditScreen,JigsawBlockEditScreen,TestBlockEditScreen,
  TestInstanceBlockEditScreen}.java`; blok entity verisinin istemciye gelişi (`BlockEntityData`).
- Rust: komut önerisi (`crates/voxel-render/src/gui/chat/` komut önerici), blok entity durumu.

## Yapılacaklar
1. Komut bloğu ekranı (komut önerisiyle), mod/koşul/her zaman aktif düğmeleri, paket.
2. Yapı bloğu ekranı (modlar, alanlar), paket.
3. Jigsaw ve test blokları.
4. G tarifi: ekran başına operation.

## Oracle
G; paket baytları `wire`.

## Kabul
`CHECK OK`.

## Kapsam dışı
Yapı sınırlarının dünyada çizimi (blok entity çizicisi, M8).

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
