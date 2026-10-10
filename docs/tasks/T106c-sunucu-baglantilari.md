# T106c — Sunucu bağlantıları diyaloğu

**Amaç:** `ClientboundServerLinksPacket` çözümü (bileşenli/bilinen bağlantı türü) ve `ServerLinksDialogScreen`: her bağlantı bir `OpenUrl` düğmesi.
**Bağımlılık:** T106. **Boy:** M.

## Oku
- Java: `client/gui/screens/dialog/ServerLinksDialogScreen.java`, `server/ServerLinks.java`, `server/dialog/ServerLinksDialog.java`.
- Rust: `crates/photon-render/src/gui/screen/dialog.rs` (`pack_into_columns`, `button_rect`), `crates/photon-game/src/dialog.rs`, `crates/photon/src/controls/dialogs.rs`; oracle `tests/behavior/java/GuiDialogOracle.java` ve `tools/behavior/cases/gui/dialogs.py` (`listing` örnek).

## Yapılacaklar
Paketi çöz (T100'den devredildi), oturumda sakla; `Dialog::ServerLinks`; ekranda `button_list()` yolunu yeniden kullan.

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
