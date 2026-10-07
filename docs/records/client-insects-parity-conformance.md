# T33b — Gümüş balığı, endermite ve arı — [client-parity]

Bu kayıt, gümüş balığı, endermite (gövde parçalarının dalgası) ve arının (kanatlar, bacaklar,
sallanma, yan yatma, iğne; yetişkin ve bebek) modellerinin Java 26.2 ile Rust'ta aynı köşe
konum, doku koordinatı ve normali üretmesini ve arının istemci tikinin (yuvarlanma miktarı)
aynı ilerlemesini kapsar. Kanıt sınıfı `client-parity`'dir (model, tik) ve canlı gözlemdir:
gerçek `SilverfishModel`, `EndermiteModel`, `AdultBeeModel` ve `BabyBeeModel` katman
tablosundan pişirilip gerçek `setupAnim` ile pozlanır; arı tiki gerçek `Bee` varlığıyla ve
gerçek veri paketiyle karşılaştırılır.

## Java kaynak incelemesi

- `SilverfishModel`/`EndermiteModel.setupAnim`: her parça yaşa göre `cos` ile y'de sallanır ve
  `sin` ile yana kayar; sapma ve kayma ortadan (2. parça) uzaklıkla büyür; gümüş balığının üç
  üst katmanı altındaki parçaları izler. İkisinin de `getFlipDegrees`'i 180.
- `BeeModel.setupAnim`: iğne gövde verisine göre görünür; havadayken kanat çırpar (yaş çarpı
  120,32113 derece) ve bacaklar sarkar; öfkeli olmayan havadaki arı yukarı aşağı sallanır
  (yetişkinde antenler de); yuvarlanıyorsa `bone` öne devrilir (`rotLerpRad`). Yerde duran arı
  (`onGround` ve hız karesi 1e-7'den küçük) kanat ve bacak pozunu korur.
- `Bee`: `tick` sonunda `updateRollAmount` (yuvarlanıyorsa beşte bir artar, değilse yaklaşık
  dörtte bir azalır); bayrak baytı yuvarlanma (2), sokmuş (4), nektar (8); öfke bitiş zamanı
  (uzun veri) oyun zamanından sonraysa öfkeli (`NeutralMob.isAngry`).
- Doku: yetişkin/bebek × öfkeli/sakin × nektarlı/nektarsız sekiz doku.

## Veri

`Bee.DATA_FLAGS_ID` (18, bayt) ve `DATA_ANGER_END_TIME` (19, uzun, serileştirici 2)
`entity/variant-fields`'ta karşılaştırılır.

## Kapsam

`entity/segmented-model` 8 satır, `entity/bee-model` 14 satır, `living-entity-animations`'a altı
arı satırı (yuvarlanma, durma, sokmuş, öfke). Nektar bayrağı oracle'a alınamadı: nektarlı arı
rastgele nektar parçacığı bırakır, kafasız istemci çizemez; bayrak ayrıştırması birim testli.

## Sapmalar (açık)

- Arı iğnesi dokusu ve sokma/hedef animasyonu yok; öfke yalnız bitiş zamanı verisinden.
- Bebek arının öfke dokuları listelenir ama canlı gözlenmedi.
- Sesler, kovan etkileşimi, parçacıklar (nektar damlası) PLAN-M7/M8'de.
