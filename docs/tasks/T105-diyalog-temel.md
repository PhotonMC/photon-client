# T105 — Diyaloglar 1: `ShowDialog`, bildirim ve onay diyalogları

**Amaç:** `ShowDialog`/`ClearDialog`, diyalog kaydı, `SimpleDialogScreen` (bildirim, onay), gövde türleri (düz mesaj, eşya) ve eylem düğmeleri; `CustomClickAction` yanıtı.
**Bağımlılık:** —. **Boy:** L.

## Oku
- Java: `client/gui/screens/dialog/{DialogScreen,DialogScreens,SimpleDialogScreen,WaitingForResponseScreen}.java`, `dialog/body/*`, `network/protocol/common/{ClientboundShowDialogPacket,ClientboundClearDialogPacket,ServerboundCustomClickActionPacket}.java`, `server/dialog/*` (veri kaydı).
- Rust: `crates/photon-game/src/text.rs` (`ClickEvent`; diyalog eylemi şimdi okunmuyor), ekran kalıbı `photon-render/src/gui/screen/`.

## Yapılacaklar
1. Diyalog veri çözücüsü (kayıt ve satır içi).
2. Ekran düzeni G tarifiyle; ekran yığınına bağlama.
3. `ClickEvent`'in diyalog eylemi.

## Oracle
G (`dialog_simple_screen`).

## Kabul
`CHECK OK`; canlıda Pumpkin'e karşı bir bağlantı (`PHOTON_SCRIPT`), `server-conformance` kaydıdır, `client-parity` değildir.

## Kapsam dışı
Çok düğmeli, listeli, girdili diyaloglar (T106).

## Belgeler
`docs/gui-oracle.md`, kayıt `client-dialog-conformance.md`.

## Devir notu
(boş)
