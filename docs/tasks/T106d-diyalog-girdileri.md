# T106d — Diyalog girdileri ve dinamik eylemler

**Amaç:** `DialogControlSet` girdileri (metin, onay kutusu, tek seçim, sayı aralığı) ve `dynamic/run_command`, `dynamic/custom` eylemleri.
**Bağımlılık:** T106. **Boy:** L.

## Oku
- Java: `client/gui/screens/dialog/{DialogControlSet,input/*}.java`, `server/dialog/action/*`, `server/dialog/input/*`.
- Rust: `crates/photon-render/src/gui/screen/dialog.rs` (`pack_into_columns`, `button_rect`), `crates/photon-game/src/dialog.rs`, `crates/photon/src/controls/dialogs.rs`; oracle `tests/behavior/java/GuiDialogOracle.java` ve `tools/behavior/cases/gui/dialogs.py` (`listing` örnek).

## Yapılacaklar
Girdileri okuyup gövdeye ekle (`Input`), değerlerle komut şablonu/özel yük doldur; gerekirse metin/onay/seçim/kaydırıcı olarak böl.

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
