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
boşluk, sol shift, sol ctrl ve fare. Kamera oyuncunun gözünde; çizilen kutu
oyuncu kenarına yaklaşınca yeniden kuruluyor.

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
  yaklaşıklanıyor, su rengi biyom
  yerine varsayılan biyomun rengi, yarı saydam yüzler sıralanmıyor, cam ve
  yaprak yanındaki su kaplama dokusu yok, ışık yok.
- Animasyonlu dokular (su, deniz otu, yosun, lav) atlasa ilk kareleriyle
  giriyor; referansın kare boyutu kuralı uygulanıyor ama animasyon
  oynatılmıyor.
- Gökyüzü referansın veri yolundan hesaplanıyor: gök rengi, sis rengi,
  güneş/ay/yıldız açıları, yıldız parlaklığı, ay evresi ve gün doğumu rengi
  yerel vanilla veri paketindeki boyut, biyom ve timeline dosyalarından,
  varsayılan → boyut → kameranın etrafında Gauss ağırlıklı biyom →
  timeline sırasıyla değerlendiriliyor. Saat `set_time` paketinin dünya
  saatlerinden alınıp tick'lerle ilerliyor; değerler tick başına hesaplanıp
  kareler arasında aradeğerleniyor. Çizim: sisle ufka karışan gök diski,
  gün doğumu yelpazesi, güneş ve ay evresi (en yakın texel), tohumlu 1500
  yıldız, ufuk altı koyu disk; ekran sis rengine temizleniyor. Farklar:
  yağmur ve gök gürültüsü karartması, bulutlar, arazinin sisi yok, su altı
  görüş süresi sabit, dünya sınırı ve End gökyüzü çizilmiyor; referans
  karelerle karşılaştırılmadı.
- Biyom renkleri (çim, yaprak, kuru yaprak, su) referansın tint yoluyla:
  biyomun sıcaklık/yağıştan colormap rengi ya da kendi rengi, koyu orman ve
  bataklık değiştiricileri (bataklık için tohumu 2345 olan simplex gürültüsü),
  biyom yöneticisinin hash'lenmiş tohumla bulanık yakınlaştırması ve 5×5
  sütun ortalaması (varsayılan karışım yarıçapı 2). Uzun çimin üst yarısı
  alttaki bloğun rengini, nilüfer dünya sabitini alıyor. Farklar: kızıltaş
  ve sap gibi duruma bağlı tonlar varsayılan durumun sabiti; referans
  karelerle karşılaştırılmadı.
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
  sıralanıyor. Farklar: blok ışığı titremesi, gece görüşü, karanlık etkisi
  ve patron sisi yok; ışık paketleri gelince mesh yeniden kurulmuyor;
  referans karelerle karşılaştırılmadı.
- Kare zamanlaması referansın zamanlayıcısı değil; tick'ler 50 ms birikimle
  sürülüyor.
