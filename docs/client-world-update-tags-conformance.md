# P09.z — ClientWorld `ClientboundUpdateTagsPacket` block tag binding — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 common `ClientboundUpdateTagsPacket` içindeki
`minecraft:block` payload'ının block registry ID'lerinden `ClientWorld` state
tag üyeliğine uygulanmasını kanıtlar. Kanıt sınıfı `client-parity`'dir; canlı
sunucu smoke sonucu aşağıda ayrı adjunct olarak tutulur.

## Java kaynak incelemesi

İncelenen yollar:

- `network/protocol/common/ClientboundUpdateTagsPacket.java:12-40`, packet'in
  registry key → network payload map'ini taşır ve `handleUpdateTags` callback'ini
  çağırır.
- `tags/TagNetworkSerialization.java:48-56,59-73`, payload içindeki tag
  Identifier'larını `TagKey`'e, signed registry entry ID'lerini registry
  holder'larına çözer.
- `client/multiplayer/ClientPacketListener.java:1814-1825`, play phase'te
  her payload'ı registry üzerinde `prepareTagReload(...).apply()` ile uygular.
- `client/multiplayer/ClientConfigurationPacketListenerImpl.java:80-89` ve
  `client/multiplayer/RegistryDataCollector.java:30-49,144-151`, configuration
  phase'te tags payload'ını play'e geçişte uygulanacak collector'a ekler.
- `world/entity/LivingEntity.java:1721-1730`, uygulanan block tag'lerini
  `onClimbable()` ve fall-flying kısa devresinde sorgular.

## Davranış sözleşmesi

- `minecraft:block` payload'ındaki değerler block registry ID'sidir; block
  state ID'si değildir. Pinned 26.2'de air `0`, stone `1`, ladder `221` ve
  vine `366` block ID'leridir.
- Rust registry, desteklenen state ailesini block ID'lerine çözer: ladder
  state'leri `5719–5726`, vine default state'i `8389` ve stone state'i `1`.
- Packet uygulaması `minecraft:climbable` ve
  `minecraft:can_glide_through` tag kümelerini payload'daki değerlerle
  yeniler; ilgili tag payload'da yoksa o tag'in izlenen üyeliği temizlenir.
- Tag uygulamasından sonra grounded vine ve ladder state'leri yaşayan,
  spectator olmayan Player'ı pushability collision'ından çıkarır; fall-flying
  vine `CAN_GLIDE_THROUGH` kısa devresi nedeniyle collision'a girer.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`world/client-collision/living-pushable-update-tags-packet` kaydı tek
`minecraft:block` packet payload'ı ve aynı oak boat source ile üç gözlem taşır:

| Gözlem | State ID | Fall-flying | Payload tag değerleri | Java sonucu |
| --- | ---: | :---: | --- | --- |
| `packet-vine-grounded-player-is-not-pushable` | `8389` | `false` | `climbable=[221,366]`, `can_glide_through=[366]` | `no_collision=true`, `no_entity_collision=true` |
| `packet-vine-fall-flying-player-is-pushable` | `8389` | `true` | `climbable=[221,366]`, `can_glide_through=[366]` | `no_collision=false`, `no_entity_collision=false` |
| `packet-ladder-grounded-player-is-not-pushable` | `5720` | `false` | `climbable=[221,366]`, `can_glide_through=[366]` | `no_collision=true`, `no_entity_collision=true` |

Java oracle her girdide gerçek `ClientboundUpdateTagsPacket` nesnesini,
`TagNetworkSerialization.NetworkPayload`'ı ve built-in block registry'nin
`prepareTagReload(...).apply()` yolunu kullanır. `Block.stateById` state'i
çözer; payload değerleri block registry ID alanında bırakılır. Gerçek Player,
boat ve transient section collision çağrıları korunur. Beklenen sonuç Rust
çıktısından türetilmez.

Java referans kimliği:
`76d0b66bd958bde3113be2d4d67f9a0b62d3d751cfb69c0d9bca9690c5bb2223`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2937 kayıt
karşılaştırıldı.

## Rust doğrulaması

`ClientBlockStateRegistry::vanilla_26_2` desteklenen pinned state → block
registry ID eşleşmelerini tutar. `ClientWorld::apply_update_tags_packet`,
decoded `ClientboundUpdateTagsPacket` içindeki block registry payload'ını
`CLIMBABLE` ve `CAN_GLIDE_THROUGH` metadata'sına bağlar. Vanilla connection
probe configuration phase'te aldığı packet'ı world kopyasına uygular.

Debug adapter SHA-256 değeri
`d4f3fb83d078b3f093f6d605160ab510ee4d4ca4e0b70f43014030e70cb65669` ile
**2937/2937** eşleşti; fark `null` kaldı.
Release adapter SHA-256 değeri
`743f11a8a1086487cc1044e7bd3d65ce2d4b238a8f145f1c7e7d99e84ae8890a` ile
aynı capture üzerinde **2937/2937** eşleşti; fark `null` kaldı.

Çalıştırılan kontroller:

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo test --locked --offline client_world_applies_block_update_tags_to_state_registry
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

## Vanilla server adjunct smoke

Mojang vanilla 26.2 server jar SHA-1'i
`823e2250d24b3ddac457a60c92a6a941943fcd6a` olan yerel server
`127.0.0.1:25568` üzerinde `LiveTags` adıyla çalıştırıldı. Rust client
`update_tags_packet_count=1`, `play_login=true`, `world_constructed=true`, 24
chunk, 576 section ve 10 tick (`world_game_time=10`,
`local_player_tick_count=10`) gözledi; world copy'sinde
`block_tag_vine_climbable=true` ve `block_tag_vine_glide_through=true` kaldı.
Server logu login, join ve disconnect olaylarını kaydetti.

Bu smoke sonucu `server-conformance` adjunct'ıdır; packet tag binding'inin
Java istemcisiyle tam eşitlik kanıtı olarak kullanılmaz.

## Açık kapsam

Tam 32366 state → block registry eşleşmesi, vine dışındaki bütün tag üyeleri,
generic registry tag uygulaması, configuration collector'ın gerçek phase
birleştirme zamanlaması, play phase common update-tags dispatch'i ve packet
ile güncellenen tag'lerin tüm gameplay/render çağıranları bu kaydın dışındadır.
