# T08 — `ChatOptionsScreen` + sohbet ayarları çizime

**Amaç:** Sohbet ayarları ekranı ve ayarların (genişlik, ölçek, opaklık, satır aralığı, yükseklikler, arka plan opaklığı, görünürlük) `ChatComponent` çizimine uygulanması.
**Bağımlılık:** T06. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/gui/screens/options/ChatOptionsScreen.java` (42), `.data/minecraft/source/net/minecraft/client/gui/components/ChatComponent.java` (`getWidth`, `getScale`, `getHeight`, `extractRenderState` ölçek/opaklık kullanımları: `grep -n "options\.\|getScale\|getWidth\|opacity\|lineSpacing"`), `.data/minecraft/source/net/minecraft/client/Options.java` (`chatVisibility` etkisi `Hud`/`ClientPacketListener` filtre: `grep -rn "chatVisibility" .data/minecraft/source/net/minecraft/client/ | head`).
- Rust: `crates/voxel-render/src/gui/chat.rs` (`WIDTH`, `LINES_*`, çizim), `crates/voxel-render/src/gui/chat.rs` testleri, T04 `Options`, T07 `OptionsList`.
- Oracle: `GuiOracle.java chat(...)` (~505) sahte `Options`'ı nasıl kuruyor (`grep -n "chatScale\|chatOpacity\|set(.*options" tests/behavior/java/GuiOracle.java`).

## Yapılacaklar
1. `ChatOptionsScreen` (`OptionsList` ile: görünürlük, renkler, web bağlantıları, `chat.options` satırları; Java'da hangileri varsa hepsi çizilir, yalnız uygulananlar etkin).
2. `ChatScreen`/`chat.rs` çizimi `Options`'tan okur: genişlik (`getWidth` formülü), ölçek, opaklık, yükseklikler (açık/kapalı), satır aralığı, arka plan opaklığı, `hidden`/`commands only` görünürlük.
3. Ayar değişince sohbet anında yeniden düzenlenir (sarma genişliğe bağlı).

## Oracle (G)
Mevcut `chat` operation'ı ayar alanları alsın (`options` nesnesi satırda: `scale`, `opacity`, `width`, `height_focused`, `height_unfocused`, `line_spacing`, `background_opacity`); `gui_cases` içindeki sohbet satırlarına ayarlı varyantlar, **yeni capture**. Ayrıca `chat_options_screen` düzen operation'ı.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Konuşma anlatıcısı (narrator), sohbet bildirimleri.

## Belgeler
`docs/gui-oracle.md`, `docs/remaining-work.md` M5 "sohbet seçenekleri" cümlesi.

## Devir notu
- `gui/chat_options.rs` (`SubScreen` kabuğu; 18 satır, `choices()` ile Options'a yazılır) + `ChatSettings` (`chat.rs`: `lines_per_page`, `set_settings` = `rescaleChat`). Sohbet çizimi Java'nın pose'unu (scale, translate 4) taklit eder: yerel koordinat + `scale_from`; `DrawOp` dikdörtgenleri `f32`, adaptör f32'yi en kısa ondalıkla yazar (Java float'ı gibi). Ortak seçenek kurucuları (`cycle/boolean/slider/named`) `options_list.rs`'e taşındı.
- Oracle: mevcut `chat` operation'ına `options`/`rescale` alanları (+32 satır), yeni `chat_options_screen` (+66 satır; `settingsScreen` Java yöntemi video ile ortak, `hud.chat` ve `isActive()=false` sahteleri).
- Sapma/açık: görünürlük yalnız değer (sunucuya bildirim `session.rs` sabit Full); sohbet kuyruğu ve kısıtlı sohbet istemi çizilmez; sohbet fare üzerine gelme için pose tersi f32 ile hesaplanır ama oracle fare (0,0) kullanır; ölçek 0 oracle'da yok. Canlı denenmedi.
