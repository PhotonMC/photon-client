# P09.y — ClientWorld LivingEntity `CAN_GLIDE_THROUGH` fall-flying branch — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.onClimbable()` içindeki fall-flying
ve `BlockTags.CAN_GLIDE_THROUGH` kısa devresinin pinned block-state registry
üzerinden `ClientWorld` entity collision sorgusuna bağlandığını kanıtlar.
Kanıt sınıfı `client-parity`'dir; canlı sunucu kabulü değildir.

## Java kaynak incelemesi

İncelenen yollar:

- `world/entity/LivingEntity.java:1721-1730`, mevcut block state'i önce
  `BlockTags.CAN_GLIDE_THROUGH` ile sınar; entity fall-flying ise bu dal
  `onClimbable()` sonucunu false yapar, aksi halde `CLIMBABLE` üyeliğine
  devam eder.
- `data/tags/VanillaBlockTagsProvider.java:252-260,293-300`, vine ailesinin hem
  `CLIMBABLE` hem `CAN_GLIDE_THROUGH` tag'lerine bağlandığını listeler.
- `world/level/block/Block.java:127-138` ve
  `core/MappedRegistry.java:397-421`, state ID çözümünü ve client tag
  reload'ında holder tag'lerinin yeniden bağlanmasını tanımlar.
- `world/entity/LivingEntity.java:3375-3377`, `onClimbable()` sonucunu
  `isPushable()` kapısına taşır.
- `world/entity/vehicle/boat/AbstractBoat.java:109-125` ve
  `world/level/EntityGetter.java:56-75`, Player adayını source boat collision
  dispatch'ine ve transient section AABB sorgusuna bağlar.

## Davranış sözleşmesi

- Pinned 26.2 vine default state ID'si `8389`, iki tag'in de üyesidir;
  fall-flying olmayan yaşayan Player bu durumda `onClimbable()` üzerinden
  pushable değildir ve overlapping vanilla boat collision sonucundan çıkar.
- Aynı vine state'inde Player fall-flying olduğunda
  `CAN_GLIDE_THROUGH` kısa devresi `onClimbable()` sonucunu false yapar;
  Player pushable olur ve entity collision'a girer.
- Stone state `1`, `CAN_GLIDE_THROUGH` veya `CLIMBABLE` üyesi değildir;
  fall-flying Player bu durumda da pushable olur.
- Rust world copy'si bu dilimde pinned vine state ID'sini iki tag metadata
  kümesinde tutar ve fall-flying shared flag'ini entity state'ine uygular.
  Gerçek tag packet binding'i ve bütün vanilla block-state ailesi bu dilimin
  açık kapsamıdır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`world/client-collision/living-pushable-glide-through` kaydı aynı oak boat
source ve Player adayıyla üç gözlem taşır:

| Gözlem | State ID | Fall-flying | Java sonucu |
| --- | ---: | :---: | --- |
| `vine-grounded-player-is-not-pushable` | `8389` | `false` | `no_collision=true`, `no_entity_collision=true` |
| `vine-fall-flying-player-is-pushable` | `8389` | `true` | `no_collision=false`, `no_entity_collision=false` |
| `stone-fall-flying-player-is-pushable` | `1` | `true` | `no_collision=false`, `no_entity_collision=false` |

Java oracle gerçek `Block.stateById`, vine default state ID'sini ve
`ServerPlayer` `onClimbable()` çağrısını kullanır. Her gözlemde gerçek vine
holder'ı iki tag'e, negatif stone gözleminde boş tag setine
`MappedRegistry.prepareTagReload(...).apply()` yoluyla bağlanır; gerçek Player,
boat ve transient section collision çağrıları korunur. Beklenen sonuç Rust
çıktısından türetilmez.

Java referans kimliği:
`44b1b687bc8bc9539e50f1650334e657b1794df8f0d1924d0573027da37ec97a`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2936 kayıt
karşılaştırıldı.

## Rust doğrulaması

`ClientBlockStateRegistry::vanilla_26_2` vine state ID'sini
`BlockTags.CLIMBABLE` ve `BlockTags.CAN_GLIDE_THROUGH` metadata'sına bağlar.
`ClientWorld` iki üyeliği ve fall-flying durumunu
`LivingEntity.onClimbable` eşdeğerine uygular; indexed behavior adapter context
state ID'sini ve fall-flying değerini aynı setter'a iletir.

Debug adapter SHA-256 değeri
`31a58d0654e75abb28e6b5416394dc7fec6c1ea454fdbd0e4b38cde4fa160406` ile
**2936/2936** eşleşti; fark `null` kaldı.
Release adapter SHA-256 değeri
`3f6ff5a30b300e9a8e2bb2a95b20b855e2d8ff61ac8d339a988d34f0e20656df` ile
aynı capture üzerinde **2936/2936** eşleşti; fark `null` kaldı.

Çalıştırılan kontroller:

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo test --locked --offline indexed_player_collision_respects_glide_through_tag
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

## Vanilla server adjunct smoke

Mojang vanilla 26.2 server jar SHA-1'i
`823e2250d24b3ddac457a60c92a6a941943fcd6a` olan yerel server
`127.0.0.1:25568` üzerinde `LiveGlide` adıyla çalıştırıldı. Rust client
bağlantısı kabul edildi; `play_login=true`, `world_constructed=true`, 24 chunk,
576 section ve 10 tick (`world_game_time=10`, `local_player_tick_count=10`)
gözlendi. Server logu login, join ve disconnect olaylarını kaydetti.

Bu smoke sonucu `server-conformance` adjunct'ıdır; `CAN_GLIDE_THROUGH`
fall-flying davranışının Java istemcisiyle eşitlik kanıtı olarak kullanılmaz.

## Açık kapsam

Gerçek `ClientboundUpdateTagsPacket` binding'i, vine dışındaki climbable ve
glide-through tag üyeleri, tam 32366 state registry'si, block position'dan
entity'nin gerçek `getInBlockState()` sorgusu, Player abilities,
`refreshDimensions`, scale ve çözülen movement AABB bu kaydın dışındadır.
