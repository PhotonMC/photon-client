# T30b — Keçi ve mantar ineği — [client-parity]

Bu kayıt, keçinin (yetişkin ve bebek; boynuzlar, ram için eğilen baş) ve mantar ineğinin
(yetişkin ve bebek, kırmızı ve kahverengi; sırtındaki ve başındaki üç mantar) Java 26.2 ile
Rust'ta aynı köşe konum, doku koordinatı ve normali, aynı mantar dönüşümleri ve aynı keçi
zamanlayıcısıyla çizilmesini kapsar. Kanıt sınıfı `client-parity`'dir (model, dönüşümler,
zamanlayıcı) ve canlı gözlemdir (çizilen birleşim): gerçek `GoatModel`, `BabyGoatModel` ve
`CowModel` katman tablosundan pişirilip gerçek `setupAnim` ile pozlanır; mantar katmanı
gerçek `MushroomCowMushroomLayer` ile yakalayıcı toplayıcıda okunur; keçinin eğilme zamanlayıcısı
gerçek `Goat` varlığının tikiyle karşılaştırılır.

## Java kaynak incelemesi

- `GoatModel.setupAnim`: dört ayaklı poz; sol ve sağ boynuz verideki bayrağa göre görünür;
  `rammingXHeadRot` sıfırdan farklıysa başın eğimi odur. Bebek model (`BabyGoatModel`): ram
  yokken baş sekizde bir pi öne eğik.
- `Goat`: olay 58 başı indirmeyi başlatır, 59 durdurur; her tik `lowerHeadTick` inerken bir
  artar, yoksa iki azalır, 0 ile 20 arasında; `getRammingXHeadRot` tik / 20 * (bebekte 52,5,
  yetişkinde 30 derece).
- `MushroomCow`: `DATA_TYPE` (int, kırmızı 0, kahverengi 1; dışı son değere sıkışır) doku ve
  mantarı seçer. Mantar katmanı yalnız yetişkinde, görünürken: iki mantar sırtta, biri başın
  pozunda; blok durumu `RED_MUSHROOM`/`BROWN_MUSHROOM`.
- Mantar ineğinin modeli inekle aynı sınıftır (`CowModel`, bebekte `BabyCowModel`); bu yüzden
  dörtlü model parametrik test satırıdır.

## Veri

`Goat.DATA_HAS_LEFT_HORN` (19), `DATA_HAS_RIGHT_HORN` (20) (boolean, varsayılan var) ve
`MushroomCow.DATA_TYPE` (18, int) indeksleri `entity/variant-fields` satırlarıyla her koşuda
karşılaştırılır.

## Kapsam

`entity/goat-model` 14 satır (iki katman × yedi poz), `quadruped-model`'a mantar ineği için 14
satır, `entity/block-layers`'a iki mantar satırı (üçer poz), `living-entity-animations`'a üç keçi
satırı.

## Sapmalar (açık)

- Tavşan yapılmadı: model `KeyframeAnimation` ister (T43b); T30c'ye taşındı.
- Mantar ineğinin mantarları görünmez ve parlayan hayvanda dış hat olarak çizilir; dış hat yok.
- Keçi sesleri, çarpma etkisi, boynuz kırma/çıkarma etkileşimi yok.
