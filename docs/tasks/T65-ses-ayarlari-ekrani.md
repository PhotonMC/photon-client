# T65 — Ses ayarları ekranı

**Amaç:** `SoundOptionsScreen`: ana ses ve kategori kaydırıcıları (`SoundSource`), ses aygıtı satırı kapsam dışı; değerler `options.txt`'e yazılır ve miksere anında yansır.
**Bağımlılık:** T63, T06 (Options ekranı, bitti). Ses yığını kararı: [`audio-backend-decision.md`](../audio-backend-decision.md) (çıkış `cpal`, çözücü `lewton`).

## Oku
- Java: `client/gui/screens/options/SoundOptionsScreen.java`, `client/Options.java` (ses değerleri), GUI tarifi (README "G").
- Rust: `crates/voxel-render/src/gui/` (Options/Video ekranı kalıpları), `crates/voxel-game/src/options.rs`.

## Yapılacaklar
1. `Options`'a `soundSourceVolumes` ve ana ses; `options.txt` anahtarları.
2. Ekran: G tarifi (oracle senaryosu, `gui.rs` kolu, `gui-oracle.md` satırı).

## Oracle
GUI oracle (`gui_sound_options_cases`) gerçek Java ekranına karşı.

## Kabul
`tools/voxel.py parity --capture --family gui` yeşil; `check --capture` → `CHECK OK`.

## Kapsam dışı
Aygıt seçimi, altyazı, ses önizlemesi (`SoundPreviewHandler`).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
