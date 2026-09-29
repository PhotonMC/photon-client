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

## Can, zırh, yiyecek, hava

`Hud.extractPlayerHealth` taşındı:

- **Kalp türü:** zehir, sonra solgunluk, sonra tamamen donmuşluk, yoksa
  normal.
- **Yanıp sönme:** Can düşünce ve oyuncu dokunulmazken 20 tik, can
  artınca 10 tik boyunca her üç tikte bir yanıp sönüyor. Eski can bir
  saniye gösterilmeye devam ediyor.
- **Emilim:** Kalpleri canın arkasından ayrı satırlarda.
- **Satır aralığı:** Satır sayısı arttıkça daralıyor (en az 3).
- **Titreme:** Can ve emilim toplamı 4 ya da altındaysa kalpler rastgele
  titriyor. Tohum HUD'un tik sayısı × 312871, bir `int` çarpımı.
- **Yenilenme:** Yenilenme etkisinde bir kalp sırayla iki birim zıplıyor.
- **Zorlu mod:** Kalpler zorlu mod sprite'larıyla çiziliyor.
- **Zırh:** Değeri (özelliğin tabanı) kalplerin üstünde.
- **Yiyecek:** Doygunluk sıfırsa titriyor, açlık etkisinde açlık
  sprite'larıyla çiziliyor.
- **Hava baloncukları:** Suyun altındayken ya da hava eksikken, patlayan
  baloncuk ve boş baloncuklarla.
- **Artı işaretinin altında saldırı göstergesi:** Saldırı gücü dolmamışken
  çiziliyor (varsayılan seçenek). Canlı hedeflenmediği için "tam"
  göstergesi çıkmıyor.

Veriler:

- Can istemcide `LocalPlayer.hurtTo` ile tutuluyor. Can düşünce 20, artınca
  10 tik dokunulmazlık ve incinme sayacı başlıyor. İlk güncelleme yalnızca
  değeri koyuyor.
- En yüksek can, zırh ve saldırı hızı sunucunun özellik paketlerinden
  geliyor. Kayıt kimlikleri 23, 1 ve 5; değer referansın işlem sırasıyla
  ve özelliğin aralığında hesaplanıyor.
- Etkiler (yenilenme 9, açlık 16, zehir 18, solgunluk 19) etki
  paketlerinden geliyor.
- Emilim, hava ve donma varlık verisinden geliyor. Oyuncu emilimi 17,
  skin parçaları 16. indeks.

Araç canı, bağlamsal çubuktaki konum ve atlama çubukları, etkilerin sağ
üst simgeleri, deneyim seviyesi yazısı (yazı tipi) ve kısayol çubuğundaki
eşyalar henüz yok. Baloncuk sesi çalmıyor.

## Yuva seçimi

`src/hotbar.rs`:

- 1–9 tuşları yuvayı doğrudan seçiyor.
- Tekerlek referansın işleyicisiyle ilerliyor: kesirli adımlar
  biriktiriliyor, yön değişince birikim sıfırlanıyor, yukarı sola gidiyor,
  uçlarda başa sarıyor.
- İzleyici yuva seçmiyor. Tekerlekle uçuş hızı ayarı henüz yok.
- Seçilen yuva sonraki tikte oyuncudan önce sunucuya bildiriliyor.
