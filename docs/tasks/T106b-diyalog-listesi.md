# T106b — Diyalog listesi

**Amaç:** `DialogListDialogScreen`: `dialogs` alanı (kimlik, `#etiket` veya satır içi liste) çözülür; her düğme alt diyaloğun `computeExternalTitle`ı ile `ShowDialog` açar.
**Bağımlılık:** T106. **Boy:** M.

## Oku
- Java: `client/gui/screens/dialog/DialogListDialogScreen.java`, `server/dialog/DialogListDialog.java`, `Dialog.LIST_CODEC`.
- Rust: `crates/photon-render/src/gui/screen/dialog.rs` (`pack_into_columns`, `button_rect`), `crates/photon-game/src/dialog.rs`, `crates/photon/src/controls/dialogs.rs`; oracle `tests/behavior/java/GuiDialogOracle.java` ve `tools/behavior/cases/gui/dialogs.py` (`listing` örnek).

## Yapılacaklar
`Dialog::DialogList` verisi (`columns`, `button_width`, `exit_action`); oturumun diyalog kayıt defterinden ve etiketlerinden çözüm (etiket kaynağı var mı bak); ekranda `button_list()` yolunu yeniden kullan.

## Oracle
`dialog_screen` operation'ına yeni `kind` ve satırlar (G tarifi).

## Kabul
`CHECK OK`; `client-parity` (ekran) — canlı Pumpkin denemesi `server-conformance` kaydıdır.

## Kapsam dışı
T106'nın öteki dilimleri.

## Belgeler
`docs/gui-oracle.md` satırı, `docs/records/client-dialog-conformance.md`.

## Devir notu
(boş)
