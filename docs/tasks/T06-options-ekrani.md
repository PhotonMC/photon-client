# T06 — `OptionsScreen`

**Amaç:** Duraklatma menüsünün "Options..." düğmesini açan ana seçenekler ekranı (düğme ızgarası + Done).
**Bağımlılık:** T03, T05. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/gui/screens/options/OptionsScreen.java` (139, satır satır), `.data/minecraft/source/net/minecraft/client/gui/screens/options/OptionsSubScreen.java` (82), `.data/minecraft/source/net/minecraft/client/gui/screens/Screen.java` (`addRenderableWidget`, `clearWidgets`, `rebuildWidgets`), `.data/minecraft/source/net/minecraft/client/gui/layouts/{HeaderAndFooterLayout,GridLayout,LinearLayout}.java` (`grep -n "HeaderAndFooterLayout" OptionsScreen.java`).
- Rust: `crates/voxel-render/src/gui/screen/pause.rs` (`PauseScreen::new` düzeni — benzer ızgara kalıbı), `layout.rs` (`Grid`, `horizontal`; **HeaderAndFooter** yoksa burada ekle, yalnız gerekli kısmı), `widgets.rs`, T03'ün `ScreenStack`i, `crates/voxel/src/controls.rs` (`PauseAction` kolları: Options düğmesi bugün etkisiz).

## Yapılacaklar
1. `gui/options.rs::OptionsScreen`: Java düzeni: başlık `options.title`, FOV kaydırıcısı (T05), `OptionsList` değil `GridLayout` 2 sütun: Skin/Music/Video/Controls/Language/Chat/Resource Packs/Accessibility/Telemetry/Credits/Online düğmeleri; **uygulanmamış alt ekranlar devre dışı çizilmez, Java gibi etkin çizilir ama basınca etkisiz** (veya `active=false` Java'da yoksa etkisiz bırak; devir notuna yaz). Done → önceki ekrana dön. Footer yüksekliği `HeaderAndFooterLayout`.
2. Pause "Options..." → `OptionsScreen` (T03 yığını), Esc/Done → pause'a dön. FOV kaydırıcısı `Options.fov`'u değiştirir ve anında `camera`'ya yansır; değişiklikte `options.txt` kaydı (`Options.save` çağrısı `removed` anında Java'da; ekran kapanırken kaydet).
3. Alt ekran düğmeleri T07–T10 geldikçe bağlanır (bu görevde Video/Chat/Mouse/Language etkisiz).

## Oracle (G)
`options_screen` (gerçek `OptionsScreen`, `GuiOracle.pauseScreen` kurulum kalıbı; `minecraft.options` sahtesi FOV değeriyle): boyutlar (427×240, 320×240, 640×360, 200×150), üzerine gelme, FOV değerleri (30/70/110 etiketleri).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. İsteğe bağlı canlı: Esc → Options → FOV'u sürükle (`shot` ile bir kare).

## Kapsam dışı
Skin/Music/Resource Packs/Accessibility/Telemetry/Credits/Online ekranları (listede etkisiz kalır; `remaining-work.md`'ye yaz).

## Belgeler
`docs/gui-oracle.md`; `docs/remaining-work.md` pause ve ayarlar satırları.

## Devir notu
- `gui/options.rs::OptionsScreen` (Java `HeaderAndFooterLayout(61, 33)`; `layout::place_in_frame` FrameLayout çocuğunu `AbstractChildWrapper` yuvarlamasıyla yerleştirir). Telemetry çevrimdışı olduğu için `active=false` (Java'da `allowsTelemetry()` yanlış). Sapma: ekran yığını yok — `Controls.options_screen: Option<OptionsScreen>` pause'un üstünde, Done/Esc kapatıp `options.save` çağırır (T03'teki YAGNI kararıyla aynı).
- Oracle `options_screen` (47 satır) kuruldu: Tab/Shift+Tab/Enter/ok/tık/sürükle/bırak, FOV değeri. Ok tuşunu kaydırıcı almazsa Java odak gezdirir (T11); o satırlar oracle'a konmadı. Alt ekran açan düğmelere basılmadı (Java ekranı kurar, oracle'da çöker); Rust'ta bu düğmeler `OptionsAction` döndürür, `Controls` yoksayar.
- Yalnız Video/Chat/Mouse/Language vb. bağlanmadı; T07–T10'da `options_input` içindeki eşlemeye eklenir. Canlı oyunda denenmedi (oracle yeterli, görev isteğe bağlı demişti).
