# T36b — Yarasa, vex ve allay — [client-parity]

Bu kayıt, yarasanın, vex'in ve allay'ın modelinin ve istemci tikinin (yarasanın animasyon durumları ve asılı
konumu, allay'ın kol ve dans ilerlemesi, vex'in yüklenme bayrağı) Java 26.2 ile aynı sonucu vermesini kapsar.
Kanıt sınıfı `client-parity`'dir: modeller gerçek `BatModel`, `VexModel` ve `AllayModel`'e, tik gerçek
`Bat`, `Vex` ve `Allay` varlıklarına karşı karşılaştırılır. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `BatModel.setupAnim`: yalnız asılıyken baş dönüşü alınır; uçma ve asılma animasyonları, durumları başlamışsa.
  `Bat.tick` (istemci, `super.tick`'ten sonra): asılıyken hareket sıfırlanır ve konum **yukarıdaki bloğa**
  yapışır (`kat(y) + 1 − boy`); uçarken dikey hareket 0,6 ile çarpılır; durumlar bayrağa göre değişir (başlamışsa
  yeniden başlatılmaz).
- `VexModel.setupAnim`: kollar `cos(yaş × 5,5°) × 0,1` ile sallanır, gövde `π/20` eğik (yüklenirken düz); yüklenirken
  ellerde eşya yoksa iki kol ileri uzanır, varsa yalnız eşyalı kol kalkar; kanatlar `45,84°` hızla çırpılır.
  Renderer blok ışığını on beş yapar, yüklenirken öteki dokuyu kullanır.
- `AllayModel.setupAnim`: kanatlar yürüme konumu ve yaşla çırpılır, gövde uçtukça eğilir (`min(hız/0,3, 1)`), kollar
  elindekine göre kalkar, havada asılıyken `Math.cos` ile hafif iner çıkar; dans ederken (dönüş ve sallanma) baş
  ve kök eğilir. `Allay.tick` (istemci): eşya ilerlemesi 5 tikte dolar, dans sayacı artar, dönme ilerlemesi 55 tikte
  ilk 15 tikte yükselir, sonra düşer. Renderer blok ışığını on beş yapar.
- Veriler: yarasa bayrakları 16 (bayt, bit 1: asılı), vex bayrakları 16 (bayt, bit 1: yükleniyor), allay dansı 16
  (boolean).
- Vex ve allay `entityTranslucent` çizer (dokularında 160 alfa var); entity boru hattı zaten `SRC_ALPHA`
  ile harmanlar ve derinliğe yazar, ek boru hattı gerekmedi.

## Kapsam

`entity/flier-model` 28 satır (yarasa: uçma, asılma, dönmüş baş, durumsuz; vex: bakış, yüklenme, sağ, sol,
iki elde eşya; allay: asılı kalma, uçma, tutma, dans, dönme ve birleşimleri); `living-entity-animations`'a 9 satır
(yarasa uçar, asılır, uyanır; vex yüklenir; allay asılı kalır, eşya alır ve bırakır, dans eder, durur);
`entity/variant-fields`'e üç satır.

## Sapmalar (açık)

- Vex'in ve allay'ın elindeki eşya (`ItemInHandLayer` ve `translateToHand`) çizilmez; yalnız vex'in kollarının
  duruşu eşyanın varlığını okur (T36d). Allay'ın bırakma ve alma sesleri, vex'in çıkış sesi, parçacıklar yok.
- Yarasa yapışırken sınırlayıcı kutusu izlenmez; boy türün boyundan alınır. Doku yolları renderer sabitinden elle
  yazıldı. Gölge yok. Canlı gözlem yapılmadı.
