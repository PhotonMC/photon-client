# T94b — Tuş atamaları ekranı

**Amaç:** `KeyBindsScreen` + `KeyBindsList`: kategoriler (`KeyMapping.Category.SORT_ORDER`), eşlemeler
(`KeyMapping.compareTo`: kategori, `order`, çevrilmiş ad), atama düğmesi (tuş/fare bekleme, Esc ile
boşaltma), çakışma işareti (kırmızı `[ ]`), satır başına sıfırla, "Reset Keys"; Controls ekranındaki
"Key Binds..." düğmesinden açılır ve `options.txt`'ye yazar.
**Bağımlılık:** T94. **Boy:** M.

## Oku
- Java: `client/gui/screens/options/controls/{KeyBindsScreen,KeyBindsList}.java`, `client/KeyMapping.java`
  (`compareTo`, `same`, `getTranslatedKeyMessage`).
- Rust: `crates/photon-game/src/options/bindings.rs`, `crates/photon-render/src/gui/options/list*`,
  `crates/photon/src/controls/options.rs`.

## Oracle
G: `key_binds_screen` (seçili satır, çakışma, bekleme hâli).

## Kabul
`CHECK OK`; atanan tuş oyunda çalışır.

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
