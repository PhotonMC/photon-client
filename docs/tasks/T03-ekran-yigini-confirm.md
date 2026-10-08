# T03 — Ekran yığını + `ConfirmScreen`

**Amaç:** Bugün her ekran (`pause`, `death`, `disconnected`, kap, sohbet) `Controls`'te ayrı `Option` alanı; ekrandan ekrana geçiş (pause → seçenekler → geri) ve onay iletişim kutusu yok. Java'nın `setScreen`/`onClose` yığın davranışını ve `ConfirmScreen`'i getir; ölüm ekranındaki "Title Screen" onay ister (`deathScreen.quit.confirm`).
**Bağımlılık:** yok. **Boy:** M.

## Oku
- Java (satır satır): `.data/minecraft/source/net/minecraft/client/gui/screens/ConfirmScreen.java` (105), `.data/minecraft/source/net/minecraft/client/gui/screens/DeathScreen.java` (`handleExitToTitleScreen`, `exitToTitleScreen`, onay kolu), `.data/minecraft/source/net/minecraft/client/gui/screens/PauseScreen.java` (yalnız `onDisconnect`), `.data/minecraft/source/net/minecraft/client/gui/screens/Screen.java` (`setScreen` dolayısı, `onClose`, `lastScreen` kalıbı: `grep -n "lastScreen\|onClose\|keyPressed" `), `.data/minecraft/source/net/minecraft/client/gui/components/Button.java`.
- Rust: `crates/photon/src/controls.rs` (`pause`, `death`, `disconnected` alanları, `open_pause`/`close_pause`, `death_event`, `scripted_press`), `crates/photon-render/src/gui/{pause,death,widgets,layout}.rs`.

## Yapılacaklar
1. `photon-render/src/gui/confirm.rs`: `ConfirmScreen::new(font, lang, size, title, message, yes_text, no_text)` — Java düzeni (ızgara: başlık, ileti sarma, iki düğme `FrameLayout`/`LinearLayout` yatay; gecikme `delayedCallback` 20 tik isteğe bağlı). `ConfirmAction::{Yes, No}`; Tab/Enter/Esc (Esc = `onClose` → No/geri).
2. Basit ekran yığını: `Controls` içinde geçerli ekran + `önceki` kaydı (Java `lastScreen`) yerine küçük bir `ScreenStack` (yalnız gereken geçişler: pause → confirm, death → confirm). Alanları tek enum'a toplamak büyük yeniden yazım ise YAGNI: ConfirmScreen'i `Option<(Confirm, Origin)>` olarak ekle ve `Origin` ile geri dön.
3. Ölüm ekranı "Title Screen" → (hardcore değilse Java'da hemen `ConfirmScreen`: `deathScreen.quit.confirm`) onayda `request_close`, hayırda ölüm ekranına dön. Pause "Disconnect" Java'da onaysız kalır; değiştirme.
4. Script: `press` adımı ConfirmScreen'e de yönlensin.

## Oracle (G)
Operation `confirm_screen`: gerçek `ConfirmScreen` (`GuiOracle.pauseScreen`/`deathScreen` kurulum örneği), satırlar: kısa/uzun ileti, boyutlar, düğme metinleri, üzerine gelme, gecikmeli (tik 0 ve 20). Rust: `confirm::ConfirmScreen`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gerekmez (oracle yeter); yine de isteniyorsa `shot` ile kill → Title Screen.

## Kapsam dışı
Başlık ekranı, gerçek "başlığa dön" (bağlantıyı koparıp menüye) — `request_close` kalır.

## Belgeler
`docs/gui-oracle.md` satırı; `docs/remaining-work.md`'de ölüm ekranı "ConfirmScreen yok" cümlesini sil/güncelle.

## Devir notu
- `photon-render/src/gui/confirm.rs`: `ConfirmScreen` (düzen Java'nın dikey/yatay `LinearLayout`'u; ileti `MultiLineLabel` 15 satır + üç nokta; `set_delay`/`tick`/`escape`/`tab`/`pressed`/`clicked`, `ConfirmBackground::{Menu, Death}`, `resized`). `death::draw_background` paylaşılır.
- Oracle `confirm_screen` (44 satır, geri çağrı cevabı dahil) ilk koşuda eşit; parity 929. Java'da `SilentSounds` ile düğme sesi susturuldu.
- Yığın yerine `Controls.confirm: Option<ConfirmScreen>` (YAGNI): yalnız ölüm → "Title Screen" kullanıyor; evet `request_close`, hayır `respawn` + ölüm ekranını kapat. Pause "Disconnect" onaysız kaldı (Java'daki gibi).
- Fare konumu güncellemesi `move_gui_mouse` yardımcısında toplandı. Script `press`/`escape` adımı confirm'e yönlenir.
- Açık: genel ekran yığını (pause → seçenekler → geri) T06'da gerekirse; canlı kill → Title Screen denenmedi (oracle yeterli).
