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
- Jukebox songs (1010, 1011): a song starts from the middle of the block at volume 4 over the
  records category and is stopped by the next song or by 1011; a registry number with no song
  does nothing, not even a stop. The oracle records the sound manager's play and stop calls.
- Jukebox parrots (1010, 1011): the living entities whose box meets the jukebox's block inflated
  by three are told the record plays or stops; a parrot dances until its jukebox is 3.46 or more
  from it or is no longer a jukebox. The oracle compares which parrots dance after each tick, not
  the clearing on a parrot's own tick.
- Block events on block entities (#47): the block as it stands takes the event (the packet's block
  is ignored). A decorated pot notes when it started to wobble and in which style (only 0 and 1
  are styles), an end gateway's cooldown starts at 40, a spawner's delay is reset to its minimum
  and a potent sulfur block notes the game time of its eruption; a potent sulfur block entity
  made by the level starts with the game time it was made at. The oracle compares the state the
  entities hold; what reads it (the pot and gateway renderers, the spawner's and the sulfur's
  tickers) is still to come.
- Piston events (#47): the client runs `PistonBaseBlock.triggerEvent` itself: the structure the
  push moves (twelve blocks at most, slime and honey holding on to what is beside them) becomes
  moving pistons, the blocks it breaks go with a break event, the piston and its arm change, and
  the pitch of the sound is drawn from the level's random source (the client plays no sound of its
  own; the server's reaches the player). The oracle compares every block of a box and each moving
  piston's data right after the events. Not yet compared: the blocks a push breaks (their break
  particles take seeds the oracle cannot give), and the shapes that only blocks with a port of
  `updateShape` take (the others keep their state here).
- Moving pistons tick (#47): the level's block entities tick after its entities, in the order they
  were set up (one set up during a pass waits for the next, a removed one is dropped as the pass
  reaches it). A moving piston moves half the way each tick, waits five ticks once it arrived
  and settles as the block it carried, shaped by its neighbours (a waterlogged one dries). The
  oracle ticks the block entities only for the rows that ask for it. Entities caught by a moving
  block are not pushed yet.
- Moving pistons are drawn (#47, #48): the block each carries (or the head of a piston, as short or
  long as its progress), at the place it has moved to by the partial tick, and the base a
  retracting arm leaves behind, lit and tinted as a falling block is. No parity row compares the
  drawing; a live run against the vanilla server showed a sticky piston and a normal one in
  the middle of a push.
- Blocks that animate by themselves (#45): the wither rose's and the end portal's smoke, the dried
  ghast's wisps (and in water its happy specks), an enchanting table's glyphs towards each
  bookshelf with a clear way to it (the offsets go as `BlockPos.betweenClosed` does, x fastest then
  y then z), honey dripping from a hive of honey level five or more (the bound of its loop is
  asked again on every pass), dust under suspicious sand and gravel, and the idle sounds of
  pale moss, open eyeblossoms, dead bushes and dry grass. The oracle's sound manager leaves the
  ambient sounds out, so they show only through the draws they make; dry grass's sound is the
  player's own and draws only where the client has a player (the oracle has none).
- A stalactite's tip drips (#45): a free hanging tip lets a drop go on one tick in about eight
  (the fluid above it picks the kind, and only still water and lava fill a cauldron, so the
  rest drip a sixth as often), and a lightning rod in a thunderstorm sheds one or two sparks
  when it stands at the top of its column, the more likely the later in the second it is. The
  dripstone's dimension is now the one the session is in on every tick, not only on a level
  event. The oracle's chunks carry the world surface and motion blocking heightmaps (the server
  sends them with the chunk; the reference's client otherwise reads zeros).
