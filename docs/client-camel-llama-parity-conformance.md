# T31b — Deve, deve kabuğu, lama ve tüccar lamasi — [client-parity]

Bu kayıt, devenin (yetişkin ve bebek; baş sınırları, yürüyüş, oturma/ayağa kalkma/boşta/koşu
anahtar kareleri), deve kabuğunun ve lama ile tüccar lamasının (yetişkin ve bebek, sandık) modellerinin
Java 26.2 ile Rust'ta aynı köşe konum, doku koordinatı ve normali üretmesini; devenin kendi istemci
tikinin (pozdan animasyon durumları, koşunun bitişi, yürüyüş hızı) aynı ilerlemesini kapsar. Kanıt
sınıfı `client-parity`'dir (model ve tik) ve canlı gözlemdir: gerçek `AdultCamelModel`,
`BabyCamelModel`, `LlamaModel` ve `BabyLlamaModel` katman tablosundan pişirilip gerçek `setupAnim` ile
pozlanır; deve tiki gerçek `Camel` varlığıyla, gerçek veri paketleri ve oyun zamanıyla karşılaştırılır.

## Java kaynak incelemesi

- `CamelModel.setupAnim`: baş yatayda ±30, dikeyde −25..45 dereceye sıkışır; koşu bekleme süresi
  varken baş `45 * kalan / 55` derece daha öne iner (en çok 70); sonra yürüyüş
  (`applyWalk(konum, hız, 2, 2,5)`: süre `konum * 50 * 2` ms, ölçek `min(hız * 2,5, 1)`) ve başlamış her
  durumun animasyonu (`oturma`, `oturma duruşu`, `kalkma`, `boşta`, `koşu`); bebeğin kendi tanımları var.
- `Camel.tick` (istemci): yerdeyken ya da binekliyken koşu bayrağı kapanır; boşta animasyonu 80 + `nextInt(40)`
  tikte bir başlar; pozdan: görünürde oturuyorsa (`poseTime < 0 != oturuyor`) kalkma ve koşu durur, oturma
  başladıysa (oturuyor ve poseTime 0..40) oturma, sonra oturma duruşu; ayaktaysa koşu bayrağına göre koşu ve
  geçişteyse kalkma. `poseTime` oyun zamanından `|LAST_POSE_CHANGE_TICK|` çıkarılarak bulunur (negatif sayı
  oturan deve); oyun zamanını `SetTime` paketi eşler.
- `Camel.updateWalkAnimation`: yalnız ayakta duran ve koşmayan deve yürür; hedef `min(mesafe * 6, 1)`, çarpan 0,2;
  bebekte konum ölçeği üç.
- `LlamaModel.setupAnim`: baş bakışla; bacaklar çaprazlama sallanır; yetişkinde yüklüyse sandıklar görünür.
- Doku: deve (yetişkin, bebek), kabuk (hep yetişkin modeli), lama dört renk × yetişkin/bebek (`DATA_VARIANT_ID`
  21, en çok üçe sıkışır).

## Veri

`Camel.DASH` (19, boolean), `Camel.LAST_POSE_CHANGE_TICK` (20, uzun) ve `Llama.DATA_VARIANT_ID` (21, int)
`entity/variant-fields`'ta karşılaştırılır.

## Kapsam

`entity/camel-model` 24 satır, `entity/llama-model` 10 satır, `living-entity-animations`'a beş deve satırı
(boşta, oturma, kalkma, koşu, inene dek koşu); animasyon tanımları `entity/animation-definitions`'ta.

## Sapmalar (açık)

- Devenin suda oturmaktan anında kalkması (`standUpInstantly`) ve `isInLiquid` ile koşunun bitmesi: suda olma
  mobda izlenmiyor.
- Koşu bekleme süresi (`dashCooldown`) yalnız kendi oyuncunun bindiği deve için sayılır: 0 kabul edilir.
- Bebeklerin yürüyüş konum ölçeği (üç) genel yürüyüşte yok (`follow`); yalnız devede uygulanır.
- Lamanın halı katmanı (`LlamaDecorLayer`, tüccar lamasının mavi halısı), eyer ve binme yok.
- Deve ve lama sesleri, tükürük (T39).
