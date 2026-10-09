# T85b — Başlık ekranının panoraması

**Amaç:** `PanoramaRenderer`/`CubeMap`: altı yüzlü panorama, zamanla dönen kamera ve bulanıklık
(`menuBackgroundBlurriness`); başlık ve menü ekranlarının arka planı.
**Bağımlılık:** T85. **Boy:** M.

## Oku
- Java: `client/renderer/PanoramaRenderer.java`, `client/renderer/CubeMap.java`,
  `client/gui/screens/Screen.java` (`extractPanorama`, `extractBlurredBackground`),
  `client/renderer/GameRenderer.java` (bulanıklık geçişi).
- Rust: `crates/photon-render/src/vulkan/` (geçiş tablosu), T85'in başlık ekranı.

## Yapılacaklar
1. Altı dokunun yüklenmesi (`textures/gui/title/background/panorama_*.png`), küp çizimi, dönüş hızı.
2. Bulanıklık geçişi (Java'nın blur post efektinin yarıçapı ve geçiş sayısı).

## Oracle
Küpün kamera matrisi ve köşeleri `math` ailesinde; görüntü kabulü `shot` ile.

## Kabul
`CHECK OK`; başlık ekranında dönen panorama.

## Kapsam dışı
Menü dışı ekranlarda bulanıklık (oyun içi ekranlar ayrı).

## Belgeler
`docs/gui-oracle.md` satırı.

## Devir notu
- Yapılan: `panorama_*` küp dokusu (altı katman, `CubeMapTexture` sırası ve satır çevirme), `PANORAMA` boru hattı (`vulkan/panorama.rs`, `Pass::Panorama`), model-view/dönüş (`photon-render/src/panorama.rs`, `panorama_view` oracle'ı JOML ile bit eşit), menü döngüsünde dönüş.
- Sapma: `panoramaScrollSpeed` seçeneği okunmuyor, hız 1.0 (T95 erişilebilirlik ayarları). `jar/assets` içindeki panorama dosyaları 1x1 yer tutucu; görsel kontrol `named` kopyalarıyla yapıldı.
- Açık: bulanıklık [T85c](T85c-menu-bulaniklik.md)'e bölündü.
