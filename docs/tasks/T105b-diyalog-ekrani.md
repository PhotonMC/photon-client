# T105b — Diyalog ekranı: `SimpleDialogScreen`, uyarı düğmesi, bekleme ekranı

**Amaç:** T105'in okuduğu bildirim ve onay diyaloğunu ekran olarak çizmek ve oyuna bağlamak: başlık + uyarı düğmesi, kaydırılabilir gövde (düz mesaj, eşya), ana eylem düğmeleri, `runAction` (CLOSE/NONE/WAIT_FOR_RESPONSE), `WaitingForResponseScreen`, `ClearDialog`, eylem olarak `run_command`/`show_dialog`/`custom`.
**Bağımlılık:** T105. **Boy:** L.

## Oku
- Java: `client/gui/screens/dialog/{DialogScreen,SimpleDialogScreen,WaitingForResponseScreen,DialogControlSet}.java`, `dialog/body/DialogBodyHandlers.java`, `client/gui/layouts/{HeaderAndFooterLayout,LinearLayout}.java`, `client/gui/components/{ScrollableLayout,FocusableTextWidget,ItemDisplayWidget}.java`.
- Rust: `photon-game/src/dialog.rs`, `photon-net` `take_dialog_events`; ekran kalıbı `photon-render/src/gui/screen/{pause,confirm}.rs`, bağlama `crates/photon/src/controls/pack_prompt.rs`.

## Oracle
G (`dialog_simple_screen`): bildirim, onay, çok satırlı gövde, eşya gövdesi, Esc kapatılamaz.

## Kabul
`CHECK OK`; canlıda Pumpkin'e karşı bir bağlantı (`PHOTON_SCRIPT`) `server-conformance` kaydıdır.

## Kapsam dışı
Çok düğmeli, listeli, girdili diyaloglar (T106).

## Belgeler
`docs/gui-oracle.md`, kayıt `client-dialog-conformance.md` (ekran bölümü).

## Devir notu
- Yapıldı: `photon-render` `screen/{dialog,waiting}.rs` (bildirim/onay ekranı, bekleme ekranı), `photon_game::dialog::run_action`, `crates/photon/src/controls/dialogs.rs` (ShowDialog/ClearDialog, uyarı ekranı, bekleme, sohbette `show_dialog`/`custom` tıklaması); oracle `dialog_screen` (86 satır, `GuiDialogOracle.java`).
- Java'dan sapma: `run_command` doğrudan gider (`sendUnattendedCommand` onay penceresi yok); gövde metninin tıklanabilir stili, oklarla odak ve eşya ipucu yok; eşya simgesi parity dışı (Java oracle eşya kaydetmiyor), kutudan taşan simge çizilmez.
- Ekran yığını yok: her ekran kendi alanında, diyalog üstte çizilir ve girdiyi önce alır.
- Açık: canlı Pumpkin denemesi yapılmadı (`PHOTON_SCRIPT` ile diyalog gönderen sunucu komutu gerek); T106 çok eylemli/listeli/girdili diyaloglar. `dynamic/*` eylemler hâlâ çalışmaz.
