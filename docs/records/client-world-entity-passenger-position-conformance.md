# P08.ax-entity-passenger-position — [server-conformance] `positionRider` tekne koltuğu

Bu kayıt, `ClientWorld` passenger tick'inin ardından normal ahşap tekneye
bağlı snowball'ın Java `positionRider` geometrisiyle taşınmasını kapsar. Kanıt
sınıfı `server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated
server'a bağlanır. Java istemcisiyle aynı girdide state karşılaştırılmadığı
için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:2380-2412`, `rideTick` içinde hareketi sıfırlayıp entity tick'ini
çalıştırdıktan sonra passenger ise aracın `positionRider` yolunu çağırır.
Konumlandırma, aracın passenger riding position'ından passenger'ın kendi
vehicle attachment noktasını çıkarır; tick başındaki eski transform bu
çağrıyla değiştirilmez.

`Entity.java:2402-2417` ve `EntityAttachments.java:47-70`, varsayılan
passenger attachment'ının araç rotasyonuyla çevrildiğini ve araç attachment'ı
olmayan entity için feet noktasının `(0,0,0)` olduğunu belirler. Snowball bu
varsayılan feet attachment'ını kullanır.

`AbstractBoat.java:132-150,622-624`, normal boat passenger noktası için tek
passenger offset'ini sıfır, birden fazla passenger'da sıralı offset'leri
`0.2` ve `-0.6` olarak seçer. `Boat.java:15-17`, ride yüksekliğini boat
dimensions yüksekliğinin üçte biri olarak hesaplar. `EntityTypes.java` içindeki
normal ahşap boat kayıtları aynı `1.375 × 0.5625` dimensions değerini taşır;
oak boat type ID'si 89'dur.

## Sözleşme

- Bağlı normal ahşap boat, `ClientWorld::tick` içinde kök olarak bir kez
  işlenir; snowball passenger `rideTick` hareket sıfırlaması ve projectile
  tick'inden sonra konumlandırılır.
- Normal boat passenger olan snowball için tek passenger koltuk noktası
  vehicle konumuna `(0, 0.1875, 0)` eklenerek hesaplanır.
- Birden fazla passenger state'i varsa normal boat'ın Java sıralı z-offset'i
  uygulanır ve vehicle yaw ile döndürülür.
- Passenger'ın tick başındaki `previous_position` değeri korunur; yalnız
  current position `positionRider` sonucuna taşınır.
- Bu dilim yalnız normal ahşap boat ve snowball passenger geometrisini
  uygular. Player attachment, chest boat, raft, boat rotation/yaw yan
  etkileri, vehicle control ve diğer passenger entity geometrileri açık
  kapsamdır.

## Rust doğrulaması

`ClientWorld::tick`, passenger entity'sini tick ettikten hemen sonra
`position_entity_passenger` çağrısını yapar. Yardımcı normal ahşap boat type
ID tablosunu, passenger sırasını ve Java'nın yaw ile dönen seat offset'ini
uygular. Snowball'ın feet attachment'ı sıfır olduğu için oak boat tek
passenger sonucu boat konumuna `0.1875` y ekler.

`passenger_tick_positions_snowball_on_regular_boat_seat` unit testi oak boat'ı
`(2,2,2)`, snowball'ı `(2,12,2)` konumunda başlatır. Bir world tick sonrası
passenger'ın eski konumunun korunduğunu, current konumunun `(2,2.1875,2)`
olduğunu ve ride tick movement reset'inden sonra snowball gravity sonucunun
korunduğunu kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken entity'ler observer oyuncusunun konumunda oluşturuldu
ve snowball oak boat'a bindirildi:

```text
cargo run --locked --offline --example world_entity_passenger_position_probe -- 127.0.0.1:25568 P08AxPosObs26
execute as P08AxPosObs26 at @s run summon minecraft:oak_boat ~ ~ ~ {Tags:["P08AXV26"],NoGravity:1b}
execute as P08AxPosObs26 at @s run summon minecraft:snowball ~ ~ ~ {Tags:["P08AXP26"],NoGravity:1b,Motion:[0.125d,0.0d,0.0d]}
ride @e[type=minecraft:snowball,tag=P08AXP26,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08AXV26,limit=1,sort=nearest]
```

Vanilla konsolu snowball'ın oak boat'a bindiğini bildirdi. Rust probe'u
uygulanan ilişki packet'inden sonra bir world tick çalıştırdı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `vehicle_id` | `7276` |
| `vehicle_type_id` | `89` |
| `vehicle_position` | `(27.5, 70.0, -36.5)` |
| `passenger_id` | `7277` |
| `passenger_type_id` | `120` |
| `passenger_position` | `(27.5, 70.0, -36.5) → (27.5, 70.1875, -36.5)` |
| `passenger_motion` | `(0.12500762986022096, 0.0, 0.0) → (0.0, 0.0, 0.0)` |
| `passenger_position_observation_count` | `1` |

Aynı Java branch'inin birden fazla passenger yolunu kontrol etmek için iki
snowball aynı oak boat'a sırayla bindirildi:

```text
cargo run --locked --offline --example world_entity_passenger_multi_position_probe -- 127.0.0.1:25568 P08AxMultiObs27
execute as P08AxMultiObs27 at @s run summon minecraft:oak_boat ~ ~ ~ {Tags:["P08AXMV27"],NoGravity:1b}
execute as P08AxMultiObs27 at @s run summon minecraft:snowball ~ ~ ~ {Tags:["P08AXM127"],NoGravity:1b}
execute as P08AxMultiObs27 at @s run summon minecraft:snowball ~ ~ ~ {Tags:["P08AXM227"],NoGravity:1b}
ride @e[type=minecraft:snowball,tag=P08AXM127,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08AXMV27,limit=1,sort=nearest]
ride @e[type=minecraft:snowball,tag=P08AXM227,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08AXMV27,limit=1,sort=nearest]
```

Rust probe'u ikinci ilişki packet'inde iki passenger ID'sini aldı ve aynı
world tick'inde sıralı seat offset'lerini uyguladı:

| Gözlem | Sonuç |
| --- | ---: |
| `passenger_ids` | `[7564, 7565]` |
| birinci passenger position | `(23.5, 63.1875, -41.3)` |
| ikinci passenger position | `(23.5, 63.1875, -42.1)` |
| `passenger_observation_count` | `1` |

Bu canlı kayıt vanilla server'ın passenger tick sonrası koltuk konumunu ve
Rust world kopyasındaki uygulamayı gösterir; Java istemcisiyle tam parity
iddiası değildir.

## Açık kapsam

Pose/scale/dimensions'a bağlı entity vehicle attachment noktaları, chest boat/raft ride
yükseklikleri, boat `deltaRotation` ve passenger head/body rotation, vehicle
control, nested passenger geometry, collision ve genel entity lifecycle açık
kalır. Default Entity ve generic living attachment [P08.bg kaydında](client-world-entity-passenger-generic-position-conformance.md)
izlenir. Bu kayıt P08.aw passenger tick sırasını genişletir; ilişki state'i
[P08.av kaydında](client-world-entity-passengers-conformance.md), passenger
tick/rideTick davranışı [P08.aw kaydında](client-world-entity-passenger-tick-conformance.md)
izlenir. Java istemcisiyle `client-parity` karşılaştırması ayrıca açıktır.
