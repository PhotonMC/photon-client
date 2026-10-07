# T89 — Tabela düzenleme (`OpenSignEditor`, `SignUpdate`)

**Amaç:** `SignEditScreen`/`HangingSignEditScreen`: tabela modelinin ekranda çizimi, dört satır
düzenleme, imleç; `ServerboundSignUpdatePacket`. Tabelanın dünyada çizimi T80d'dedir.
**Bağımlılık:** T80d. **Boy:** M.

## Oku
- Java: `client/gui/screens/inventory/AbstractSignEditScreen.java`, `SignEditScreen.java`,
  `HangingSignEditScreen.java`, `client/gui/font/TextFieldHelper.java`,
  `ClientPacketListener.handleOpenSignEditor`.
- Rust: T80d tabela çizicisi, `crates/voxel-render/src/gui/screen/` kalıbı.

## Yapılacaklar
1. Paket → ekran (ön/arka yüz), satır düzenleme ve imleç yanıp sönmesi.
2. Kapatınca `SignUpdate`.
3. G tarifi: `sign_edit_screen`, `hanging_sign_edit_screen`.

## Oracle
G.

## Kabul
`CHECK OK`.

## Kapsam dışı
Renkli/parlak tabela boyası (dünya çiziminde).

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
