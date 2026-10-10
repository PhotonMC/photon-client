# T105 — Diyaloglar 1: `ShowDialog`, veri ve paketler

**Amaç:** `ShowDialog`/`ClearDialog`, diyalog kaydı, bildirim/onay verisi, gövde türleri (düz mesaj, eşya), eylemler ve `CustomClickAction` yanıtı. Ekran: [T105b](T105b-diyalog-ekrani.md).
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
Yapıldı: `photon-game::dialog` (bildirim, onay, gövde, eylem), `ClickEvent::{ShowDialog, Custom(id, payload)}`, paketler `139/140/68`, `PlaySession::{take_dialog_events, dialog_by_name, send_custom_click_action}`, kayıt defteri girdileri. Sapma: yapılandırma evresi diyaloğu yok sayılır; bilinen paket girdisi verisiz. Açık: ekran, oracle ve oyuna bağlama T105b. Kayıt: [client-dialog-conformance.md](../records/client-dialog-conformance.md).
