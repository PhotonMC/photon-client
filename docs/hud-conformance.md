# HUD

Kanıt sınıfı: `client-parity` (yerleşim referansın `Hud` sınıfından
taşındı; çizilen karelerle karşılaştırılmadı).

## Çizim

GUI, pencerenin piksellerinin GUI ölçeğine bölümüyle bulunan GUI
birimlerinde kuruluyor. Ölçek varsayılan otomatik seçimle bulunuyor: GUI'yi
en az 320×240 tutan en büyük tam sayı (`WindowScale`). Sprite'lar
referansın `gui/sprites` klasöründen tek bir atlasa paketleniyor ve texel
texel örnekleniyor.

Referansın dokulu GUI shader'ında olduğu gibi texel köşe rengiyle
çarpılıyor, tamamen saydam olan atılıyor. Normal öğeler saydam karışımla,
artı işareti arkasını ters çeviren karışımla (`1 - hedef` ve `1 - kaynak`)
çiziliyor. GUI karenin en sonunda, derinlik testi olmadan çiziliyor.

## Öğeler

- **Artı işareti:** Yalnız birinci şahısta ve izleyici değilken çiziliyor.
- **Kısayol çubuğu ve seçili yuva:** İzleyici değilken çiziliyor.
- **Hayatta kalma ve macera kipinde:** kalpler, zırh, yiyecek ve deneyim
  çubuğu.

## Yuva seçimi

`src/hotbar.rs`:

- 1–9 tuşları yuvayı doğrudan seçiyor.
- Tekerlek referansın işleyicisiyle ilerliyor: kesirli adımlar
  biriktiriliyor, yön değişince birikim sıfırlanıyor, yukarı sola gidiyor,
  uçlarda başa sarıyor.
- İzleyici yuva seçmiyor. Tekerlekle uçuş hızı ayarı henüz yok.
- Seçilen yuva sonraki tikte oyuncudan önce sunucuya bildiriliyor.
