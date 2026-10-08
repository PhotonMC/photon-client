# T78, T78b — Seviye olaylarının sesleri ve parçacıkları — [client-parity]

Bu kayıt `LevelEventHandler.levelEvent`'in ses ve basit parçacık olaylarını kapsar. Kapsanan
olaylar: yalnız ses çalan 1000–1052, söndürme 1009, lav/meşale/portal çerçevesi dumanları
1501–1503, dağıtıcı dumanı 2000 ve 2010, patlama 2008, bulut 2009, blok kırılması 2001. Kanıt
sınıfı `client-parity`'dir. `movement/particle-engine`'de gerçek `ClientLevel.levelEvent`
sessiz ses yöneticisi ve tohumlu motorla koşar. Çalınan seslerin sırası, kategorisi, sesi,
perdesi, yeri ve üretilen parçacıklar Rust'la bit bit karşılaştırılır.

## Java kaynak incelemesi

- Rastgele seviyenin rastgelesidir. Her `playLocalSound` önce perdeyi çeker, sonra ses
  örneğinin tohumu için bir `nextLong` daha çeker; sıra korunmazsa ardından gelen parçacıklar
  kayar. Ses bloğun ortasında çalar.
- Perde türleri: sabit, `taban + (r−r)*ölçek` (mob sesleri 1 ve 0.2, söndürme 2.6 ve 0.8),
  `r*0.1+0.9` ve rüzgâr yükünün `0.4/(r*0.4+0.8)`'i.
- 1009 veriye göre ateş ya da genel söndürme sesidir; başka veride sessizdir.
- Dumanlar sesten sonra `Level.addParticle` ile tek tek eklenir. Her eklemede seçenek düzeyi
  için seviye rastgelesi yeniden çekilebilir.
- `shootParticles`: `Direction.from3DDataValue(data)` yönünde 10 duman, güç
  `r*0.2+0.01`, yüze yayılmış konum, Gauss sapmalı hız.
- Küresel olaylar (1023, 1028, 1038) `levelEvent`'e gitmez.

- T78b: kompostçu (1500) önce sesini çalar, sonra içeriğinin üstüne Gauss itişli on zerre
  saçar. Sıçrayan iksirde (2002, anlık 2007) sekiz şişe kırıntısı, rengini taşıyan yüz büyü
  zerresi (`colorFromFloat`, parlaklık `0.75+r*0.25`, güç uzaklık) ve `NEUTRAL` şişe sesi vardır.
  Ender gözü (2003) sekiz kırıntı ve `π/20` adımlı iki hızlı portal halkası bırakır. Spawner
  (2004) yirmi duman ve alev çıkarır. Kemik tozu ve arı (2011, 2012) bloğun dış şekli
  yüksekliğinde mutlu köylü zerreleri saçar (`spawnParticleInBlock`).

## Kapsam ve dal kararı

Dört satır var: tüm ses olayları, duman/bulut/patlama, `DECREASED` seçeneğiyle duman, altı
yön ve negatif veriyle atış. 2001 parçacıkları `movement/particle-terrain`, sesi
`movement/breaking` ile sınanır. Olaylar ağdan sırayla kuyruğa girer ve motor seviye
rastgelesiyle işler.

## Sapmalar (açık)

- `level-event-shoot` özeti Q04 yüzünden tutmuyor: hızdaki Gauss bir ulp sapar (JVM
  `Math.log`). İlk 16 parçacık eşit.
- Ses tohumu çekiliyor ama ses motoruna verilmiyor; varyantı motorun kendi rastgelesi seçer.
- `level-event-more` özeti ender gözü halkasındaki bir `Math.sin`'in 1 ulp farkıyla tutmuyor (JVM
  intrinsic'i, Q04 sınıfı). Topuz darbesi (2013) satırda yok; Java'da model yöneticisi ister.
- Kalan olaylar (kemik tozu büyümesi 1505, sarkıt 1504, ejderha nefesi 2006, 3000'ler, küresel ve
  jukebox) T78e'de.
