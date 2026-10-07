# P09.x — ClientWorld LivingEntity `BlockTags.CLIMBABLE` branch — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.onClimbable()` içindeki
`state.is(BlockTags.CLIMBABLE)` dalının pinned block-state registry üzerinden
`ClientWorld` entity collision sorgusuna bağlandığını kanıtlar. Kanıt sınıfı
`client-parity`'dir; canlı sunucu kabulü değildir.

## Java kaynak incelemesi

İncelenen yollar:

- `world/entity/LivingEntity.java:1721-1730`, spectator ve fall-flying
  kontrollerinden sonra mevcut block state'in `BlockTags.CLIMBABLE` üyeliğini
  sorgular.
- `data/tags/VanillaBlockTagsProvider.java:252-260`, ladder'ın vanilla
  climbable tag'inin bir üyesi olduğunu ve aynı tag'e ait diğer block ailelerini
  listeler.
- `world/level/block/Block.java:127-138` ve
  `core/MappedRegistry.java:397-421`, state ID çözümünü ve client tag
  reload'ında holder tag'lerinin yeniden bağlanmasını tanımlar.
- `world/entity/LivingEntity.java:3375-3377`, `onClimbable()` sonucunu
  `isPushable()` kapısına taşır.
- `world/entity/vehicle/boat/AbstractBoat.java:109-125` ve
  `world/level/EntityGetter.java:56-75`, Player adayını source boat collision
  dispatch'ine ve transient section AABB sorgusuna bağlar.

## Davranış sözleşmesi

- Pinned 26.2 ladder state ID'leri `5719–5726`, block state'in
  `BlockTags.CLIMBABLE` üyeliği nedeniyle `onClimbable()` sonucunu true yapar;
  `WATERLOGGED` ve `FACING` state değerleri bu tag branch'ini değiştirmez.
- Stone state `1` ve air state `0` tag üyesi değildir; aynı yaşayan,
  spectator olmayan Player bu durumda pushable olarak collision'a girer.
- Tag üyesi Player'ın `can_be_collided_with=false` olması, overlapping vanilla
  boat source için entity collision sonucundan çıkmasını sağlar.
- Rust world copy'si bu dilimde pinned ladder state ID'lerinin tag üyeliğini
  registry metadata'sında tutar. Tam tag packet binding'i ve bütün vanilla
  climbable block state ailesi bu dilimin açık kapsamıdır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`world/client-collision/living-pushable-climbable-tag` kaydı aynı oak boat
source ve Player adayıyla tek parametrik akışta dört gözlem taşır:

| Gözlem | State ID | Java sonucu |
| --- | ---: | --- |
| `ladder-state-is-climbable` | `5720` | `no_collision=true`, `no_entity_collision=true` |
| `waterlogged-ladder-state-is-climbable` | `5719` | `no_collision=true`, `no_entity_collision=true` |
| `stone-state-is-not-climbable` | `1` | `no_collision=false`, `no_entity_collision=false` |
| `air-state-is-not-climbable` | `0` | `no_collision=false`, `no_entity_collision=false` |

Java oracle gerçek `Block.stateById` ve ladder state ID'lerini kullanır. Her
gözlemde `BlockTags.CLIMBABLE`, gerçek built-in ladder holder'ına veya negatif
 kontrol için boş tag setine `MappedRegistry.prepareTagReload(...).apply()`
 yoluyla bağlanır; gerçek Player, boat ve transient section collision
 çağrıları korunur. Beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`dd248f039e4d7d8c554650901f6f18bea0c8185707258fff39431a9cd99412bc`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2935 kayıt
karşılaştırıldı.

## Rust doğrulaması

`ClientBlockStateRegistry::vanilla_26_2` ladder'ın sekiz pinned state ID'sini
`BlockTags.CLIMBABLE` metadata'sına bağlar. `ClientWorld` bu üyeliği yaşayan
entity state'ine uygular; indexed behavior adapter state ID'sini aynı setter'a
iletir ve mevcut alive/spectator, flying ve boat source dispatch yollarını
korur.

Debug adapter SHA-256 değeri
`37b4016c9b9db083b217fc154b82377bc7834a3636cd6471d55f0e14a6e23abb` ile
**2935/2935** eşleşti; fark `null` kaldı.
Release adapter SHA-256 değeri
`6ce898d9d94600187df3fbf76321b30a8a8b8fcb47bda304bc0c3b9984227220` ile
aynı capture üzerinde **2935/2935** eşleşti; fark `null` kaldı.

Çalıştırılan kontroller:

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo test --locked --offline indexed_player_collision_respects_climbable_block_state_tag
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

Gerçek `ClientboundUpdateTagsPacket` binding'i, ladder dışındaki climbable tag
üyeleri, tam 32366 state registry'si, block position'dan entity'nin gerçek
`getInBlockState()` sorgusu, `BlockTags.CAN_GLIDE_THROUGH`, fall-flying, Player
abilities, `refreshDimensions`, scale ve çözülen movement AABB bu kaydın
dışındadır.
