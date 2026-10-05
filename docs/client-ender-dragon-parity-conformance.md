# T37h — Ender ejderhası (canlı hâl) — [client-parity]

Bu kayıt, ender ejderhasının modelinin (uçuş geçmişinden poz alan boyun, kafa, kuyruk; kanat çırpışı; bacaklar) ve
istemci durumunun (uçuş geçmişi, çırpış zamanı, evre, ölüm sayacı, öldürücü darbe) Java 26.2 ile aynı sonucu
vermesini kapsar. Kanıt sınıfı `client-parity`'dir: model gerçek `EnderDragonModel`'e, durum gerçek `EnderDragon`
varlığının tikine karşı karşılaştırılır. Çizim (dönüşler, göz katmanı, ışık) oracle ile karşılaştırılmaz. Ölüm çizimi,
kristal ışını ve parçalar ayrı görevlerdedir ([T37h2](tasks/T37h2-ejderha-olumu.md), [T37h3](tasks/T37h3-ejderha-kristal-isini.md),
[T37h4](tasks/T37h4-ejderha-parcalari.md)). Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `DragonFlightHistory`: 64 örnekli halka, ilk kayıt hepsini doldurur; `get(gecikme, kısmi)` iki örneği (double `lerp`,
  `rotLerp`) aradeğerler. `EnderDragonModel.setupAnim`: çene ve gövde zıplaması çırpış zamanıyla; boyun beş parçası ve
  kafa geçmişin dönüş ve yüksekliklerini zincirleme izler; kanatlar çırpar (sol/sağ ayna), bacaklar zıplar; kuyruk on
  iki parçası geçmişi arkada izler. Boyun eğimi iner/kalkarken yumurtaya uzaklıkla (`part / max(uzaklık/4, 1)`),
  otururken parça numarasıyla belirlenir.
- `EnderDragon.aiStep` (istemci) `LivingEntity.aiStep`'in yerini alır: hareket sönmez, yürüyüş animasyonu hesaplanmaz,
  baş bakışa uymaz. Ölmüyorsa çırpış: `0.2/(yatay hız·10 + 1) · 2^(hız.y)`, otururken 0.1 sabit, yapay zekâsızken 0.5'te
  durur; yaw `wrapDegrees`'e alınır, yükseklik ve yaw geçmişe yazılır, sonra konum son paketin konumuna yaklaşır ve
  gövde yaw'ı yaw olur. Ölürken (`isDeadOrDying`) çırpış ve geçmiş durur.
- `EnderDragon.DATA_PHASE` (veri 16): evre kimlikleri; oturma evreleri ve beklemede `isSitting`. İlk evre beklemedir
  (oturan). `handleKillingBlow`: ölüm olayı (3) oturmayan ejderhayı öldürmez: canı 1 olur ve evresi ölüm evresi olur.
- `EnderDragon.tickDeath`: `deathTime` değil kendi `dragonDeathTime` sayacı artar.
- `EnderDragonRenderer`: dönüş geçmişin 7 tik önceki yaw'ıyla (`−yaw`), 5 ve 10 tik önceki yüksekliklerin farkının 10 katı
  kadar öne eğim, sonra `(0,0,1)`, `scale(−1,−1,1)`, `(0,−1.501,0)`; ışık göz konumundan; gözler ayrı katman.
  Yumurtaya uzaklık, podyumun (başlangıçta orijin) yaprak sayılmayan en yüksek hareket engelleyen bloğunun merkezine
  uzaklığın karesidir (`MOTION_BLOCKING_NO_LEAVES`; yaprak ayrımı blok adının `_leaves` ile bitmesiyle yapılır).

## Kapsam

`entity/ender-dragon-model` 12 satır (tek örnekli geçmiş, düz, tırmanıp sola/inip sağa dönen, halkayı aşan, kısmi tikler,
çırpışın iki ucu, inerken yumurtaya uzak ve yakın, oturan); `living-entity-animations`'a 6 satır (bekleme, uçuş, uçup
dönme (göreli hareket ve dönüşle), iniş, beklerken ve uçarken ölüm olayı).

## Sapmalar (açık)

- Ölüm çizimi (çözünme, ışınlar), kristal ışını ve parçalar (vuruş kutuları, seçilebilirlik) yok. Ölürken ejderha
  normal çizilir. Sesler, parçacıklar, evrelerin sunucu mantığı yok. Yumurta konumu için savaş başlangıcı (`fightOrigin`)
  istemcide sıfır alınır. Doku yolları elle yazıldı; canlı gözlem yapılmadı.
