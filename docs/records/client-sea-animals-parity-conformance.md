# T35d — Yunus ve kaplumbağa — [client-parity]

Bu kayıt, yunusun ve kaplumbağanın (yetişkin ve bebek) modelinin, yunusun taşıdığı eşyanın
konumunun ve kaplumbağanın istemci durumunun (karada olma, yumurta) Java 26.2 ile aynı sonucu
vermesini kapsar. Kanıt sınıfı `client-parity`'dir: modeller gerçek `DolphinModel`,
`BabyDolphinModel`, `AdultTurtleModel`, `BabyTurtleModel`'e, eşya konumu gerçek
`DolphinCarryingItemLayer`'a, durum gerçek `Turtle` ve `Dolphin` varlıklarına karşı karşılaştırılır.
Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `DolphinModel.setupAnim`: gövde başın dönüşünü ve eğimini alır; hareketliyken `-0,05 - 0,05·cos(0,3·yaş)`
  eklenir, kuyruk `-0,1·cos`, kuyruk yüzgeci `-0,2·cos` eğilir. Hareketli: istemcideki hareketin
  yatay uzunluğunun karesi `1e-7`'den büyük. Hareket paketten gelir, `LivingEntity.aiStep` onu eritir
  (T35b'de yazıldı).
- `DolphinCarryingItemLayer`: eşya çenenin altında, `(0; 1 ∓ …; -1 ± …)` kaymasıyla; eğim negatifse
  `1 - %·0,5` ve `-1 + %·0,5`, değilse `1 + %·0,8` ve `-1 + %·0,2` (`% = |eğim| / 60`). Eşya, ana
  eldeki yığındır (yer görünümü).
- `TurtleModel.setupAnim`: dört ayaklının yürüyüşü, sonra karada ön yüzgeçler y'de `∓cos(k·5·konum)·8·hız·a`
  (yumurtlarken `k = 4`, `a = 2`), arka yüzgeçler `∓cos(5·konum)·3·hız`; suda ya da havada arka
  yüzgeçler x'te `±cos(0,6662·0,6·konum)·0,5·hız`, ön yüzgeçler z'de `∓` aynı. Yetişkinde karın yumurtası
  `hasEgg` ile görünür ve model bir onaltılık alçalır. Bebekte yumurta yok.
- `TurtleRenderer`: karada olma = suda değil ve yerde; yumurta yalnız yetişkinde. Veriler 18 (yumurta)
  ve 19 (yumurtlama), boolean. Suda olma durumu kaplumbağa için de izlenir.

## Kapsam

`entity/dolphin-model` 8, `entity/dolphin-item-pose` 9, `entity/turtle-model` 14 satır;
`entity/variant-fields`'e iki satır; `living-entity-animations`'a 10 satır (kaplumbağa suda, karada,
sığ suda yerde, havada, yumurtalı, yumurtlarken; yunus durgun, yüzerken, durana dek, yalnız yukarı).

## Sapmalar (açık)

- Bebek kaplumbağa modeli `entityCutoutCull` ile çizilir (arka yüzler atılır); istemci varlık çizimi
  yüz atmayı ayırt etmediği için sıfır kalınlıklı yüzgeçler iki yüzlü görünür.
- Yunusun suda parçacıkları, sesleri ve gölgesi yok; kaplumbağanın yumurtlama parçacığı ve sesi yok.
  Canlı gözlem yapılmadı.
