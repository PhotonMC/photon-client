# HUD

Kanıt sınıfı: `client-parity` (artı işareti, saldırı göstergesi, kısayol
çubuğu, zırh, kalpler, yiyecek, hava baloncukları ve deneyim çubuğu gerçek
`Hud`'ın çizdiği karelerle karşılaştırıldı (`hud_hotbar`, `hud_status`,
[GUI oracle](../gui-oracle.md)); araç canı, atlama çubuğu, etki simgeleri ve
seçili eşya adının yerleşimi referansın `Hud` sınıfından taşındı veya henüz
yok ve karelerle karşılaştırılmadı).

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
- **Saldırı göstergesi:** `attackIndicator` seçeneğine göre (Video
  ayarlarından değişir, `options.txt`'e yazılır) artı işaretinin altında,
  kısayol çubuğunun yanında ya da hiç çizilmiyor. Saldırı gücü dolmamışken
  ilerleme çiziliyor; dolmuşken artı işareti canlı ve ölmemiş bir varlığın
  üzerindeyse ve saldırı gecikmesi 5 tikten uzunsa (yavaş silah) "tam"
  gösterge çıkıyor. Oracle: saldırı gücü 0'dan doluya, iki saldırı hızı, dört
  hedef türü, izleyici, üçüncü şahıs ve pencere boyutları. Açık: eşyanın
  kendi saldırı menzili (mızrak; hedef menzil dışındaysa tam gösterge
  çıkmaz), sol elini kullanan oyuncu (kısayol çubuğundaki gösterge ve
  yan el simgesi karşı tarafta), izleyicinin menülü bir bloğa/varlığa
  baktığında artı işareti, üç boyutlu artı işareti hata ayıklama girdisi,
  yan elde eşya.

`hud_status` oracle'ı (150 satır): gerçek `LocalPlayer` ile can (yarım kalpten kırk kalbe), emilim
(en çok emilim özniteliğiyle), zırh, yiyecek ve doygunluk (titreme), hava (suyun altında ve dışında,
patlayan ve boşalan baloncuklar), zehir/solgunluk/donma/yenilenme/açlık, zorlu mod, deneyim seviyesi ve
ilerlemesi, üç pencere boyutu ve kareler boyunca can düşüşü/artışı (yanıp sönme, gösterilen canın
yetişmesi, saat ve HUD tik sayısıyla); sprite adı ve yerleri bit bit eşit. Açık: araç canı, atlama ve konum
çubuğu bu oracle'da yok (konum çubuğunun kendi oracle'ı var), eşya adı ve etki simgeleri yok.

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

Deneyim seviyesi, sıfırdan büyükse çubuğun üstünde ortalanıyor: dört
siyah kopya bir birim kaydırılmış, üstünde yeşil (`0xFF80FF20`).

Araç canı, bağlamsal çubuktaki konum ve atlama çubukları, etkilerin sağ
üst simgeleri ve kısayol çubuğundaki eşyalar henüz yok. Baloncuk sesi
çalmıyor.

## Yazı tipi

`crates/photon-render/src/gui/font.rs`, varsayılan yazı tipini referansın tanımından
kuruyor:

- `font/default.json` ve başvurduğu tanımlar sırayla okunuyor.
  Seçeneklerin varsayılanıyla (tek tip değil, Japonca varyant kapalı)
  süzgeçten geçmeyen sağlayıcılar atlanıyor.
- Bir kod noktasını ilk sağlayan sağlayıcı kullanılıyor.
- Boşluk sağlayıcısı yalnız ilerleme veriyor.
- Bitmap sağlayıcısı resmi `chars` ızgarasına bölüyor (satırlar Java'daki
  gibi kod noktalarına ayrılıyor). Her glifin genişliği sağdan ilk alfalı
  sütuna kadar; ilerlemesi yüksekliğe göre ölçeklenmiş genişliğin
  yuvarlanması artı bir.
- Glif dörtgeni sol kenardan başlıyor; üstü `7 − ascent`, boyutu hücre
  × ölçek.
- Metin genişliği ilerlemelerin toplamının yukarı yuvarlanması.

Unifont yedeği okunmuyor; yalnız onda olan kod noktaları çizilmiyor. Gölge,
kalın ve eğik yazı henüz yok.

## Yuva seçimi

`crates/photon-game/src/input/hotbar.rs`:

- 1–9 tuşları yuvayı doğrudan seçiyor.
- Tekerlek referansın işleyicisiyle ilerliyor: kesirli adımlar
  biriktiriliyor, yön değişince birikim sıfırlanıyor, yukarı sola gidiyor,
  uçlarda başa sarıyor.
- İzleyici yuva seçmiyor. Tekerlekle uçuş hızı ayarı henüz yok.
- Seçilen yuva sonraki tikte oyuncudan önce sunucuya bildiriliyor.
