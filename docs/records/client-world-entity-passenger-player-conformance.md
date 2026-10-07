# P08.ay-entity-passenger-player — [server-conformance] Player vehicle attachment

Bu kayıt, normal ahşap boat passenger'ı olan Player state'inin Java
`positionRider` hesaplamasında Player'a ait vehicle attachment noktasının
uygulanmasını kapsar. Kanıt sınıfı `server-conformance`'dır: Rust probe'u
Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle aynı
girdide state karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientLevel.java:479-489`, Player passenger'ı ticking entity olmasa bile
`rideTick` yoluna alır. `Entity.java:2380-2417`, tick sonrasında aracın
passenger riding position'ını hesaplar ve passenger'ın vehicle attachment
noktasını bu konumdan çıkararak `setPos` çağrısı yapar.

`Avatar.java:17-23` ve `EntityTypes.java:1163-1172`, Player standing state için
vehicle attachment'ı `(0,0.6,0)` olarak tanımlar. Normal boat için
`Boat.java:15-17` ride height `0.5625 / 3 = 0.1875` olduğundan, yaw sıfırken
Player passenger'ın feet position'ı boat position'ına göre `y - 0.4125` olur.

## Sözleşme

- Normal ahşap boat'a bağlı remote Player, passenger tick'inde bir kez işlenir
  ve `rideTick` hareket sıfırlamasından sonra seat konumuna taşınır.
- Standing Player için vehicle attachment y bileşeni `0.6` çıkarılır;
  normal oak boat tek passenger seat'i için sonuç boat konumundan
  `(0, -0.4125, 0)` farkıdır.
- Passenger'ın tick başındaki `previous_position` değeri korunur; seat
  konumu current position'a yazılır.
- Aynı attachment hesabı local player state'ine de uygulanır.
- Bu dilim normal ahşap boat ve standing Player attachment'ını kapsar.
  Crouching/swimming pose attachment ayrıntıları, chest boat/raft, boat yaw
  yan etkileri, vehicle control, diğer living entity türleri ve
  `client-parity` açık kapsamdır.

## Rust doğrulaması

`ClientWorld::tick`, local player passenger'ı için parent vehicle ID'sini
taşıyarak `tick_local_player` çağrısını yapar. Remote Player entity'si için
aynı vehicle ID `tick_entity_tree` yoluna verilir. Ortak
`position_entity_passenger`, Player attachment y değerini snowball'ın feet
attachment'ından ayırır ve sonucu `ClientEntity` veya `ClientPlayerState`
konumuna yazar.

`passenger_tick_positions_remote_player_from_vehicle_attachment` unit testi
remote Player'ı `(2,12,2)` ve oak boat'ı `(2,2,2)` konumunda başlatır; sonucu
`(2,1.5875,2)` olarak kontrol eder. `passenger_tick_positions_local_player_from_vehicle_attachment`
aynı sonucu local player world state'inde kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer ve remote Player mover probe'ları birlikte açıkken mover observer'ın
konumuna taşındı, yanında no-gravity oak boat oluşturuldu ve remote Player
boat'a bindirildi:

```text
cargo run --locked --offline --example world_entity_player_passenger_position_probe -- 127.0.0.1:25568 P08AyObs26 P08AyMove26
tp P08AyMove26 P08AyObs26
execute as P08AyMove26 at @s run summon minecraft:oak_boat ~ ~ ~ {Tags:["P08AYV26"],NoGravity:1b}
ride P08AyMove26 mount @e[type=minecraft:oak_boat,tag=P08AYV26,limit=1,sort=nearest]
```

Vanilla konsolu Player'ın oak boat'a bindiğini bildirdi. Observer Rust
probe'u remote Player passenger ilişkisinden sonra bir world tick çalıştırdı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `mover_movement_packets_sent` | `1` |
| `vehicle_id` | `7709` |
| `vehicle_type_id` | `89` |
| `vehicle_position` | `(27.5,75.0,-27.5)` |
| `passenger_id` | `7708` |
| `passenger_type_id` | `156` |
| `passenger_position` | `(26.5,77.0,-30.5) → (27.5,74.5875,-27.5)` |
| `passenger_observation_count` | `1` |

Beklenen seat y değeri `75.0 + 0.1875 - 0.6 = 74.5875` ile canlı sonuç
aynıdır. Bu kayıt vanilla server kabulü ve Rust world uygulamasını gösterir;
Java istemcisiyle tam parity iddiası değildir.

## Açık kapsam

Player'ın crouching/swimming pose dimensions'ı, local player'ın gerçek input ve
movement yoluyla tekneye binmesi, boat `deltaRotation` ile passenger body/head
rotation, chest boat/raft geometrisi, pose/scale/dimensions'a bağlı generic
attachment, vehicle control, collision, render ve Java istemcisiyle
`client-parity` karşılaştırması açık kalır. Default generic attachment [P08.bg kaydında](client-world-entity-passenger-generic-position-conformance.md)
izlenir.
Animal dışındaki living body/head dispatch'i [P08.bf kaydında](client-world-entity-passenger-living-conformance.md)
izlenir. Normal boat ve
snowball seat geometry [P08.ax kaydında](client-world-entity-passenger-position-conformance.md),
passenger tick/rideTick sırası [P08.aw kaydında](client-world-entity-passenger-tick-conformance.md)
izlenir.
