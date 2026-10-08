# T72 — Yükselen duman/ateş ve kül parçacıkları — [client-parity]

Bu kayıt, `RisingParticle` ailesinin (`SoulParticle`, `LavaParticle`, `CampfireSmokeParticle`,
`PlayerCloudParticle`; `FlameParticle` T70'te) ve `BaseAshSmokeParticle` ailesinin (`LargeSmoke`,
`WhiteSmoke`, `Ash`, `WhiteAsh`, `DustPlume`; `Smoke` T70'te) istemci davranışını kapsar: sağlayıcının
çekilişleri, kurucunun değerleri, her tik konum/hız/yaş/renk/opaklık, ömür sonu, dörtgen boyu ve ışık.
Kanıt sınıfı `client-parity`'dir: gerçek Java sağlayıcıları tohumlanmış rastgeleyle
`movement/particle-ticks`'te (tek parçacık) ve `movement/particle-engine`'de (motor, seviye eklemesi)
koşturulur, alanlar bit bit karşılaştırılır.

## Java kaynak incelemesi

- `client/particle/RisingParticle.java`: sürtünme 0.96, hız yüzde biri + itiş, konum ±0.05 sarsıntı,
  ömür `8/(r*0.8+0.2)+4`. `SoulParticle`: ×1.5, resmi yaşla, `TRANSLUCENT`; `EmissiveProvider`
  (`sculk_soul`) blok ışığını 15 yapar.
- `LavaParticle.java`: sıfır itişli fırlatma, yerçekimi 0.75, sürtünme 0.999, dikey hız
  `r*0.4+0.05`, boy `r*2+0.2` çarpanı, ömür `16/(r*0.8+0.2)`, boy `1-s²`, ışık 15; her tik
  `random > yaş/ömür` ise seviyeden duman ister (`ClientLevel.addParticle`).
- `CampfireSmokeParticle.java`: ×3, kutu 0.25, ömür `nextInt(50)+80` (sinyal `+280`), yerçekimi
  3e-6, kendi tiki (yan sapma `r/5000`, son 60 tikte opaklık −0.015), opaklık 0.9/0.95.
- `PlayerCloudParticle.java`: hız onda biri + itiş, renk `1-r*0.3`, boy ×1.875, ömür
  `max(int(8/(r*0.8+0.3))*2.5, 1)`, fiziksiz; 2 blok içindeki seyirci olmayan en yakın oyuncunun
  üstündeyse beşte bir yaklaşır. `SneezeProvider`: renk (0.22, 1, 0.53), opaklık 0.4.
- `BaseAshSmokeParticle` değişkenleri: `LargeSmoke` ölçek 2.5; `WhiteSmoke` renk
  (0.729, 0.694, 0.761); `Ash` (0.1,−0.1,0.1), 0.5, 20, 0.1, fiziksiz; `WhiteAsh` itişi sağlayıcının
  rastgelesinden, renk `0xBAB1C2`; `DustPlume` (0.7,0.6,0.7), itiş +0.15, 0.5, 7, 0.5, her tik
  yerçekimi ×0.88 ve sürtünme ×0.92, renk `0xBAB1C2 − r*0.2`.

## Kapsam ve dal kararı

`particle-ticks`'e 20 satır (her sağlayıcı; tavan, zemin ve taş içi dalları), `particle-engine`'e 4
satır (lav dumanı: yakın/uzak, `DECREASED`; on bir sağlayıcı birlikte). Lav, motor satırlarında
sınanır (tek parçacık sahnesinde seviye eklemesi yok).

## Sapmalar (açık)

- Yarı saydam katman (`Soul`, kamp ateşi dumanı, bulut) çizimde saydam değil: Rust parçacık hattı
  kesik (cutout) çiziyor, opaklık yok sayılıyor (`Particle::translucent` hazır).
- Bulutun oyuncuya yerleşmesi oracle'da sınanmıyor (sahnede oyuncu yok).
