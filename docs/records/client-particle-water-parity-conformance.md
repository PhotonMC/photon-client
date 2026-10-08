# T73 — Su ve kabarcık parçacıkları — [client-parity]

Bu kayıt suyun parçacıklarını kapsar: `BubbleParticle`, `BubbleColumnUpParticle`, `BubblePopParticle`,
`WaterDropParticle` (`rain`) ve `SplashParticle`, `WakeParticle` (`fishing`),
`WaterCurrentDownParticle` (`current_down`), `SquidInkParticle` (`squid_ink`, `glow_squid_ink`).
Kanıt sınıfı `client-parity`'dir: gerçek Java sağlayıcıları tohumlanmış rastgeleyle
`movement/particle-ticks`'te su havuzu, zemin ve basamak sahnelerinde koşturulur; her tikin konum,
hız, yaş/ömür, kutu, renk, opaklık, resim ve ışığı bit bit karşılaştırılır.

## Java kaynak incelemesi

- Kabarcıklar: kutu 0.02, boy `×(r*0.6+0.2)`, itişin beşte biri ± `(r*2−1)*0.02`; `Bubble` ömrünü
  geri sayar (`lifetime--`), dikey +0.002, ×0.85, sudan çıkınca gider; `ColumnUp` yerçekimi −0.125,
  sürtünme 0.85, düz tik sonra su denetimi; `Pop` dört tik, yerçekimi 0.008, resmi yaşla.
- Damla: sıfır itişli fırlatma, ×0.3, dikey `r*0.2+0.1`, kutu 0.01, yerçekimi 0.06; tik geri sayar,
  ×0.98, yere değince yarı olasılıkla gider (×0.7); bulunduğu bloğun çarpışma şeklinin o sütundaki
  tepesi ya da akışkan yüksekliği (üstü aynı akışkansa 1) altına inince gider. `Splash`: yerçekimi
  0.04, yalnız yana itilmişse (dikey 0) yana ve 0.1 yukarı.
- `Wake`: itiş olduğu gibi, yerçekimi 0; boyu `(60−ömür)*0.001`, resmi `(60−ömür)%4` / 4.
- `CurrentDown`: fiziksiz, ömür `int(r*60)+30`, açı her tik +0.08; yatay `0.6*cos/sin(açı)` eklenip
  ×0.07; sudan çıkınca ya da yere değince gider.
- `SquidInk` (`SimpleAnimatedParticle`): sürtünme 0.92, boy 0.5, ömür `int(0.5*12/(r*0.8+0.2))`,
  fiziksiz, ikinci yarısında opaklık azalır (tik içinde iki kez), havada her tik −0.0074; tam
  parlak ışık; parlayan mürekkep `colorFromFloat(1, .2, .8, .6)`'nın 8 bitlik kanalları.

## Kapsam ve dal kararı

`particle-ticks`'e 20 satır: havuz içinde/dışında, yüzeye çıkış, zemin, su ve basamak üstüne yağmur,
yana/yukarı sıçrama, iz, akıntının zemine ve sudan çıkışı, havada/suda mürekkep.

## Sapmalar (açık)

- Mürekkep yarı saydam katmanda; Rust parçacık hattı kesik çiziyor (T72'deki gibi).
- Akan suyun farklı seviyeleri sahnede yok (yalnız kaynak); yükseklik kuralı koddan.
