# P09.u — ClientWorld LivingEntity onClimbable pushability gate — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.isPushable()` içindeki
`!onClimbable()` kapısının `ClientWorld` entity collision sorgusuna
bağlandığını kanıtlar. Kanıt sınıfı `client-parity`'dir; canlı sunucu kabulü
değildir.

## Java kaynak incelemesi

İncelenen temel yollar:

- `world/entity/LivingEntity.java:3375-3377`, pushability sonucunda canlılık ve
  spectator kapılarından sonra `!onClimbable()` koşulunu uygular.
- `world/entity/LivingEntity.java:1721-1748`, spectator kısa devresinden sonra
  fall-flying/glide-through, `BlockTags.CLIMBABLE` ve açık trapdoor'un altındaki
  facing'i ladder ile eşleştiren özel dalı değerlendirir.
- `world/entity/player/Player.java:2024-2025`, Player uçuyorsa `onClimbable()`
  sonucunu false yapar; aksi halde LivingEntity block-state yoluna devreder.
- `world/entity/vehicle/boat/AbstractBoat.java:109-125`, source boat'ın
  `canBeCollidedWith(source) || isPushable()` dispatch'inde bu sonucu kullanır.
- `world/level/EntityGetter.java:56-75`, dispatch sonucunu transient section
  AABB sorgusuna ve collision shape listesine bağlar.

## Davranış sözleşmesi

- Yaşayan, spectator olmayan ve tırmanmayan Player adayı, `canBeCollidedWith`
  false olsa bile overlapping vanilla boat source için pushable collision'a
  girer.
- Aynı Player `onClimbable=true` olduğunda `isPushable()` false olur ve boat
  source collision sonucundan çıkar.
- Bu dilim, Java runtime dalını kontrollü fixture state'iyle izole eder;
  Player'ın gerçek block-state, fall-flying, trapdoor/ladder ve ability
  bağlamı henüz ClientWorld state modeline bağlanmaz.
- Player'ın pinned standing dimensions'ı `0.6 × 1.8` feet-origin kutu olarak
  ve source exclusion/transient section lookup mevcut yoldaki gibi korunur.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`world/client-collision/living-pushable-climbable` kaydı aynı oak boat source
ve Player adayıyla iki gözlem taşır:

| Gözlem | `alive` | `spectator` | `on_climbable` | Java sonucu |
| --- | --- | --- | --- | --- |
| `non-climbable-player-is-pushable` | `true` | `false` | `false` | `no_collision=false`, `no_entity_collision=false` |
| `climbable-player-is-not-pushable` | `true` | `false` | `true` | `no_collision=true`, `no_entity_collision=true` |

Java oracle gerçek `ServerPlayer` tabanlı Player fixture'ını, gerçek Boat
source sınıfını ve transient section getter'ını kullanır. Fixture yalnız
`isAlive`, `isSpectator` ve `onClimbable` dallarını kontrollü biçimde verir;
beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`be45cd4787120d364c00000d9d97e5e53f697c8a55028e656e77d4d03b655691`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2932 kayıt
karşılaştırıldı. Debug adapter SHA-256 değeri
`3b9d45296362a6188daf36396ab23a1ec65a597c3574f02ede5e3110e1d80176` ile
**2932/2932** eşleşti; fark `null` kaldı. Release karşılaştırması aynı capture
üzerinde **2932/2932** eşleşti; fark `null` kaldı. Release adapter SHA-256
değeri `90da2e29864b6b435007a50ad66d81bdc4658e09eee2e81ede42eb3b7bb27fe4`.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

Rust tarafında `ClientEntity` `on_climbable` state'ini saklar ve Player
pushability kararında canlılık/spectator kapılarından sonra bu state'i
uygular. State setter ve indexed `ClientWorld` collision yolu bu dalı aynı
source dispatch'ine bağlar.

## Açık kapsam

Gerçek `onClimbable()` block-state/in-block sorgusu, `BlockTags.CLIMBABLE`
registry/tag çözümü, fall-flying `CAN_GLIDE_THROUGH`, açık trapdoor ile ladder
facing eşleşmesi ve Player flying ability packet binding'i açık kalır. Tam
EntityType registry/context, diğer özel collidable türler, `refreshDimensions`,
scale ve çözülen movement AABB de bu kaydın dışındadır.
