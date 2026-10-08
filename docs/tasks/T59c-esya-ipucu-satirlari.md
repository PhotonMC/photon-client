# T59c — Kap, harita, iksir ve kitap ipucu satırları; F3+H

**Amaç:** `appendHoverText` satırları: shulker kutusu içeriği (`ItemContainerContents`), harita kimliği/ölçeği,
iksir etkileri (simya şablonu), yazılı kitap yazarı/kuşağı; gelişmiş ipucu (F3+H: eşya kimliği, dayanıklılık,
bileşen sayısı).
**Bağımlılık:** T59. **Boy:** M.

## Oku
Java: `world/item/Item.java` ve alt sınıfların `appendHoverText`i, `world/item/ItemStack.java`
(`getTooltipLines`, `addDetailsToTooltip`). Rust: `crates/photon-game/src/tooltip/lines.rs`.

## Oracle
`tooltip` oracle satırları.

## Kabul
`tools/photon.py check --capture` → yalnız bilinen Q04 farkı dışında yeşil.

## Belgeler
`docs/records/generic-container-conformance.md`, `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3.

## Devir notu
- Yeni `gui/tooltip-lines` oracle'ı (`GuiTooltipOracle.java`, `cases/gui/tooltips.py`): Java'nın
  `getTooltipLines`ı ile Rust `tooltip::lines` stilli parçalarla, 18 satır birebir.
- Eklenen: harita (istemci harita verisi `Maps`), kap içeriği, yazılı kitap (+ başlık adı), demirci
  şablonu, gelişmiş satırlar (`Flags::advanced`, bileşen sayısı `component_sets.rs` tablosundan).
- Düzeltilen: bayrak desenleri Java sırasına; dil `deprecated.json`ı uygular (oyunda da).
- Açık: arı/ganimet/saksı/mermi/balık/profil/varlık/spawner satırları; `F3+H` anahtarı T93'te.
