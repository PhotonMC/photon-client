# T09 — `MouseSettingsScreen` + duyarlılık

**Amaç:** Fare ayarları ekranı; duyarlılık, ters fare ve kaydırma duyarlılığı kamerayı/kaydırmayı gerçekten etkiler.
**Bağımlılık:** T06. **Boy:** S.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/gui/screens/options/MouseSettingsScreen.java` (37), `.data/minecraft/source/net/minecraft/client/gui/screens/options/controls/ControlsScreen.java` (43; `Controls` düğmesi → sonra), `.data/minecraft/source/net/minecraft/client/Options.java` (`sensitivity`, `invertMouse`, `mouseWheelSensitivity`, `discreteMouseScroll`, `touchscreen`, `rawMouseInput`: `grep -n`), `.data/minecraft/source/net/minecraft/client/MouseHandler.java` (`turnPlayer`, kaydırma: `grep -n "sensitivity\|invert\|scroll"`).
- Rust: `crates/voxel/src/controls.rs` (~880 duyarlılık ölçeği, `invert` yok; kaydırma `crates/voxel-game/src/scroll.rs`), `docs/mouse-*`/`smooth-double-conformance.md` zaten `SmoothDouble` parity'si.

## Yapılacaklar
1. `MouseSettingsScreen` (Java satırları; `OptionsList`): duyarlılık kaydırıcısı ("SENSITIVITY: 100%", `options.sensitivity.min/max` özel metinleri), ters fare, kaydırma duyarlılığı, ayrık kaydırma; `rawMouseInput`/`touchscreen` etkisiz çizilir.
2. `controls.rs`: `Options.sensitivity` ve `invert_mouse` dönüşüne bağlanır (T04'te sabit 0.5'ti).

## Oracle (G)
`mouse_settings_screen` düzen operation'ı (kaydırıcı etiketleri sınır değerleriyle).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`; `crates/voxel/src/controls.rs` birim testi: sensitivity 1.0 ve invert ile delta işareti/ölçeği Java `MouseHandler.turnPlayer` formülüyle.

## Kapsam dışı
Tuş atamaları ekranı (`ControlsScreen` alt ekranları).

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
- `gui/mouse_options.rs`: `MouseSettingsScreen` (6 satır) + `ControlsScreen` (Java'da fare ekranının tek giriş yolu; Key Binds düğmesi ölü). `ButtonControl`/`button` listeye düğme ekler (basış "değişti" sayılır). Tekerlek kaydırıcısı `MappedIntRange<f64, fn, fn>` (`log_mouse/unlog_mouse` artık pub). Seçenekler `choices()` ile `Options`'a yazılır; Esc/Done kaydeder.
- Oracle: `settingsScreen` Java yöntemi `mouse`/`controls` türleriyle; `mouse_settings_screen` 37, `controls_screen` 25 satır; ham fare satırı Java'da da yok çıktı (`isRawMouseInputSupported` yanlış). Düğmeler (Mouse Settings, Key Binds) basılmadı (Java `setScreen` sahtede çöker).
- `controls.rs`: `turn_by` (Java `turnPlayer` formülü) birim testli. Açık: Key Binds ekranı, Tab/ok odağı (T11), ayrık kaydırma gerçek etkisi.
