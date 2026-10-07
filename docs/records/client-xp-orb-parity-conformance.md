# T25b — Deneyim küresi çizimi — [client-parity]

Bu kayıt, sunucunun eklediği `ExperienceOrb`'un çizilen dörtgeninin
(`ExperienceOrbRenderer.submit`) Java 26.2 ile Rust'ta aynı köşe konumu, renk,
doku koordinatı ve normali üretmesini kapsar. Kanıt sınıfı `client-parity`'dir:
Java'nın gerçek çizicisi sahte bir toplayıcıyla koşturulur ve yazdığı her köşe
bit bit karşılaştırılır. Canlı kabul (izleyici kipinde beş değerli küre görünür)
`server-conformance` gözlemidir; yarı saydam karışım ve ışık haritası oracle'da
yok.

## Java kaynak incelemesi

- `client/renderer/entity/ExperienceOrbRenderer.java:34-72`: dörtgen kameraya
  dönüktür (`camera.orientation`), 0.3 ölçekli, 0.1 yukarıda; ikonun 16×16'lık
  hücresi 64×64 dokuda (`icon % 4`, `icon / 4`); renk yaşa bağlı sinüs
  (`ageInTicks / 2`: kırmızı `(sin + 1) * 0.5`, mavi `(sin(+4π/3) + 1) * 0.1`,
  yeşil 255, alfa 128); köşe normali `(0,1,0)`'ın dönüşmüş hâli, ışığı bununla
  ışıklandırılır (`ENTITY_TRANSLUCENT_CULL` yüz başına ışıklandırma tanımlamaz).
  Blok ışığı bulunduğu bloğunkinden yedi fazladır (`getBlockLightLevel`).
- `world/entity/ExperienceOrb.java:31,320-345`: değer `DATA_VALUE` (int) verisinde
  (veri indeksi 8, oracle ile doğrulandı), ikon `getIcon` eşikleriyle seçilir.

## Kapsam ve dal kararı

`entity/xp-orb` 25 satır: on bir ikon, sekiz yaş (sinüs eğrisi), altı bakış
yönü (dönmüş kameralar), üç konum ve dört ışık değeri. `entity/variant-fields`'e
küre değerinin veri indeksi eklendi. Eşik tablosu (`icon`) birim testle sınanır
(oracle gerçek `ExperienceOrb` kurmaz).

## Sapmalar (açık)

- Çizim `ENTITY_TRANSLUCENT_CULL`'un karşılığıdır: yarı saydam karışım, derinlik
  yazmadan; ışık haritası ve yüz ışığı köşe renginde, CPU'da hesaplanır.
  Dokunun alfa kesimi Java'da çarpımdan önce, burada sonra (küre dokusunda
  yalnız 0 ve 255 alfa var).
- Işık, kürenin göz yüksekliğindeki bloktan tek örnek.
- Toplama animasyonu ve birleşme kapsam dışı; sunucu kürenin konumunu gönderir,
  istemci yalnız `tick_experience_orb` ile aralarda ilerletir.
