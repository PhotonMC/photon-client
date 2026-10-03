# T31 — At, eşek, katır ve ölü atlar — [client-parity]

Bu kayıt, atın (yedi renk, beş işaret), eşeğin, katırın, iskelet atının ve zombi atının
(yetişkin ve bebek) modellerinin Java 26.2 ile Rust'ta aynı köşe konum, doku koordinatı ve normali
üretmesini; hayvanın kendi istemci tikinin (otlama, şahlanma ve ağız açma animasyonları, kuyruk
sallama) aynı ilerlemesini kapsar. Kanıt sınıfı `client-parity`'dir (model ve tik) ve canlı
gözlemdir (çizilen birleşim): gerçek `HorseModel`, `BabyHorseModel`, `DonkeyModel` ve
`BabyDonkeyModel` katman tablosundan pişirilip gerçek `AbstractEquineModel.setupAnim` ile
pozlanır; animasyonlar ve kuyruk gerçek `AbstractHorse` varlığıyla ve gerçek veri paketiyle
karşılaştırılır.

## Java kaynak incelemesi

- `AbstractEquineModel.setupAnim`: baş yatayda ±20 dereceye sıkışır; yürürken hız 0,2'nin
  üstündeyse başa küçük bir sallanma biner; baş otlama, şahlanma ve olağan poz arasında
  `eatAnimation`/`standAnimation` ile karışır; ön bacaklar şahlanırken yukarı ve geriye, arka
  bacaklar adımla gider (suda adım 0,2 katına iner); gövde şahlanırken −45 derece yatar; kuyruk
  hızla kalkar ve `animateTail` iken `cos(yaş * 0,7)` ile salınır; yaş ölçeği kuyruğun yerini
  kaydırır.
- Bebek at (`BabyHorseModel`): ayakta duruş kaydırmaları ve kuyruk eğimi farklı; bebek eşek
  (`BabyDonkeyModel`): parçalar `body` altında, kendi sabitleri ve baş pozu −30 derecelik sabit
  eğimle yeniden hesaplanır; arka bacaklar şahlanırken biraz iner. Eşek ve katır: sandık parçaları
  `hasChest` ile görünür.
- `AbstractHorse`: `DATA_ID_FLAGS` bayt: 16 otluyor, 32 şahlanmış, 64 ağız açık. İstemci
  `tick`'inde üç animasyon yumuşar (`eatAnim`: ±0,4 ve ±0,05 adımla, şahlanırken sıfırlanır;
  `standAnim`: şahlanırken 0,4/0,05 adımla artar, değilse küp terimiyle düşer; `mouthAnim` 0,7);
  `aiStep` başında 200'de bir şansla kuyruk sallanır (`tailCounter` 1'den 8'e sayar).
- Kuyruk şansı varlığın tohumsuz rastgele kaynağındandır ve `Mob.baseTick`'in ambiyans ses
  yoklamasıyla (`nextInt(1000) < ambientSoundTime++`, bulununca iki `nextFloat` ses perdesi
  için) aynı kaynağı paylaşır; sıra baseTick, sonra kuyruk şansıdır.
- Doku: `Horse.getVariant`/`getMarkings` (`DATA_ID_TYPE_VARIANT` düşük bayt renk, sonraki bayt işaret;
  yedi ve beş değerle sarar); işaret katmanı atın üstüne aynı modeli işaret dokusuyla çizer
  (görünmezken değil).

## Veri

`AbstractHorse.DATA_ID_FLAGS` (18, bayt), `Horse.DATA_ID_TYPE_VARIANT` (19, int) ve
`AbstractChestedHorse.DATA_ID_CHEST` (19, boolean) `entity/variant-fields`'ta karşılaştırılır.

## Kapsam

`entity/equine-model` 80 satır (sekiz katman × on poz), `living-entity-animations`'a yedi at
satırı (otlama, bırakma, şahlanma, ikisi birden, ağız, kuyruk).

## Sapmalar (açık)

- Atın suda olduğu (`isInWater`) mobda izlenmiyor: adım 0,2 katı yalnız oracle'da.
- Kuyruk ve ambiyans yoklaması rastgele kaynağı üretimde saatle tohumlanır (referans da
  tohumlamaz); karşılaştırma için tohum sabitlenir.
- İşaret katmanı `entityTranslucent` yerine varlık hattıyla çizilir: onda alfa 0,1'in altı
  kesilir (referansta kesme yok); işaret dokularında yarı saydam piksel varsa fark görünür.
- Eyer ve at zırhı katmanları (`SimpleEquipmentLayer`) yok; binme (T44).
- Deve, lama ve tüccar lamasının modelleri T31b'de (bu varlıklar `AbstractHorse` durumunu zaten taşır).
