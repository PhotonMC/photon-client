# T33c — Parlayan göz katmanı (örümcek, mağara örümceği, enderman) — [client-parity]

Bu kayıt, `EyesLayer`'ın üç tüketicisinin (`SpiderEyesLayer`, `EnderEyesLayer`) ne
gönderdiğinin ve gönderdiği render türünün hattının (`RenderPipelines.EYES`) Java 26.2 ile
Rust'ta aynı olmasını kapsar. Kanıt sınıfı `client-parity`'dir (katmanın gönderdiği
değerler ve hat durumu) ve canlı gözlemdir (çizim): gerçek katman sınıfları, yakalayıcı
bir `SubmitNodeCollector` ile çağrılır; çizim sırası, doku, ışık, örtü ve render türünün hattı
(konum, shader anahtarları, harman, derinlik, yüz eleme) okunup karşılaştırılır. Modelin
kendisi aynı poz ve aynı köşelerdir (`spider_model`, `enderman_model` kayıtları).

## Java kaynak incelemesi

- `EyesLayer.submit`: `order(1)` ile (sıradan katmanlardan sonra) üst modeli aynı pozda, kendi
  render türüyle, katmanın aldığı ışıkla ve `NO_OVERLAY` ile gönderir; görünmezlik denetlemez
  (görünmez örümceğin gözleri görünür).
- `RenderPipelines.EYES`: entity shader'ı `EMISSIVE` (ışık haritası yok), `NO_OVERLAY`,
  `NO_CARDINAL_LIGHTING` (yüz başına gölge yok) ile; `TRANSLUCENT` harman; derinlik testi
  büyük-eşit, yazma yok; yüz eleme açık. Alfa kesme yok.
- Dokular: `entity/spider/spider_eyes` (örümcek ve mağara örümceği), `entity/enderman/enderman_eyes`.

## Rust

`photon-render/src/render_types.rs` hattın ve katmanların olgularını (`EYES`, `EYES_ORDER`,
`NO_OVERLAY`, `eyes_texture`) taşır; oyun dokuyu buradan alır. `mob_vertices_glow` modeli
ışıksız ve örtüsüz, tint'i beyaz üretir; Vulkan'da gölge hattıyla aynı shader'lar ve durum
(doku çarpı renk, ışık yok, kesme yok, harman, derinlik testli yazmasız) `glow` tamponuyla
varlıklardan sonra çizilir.

## Kapsam

`entity/eyes-layers` 2 satır (örümcek, enderman).

## Sapmalar (açık)

- Hat yüz elemesi: referans arka yüzleri eler; Rust varlık ve gölge hatları eleme yapmaz.
  Göz dokusu yalnız ön yüz bölgesinde piksel taşıdığından ve arka yüzler derinlik testinde
  elendiğinden gözlenen fark yok.
- `ColorModulator` ve sis ortam renkleri kare başına varsayılan değerlerle okunur (diğer
  varlıklar gibi).
- Şarjlı creeper girdabı (`EnergySwirlLayer`) T33d'de.
