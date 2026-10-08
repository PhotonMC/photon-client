# T13 — Sohbet tıklama eylemleri + `ConfirmLinkScreen`

**Amaç:** Sohbette bileşene tıklayınca Java'daki gibi: `run_command` (komutu gönderir), `suggest_command` (giriş kutusunu doldurur), `copy_to_clipboard`, `open_url` (onay ekranı), `insertion` (Shift+tık).
**Bağımlılık:** T03 (ConfirmScreen yığını), T12. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/gui/screens/ChatScreen.java` (186–260: `mouseClicked`, `handleComponentClicked`, `insertionClickMode`), `.data/minecraft/source/net/minecraft/client/gui/screens/Screen.java` (`defaultHandleGameClickEvent`, `clickUrlAction`, `handleClickEvent` — `grep -n "ClickEvent\|CONFIRM\|openUrl"`), `.data/minecraft/source/net/minecraft/client/gui/screens/ConfirmLinkScreen.java` (128), `.data/minecraft/source/net/minecraft/network/chat/ClickEvent.java`.
- Rust: `crates/photon-render/src/gui/chat.rs` (`ChatScreen`, `click`), T12 `style_at`, T03 `ConfirmScreen`, `crates/photon/src/controls.rs` (`chat` kolları; `send` yardımcıları).

## Yapılacaklar
1. `ChatScreen::clicked(mouse, shift) -> Option<ChatClick>` (`Run(command)`, `Suggest(text)`, `Copy(text)`, `OpenUrl(url)`, `Insert(text)`); uygulama bunları işler: `run_command` Java gibi `ChatCommand` (başında `/`) olarak gönderir (imza gereken komutlarda Java'da onay: `SignableCommand`; imza yok → doğrudan `ChatCommand`), pano için GLFW pano (`glfw::Window::set_clipboard_string`; yoksa not).
2. `open_url`: `ConfirmLinkScreen` (T03 `ConfirmScreen` üzerine; "Open Link/Copy to Clipboard/Cancel" üç düğme); onayda sistem tarayıcısı **açılmaz** (yeni bağımlılık yok): URL panoya kopyalanır ve ekranda not (`remaining-work.md`'ye yaz) veya `open`/`xdg-open` ile `std::process::Command` (Java `Util.getPlatform().openUri` eşiti) — ikincisini yap, platforma göre.
3. `run_command` yalnız `/` ile başlayan komutu kabul eder (Java 26.2: `ClickEvent.RunCommand` komut metni `/`'sız), kontrol et.

## Oracle (G)
`confirm_link_screen` operation'ı (gerçek `ConfirmLinkScreen`, URL kısa/uzun, boyutlar, düğme üzerine gelme). Tıklama davranışı için Java kaynağından birim testleri (hangi eylem hangi çıktıyı verir).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı (isteğe bağlı, `shot`): `tellraw @a {"text":"x","click_event":{"action":"run_command","command":"/say hi"}}` sonrası tıkla.

## Kapsam dışı
`ShowDialog`/`Custom` eylemleri (yok sayılır, günlük satırı), imzalı komut zinciri.

## Belgeler
`docs/gui-oracle.md`; `docs/remaining-work.md` M5.

## Devir notu
- `ChatLog::clicked_style` (Java `ClickableStyleFinder`: son glif, tıklama olayı veya Shift'te insertion; aynı pose/glif alanları), `Controls::style_clicked`: `run_command` (başındaki tek `/` atılır, `ChatCommand` doğrudan; Java'daki komut doğrulama/onay pencereleri — ayrıştırma hatası, imza, izin — yok), `suggest_command` (kutu değiştirilir), `copy_to_clipboard`, `open_url` (`ConfirmScreen::link` = `ConfirmLinkScreen`, güvenilmez; Yes sistem açıcısıyla (`open`/`xdg-open`/`start`), Copy panoya, No/Esc sohbete), `open_file`; `change_page`/`custom` yok sayılır. `chatLinks`/`chatLinksPrompt` seçenekleri tutulmuyor (hep sorar).
- Oracle `confirm_screen`'e `link` alanı (+25 satır: 3×100 düğme, uyarı); Copy Java'da klavye ister, basılmadı. Tıklama eylemleri için birim test (`clicked_style`), eylem eşlemesi canlıda denenmedi.
- Açık: komut doğrulama pencereleri, `chatLinks` seçenekleri, güvenilen bağlantı yolu (pause simgeleri `ConfirmLinkScreen.confirmLink` hâlâ bağlı değil).
