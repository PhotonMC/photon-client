# P08.az-entity-passenger-boat-family — [server-conformance] Chest boat ve raft `positionRider`

Bu kayıt, `ClientWorld` passenger tick'inin ardından chest boat, bamboo raft
ve bamboo chest raft üzerindeki tek snowball passenger'ın Java
`positionRider` geometrisiyle taşınmasını kapsar. Kanıt sınıfı
`server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated server'a
bağlanır. Java istemcisiyle aynı girdide state karşılaştırılmadığı için bu
kayıt `client-parity` değildir.

## Java kaynak incelemesi

`AbstractBoat.java:132-150`, passenger noktası için tek passenger'da
`getSinglePassengerXOffset()` değerini, tekne ride yüksekliğini ve tekne
yaw'ına göre dönen z-offset'i birleştirir. `AbstractChestBoat.java:43-50`,
chest boat ailesinde tek passenger offset'ini `0.15F` yapar ve kapasiteyi bir
passenger ile sınırlar.

`Boat.java:15-17` ve `ChestBoat.java:15-17`, `0.5625F / 3.0F` ride height
hesabını kullanır. `Raft.java:15-17` ve `ChestRaft.java:15-17`, aynı entity
dimensions yüksekliğini `0.8888889F` ile çarpar; Java f32 hesabı bu fixture'da
`0.5` verir. `EntityTypes.java:149-157,197-213,219-227,273-289,326-342,
598-614,655-671,703-719,734-750,938-954`, bütün ahşap boat/chest boat ve
bamboo raft/chest raft kayıtlarının aynı `1.375 × 0.5625` dimensions değerini
taşıdığını gösterir.

## Sözleşme

- Tek passenger'lı normal ahşap boat türleri `[0, 12, 23, 33, 74, 81, 89,
  94, 125]` için ride height `0.1875`, z-offset `0`'dır.
- Tek passenger'lı chest boat türleri `[1, 13, 24, 34, 75, 82, 90, 95,
  126]` için ride height `0.1875`, z-offset Java `0.15F` değerinin f64'e
  genişletilmiş karşılığı `0.15000000596046448`'dir.
- Bamboo raft type `9` için ride height `0.5`, z-offset `0`'dır; bamboo chest
  raft type `8` için ride height `0.5`, z-offset `0.15000000596046448`'dir.
- Offset, teknenin yaw'ı ile Java `Vec3.yRot(float)` yolunda döndürülür ve
  passenger vehicle attachment noktası sonuçtan çıkarılır.
- Bu dilim tek passenger geometrisini kapsar. Çoklu passenger sırası, Player
  attachment'ı ve tekne dönüşünün passenger rotation yan etkileri ilgili açık
  kapsam olarak kalır.

## Rust doğrulaması

`VANILLA_BOAT_PASSENGER_GEOMETRY` tablosu, normal boat/chest boat/raft/chest
raft type ID'lerini ride height ve tek passenger z-offset'iyle eşler. Kaynakta
f32 olan `0.15F` ve `0.8888889F` hesapları önce f32 olarak tutulur, Java'daki
entity konumuna benzer biçimde hedef f64 hesabına sonra genişletilir.

`passenger_tick_positions_all_vanilla_boat_geometry_rows` tek parametrik testte
20 tablo satırını yürütür. Her fixture aynı world tick'inde vehicle ve snowball
oluşturur; passenger'ın eski konumu ve seat hedefi karşılaştırılır. Bu test
Java source'tan türetilen geometri tablosunu kullanır ve canlı server sonucunu
başarı ölçütü olarak yeniden üretmez.

`world_entity_passenger_boat_family_probe` aynı probe'u beklenen vehicle type,
ride height ve offset argümanlarıyla chest boat veya raft ailesine uygular.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken her fixture observer oyuncusunun konumunda oluşturuldu,
snowball tekneye bindirildi ve Rust probe SetPassengers sonrasında bir world
tick çalıştırdı.

Oak chest boat:

```text
cargo run --locked --offline --example world_entity_passenger_boat_family_probe -- 127.0.0.1:25568 P08ChestGeom26 90 0.1875 0.15000000596046448
execute as P08ChestGeom26 at @s run summon minecraft:oak_chest_boat ~ ~ ~ {Tags:["P08CGV26"],NoGravity:1b}
execute as P08ChestGeom26 at @s run summon minecraft:snowball ~ ~ ~ {Tags:["P08CGP26"],NoGravity:1b}
ride @e[type=minecraft:snowball,tag=P08CGP26,limit=1,sort=nearest] mount @e[type=minecraft:oak_chest_boat,tag=P08CGV26,limit=1,sort=nearest]
```

| Gözlem | Sonuç |
| --- | ---: |
| `vehicle_id` | `8146` |
| `vehicle_type_id` | `90` |
| `vehicle_position` | `(23.5,63.0,-39.5)` |
| `passenger_id` / type | `8147` / `120` |
| `passenger_position` | `(23.5,63.0,-39.5) → (23.5,63.1875,-39.349999994039536)` |

Bamboo raft:

```text
cargo run --locked --offline --example world_entity_passenger_boat_family_probe -- 127.0.0.1:25568 P08RaftGeom26 9 0.5 0.0
execute as P08RaftGeom26 at @s run summon minecraft:bamboo_raft ~ ~ ~ {Tags:["P08RGV26"],NoGravity:1b}
execute as P08RaftGeom26 at @s run summon minecraft:snowball ~ ~ ~ {Tags:["P08RGP26"],NoGravity:1b}
ride @e[type=minecraft:snowball,tag=P08RGP26,limit=1,sort=nearest] mount @e[type=minecraft:bamboo_raft,tag=P08RGV26,limit=1,sort=nearest]
```

| Gözlem | Sonuç |
| --- | ---: |
| `vehicle_id` | `8297` |
| `vehicle_type_id` | `9` |
| `vehicle_position` | `(21.5,63.0,-39.5)` |
| `passenger_id` / type | `8298` / `120` |
| `passenger_position` | `(21.5,63.0,-39.5) → (21.5,63.5,-39.5)` |

Bamboo chest raft:

```text
cargo run --locked --offline --example world_entity_passenger_boat_family_probe -- 127.0.0.1:25568 P08CRGeom26 8 0.5 0.15000000596046448
execute as P08CRGeom26 at @s run summon minecraft:bamboo_chest_raft ~ ~ ~ {Tags:["P08CRV26"],NoGravity:1b}
execute as P08CRGeom26 at @s run summon minecraft:snowball ~ ~ ~ {Tags:["P08CRP26"],NoGravity:1b}
ride @e[type=minecraft:snowball,tag=P08CRP26,limit=1,sort=nearest] mount @e[type=minecraft:bamboo_chest_raft,tag=P08CRV26,limit=1,sort=nearest]
```

| Gözlem | Sonuç |
| --- | ---: |
| `vehicle_id` | `8444` |
| `vehicle_type_id` | `8` |
| `vehicle_position` | `(13.5,70.0,-25.5)` |
| `passenger_id` / type | `8445` / `120` |
| `passenger_position` | `(13.5,70.0,-25.5) → (13.5,70.5,-25.349999994039536)` |

Üç canlı sonuç da Java source'tan hesaplanan height ve f32 offset ile
eşleşir. Bu kayıt vanilla server kabulü ve Rust world uygulamasını gösterir;
Java istemcisiyle tam parity iddiası değildir.

## Açık kapsam

Chest boat kapasite reddi, çoklu passenger ve animal offset dalı, boat yaw'ın
seat dışındaki passenger body/head rotation yan etkileri, `deltaRotation`,
vehicle control, pose/scale/dimensions'a bağlı passenger attachment'ları,
genel entity tick/lifecycle, collision, render ve Java
istemcisiyle `client-parity` karşılaştırması açık kalır. Normal boat ve
snowball seat geometry [P08.ax kaydında](client-world-entity-passenger-position-conformance.md),
default passenger attachment [P08.bg kaydında](client-world-entity-passenger-generic-position-conformance.md),
Player vehicle attachment [P08.ay kaydında](client-world-entity-passenger-player-conformance.md)
ve passenger tick/rideTick sırası [P08.aw kaydında](client-world-entity-passenger-tick-conformance.md)
izlenir.
