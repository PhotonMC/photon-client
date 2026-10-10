# T95c — Kalan ayar ekranları ve oyuncu bilgisi

**Amaç:** [T95](T95-kalan-ayar-ekranlari.md)'in kalanı: `AccessibilityOptionsScreen`, `OnlineOptionsScreen`, `FontOptionsScreen`, `WorldOptionsScreen` (zorluk düğmesi); `ClientInformation`'ın kalan alanlarının (`chatColors`, `particles`, `allowServerListing`) seçeneklerden gelmesi; etkisiz video değerlerinin (bulanıklık, parlaklık) bağlanması.
**Bağımlılık:** T95b. **Boy:** L (sığmazsa ekran başına böl).

## Oku
Java: `client/gui/screens/options/{AccessibilityOptionsScreen,OnlineOptionsScreen,FontOptionsScreen,WorldOptionsScreen,OptionsSubScreen}.java`, `client/Options.java`. Rust: `crates/photon-render/src/gui/options/skin.rs` (en yeni örnek, G tarifi: [görev panosu](README.md)), `crates/photon/src/controls/skin.rs`.

## Oracle
G (`settingsScreen` anahtarı + `gui_<ad>_cases` + Rust kolu).

## Kabul
`CHECK OK`.

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
(boş)
