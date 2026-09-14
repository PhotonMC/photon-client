# P08.bd-entity-passenger-animal-position — [server-conformance] Animal passenger `positionRider` seat attachment

Bu kayıt, boat `positionRider` akışında pinned `Animal` passenger için
uygulanan varsayılan vehicle attachment ve iki passenger seat offset'ini
kapsar. Kanıt sınıfı `server-conformance`'dır: Rust probe'u Mojang vanilla
26.2 dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:2380-2405`, passenger `rideTick` içinde hareketi sıfırlayıp
entity tick'ini çalıştırdıktan sonra vehicle'ın `positionRider` yolunu çağırır.
Bu yol, vehicle'ın passenger riding position'ından passenger'ın
`getVehicleAttachmentPoint` sonucunu çıkararak current konumu yazar.
Passenger'ın tick başındaki eski konumu bu çağrıdan önce saklanır.

`Entity.java:2403-2405`, `EntityAttachment.java:6-26` ve
`EntityAttachments.java:44-80`, `VEHICLE` attachment'ı yoksa feet fallback'inin
`(0,0,0)` olduğunu ve attachment noktalarının vehicle yaw'ıyla döndürüldüğünü
gösterir. Pinned Animal sınıflarında `getVehicleAttachmentPoint` override'ı
yoktur; `EntityTypes.java` içindeki Animal `passengerAttachments` kayıtları
Animal'ın kendi passenger taşıdığı konumu tanımlar, Animal'ın bir vehicle
üzerindeki attachment'ını değiştirmez.

`AbstractBoat.java:132-150`, passenger listesi birden uzunsa ilk seat için
`0.2F`, sonraki seat için `-0.6F` seçer ve passenger `Animal` ise seçilen
offset'e `0.2F` ekler. `AbstractBoat.java:755-757` normal boat kapasitesini
`2`, `AbstractChestBoat.java:48-50` chest boat kapasitesini `1` yapar.
`Boat.java:15-17` normal boat ride yüksekliğini dimensions yüksekliğinin
üçte biri olarak hesaplar; oak boat dimensions yüksekliği `0.5625F` olduğu
için ride height `0.1875` olur.

P08.bc'de sabitlenen protocol `Animal` type ID tablosu burada da kullanılır:
`4, 7, 11, 19, 20, 21, 26, 30, 36, 54, 55, 58, 62, 64, 66, 78, 86, 87,
88, 91, 96, 98, 100, 104, 108, 111, 116, 119, 129, 135, 138, 149, 152,
153`. Cow type ID `30` canlı witness'tır.

## Sözleşme

- Pinned Animal passenger'ın vehicle attachment'ı varsayılan feet noktası
  `(0,0,0)`'dır; bu yüzden passenger riding position'ından ek bir y
  çıkarılmaz.
- Normal oak boat'ta tek passenger Animal konumu vehicle konumuna
  `(0, 0.1875, 0)` eklenerek hesaplanır.
- İki passenger'lı normal boat'ta ilk Animal için z offset'i Java f32 işlem
  sırasıyla `0.2F + 0.2F = 0.4F` olur. Offset vehicle yaw'ıyla döndürülür.
- Aynı listede ikinci passenger Animal değilse onun z offset'i `-0.6F` olarak
  kalır; Animal ise aynı `+0.2F` dalı uygulanır.
- Passenger'ın tick başındaki `previous_position` değeri korunur; current
  position `positionRider` sonucuna taşınır. Önceki P08.bc rotation clamp
  davranışı bu konumlandırmadan sonra çalışmaya devam eder.

## Rust doğrulaması

`position_entity_passenger`, pinned Animal type ID'lerini varsayılan feet
attachment yoluna alır. Çoklu boat seat hesabı, `0.2F`/`-0.6F` tabanını ve
Animal `+0.2F` ekini Java ile aynı f32 toplama sırasıyla yapıp f64'e genişletir.
Tek passenger offset'i de Java f32 sonucu olarak taşınır. Rotation clamp
ve kapasite/parite dalları [P08.bc kaydında](client-world-entity-passenger-animal-rotation-conformance.md)
izlenen ortak yoldan yürür.

`passenger_tick_positions_animal_on_full_boat_seat_offset` unit testi oak
boat type `89`, Cow type `30` ve ikinci Snowball type `120` ile ilk Animal
seat'inin `(2, 2.1875, 2.0 + (0.2F + 0.2F))` konumunu kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer oyuncusunun konumunda no-AI/no-gravity Cow ilk passenger, no-gravity
Snowball ikinci passenger olarak oak boat'a bindirildi. Rust probe relation
packet'inden sonra bir world tick çalıştırdı:

```text
cargo run --locked --offline --example world_entity_passenger_animal_position_probe -- 127.0.0.1:25568 P08CowPos57
execute as P08CowPos57 at @s run summon minecraft:oak_boat ~ ~ ~ {Tags:["P08BDV57"],NoGravity:1b,Rotation:[0.0f,0.0f]}
execute as P08CowPos57 at @s run summon minecraft:cow ~ ~ ~ {Tags:["P08BDP57"],NoAI:1b,NoGravity:1b,PersistenceRequired:1b,Rotation:[170.0f,0.0f]}
execute as P08CowPos57 at @s run summon minecraft:snowball ~ ~ ~ {Tags:["P08BDS57"],NoGravity:1b,Motion:[0.0d,0.0d,0.0d]}
ride @e[type=minecraft:cow,tag=P08BDP57,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08BDV57,limit=1,sort=nearest]
ride @e[type=minecraft:snowball,tag=P08BDS57,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08BDV57,limit=1,sort=nearest]
```

Canlı sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
vehicle_id=14039
vehicle_type_id=89
vehicle_position=(23.5, 64, -40.5)
passenger_id=14040
passenger_type_id=30
passenger_position=(23.5, 64.0, -40.5)->(23.5, 64.1875, -40.099999994039536)
passenger_observation_count=1
world_entity_count=73
world_chunk_count=157
received_packet_count=17114
```

Oak boat type `89`, Cow type `30` ve tam relation alındı. Cow current konumu
vehicle y'sine `0.1875`, z'sine Java f32 `0.4` offset'ini ekledi; sonuç
`(23.5, 64.1875, -40.099999994039536)` oldu. Bu gözlem Rust world kopyasının
vanilla sunucusundan gelen relation/entity state'ini source-derived
`positionRider` hesabıyla uyguladığını gösterir; Java istemcisiyle tam parity
iddiası değildir.

## Açık kapsam

Bu kayıt pinned Animal türlerinin vehicle attachment ve normal two-passenger
boat seat offset dalını kapatır. Animal'ın kendi passenger taşıdığı özel
`PASSENGER` attachment noktaları, custom pose/dimensions, seat collision ve
dismount, nonzero boat `deltaRotation`, nested/lifecycle durumları, render ve
Java istemcisiyle `client-parity` karşılaştırması açık kalır. Animal dışındaki
living body/head dispatch'i [P08.bf kaydında](client-world-entity-passenger-living-conformance.md)
izlenir. `onPassengerTurned`
[P08.be kaydında](client-world-entity-passenger-turn-conformance.md), chest
boat/raft geometrisi [P08.az kaydında](client-world-entity-passenger-boat-family-conformance.md),
genel rotation clamp [P08.ba kaydında](client-world-entity-passenger-rotation-conformance.md),
Breeze istisnası [P08.bb kaydında](client-world-entity-passenger-boat-turn-exemption-conformance.md)
ve Animal rotation state'i [P08.bc kaydında](client-world-entity-passenger-animal-rotation-conformance.md)
izlenir.
