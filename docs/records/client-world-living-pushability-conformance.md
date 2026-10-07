# P09.t — ClientWorld LivingEntity alive/spectator pushability gates — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.isPushable()` kapısındaki canlılık ve
spectator dallarının `ClientWorld` entity collision sorgusuna bağlandığını
kanıtlar. Kanıt sınıfı `client-parity`'dir; canlı sunucu kabulü değildir.

## Java kaynak incelemesi

İncelenen temel yollar:

- `world/entity/LivingEntity.java:3375-3377`, `isPushable()` sonucunu
  `isAlive() && !isSpectator() && !onClimbable()` olarak kurar.
- `world/entity/LivingEntity.java:1751-1754`, `isAlive()` için entity'nin
  kaldırılmamış ve health değerinin sıfırdan büyük olmasını kullanır.
- `world/entity/player/Player.java:1675-1680`, spectator kararını game mode'dan
  verir. `Player.java:2024-2025` uçuş istisnasından sonra `onClimbable()` yolunu
  LivingEntity'ye devreder.
- `world/entity/LivingEntity.java:1721-1748`, `onClimbable()` için in-block,
  fall-flying, climbable block tag'i ve trapdoor/ladder özel dallarını taşır.
  Bu dalın block-state bağlamı bu dilimin açık kapsamındadır.
- `world/entity/vehicle/boat/AbstractBoat.java:109-125`, source boat için
  adayın `canBeCollidedWith(source) || isPushable()` sonucunu kullanır.
  Böylece `canBeCollidedWith=false` Player adayı yalnızca yaşayan ve spectator
  olmayan durumda source vehicle collision'a girebilir.
- `world/level/EntityGetter.java:56-75`, source dispatch sonucunu transient
  section AABB sorgusu ve collision shape listesine bağlar.

## Davranış sözleşmesi

- Non-null vanilla boat source ve query alanındaki Player adayı, Player'ın
  `canBeCollidedWith` sonucu false olsa da yaşayan, spectator olmayan ve
  tırmanmayan durumda pushable collision adayıdır.
- Varsayılan fixture state'i `alive=true, spectator=false` olduğunda boat ile
  çakışan Player collision'a girer.
- Aynı Player `alive=false` olduğunda veya `spectator=true` olduğunda pushable
  kapısı kapanır; entity shape'i sonuçtan çıkarılır.
- Bu dilim `onClimbable()` sonucunu sabit false olan kontrollü fixture ile
  doğrular. Block-state, fall-flying, trapdoor ve game-mode/ability metadata
  bağlamı ayrı açık dallardır.
- Player'ın pinned standing dimensions'ı `0.6 × 1.8` feet-origin kutudur;
  source exclusion, epsilon broad-phase ve transient section lookup mevcut
  `ClientWorld` yolunda korunur.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`world/client-collision/living-pushable-state` kaydı aynı oak boat source ve
Player aday kutusuyla üç gözlem taşır:

| Gözlem | `alive` | `spectator` | Java sonucu |
| --- | --- | --- | --- |
| `alive-player-is-pushable` | `true` | `false` | `no_collision=false`, `no_entity_collision=false` |
| `spectator-player-is-not-pushable` | `true` | `true` | `no_collision=true`, `no_entity_collision=true` |
| `dead-player-is-not-pushable` | `false` | `false` | `no_collision=true`, `no_entity_collision=true` |

Java oracle gerçek `ServerPlayer` tabanlı Player fixture'ını, gerçek Boat
source sınıfını ve transient section getter'ını kullanır. Fixture yalnız
`isAlive`, `isSpectator` ve `onClimbable` durumlarını kontrollü biçimde verir;
beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`73b8477e21977d41f1ce1b03f9b51b31a213238915906f1bed73ed38cc9d2f22`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2931 kayıt
karşılaştırıldı. Debug adapter **2931/2931** eşleşti; fark `null` kaldı. Release
adapter SHA-256 değeri
`06bd9383767cf974f40479d80150e9fb1fa3922ac6f0da9fe2aa745a110946ea` ile aynı
capture üzerinde **2931/2931** eşleşti; fark `null` kaldı.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

Rust tarafında `ClientEntity` canlılık ve spectator state'ini saklar. Indexed
`ClientWorld` collision yolu Player için bu iki kapıyı ve mevcut health metadata
varsa sıfırdan büyük olma koşulunu uygular; `onClimbable` state'i henüz modelde
değildir.

## Açık kapsam

Bu dilim `isPushable()` alive/spectator kapısını kapatır. `onClimbable()` için
block-state/in-block, fall-flying, climbable tag'i ve trapdoor/ladder dalları;
gerçek game-mode/ability packet binding'i; tam EntityType registry/context;
diğer özel collidable türler; `refreshDimensions`, scale ve çözülen movement
AABB dalları açık kalır. Bu kayıt canlı vanilla sunucu testi değildir.
