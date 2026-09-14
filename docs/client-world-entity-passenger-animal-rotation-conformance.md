# P08.bc-entity-passenger-animal-rotation — [server-conformance] Boat `Animal` passenger rotation offset

Bu kayıt, boat `positionRider` akışında `Animal` passenger için uygulanan
living body/head rotation ve tam kapasite parite offset'ini kapsar. Kanıt
sınıfı `server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated
server'a bağlanır. Java istemcisiyle aynı girdide state karşılaştırılmadığı
için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`AbstractBoat.java:631-642`, `CAN_TURN_IN_BOATS` tag'inde olmayan passenger'a
önce `deltaRotation` ekler, sonra `clampRotation` çağırır. `clampRotation`
living passenger'da body yaw'ı boat yaw'ına, head yaw'ı clamp edilmiş current
yaw'a yazar. Passenger `Animal` ve listenin uzunluğu boat kapasitesine
eşitse, entity ID'si çift olduğunda `90`, tek olduğunda `270` derece eklenir;
bu ek değer hem body hem head yaw'a uygulanır.

`Entity.java:2969-2977`, genel entity'de head/body setter'larının no-op
olduğunu; `LivingEntity.java:238-241,498-501,3380-3392`, Animal gibi living
entity'lerin current/old body ve head state'ini tuttuğunu gösterir.
`AbstractBoat.java:755-757` normal boat kapasitesini `2`,
`AbstractChestBoat.java:48-50` chest boat kapasitesini `1` yapar.

Pinned `EntityTypes.java` kayıtları ve `EntityTypeIds.java:12-164` üzerinden
`Animal` alt türlerinin protocol type ID tablosu şöyledir:
`4, 7, 11, 19, 20, 21, 26, 30, 36, 54, 55, 58, 62, 64, 66, 78, 86, 87,
88, 91, 96, 98, 100, 104, 108, 111, 116, 119, 129, 135, 138, 149, 152, 153`.
Cow type ID `30` bu tablodaki canlı witness'tır.

`AbstractBoat.java:132-150` aynı fixture'da Animal passenger'ın iki koltuklu
boat z-offset'ine `+0.2F` eklediğini de gösterir; bu konum attachment dalı
bu kaydın dışındadır.

## Sözleşme

- Tag istisnası dışındaki pinned `Animal` type ID'leri boat rotation clamp'inde
  living body/head setter davranışını kullanır.
- Clamp sonrasında current yaw, boat yaw'a göre `[-105,105]` içinde kalır;
  current/old yaw correction'ı aynı Java f32 düzeltmesini taşır.
- Animal passenger listesi normal boat için `2`, chest boat için `1` olan
  kapasiteye eşitse entity ID paritesiyle body/head'e `90` veya `270` eklenir.
- Tam kapasitede olmayan Animal yine living clamp ile body yaw'ı boat yaw'ına
  ve head yaw'ı clamp edilmiş current yaw'a alır; parite offset'i eklenmez.
- Genel Entity passenger'da head/body no-op davranışı korunur. Breeze tag
  istisnası [P08.bb kaydında](client-world-entity-passenger-boat-turn-exemption-conformance.md)
  tutulur.

## Rust doğrulaması

`VANILLA_ANIMAL_ENTITY_TYPE_IDS` pinned registry tablosunu tek bir kategori
olarak taşır. `vanilla_boat_max_passengers` normal ve chest boat ailelerinin
kapasitesini ayırır. Rotation clamp çağrısı hem seat geometry'si olmayan
Animal passenger fallback'inde hem bilinen passenger konum yolunda çalışır.
`ClientEntity`, Player ve Animal için living head/body state'ini günceller;
generic Entity passenger'da mevcut no-op davranışı korur. Bu kayıtta Animal
vehicle attachment konumu henüz uygulanmaz.

`passenger_tick_applies_animal_rotation_offset_by_entity_parity_at_capacity`
tek parametrik testte çift/tek entity ID'lerinin `90/270` dallarını iki
passenger normal boat ile kontrol eder. Aynı test ailesindeki
`passenger_tick_applies_animal_living_rotation_before_capacity_offset`, tek
passenger normal boat ile chest boat kapasite ayrımını ve living clamp state'ini
kontrol eder. Önceki `passenger_tick_*` testleri de bu değişiklikten sonra
geçmektedir.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer oyuncusunun konumunda no-AI/no-gravity Cow ilk passenger, no-gravity
snowball ikinci passenger olarak oak boat'a bindirildi. İki passenger relation
geldikten sonra Rust probe bir world tick çalıştırdı:

```text
cargo run --locked --offline --example world_entity_passenger_animal_rotation_probe -- 127.0.0.1:25568 P08CowObs52
execute as P08CowObs52 at @s run summon minecraft:oak_boat ~ ~ ~ {Tags:["P08ACV52"],NoGravity:1b,Rotation:[0.0f,0.0f]}
execute as P08CowObs52 at @s run summon minecraft:cow ~ ~ ~ {Tags:["P08ACP52"],NoAI:1b,NoGravity:1b,PersistenceRequired:1b,Rotation:[170.0f,0.0f]}
execute as P08CowObs52 at @s run summon minecraft:snowball ~ ~ ~ {Tags:["P08ASP52"],NoGravity:1b,Motion:[0.0d,0.0d,0.0d]}
ride @e[type=minecraft:cow,tag=P08ACP52,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08ACV52,limit=1,sort=nearest]
ride @e[type=minecraft:snowball,tag=P08ASP52,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08ACV52,limit=1,sort=nearest]
```

Gözlenen sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
vehicle_id=12616
vehicle_type_id=89
vehicle_y_rot=0
passenger_ids=[12617, 12618]
passenger_id=12617
passenger_type_id=30
passenger_rotation=(168.75, 0.0, 168.75)->(105.0, 0.0, 375.0)
passenger_body_y_rotation=168.75->270
passenger_position=(26.5, 62.0, -37.5)->(26.5, 62.0, -37.5)
passenger_observation_count=1
world_entity_count=59
world_chunk_count=157
received_packet_count=19484
```

Oak boat type ID `89`, Cow type ID `30`, tam relation, odd entity ID `12617`,
head `375` ve body `270` çıktıları Java parity dalıyla eşleşti. Position
çıktısının değişmemesi, Animal vehicle attachment konumunun bu küçük rotation
dilimine henüz dahil edilmediğini gösterir; bu sonuç o konum dalı için kanıt
sayılmaz.

## Açık kapsam

Animal passenger vehicle attachment noktaları ve iki passenger'da
`AbstractBoat.getPassengerAttachmentPoint` içindeki `+0.2F` offset'i, farklı
Animal attachment vector'ları, nonzero `deltaRotation`, `onPassengerTurned`,
diğer living türleri, collision/dismount, render ve Java istemcisiyle
`client-parity` karşılaştırması açık kalır. Seat geometry [P08.az kaydında](client-world-entity-passenger-boat-family-conformance.md),
genel clamp [P08.ba kaydında](client-world-entity-passenger-rotation-conformance.md)
ve Breeze istisnası [P08.bb kaydında](client-world-entity-passenger-boat-turn-exemption-conformance.md)
izlenir.
