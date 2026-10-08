# T73b — Damla parçacıkları — [client-parity]

Bu kayıt `DripParticle` ailesini kapsar: su, lav, bal, nektar, spor çiçeği, obsidyen gözyaşı ve
dripstone damlalarının asılı, düşen ve inen hâlleri. Kanıt sınıfı `client-parity`'dir. Gerçek
sağlayıcılar `movement/particle-ticks`'te tek tek, `movement/particle-engine`'de motor içinde
(aşamalar arası geçiş, iniş sesleri) ve `animate_tick` ile sıvı altından damlarken koşturulur.
Hepsi bit bit karşılaştırılır.

## Java kaynak incelemesi

- `DripParticle`: kutu 0.01, yerçekimi 0.06; hız her tik `yd − gravity` (0.04 çarpanı yok),
  sürtünme `0.98F`. Türünün sıvısına bloğun sıvı yüksekliğinin altında girince kaybolur.
  Yükseklik, üstte aynı sıvı varsa 1'dir.
- Asılı: yerçekimi ×0.02, ömür 40. Ömür bitince düşeni o anki yer ve hızla ister, her tik
  hızı ×0.02 olur. Soğuyan (lav) her tik önce rengini `g = 16/(56−ömür)`, `b = 4/(48−ömür)`
  yapar. Bal ve gözyaşında yerçekimi ayrıca ×0.01, ömür 100.
- Düşen: yere değince biter. İnen türü olanın ömrü `int(64/(r*0.8+0.2))` olur, iniş türünü
  ister. Bal ve dripstone damlası ayrıca kendi rastgelesinden `0.3–1` sesle `beehive.drip`
  ya da `pointed_dripstone.drip_*` çalar (seviye rastgelesinden tohumla).
- İnen: ömür `int(16/(r*0.8+0.2))`. Bal 128, gözyaşı 28 ile, sağlayıcı rastgelesinden.
- Nektarın ömrü sağlayıcı rastgelesinden 16'lık; spor çiçeğininki
  `int(64F/randomBetween(0.1, 0.9))`. Gözyaşı blok ışığı 15 ile parlar.
- `ClientLevel.trySpawnDripParticles`: geçirimsiz (`minecraft:impermeable`) bloktan damlamaz.

## Kapsam ve dal kararı

`particle-ticks`'e 16 satır var. Motor satırları: aşama zinciri (su ve lav asılı, bal,
dripstone lav ve su düşüşü, sesleriyle) ve taş/cam altından `animateTick` damlası. Parçacık
tikinin istediği ses, motorun istek sırasında (`Emitted`) seviye rastgelesinden tohum çeker.

## Sapmalar (açık)

- Yok bilinen. Sarkıt ucundan damlama (`PointedDripstoneBlock.spawnDripParticle`) T70f/T78b'de.
