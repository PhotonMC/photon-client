# T74 — Toz, büyü, totem ve parıltı parçacıkları — [client-parity]

Bu kayıt seçenek taşıyan ve renkli parçacıkları kapsar: `DustParticle` ve
`DustColorTransitionParticle` (`DustParticleOptions`, `DustColorTransitionOptions`),
`SpellParticle` (`effect`/`instant_effect` `SpellParticleOption`, `entity_effect`
`ColorParticleOption`, `witch`, `infested`/`raid_omen`/`trial_omen`), `TotemParticle` ve
`GlowParticle` ailesi (`glow`, `electric_spark`, `scrape`, `wax_on`, `wax_off`). Kanıt sınıfı
`client-parity`'dir: gerçek Java sağlayıcıları (seçenekler satırdan kurulur) tohumlanmış rastgeleyle
`movement/particle-ticks` ve `movement/particle-engine`'de koşturulur, alanlar bit bit karşılaştırılır.

## Java kaynak incelemesi

- `DustParticleBase`: itişin onda biri, sürtünme 0.96, boy `×0.75*ölçek`, ömür
  `int(max(int(8/(nextDouble*0.8+0.2))*ölçek, 1))`; renk `(r*0.2+0.8)*kanal*(r*0.4+0.6)`.
  Geçiş: iki renk aynı kuralla, çizimde `(yaş+kısmi)/(ömür+1)` ile doğrusal (JOML `fma` kapalı).
  Ağda renk int, ölçek float (0.01–4'e sıkıştırılır).
- `SpellParticle`: yan itiş sınıfın statik `RANDOM`'undan (`0.5−nextDouble`), dikey ×0.2, itiş
  yataysa yatay ×0.1, yerçekimi −0.1, fiziksiz, boy ×0.75; opaklık her tik verilen opaklığa
  `lerp(0.05)`; dürbünle yakından bakan oyuncuya görünmez (bkz. sapmalar).
- `TotemParticle` (`SimpleAnimatedParticle`): yerçekimi 1.25, sürtünme 0.6, ömür `60+nextInt(12)`,
  dörtte bir sarı yoksa yeşil, ikinci yarıda solma, tam parlak.
- `GlowParticle`: sürtünme 0.96, boy ×0.75, fiziksiz, yaşla parlar; sağlayıcılar renk, hız ve ömrü
  sağlayıcının rastgelesinden alır.

## Kapsam ve dal kararı

`particle-ticks`'e 18 satır, `particle-engine`'e bir satır (14 tür birlikte, ağ seçenekleriyle).
Motor ve ağ artık `ParticleData` taşır (blok, renk, büyü, toz, toz geçişi).

## Sapmalar (açık)

- Büyü parçacıklarının dürbün kuralı yok (oyuncunun dürbünle bakışı dünyada tutulmuyor).
- Büyü ve totem yarı saydam katmanda; Rust parçacık hattı kesik çiziyor.
- `SpellParticle.RANDOM` oyunda tohumu saatten alan ayrı bir rastgele olarak tutulur.
