# P09.v — ClientWorld Player flying ability pushability gate — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `Player.onClimbable()` içindeki flying ability
kısa devresinin `ClientWorld` entity collision sorgusuna bağlandığını kanıtlar.
Kanıt sınıfı `client-parity`'dir; canlı sunucu kabulü değildir.

## Java kaynak incelemesi

İncelenen yollar:

- `world/entity/player/Player.java:2024-2025`, Player'ın flying ability alanı
  true ise `onClimbable()` sonucunu doğrudan false yapar; false ise
  `LivingEntity.onClimbable()` yoluna döner.
- `world/entity/LivingEntity.java:3375-3377`, canlılık, spectator ve
  `onClimbable()` sonucunu birlikte `isPushable()` kararına bağlar.
- `world/entity/vehicle/boat/AbstractBoat.java:109-125`, boat source için
  adayın `canBeCollidedWith()` veya `isPushable()` sonucunu kabul eder.
- `world/level/EntityGetter.java:56-75`, source dispatch sonucunu transient
  section AABB sorgusuna taşır.

## Davranış sözleşmesi

- Yaşayan ve spectator olmayan Player adayı tırmanılabilir durumda flying false
  iken pushable değildir; overlapping boat source collision sonucundan çıkar.
- Aynı adayın flying ability değeri true olduğunda Player override'ı
  `onClimbable()` sonucunu false yapar ve aday pushable collision'a geri girer.
- `can_be_collided_with=false` olan Player ile aynı vanilla boat source, iki
  durumda da source dispatch'inin ayırt edici girdisidir.
- Bu kayıt, ability state'inin kontrollü world-copy girdisiyle bağlanmasını
  doğrular. Gerçek Player abilities packet'i ve block-state/tag çözümü bu
  kaydın açık kapsamıdır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`world/client-collision/living-pushable-flying` kaydı aynı oak boat source ve
Player adayıyla iki gözlem taşır:

| Gözlem | `on_climbable` | `flying` | Java sonucu |
| --- | --- | --- | --- |
| `climbable-grounded-player-is-not-pushable` | `true` | `false` | `no_collision=true`, `no_entity_collision=true` |
| `climbable-flying-player-is-pushable` | `true` | `true` | `no_collision=false`, `no_entity_collision=false` |

Java oracle gerçek protocol entity type registry dimensions'ını, gerçek boat
source sınıfını ve transient section getter'ını kullanır. Player fixture'ı
`isAlive`, `isSpectator`, kontrollü `onClimbable` sonucu ve flying kısa devresini
ayrı state alanlarıyla verir; beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`01db9a0fd386ac39eed257081a501931db246c9a9368e56ee50acb9fbc2e9b11`.

Capture iki Java çalıştırmasında kararlı kaldı; bu yeni kayıtla math corpus'u
2933 kayıt olarak doğrulandı. Rust implementasyonu öncesinde aynı capture'a karşı
ilk fark `world/client-collision/living-pushable-flying` kaydının ikinci state'inde
çıktı: Java `no_collision=false`, eski Rust `no_collision=true`.

## Rust doğrulaması

Rust `ClientEntity` flying ability state'ini saklar. Indexed world adapter'ı bu
state'i Player entity'ye uygular; collision dispatch'i tırmanılabilir yaşayan
Player için `flying || !on_climbable` kuralını kullanır.

Debug adapter SHA-256 değeri
`6468350eff3ab3b90ab6e9beb623521b75236f9ac8215ad7183b1990642eb84e` ile
**2933/2933** eşleşti; fark `null` kaldı. Release karşılaştırması aynı capture
üzerinde **2933/2933** eşleşti; fark `null` kaldı. Release adapter SHA-256
değeri `2275c16da65c1cf90eb4c45f807f692b407a3bc0ba6002825a4e78dbbe6821ab`.

Çalıştırılan temel kontroller:

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --example behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
cargo fmt --all -- --check
```

Ek entegrasyon smoke testi, bu parity kaydından ayrı `server-conformance`
olarak Mojang vanilla 26.2 sunucusunda çalıştırıldı:

```text
cargo run --locked --offline -- 127.0.0.1:25568 LiveFlying 10 10
client_connection=ok
play_login=true
world_constructed=true
world_chunk_count=24
world_section_count=576
ticks_run=10
world_game_time=10
local_player_tick_count=10
rendered_frames=0
```

Sunucu SHA-1'i `823e2250d24b3ddac457a60c92a6a941943fcd6a` olarak sabittir. Bu
smoke testi vanilla'nın Rust bağlantısını kabul ettiğini ve world state'in
kurulduğunu gösterir; Java istemcisi parity'si veya oynanabilir görüntü kanıtı
değildir.

## Açık kapsam

Gerçek Player abilities packet binding'i, gerçek block-state/in-block sorgusu,
`BlockTags.CLIMBABLE` ve `BlockTags.CAN_GLIDE_THROUGH` çözümü, fall-flying,
açık trapdoor ile ladder facing eşleşmesi ve tam game-mode state'i açık kalır.
Tam EntityType registry/context, diğer özel collidable türler,
`refreshDimensions`, scale ve çözülen movement AABB de bu kaydın dışındadır.
