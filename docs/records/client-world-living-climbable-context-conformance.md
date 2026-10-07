# P09.w — ClientWorld LivingEntity trapdoor/ladder onClimbable context — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.onClimbable()` içindeki açık
trapdoor ve altındaki aynı yöne bakan ladder bağlamının `ClientWorld` entity
collision sorgusuna bağlandığını kanıtlar. Kanıt sınıfı `client-parity`'dir;
canlı sunucu kabulü değildir.

## Java kaynak incelemesi

İncelenen yollar:

- `world/entity/LivingEntity.java:1721-1748`, spectator kısa devresinden sonra
  block state'i okur; fall-flying/glide-through ve `BlockTags.CLIMBABLE`
  dallarından sonra trapdoor yardımcı metoduna geçer.
- `world/entity/LivingEntity.java:1741-1748`, trapdoor'un `OPEN` değerini,
  altındaki state'in `Blocks.LADDER` olmasını ve `FACING` eşitliğini birlikte
  değerlendirir.
- `world/level/block/TrapDoorBlock.java:40-52` ve
  `world/level/block/LadderBlock.java:25-29`, ilgili state property'lerini ve
  default state'lerini tanımlar.
- `world/entity/LivingEntity.java:3375-3377`, `onClimbable()` sonucunu
  `isPushable()` kapısına taşır.
- `world/entity/vehicle/boat/AbstractBoat.java:109-125` ve
  `world/level/EntityGetter.java:56-75`, Player adayını source boat collision
  dispatch'ine ve transient section AABB sorgusuna bağlar.

## Davranış sözleşmesi

- Yaşayan ve spectator olmayan Player'ın içindeki trapdoor açık, altındaki
  block ladder ve iki state'in facing değeri eşitse `onClimbable()` true olur.
- Bu durumda `isPushable()` false olur; `can_be_collided_with=false` Player,
  overlapping vanilla boat source için entity collision sonucundan çıkar.
- Trapdoor kapalıysa, altta ladder yoksa veya facing değerleri farklıysa
  `onClimbable()` false olur; Player pushable olarak collision'a girer.
- Rust world copy'si bu dilimde packet'ten gelen trapdoor/ladder context
  descriptor'ını aynı üç koşula bağlar. Tam block-state registry ve ağdan
  property çözümü bu dilimin açık kapsamıdır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`world/client-collision/living-pushable-trapdoor-context` kaydı aynı oak boat
source ve Player adayıyla tek parametrik akışta dört gözlem taşır:

| Gözlem | Trapdoor | Alt state | Facing | Java sonucu |
| --- | --- | --- | --- | --- |
| `open-matching-ladder-player-is-not-pushable` | açık | ladder | north/north | `no_collision=true`, `no_entity_collision=true` |
| `closed-trapdoor-player-is-pushable` | kapalı | ladder | north/north | `no_collision=false`, `no_entity_collision=false` |
| `open-mismatched-ladder-player-is-pushable` | açık | ladder | north/south | `no_collision=false`, `no_entity_collision=false` |
| `open-without-ladder-player-is-pushable` | açık | air | north/north | `no_collision=false`, `no_entity_collision=false` |

Java oracle gerçek `ServerPlayer.onClimbable()` ve `LivingEntity` metodlarını,
gerçek `Blocks.OAK_TRAPDOOR`/`Blocks.LADDER` state property'lerini, gerçek
boat source sınıfını ve transient section getter'ını kullanır. Fixture yalnız
`ServerLevel.getBlockState(pos.below())` için deterministik komşu state verir;
beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`b65ddb07f9d5d3f01a1236bec697129666c0bb40d41c496fffb9a2caa7cb738e`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2934 kayıt
karşılaştırıldı. Rust implementasyonu öncesinde ilk fark bu kaydın açık ve
eşleşen trapdoor satırında çıktı: Java `no_collision=true`, eski Rust
`no_collision=false`.

## Rust doğrulaması

`ClientWorld::apply_entity_living_climbable_trapdoor_state` açık trapdoor,
ladder varlığı ve facing eşitliğini `ClientEntity.on_climbable` state'ine
uygular. Indexed behavior adapter aynı descriptor'ı world copy'ye iletir;
mevcut Player flying, alive, spectator ve boat source dispatch yolları
korunur.

Debug adapter SHA-256 değeri
`91eb07ba294119c01348cabecd0269dfa377b4caa51b38d686982c24576335d0` ile
**2934/2934** eşleşti; fark `null` kaldı.
Release adapter SHA-256 değeri
`3cfe30bc371059d6ebf2420741a1399b1406573a1822f89f383030ac469b7ac8` ile
aynı capture üzerinde **2934/2934** eşleşti; fark `null` kaldı.

Çalıştırılan kontroller:

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo test --locked --offline indexed_player_collision_respects_trapdoor_ladder_context
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

Mojang vanilla 26.2 sunucusuna yapılan ek smoke testi `server-conformance`
sınıfındadır ve bu block context parity'si yerine geçmez:

```text
cargo run --locked --offline -- 127.0.0.1:25568 LiveTrapdoor 10 10
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

Vanilla jar SHA-1'i
`823e2250d24b3ddac457a60c92a6a941943fcd6a` olarak sabittir. Bu smoke testi
bağlantı ve ClientWorld kurulmasını gösterir; trapdoor oynanışı veya Java
istemcisi parity'si göstermez.

## Açık kapsam

`BlockTags.CLIMBABLE` ve `BlockTags.CAN_GLIDE_THROUGH` registry/tag çözümü,
fall-flying kısa devresi, gerçek ClientWorld block-state property lookup'ı,
trapdoor/ladder context'in ilgili packet ve chunk state'lerinden üretilmesi,
Player abilities binding'i, `refreshDimensions`, scale ve çözülen movement
AABB bu kaydın dışındadır.
