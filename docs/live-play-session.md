# P11.e — Canlı oyun oturumu — server-conformance kaydı

**Tarih:** 2026-09-27 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** `server-conformance`

Bu kayıt istemci eşitliği göstermez. Gösterdiği şey, [pinli LocalPlayer
tick'inin](local-player-tick-conformance.md) canlı bir vanilla sunucuda
sürüldüğünde sunucunun gönderilen hareketleri kabul etmesi.

## Ne yapıldı

`src/connection/play.rs` kalıcı bir oyun oturumu: giriş ve konfigürasyon
probe'larla ortak (`open_play_connection`), paketleri bir okuyucu iş parçacığı
alıyor, çağıran kareler arasında boşaltıyor. Yığın, yığın unutma, blok ve
bölüm blok güncellemeleri, yığın önbelleği merkezi/yarıçapı, yığın grubu
onayı, keep-alive, ping, oyun modu olayı, yetenekler, respawn ve teleport
işleniyor. Seviye yüklenince (oyuncunun konumu ve altındaki yığın gelince)
`player_loaded` gönderiliyor ve oyuncu tick'lenmeye başlıyor. Her istemci
tick'i oyuncuyu çalıştırıp istediği paketleri, ardından tick sonu paketini
gönderiyor.

`voxel-client play <sunucu> [genişlik yükseklik]` bir pencere açıyor: WASD,
boşluk, sol shift, sol ctrl ve fare. Kamera oyuncunun gözünde. Bütün blok
modelleri ve tek atlas açılışta kuruluyor; dünya 16³'lük bölümler halinde
mesh'leniyor. Gelen yığın ve ışık paketleri çevredeki 3×3 yığın sütununun,
blok güncellemeleri çevredeki 3×3×3 bölümün mesh'ini kirletiyor; her kare
görüş mesafesindeki kirli bölümlerin oyuncuya en yakınları bir süre
bütçesiyle yeniden kuruluyor, unutulan yığınların bölümleri siliniyor.
Yarı saydam yüzler bölüm merkezinin kameraya uzaklığına göre uzaktan yakına
çiziliyor.

## Ölçülen sonuç

Hayatta kalma modunda, üretilmiş normal arazide (tohum 776), betikli
girdiyle (`VOXEL_SCRIPT`) yürüme, sprint, zıplama, göle girip çıkma ve eğilme
içeren 166 tick: sunucu girişteki bir yerleştirme dışında hiç teleport
göndermedi ve günlüğünde "moved wrongly" ya da "moved too quickly" yok.

## Koşma

```
cd .data/render/walk && java -jar ../../minecraft/runtime/server-26.2.jar nogui
VOXEL_ASSETS=.data/minecraft/assets/jar/assets \
  target/release/voxel-client play 127.0.0.1:25600
```

Sunucu klasörü yerel; `server.properties` çevrimdışı mod, hayatta kalma,
barışçıl ve `server-port=25600` içeriyor.

## Eşitlik olmayanlar

- Kamera: göz yüksekliği ve görüş açısı çarpanı referanstaki gibi her tick
  kalan farkın yarısı kadar yaklaşıyor ve kareler arasında aradeğerleniyor
  (eğilince bakışın birkaç tick'te inmesi, koşarken görüşün genişlemesi).
  Kamera tick'inin sırası, bakış sallanması ve suya giren kameranın sıvı
  kontrolü referansla karşılaştırılmadı (P14).
- Fare hassasiyeti varsayılan değerin ölçeği; yumuşatma ve ters eksen yok.
- Çizim [dünya çizim kaydındaki](server-world-render.md) eksikleri taşıyor.
  Yüz gizleme referansın kararı: her durumun yüz örtme şekilleri, şekil
  örtüşme tablosu ve atlanan blok çiftleri referanstan yakalanıp
  `src/render/face_occlusion_table.rs` olarak üretiliyor
  (`tools/generate/face_occlusion.py`); block ailesi tabloyu her koşuda
  karşılaştırıyor. Yapraklar örtmediği için içlerindeki kütükler görünüyor.
- Su ve lav referansın sıvı çizicisinin mantığıyla çiziliyor: köşe
  yükseklikleri, akış yönüne dönen akış dokusu, aynı sıvıya bakan yüzün
  atılması, arkadan görünen yüzey. Farklar: kısmi örtme şekli olan komşu
  sıvı yüzünü gizlemiyor sayılıyor, katılık çarpışma şekline göre
  yaklaşıklanıyor. Su rengi biyomdan, ışık sıvı çizicisinin kuralıyla.
  Referansın yarı saydam blokları ve yaprakları (`HalfTransparentBlock`,
  `LeavesBlock`; `src/render/fluid_overlay_table.rs`,
  `tools/generate/fluid_overlay.py`, block ailesinde karşılaştırılıyor)
  yanındaki su yüzleri `water_overlay` dokusuyla ve yalnız önden çiziliyor.
- Animasyonlu dokular referansın sprite animasyonuyla oynuyor: kare
  listesi (listesizse şeridin bütün kareleri), kare süreleri (`frametime`
  ve kareye özel `time`, geçersiz kareler atılıyor), her istemci tick'inde
  ilerleyen kare/alt kare sayacı; kare değişince, `interpolate` olanlarda
  her tick sonraki kareye binde bir adımlı ilerlemeyle karıştırılarak
  atlasa yazılıyor. Her karenin mip seviyeleri şeridin tamamından üretilen
  seviyelerden kesiliyor. Farklar: atlas GPU'da çizimle değil CPU kopyası ve
  bölge yüklemesiyle güncelleniyor; bir karede birden çok tick geçerse
  yalnız son durum yazılıyor; referans karelerle karşılaştırılmadı.
- Gökyüzü referansın veri yolundan hesaplanıyor: gök rengi, sis rengi,
  güneş/ay/yıldız açıları, yıldız parlaklığı, ay evresi ve gün doğumu rengi
  yerel vanilla veri paketindeki boyut, biyom ve timeline dosyalarından,
  varsayılan → boyut → kameranın etrafında Gauss ağırlıklı biyom →
  timeline sırasıyla değerlendiriliyor. Saat `set_time` paketinin dünya
  saatlerinden alınıp tick'lerle ilerliyor; değerler tick başına hesaplanıp
  kareler arasında aradeğerleniyor. Çizim: sisle ufka karışan gök diski,
  gün doğumu yelpazesi, güneş ve ay evresi (en yakın texel), tohumlu 1500
  yıldız, ufuk altı koyu disk; ekran sis rengine temizleniyor. Farklar:
  su altı
  görüş süresi sıfır sayılıyor, dünya sınırı ve End gökyüzü çizilmiyor; referans
  karelerle karşılaştırılmadı.
- Hava: yağmur ve gök gürültüsü seviyeleri oyun olaylarından (başla/dur,
  seviye değişimi; gök gürültüsü yağmurla ölçekli). Referansın hava
  katmanı timeline'lardan sonra, yalnız hava alabilen boyutlarda: gök,
  sis, bulut ve gün doğumu renkleri, gök ışığı rengi ve çarpanı, yıldızlar
  yağmur/gök gürültüsü değiştiricileriyle, yağmurun gök gürültüsü dışında
  kalan payı ve gök gürültüsü oranında karıştırılıyor. Sisin karıştığı gök
  rengi yağmurla koyulaşıyor, güneş ve ay yağmurla soluyor; yağmur sisi
  kameranın bloğundaki gök ışığı ve biyomun yağışına göre tick başına
  beşte bir yaklaşıp sis başlangıç ve bitişini çekiyor. Yağmur ve kar
  çizgileri referansın hava efekti çizicisiyle: 10 blokluk yarıçapta her
  sütun, MOTION_BLOCKING yüksekliği (durum başına bit tablosu referanstan:
  `src/world/motion_blocking_table.rs`, `tools/generate/motion_blocking.py`,
  block ailesinde karşılaştırılıyor) ile kameranın ±10 bloğu arasında
  merkeze dönük bir dörtgen; yağış biyomun yağışı ve sıcaklığından (donmuş
  değiştiricinin gürültüleri, kar çizgisi üstünde sıcaklık gürültüsüyle
  soğuma; 0,15 altı kar), sütun tohumlu hız ve kayma, kenara doğru ve
  yağmurla solan alfa, sütunun ışığı (karda aydınlatılmış), yakın texel ve
  tekrarlayan örnekleme, derinlik yazmadan, arazi sisiyle. Farklar:
  sıçrama parçacıkları ve sesler, şimşek ve gök parlaması yok; kar görsel
  olarak denenmedi; referans karelerle karşılaştırılmadı.
- Arazi sisi referansın sis çizicisinin hesabı: çevresel sis (atmosfer
  için `fog_start_distance`/`fog_end_distance`, su altında
  `water_fog_start_distance` ve görüş süresiyle çarpılan
  `water_fog_end_distance`) kameraya küresel uzaklıkla, görüş mesafesi
  sisi (son `clamp(blok/10, 4, 64)` blok) silindirik uzaklıkla ölçülüyor;
  kalını alınıp sis rengine karıştırılıyor. Farklar: körlük,
  karanlık, lav ve toz kar sisleri ve bölümlerin belirme geçişi yok (yağmur
  sisi aşağıda); referans
  karelerle karşılaştırılmadı.
- Bulutlar referansın bulut çizicisinin düzeniyle: `clouds.png`'nin her
  saydam olmayan pikseli 12×12×4 bloklük bir hücre; ızgara oyun zamanıyla
  tick başına 0,03 blok kayıyor, z'de 3,96 blok kaydırılmış ve sarılıyor.
  Hücreler kameranın hücresinden halkalar halinde, varsayılan bulut
  mesafesi 128 yığın boyunca diziliyor (süslü stil): kamera katmanın altında
  değilse üst yüz, üstünde değilse alt yüz, kenar yüzleri yalnız komşu boşsa
  ve kameraya bakıyorsa, kameranın çevresindeki 3×3 hücrede içe bakan
  yüzler. Yüz parlaklıkları, renk (`cloud_color`, gece koyulaşan
  timeline), yükseklik (`cloud_height`) ve küresel uzaklıkla saydamlaşan
  bulut sisi (`cloud_fog_end_distance`, en çok bulut mesafesi) referanstaki
  gibi. Uzak düzlem referanstaki gibi görüş mesafesinin dört katı ya da
  bulut mesafesi; derinlik referans gibi ters (yakın 1, uzak 0, büyük-eşit
  testi). Farklar: bulutlar ayrı hedefe çizilmiyor (fabulous yok), düz
  bulut stili seçilemiyor; referans karelerle karşılaştırılmadı.
- Biyom renkleri (çim, yaprak, kuru yaprak, su) referansın tint yoluyla:
  biyomun sıcaklık/yağıştan colormap rengi ya da kendi rengi, koyu orman ve
  bataklık değiştiricileri (bataklık için tohumu 2345 olan simplex gürültüsü),
  biyom yöneticisinin hash'lenmiş tohumla bulanık yakınlaştırması ve 5×5
  sütun ortalaması (varsayılan karışım yarıçapı 2). Uzun çimin üst yarısı
  alttaki bloğun rengini, nilüfer dünya sabitini alıyor. Duruma bağlı
  tonlar referansın formülleriyle: kızıltaş tozu gücüne, kavun ve balkabağı
  sapı yaşına göre (varsayılan durumlar referans tablosuyla testte
  karşılaştırılıyor). Referans karelerle karşılaştırılmadı.
- Model seçimi referansın yolu: blok başına tohum (`Mth.getSeed`; çift
  bitki ve kapı alt yarının, yatak başın sütununun tohumu), her ağırlıklı
  listeden sırayla bir çekiliş (tek başına yazılmış varyant çekiliş
  yapmıyor), çok parçalı blokların bütün parçaları. Bitkilerin konuma bağlı
  kaydırması referanstan tablo (`src/render/block_offset_table.rs`,
  `tools/generate/block_offset.py`) ve formülle; block ailesinde
  karşılaştırılıyor.
- Işık referansın model aydınlatıcısıyla hesaplanıyor: yumuşak ışıkta her
  köşe çevre blokların ışığını ve AO gölgesini karıştırıyor (kısmi yüzler
  ağırlıklı), ışık yayan bloklar ve AO kapalı modeller düz ışıkla; renk AO
  grisi → yan gölge → ton sırasıyla 8 bit adımlarla. Durum özellikleri
  (ışık yayma, AO gölgesi, görüş engelleme, sönümleme, katı çizim, tam kutu)
  referanstan tablo (`src/world/block_light_table.rs`,
  `tools/generate/block_light.py`), block ailesinde karşılaştırılıyor. Işık
  seviyeleri sunucunun yığın ve ışık paketlerinden; lightmap referansın
  lightmap shader'ının hesabıyla, 8 bit nicemleme ve doğrusal örneklemeyle
  vertex shader'da. Dörtgenler FaceBakery gibi son yönlerine göre yeniden
  sıralanıyor. Blok ışığı çarpanı referanstaki gibi 1,4 artı her tick
  rastgele adımla kayıp 0,9 ile sönen titreme (kaynak referanstaki gibi her
  çalışmada farklı tohumla); lightmap girdileri tick başına bir kez, tick
  sonundaki öznitelik değerleriyle. Farklar: gece görüşü, karanlık etkisi
  ve patron sisi yok; referans karelerle karşılaştırılmadı.
- Bölüm mesh'leme referansın bölüm derleyicisinin sırası ve iş
  parçacıkları değil: kirli bölümler ana iş parçacığında, uzaklık sırasıyla,
  kare başına 12 ms bütçeyle kuruluyor; görüş mesafesi 2–12 bölüme
  kırpılıyor; bölüm içindeki yarı saydam yüzler sıralanmıyor; görünürlük (frustum ve
  mağara) ayıklaması yok. Derinlik testi referanstaki gibi eşitliği
  geçiriyor; aynı yerdeki sonraki yüz (çimin yan örtüsü) görünüyor.
- Kare zamanlaması referansın zamanlayıcısı değil; tick'ler 50 ms birikimle
  sürülüyor.
