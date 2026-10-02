# T11b — Ayar listelerinde klavye odağı (Tab, ok tuşları)

**Amaç:** `OptionsList` (video/sohbet/fare/kontroller) ve dil listesinde Tab ve ok tuşlarıyla widget gezintisi, odaklanınca listenin görünüre kayması ve klavye odağında ipucu.
**Bağımlılık:** T11. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/gui/components/ContainerObjectSelectionList.java` (`nextFocusPath`, `Entry.focusPathAtIndex`), `.data/minecraft/source/net/minecraft/client/gui/components/ObjectSelectionList.java` (`nextFocusPath`), `AbstractSelectionList.java` (`setFocused`, `scrollToEntry`, `nextEntry`), `.data/minecraft/source/net/minecraft/client/gui/screens/Screen.java` (`keyPressed`, `changeFocus`).
- Rust: `crates/voxel-render/src/gui/navigation.rs` (düz ekran gezintisi), `options_list.rs` (`focus_on`, `Control`), `language_select.rs`.

## Yapılacaklar
1. `SubScreen.key_pressed`: Tab/Shift+Tab ve oklar; liste içinde `ContainerObjectSelectionList.nextFocusPath` (satır içinde sağ/sol, satırlar arası yukarı/aşağı, `index` korunumu), liste dışı widget'lar (Done) `navigation.rs` ile.
2. Klavye odağı listeyi görünüre kaydırır (`setSelected` klavye dalı); `input` klavye olunca ipucu `BelowOrAbove`.
3. Dil listesi: `ObjectSelectionList.nextFocusPath` ve Enter.

## Oracle (G)
`video_settings_screen`, `chat_options_screen`, `mouse_settings_screen`, `controls_screen`, `language_screen` satırlarına `key` adımları (tab, shift+tab, up/down/left/right, enter basılmayanlar).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/remaining-work.md` (video/kontroller/dil satırları), `docs/gui-oracle.md`.

## Devir notu
- Yapıldı: `OptionsList` içinde `Nav`/`next_focus` (ContainerObjectSelectionList: satır içi sağ/sol, satırlar arası sütun korunumlu yukarı/aşağı, başlıklar atlanır, etkin olmayan widget geçilir), `SubScreen.tab/key_pressed/arrow` (başlık, liste, Done), listenin ekran odağı (girdisiz de olabilir: boş satıra/çubuğa tık), klavye odağı girdiyi her zaman görünüre kaydırır; `Control::key_pressed/is_active`; video/chat/mouse/controls sarmalayıcıları + `controls.rs` `menu_key`; dil ekranı `Nav/Target` (ObjectSelectionList: uçta seçimi ve odağı düşürme, Sol/Sağ listede aynı).
- Oracle: dört ayar ekranına ve dil ekranına `key` adımları (tab, shift+tab, ok, enter, space), her satırda gerçek Java ekranı.
- Sapma: kaydırıcı Yukarı/Aşağı tuşlarını almaz (Java gibi; önceki kod Sağ sayıyordu). Enter/Space Done/Font/ekran açan düğmelerde oracle'da basılmadı (Java `setScreen` sahte Gui'de çalışmaz).
- Açık: Controls'un `KeyBindsScreen`i, Font ekranı.
