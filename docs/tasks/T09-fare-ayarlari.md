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
(boş)
