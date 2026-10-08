# T95 — Kalan ayar ekranları

**Amaç:** `AccessibilityOptionsScreen`, `SkinCustomizationScreen` (model parçaları: `ClientInformation`
maskesi, ana el), `OnlineOptionsScreen`, `FontOptionsScreen`, `WorldOptionsScreen`/zorluk düğmesi;
ayarların oyuna bağlanması (ör. skin parçaları sunucuya, ana el, hareket bulanıklığı vb.).
Video ayarlarının etkisiz değerleri (bulanıklık, parlaklık) de burada bağlanır; görüş mesafesi
T95a'da bağlandı.
**Bağımlılık:** — . **Boy:** L.

## Oku
- Java: `client/gui/screens/options/{AccessibilityOptionsScreen,SkinCustomizationScreen,
  OnlineOptionsScreen,FontOptionsScreen,WorldOptionsScreen,OptionsSubScreen}.java`,
  `client/Options.java` (`broadcastOptions`, `getEffectiveRenderDistance`).
- Rust: `crates/voxel-render/src/gui/options/video.rs` (kaydırıcı var, bağlı değil),
  `crates/voxel-net/src/connection/session.rs` (`ClientInformation` sabitleri),
  `crates/voxel/src/play/draw.rs` (`render_distance` sınırı).

## Yapılacaklar
1. Ekranlar (G tarifi, ekran başına operation).
2. `ClientInformation`'ın seçeneklerden kurulması ve değişince yeniden gönderilmesi
   (`broadcastOptions`); görüş mesafesinin çizime bağlanması.

## Oracle
G; paket baytları `wire`.

## Kabul
`CHECK OK`.

## Kapsam dışı
Anlatıcı, telemetri.

## Belgeler
`docs/gui-oracle.md`, `docs/remaining-work.md` §6 ayar maddesi.

## Devir notu
