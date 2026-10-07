# T34d — Varlık üstünde blok durumu çizimi — [client-parity]

Bu kayıt, bir blok durumunu varlığın üstünde blok modeli olarak çizen üç katmanın
(`CarriedBlockLayer`: endermanın taşıdığı blok, `SnowGolemHeadLayer`: kar golemin
balkabağı, `IronGolemFlowerLayer`: demir golemin gelinciği) Java 26.2 ile Rust'ta
aynı dönüşümü üretmesini kapsar. Kanıt sınıfı `client-parity`'dir (dönüşümler) ve canlı
gözlemdir (çizilen model): gerçek katman sınıfları, gerçek poz yığınında, bir blok
modeli durumu gönderdiğinde yakalayıcı bir `SubmitNodeCollector` ile okunur; poz yığınının
matrisi bit bit karşılaştırılır. Blok modelinin köşeleri (`BlockStateModelPart` dörtgenleri)
bu kayıtta uçtan uca karşılaştırılmaz: model ayrıştırma, yüz geometrisi, dönüş ve UV
parçaları kendi kayıtlarında `client-parity`'dir; birleşimi canlı gözlemlendi.

## Java kaynak incelemesi

- `BlockModelResolver.update`: durum için `BlockModelSet`'in modeli (varsayılan, durumun
  blok durumu modeli ve renk kaynakları) tohum 42 ile parçaları toplar; `blockLightCoords`
  durum ışımalı çiziliyorsa tam parlak, değilse durumun yaydığı ışıktır.
- `BlockModelFeatureRenderer`: her parçanın her yönündeki ve yönsüz dörtgenleri, hiçbirini
  elemeden; renk, dörtgenin renk dizinine göre durumun sabit renkleriyle çarpılır (bölge
  rengi dünya olmadan sabit değere düşer).
- Katman dönüşümleri: taşınan blok gövdenin önünde eğik ve dönük yarım boy; balkabağı
  başın pozunun üstünde ters çevrilmiş sekizde beş boy; çiçek sağ kolun pozunda yarım
  boy ve dik. Katmanlar bloğu köşesinden çizer (`translate(-0.5, -0.5, -0.5)` katmanın
  içindedir).
- Işık: bloğun ışığı varlığın ışığıyla blok ve gök için ayrı ayrı en büyük alınır
  (`LightCoordsUtil.max`).

## Veri

`EnderMan.DATA_CARRY_STATE` (16, isteğe bağlı blok durumu, sıfır boş; değer durum kimliğidir),
`SnowGolem.DATA_PUMPKIN_ID` (16, bayt, on altı balkabağı; varsayılan balkabağı var) veri
indeksleri `entity/variant-fields` satırlarıyla her koşuda karşılaştırılır.

## Kapsam

`entity/block-layers` 8 satır (taşınan blok, balkabağı üç bakış, çiçek sunma/geç sunma/saldırıda/
yürürken).

## Sapmalar (açık)

- Çiçek canlı gözlenmedi: sunucu çiçek sunmayı yalnız köylü çocukla etkileşimde başlatır;
  dönüşümü oracle'da, çizim yolu balkabağı ve taşınan blokla aynıdır.
- Özel blok modelleri (`BlockModelSet`'in durum başına özel girdileri: sandık, yatak…)
  çizilmez; taşınan blok bunlardan biriyse çizim atlanır.
- Balkabağı, görünmez ve parlayan golemde yalnız dış hat olarak çizilir (referans); dış hat
  yok, bu durumda balkabağı çizilmez.
- Endermanın titremesi (`getRenderOffset`) tohumsuz rastgele kaynakla çizilir
  (referans da tohumlamaz); bu yüzden oracle'a alınamaz.
- Eşya olarak çizilen bloklar (T25, kafadaki balkabağı) ayrı yoldur.
