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
- Küresel olaylar (1023, 1028, 1038) `levelEvent`'e gitmez; kameradan iki blok ötede, kameraya
  doğru yönde çalar (`globalLevelEvent`).

- T78e: ejderha nefesi (2006) iki yüz `dragon_breath` zerresi (`PowerParticleOption`, güç uzaklık)
  saçar, veri 1 ise fırlatmanın patlama sesi gelir; zerre yere inince yavaş yükselir. Geçit (3000)
  yayıcıyı sınırlayıcıdan geçirerek ekler (`addAlwaysVisibleParticle`), 3001 ejderha hırıltısı.
  3002 (çubuk kıvılcımı: eksen boyunca ya da yüzlerde), 3003–3005 (balmumu, kazıma) ve 3009
  (yumurta çatlağı) `ParticleUtils`'in yüz yardımcılarıyla zerre bırakır. Sculk yükü (3006) sayıya
  göre damar yüzlerinde ya da bütün blokta süzülür, sayısızsa patlar (tam blokta 40, değilse 20).
  Çığlık (3007) on gecikmeli zerre ve (suya gömülü değilse) sestir; 3018 on puf ve
  ağ sesidir.

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
- T78f: deneme spawner'ının olayları (3011–3014, 3017, 3019–3021): kıvılcımlı duman, algılama
  zerreleri (sayı `30 + min(veri,10)*5`, `LOOKAT_Y` ile yalnız dikey eksende dönen
  `trial_spawner_detection` zerresi), fırlatma alevi, ominous olma zerreleri; sesler `BLOCKS`,
  perde `(r−r)*0.2+1`. Bu zerrenin kamera dönüşü `movement/particle-quads` satırlarında eşit.
- Mesafe gecikmeli ses (`distanceDelay`) yok; oracle erteli sesi hemen çalmış sayar.
- Stalactite drip (1504): the root is searched for within 10 blocks; the fluid above it (mud counts
  as water where water does not evaporate) picks the drop, and with no fluid the dimension's
  default (lava in the Nether) applies. The client does not read a dimension type's attributes
  yet and uses the vanilla dimensions' values; the oracle is overworld only, so the Nether branch
  has no row.
- Vault activation (3015): only where the block is a vault (the level makes its block entity when
  the block is placed); the flames come after the connection particles the server's
  `shared_data` names, one burst for each connected player the level knows within reach. The
  local player is passed to the engine by the client; the oracle's level has no local player, so
  that path has no row.
