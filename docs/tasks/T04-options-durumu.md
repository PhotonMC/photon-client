# T04 — Seçenek değerleri ve `options.txt`

**Amaç:** Bugün FOV 70, GUI ölçeği, duyarlılık 0.5, sohbet genişliği 320 vb. kodda sabit (`camera.rs DEFAULT_FOV`, `controls.rs:~883`, `chat.rs WIDTH`). Java'nın `Options` durumunu (yalnız aşağıdaki alanlar) `options.txt` biçimiyle oku/yaz ve sabitleri ona bağla. Ekran yok (T06+).
**Bağımlılık:** yok. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/Options.java` (2011 satır — **bütün okuma**; `grep -n "processOptions\|private final OptionInstance<.*> \(fov\|guiScale\|sensitivity\|chatOpacity\|chatScale\|chatWidth\|chatHeightFocused\|chatHeightUnfocused\|chatLineSpacing\|textBackgroundOpacity\|chatVisibility\|language\|invertMouse\|mouseWheelSensitivity\|renderDistance\|hideGui\)\|public void load\|public void save"` ile çapaları bul, her biri için ~40 satırlık pencere), `.data/minecraft/source/net/minecraft/client/OptionInstance.java` (`ValueSet`, `IntRange`, `UnitDouble`, `CaptionBasedToString`; `grep -n "class\|record\|enum"`).
- Rust: `crates/voxel-game/src/option.rs` (zaten `OptionValues`/`SliderableEnum`/`CycleableEnum`… Java'dan kopya; oracle `option_values` ailesi var, `docs/option-values-conformance.md`). Var olan tipler yeniden kullanılır.
- Bağlanacak yerler: `crates/voxel-render/src/camera.rs` (FOV), `crates/voxel/src/controls.rs` (~880 duyarlılık), `crates/voxel-render/src/gui/chat.rs` (WIDTH 320, LINES_*, sabitler), `crates/voxel-render/src/window/scale.rs` (GUI ölçeği — zaten `gui_scale` parametreli mi bak).
- Örnek dosya: `command_history.txt` yazımı `controls.rs`/`chat.rs`'de (oyun dizini çözümü; aynı konumu kullan).

## Yapılacaklar
1. `voxel-game/src/options.rs`: `Options` yapısı; alanlar: `fov` (30–110 int, varsayılan 70), `gui_scale` (0=auto), `sensitivity` (0..1, 0.5), `invert_mouse`, `mouse_wheel_sensitivity`, `chat_visibility`, `chat_opacity`, `chat_scale`, `chat_width`, `chat_height_focused`, `chat_height_unfocused`, `chat_line_spacing`, `text_background_opacity`, `lang`. Hepsi Java varsayılanı ve aralığıyla (`OptionInstance` değerlerinden oku, tahmin etme).
2. `load(path)`/`save(path)`: Java biçimi `anahtar:değer` satırları (`processOptions` anahtar adları!), bilinmeyen anahtarlar korunur (yeniden yazımda kaybolmaz), bozuk değer varsayılana düşer. Java gibi yazım sırası.
3. Bağla: FOV (`camera.rs`), duyarlılık (`controls.rs`), sohbet sabitleri (`chat.rs`; `ChatScreen`/`ChatComponent` oluşturulurken `Options`'tan okunur), GUI ölçeği (`scale.rs`; 0 = otomatik, sabitler Java `calculateScale` ile zaten var mı bak).
4. Uygulama açılışında `options.txt` (oyun dizini) oku; yoksa yaz. Hiçbir ekran bunu henüz değiştirmez.
5. Birim testleri: Java varsayılanları, aralık sıkıştırma, round-trip, bilinmeyen anahtar korunumu (beklenen değerleri `Options.java`'dan elle çıkar).

## Oracle
Yeni ailesiz: `option_values` ailesine **gerekirse** `fov`/`chat_*` aralık satırı ekle (yalnız Java'da OptionInstance kurulup `values().validSlider()` okunabiliyorsa; headless'ta zor ise Java kaynağından satır numaralı birim test yeter, nedenini devir notuna yaz).

## Kabul
`tools/voxel.py check` → `CHECK OK`; varsayılanlarla davranış değişmez (sohbet/oracle satırları aynı: `parity` sayısı sabit).

## Kapsam dışı
Ekranlar, ses/grafik/erişilebilirlik alanları, tuş atamaları.

## Belgeler
`docs/remaining-work.md` M6 "Ayarlar" satırı; `docs/option-values-conformance.md` değişmez.

## Devir notu
(boş)
