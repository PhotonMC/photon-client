# T10b — Dil değişince sohbet geçmişini yeniden çöz

**Amaç:** Dil değiştirilince sohbetteki çevrilebilir bileşenler (sistem mesajları, `chat.type.*`) yeni dille yeniden çözülür ve satırlar yeniden sarılır.
**Bağımlılık:** T10. **Boy:** S.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/gui/components/ChatComponent.java` (`allMessages`, `refreshTrimmedMessages`), `.data/minecraft/source/net/minecraft/client/GuiMessage.java` (içerik `Component` olarak tutulur).
- Rust: `crates/photon-render/src/gui/chat.rs` (`Message.runs` çözülmüş `Run` tutuyor), `crates/photon/src/gui.rs` / `controls.rs` (mesajın eklendiği yer, `chat_log.add_message`).

## Yapılacaklar
1. `Message` içeriği `Component` olarak tutulsun; `ChatLog::set_lang(&Lang, font)` mesajları yeniden çözüp `refresh` etsin.
2. `Gui::set_language` sonrası `controls.chat_log` yenilensin.

## Oracle (G)
`chat` operation'ına `lang` alanı: önce en_us, sonra tr_tr ile `rescaleChat` benzeri yenileme.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Belgeler
`docs/remaining-work.md` Dil seçimi satırı.

## Devir notu
- `ChatLog::add_component` mesajın `Component`ini saklar, `set_lang` çevrilebilir mesajları yeniden çözüp `rescale` eder; `gui.rs` bütün sohbet eklemeleri `add_component`; `play.rs` dil değişince `Gui::reword_chat`. Silinmiş-mesaj işareti ve düz `add_message` kaynaksız kalır (eski dilde).
- Oracle `chat`: mesajda `key` (çevrilebilir), satırda `lang_after` (+6 satır, `Language.inject` + `rescaleChat`). Canlı denenmedi.
