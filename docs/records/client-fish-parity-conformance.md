# T35c — Morina, somon, tropikal balık, kirpi balığı — [client-parity]

Bu kayıt, dört balığın (morina, somon, tropikal balık, kirpi balığı) modelinin, çizim dönüşlerinin,
dokularının, boyutlarının ve suda olma durumunun Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt
sınıfı `client-parity`'dir: modeller gerçek `CodModel`, `SalmonModel`, `TropicalFish*Model` ve
`Pufferfish*Model`'e, dönüşler gerçek `*Renderer.setupRotations`'a, dokular gerçek renderer ve
katman sabitlerine, boyut ve suda olma gerçek varlıklara karşı karşılaştırılır. Canlı gözlem
`server-conformance`.

## Java kaynak incelemesi

- Modeller: morinanın `tail_fin`'i, tropikal balığın `tail`'i `-k·0,45·sin(0,6·yaş)` ile y'de
  sallanır (`k` suda 1, karada 1,5). Somonun `body_back`'i `-a·0,25·sin(h·0,6·yaş)` ile sallanır
  (suda `a = h = 1`, karada `1,3` ve `1,7`). Kirpi balığının iki yüzgeci (küçükte `right_fin` ve
  `left_fin`, ötekilerde `right_blue_fin` ve `left_blue_fin`) `∓0,2 ± 0,4·sin(0,2·yaş)` ile z'de
  çırpınır. Şişme durumuna göre üç model (0 küçük, 1 orta, ötesi büyük), somonun boyutuna göre üç
  katman (küçük ve büyük, tablodaki kök dönüşümüyle ölçeklenmiş).
- `setupRotations` (üst sınıfın dönüşü ve ölüm düşüşünden sonra): morina `4,3·sin(0,6·yaş)` y,
  somon `a·4,3·sin(h·0,6·yaş)` y, tropikal balık `4,3·sin(0,6·yaş)` y; susuzken morina
  `(0,1; 0,1; -0,1)`, ötekiler `(0,2; 0,1; 0)` kayar ve 90° z döner. Kirpi balığı yalnız
  `cos(0,05·yaş)·0,08` yukarı çıkar, **üst sınıfın dönüşlerinden önce**.
- Tropikal balık: gövde dokusu ve renk (`getModelTint` = taban boyası) taban boyuna (`tropical_a`,
  `tropical_b`) göre, desen katmanı ikinci bir modelle (`#pattern`, `0,008` büyütülmüş) desen
  dokusuyla ve desen boyasıyla çizilir; görünmezken katman çizilmez. Paketlenmiş değişken: düşük 16 bit
  desen (taban `0`/`1`, indeks `<< 8`; tanınmayan kimlik ilk desen, kob), sonraki baytlar taban ve
  desen boyası (tanınmayan boya beyaz).
- Veri: somon türü, tropikal balık değişkeni ve şişme durumu hepsi 17. veri, int. Somon boyutu
  `0..2`'ye kırpılır (varsayılan orta); `getDefaultDimensions` somonda `0,5/1/1,5`, kirpi balığında
  `0,5/0,7/1` ile ölçeklenir.
- `Entity.baseTick` suda olma durumu bu dört tür için de izlenir (kutu: tablo boyutu, somon boyutu
  ve şişme durumuyla ölçeklenmiş).

## Kapsam

`entity/fish-model` 88 satır (on bir katman × dört yaş × suda ve karada); `entity/fish-rotations`
24 satır (dört tür × suda ve karada, ikişer yaş, ölü ve ölü-karada); `entity/fish-textures` 23
satır (üç tür dokusu, on iki desen ve sekiz taşma değişkeni, boya kimlikleriyle);
`entity/variant-fields`'e üç satır (veri kimlikleri ve serileştirici); `living-entity-animations`'a
10 satır (morina ve tropikal balık suda ve susuz; somon boyutları ve kirpi balığı şişme durumları,
suyun yalnız büyük kutuların üstüne değdiği bir katmanda, kutu ve suda olma durumuyla).

## Sapmalar (açık)

- `AbstractFish.aiStep`'in karada çırpınması (sıçrama hareketi ve sesi) istemcide uygulanmaz:
  konum sunucudan gelir; hareketi yalnız çizimde görünmez.
- Tadpole (iribaş) bu görevde değil, T35d'dedir. Kova ile tutma, sesler, parçacıklar yok.
- Gölge çizilmez (yalnız domuz, inek, eşya ve orb gölgesi var). Canlı gözlem yapılmadı.
- Suyun akıntısı mob hareketine uygulanmaz (kalamarla aynı).
