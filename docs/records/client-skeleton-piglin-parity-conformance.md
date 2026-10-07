# T29 — İskelet ve piglin aileleri modeli — [client-parity]

Bu kayıt, iskelet, stray, bogged, parched, wither iskeleti ile piglin,
piglin brute ve zombifiye piglin (yetişkin ve bebek) modellerinin pozunun
(`SkeletonModel.setupAnim`, `PiglinModel`/`ZombifiedPiglinModel.setupAnim`)
Java 26.2 ile Rust'ta aynı köşe konum, doku koordinatı ve normali üretmesini
kapsar. Kanıt sınıfı `client-parity`'dir: gerçek model sınıfları katman
tablosundan pişirilip gerçek `setupAnim` ile pozlanır, parça çizicisiyle
çizilir ve her dörtgen bit bit karşılaştırılır. Canlı kabul (altı tür yan yana)
`server-conformance` gözlemidir.

## Java kaynak incelemesi

- `model/monster/skeleton/SkeletonModel.java:90-105`: insansı pozdan sonra
  saldırgan ve yay tutmayan iskelet kollarını öne `-π/2 - (saldırı...)` ile
  uzatır, sallama zamanına göre döndürür ve kolları yeniden nefes aldırır.
- `model/monster/piglin/AbstractPiglinModel.java:72-82`: kulaklar yürüme ve
  yaşla çırpınır (`frequency = yaş*0.1 + yürüme*0.5`, genlik `0.08 + hız*0.4`,
  varsayılan açı yetişkinde 30°, bebekte 5°). `PiglinModel`: dans (kulak, baş,
  kol ve gövde salınımı), diğer kollar eşya durumlarına bağlı (yay tutma, arbalet,
  hayranlık). `ZombifiedPiglinModel`: kulaklardan sonra zombi kolları.
- Renderer'lar (`AbstractSkeletonRenderer`, `StrayRenderer`, `BoggedRenderer`,
  `PiglinRenderer`): doku seçimi, stray/bogged üzerine `SkeletonClothingLayer`
  (aynı pozlu ikinci model ve `_overlay` dokusu); piglin dokusu tür ve bebeğe göre.

## Veri

`Piglin.DATA_BABY_ID` (17) ve `DATA_IS_DANCING` (19) veri indeksleri ve
serileştirici kimlikleri Java'nın erişimcilerinden alınıp `entity/variant-fields`
satırlarıyla her parity koşusunda karşılaştırılır; bebek ve dans bayrakları
dünyaya okunur.

## Kapsam ve dal kararı

`entity/skeleton-model` 63 satır (yedi katman × dokuz poz: durma, bakış, yürüme,
yaylı ve yaysız saldırgan, üç sallama aşaması, yüzme); `entity/piglin-model` 47
satır (beş katman × yedi poz, piglin katmanlarında iki dans). Aile içi türler
tablo satırıdır.

## Sapmalar (açık)

- Elde eşya okunmuyor: iskeletin `isHoldingBow` değeri bilinmediğinden canlıda yay
  tutuyor sayılır (saldırgan poz atlanır); yay çekme, arbalet, hayranlık ve silah
  kaldırma pozları ile eşyalı kollar T41'de. Saldırı sallaması `Animate`
  paketiyle (T43).
- Bogged'ın mantarları hep görünür (`DATA_SHEARED` okunmuyor); bogged/iskelet
  titremesi (`isShaking`) ve piglinin `isConverting` titremesi yok.
- Mob yüzme (`swimAmount`) tiklenmediğinden yüzüş kolları canlıda çalışmaz.
- Zırh, kafa eşyası (T41); parched'ın dış katmanı yok (renderer eklemiyor).
