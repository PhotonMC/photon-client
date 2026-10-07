# T94 — Tuş atamaları ekranı (Key Binds)

**Amaç:** `KeyBindsScreen` + `KeyBindsList`: kategoriler, atama düğmeleri, çakışma işareti, sıfırla,
Reset All; `options.txt` `key_*` anahtarlarına yazım ve oyunun bu atamaları okuması (bugün tuşlar
sabit).
**Bağımlılık:** — . **Boy:** M.

## Oku
- Java: `client/gui/screens/options/controls/KeyBindsScreen.java`, `KeyBindsList.java`,
  `client/KeyMapping.java` (kategoriler, sıralama, `same`), `client/Options.java` (`key_` alanları).
- Rust: `crates/voxel-game/src/options/keys.rs`, `crates/voxel/src/controls.rs` (sabit tuşlar),
  `crates/voxel-render/src/gui/options/list*`.

## Yapılacaklar
1. Tuş tablosu (Java'nın kayıt sırası, kategori), kontrollerin tablodan okunması.
2. Ekran ve atama akışı (tuş/fare düğmesi bekleme, Esc ile boşaltma).
3. G tarifi: `key_binds_screen`.

## Oracle
G.

## Kabul
`CHECK OK`; atanan tuş oyunda çalışır.

## Kapsam dışı
Erişilebilirlik ve kalan ayar ekranları (T95).

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
