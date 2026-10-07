# P08.be-entity-passenger-turn — [server-conformance] `Entity.turn` ve boat `onPassengerTurned`

Bu kayıt, passenger'ın doğrudan döndürülmesi sonrasında vehicle callback'inin
çalışmasını kapsar. Java `Entity.turn`, bağlı vehicle'ın
`onPassengerTurned` yolunu çağırır; boat bu callback'te rotation clamp'i
`CAN_TURN_IN_BOATS` kontrolü olmadan uygular. Kanıt sınıfı
`server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated
server'a bağlanır ve relation state'i aldıktan sonra aynı client-world turn
girdisini uygular. Java istemcisiyle aynı girdide state karşılaştırılmadığı
için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:490-501`, `turn(double xo, double yo)` içinde mouse deltasını
önce Java f32 olarak `xDelta = (float) yo * 0.15F` ve
`yDelta = (float) xo * 0.15F` biçiminde üretir. Current x/y rotation'ı,
ardından old x/y rotation'ı günceller; current ve old x rotation'ı
`[-90,90]` aralığına sıkıştırır. Entity'nin vehicle'ı varsa dönüşün sonunda
`vehicle.onPassengerTurned(this)` çağrılır.

`AbstractBoat.java:677-688`, boat callback'inin `clampRotation` yolunu
çağırdığını gösterir. Bu callback, `positionRider` içindeki
`CAN_TURN_IN_BOATS` koşulunu tekrar kullanmaz: passenger ile boat arasındaki
wrapped yaw farkını `[-105F,105F]` aralığına indirir, correction'ı current ve
old yaw'a ekler, head yaw'ı current yaw'a ve living passenger body yaw'ını
boat yaw'ına yazar. `AbstractBoat.java:631-642` içindeki tag istisnası yalnız
aynı tick'teki `positionRider` dönüş bloğuna aittir.

`Breeze.java:41` Breeze'in `Monster`, dolayısıyla living entity olduğunu
gösterir. Bu nedenle Breeze tick sırasında tag istisnasıyla dönüşünü korurken
doğrudan `turn` callback'inde living head/body state'i clamp sonucuna alınır.
Animal kapasite paritesi callback'in kendisinde eklenmez; `90/270` offset'i
P08.bc'de izlenen `positionRider` dalındadır.

## Sözleşme

- `turn_entity(id, xo, yo)`, Java `Entity.turn` ile aynı f32 delta ve current/
  old x/y rotation sırasını kullanır.
- Bağlı vehicle vanilla boat ailesindeyse `onPassengerTurned` callback'i
  çağrılır ve yaw correction current ile old passenger yaw'a uygulanır.
- Callback yolunda wrapped passenger/boat yaw farkı `[-105,105]` içinde
  kalır; Player ve Breeze için head yaw current yaw'a, body yaw boat yaw'ına
  eşitlenir.
- Breeze'in `positionRider` tick istisnası doğrudan turn callback'ine
  taşınmaz. Böylece Breeze tick'te serbest, doğrudan dönüşte clamp edilmiş
  olur.
- Callback passenger'ı yeniden konumlandırmaz. Passenger position state'i
  aynı kalır ve Animal kapasite `90/270` offset'i uygulanmaz.
- Boat olmayan vehicle için Entity'in varsayılan no-op callback davranışı
  korunur. `deltaRotation` ile boat input/control akışı bu kaydın dışındadır.

## Java oracle ve Rust doğrulaması

Pinned Java 26.2 JAR'ındaki `Mth.wrapDegrees` ve `Mth.clamp` sınıfları,
`initial=168.75F`, `vehicle=0F`, `xo=-100F`, `yo=0F` girdisiyle callback
clamp hesabını çalıştırdı:

```text
x_delta=0.0
y_delta=-15.000001
after_rotation=105.0
after_previous_rotation=105.0
after_head=105.0
after_body=0.0
```

`ClientWorld::turn_entity`, `ClientEntity` ve `ClientPlayerState` bu dönüş
sırasını taşır. `passenger_turn_calls_boat_on_passenger_turned` Player
passenger'da callback clamp'ini, current/previous rotation ve position'ın
korunmasını kontrol eder. `passenger_turn_clamps_breeze_despite_boat_turn_exemption`
aynı gövdeyi Breeze ile kontrol ederek P08.bb'nin tick exemption dalından
farklı callback sonucunu sabitler.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer oyuncusunun konumunda no-AI/no-gravity Breeze ve oak boat oluşturuldu,
Breeze boat'a bindirildi. Rust probe relation packet'inden sonra bir world
tick çalıştırdı; bu tick'te P08.bb dalı Breeze rotation'ını değiştirmedi.
Ardından probe, Breeze üzerinde `xo=-100`, `yo=0` turn girdisini uyguladı:

```text
cargo run --locked --offline --example world_entity_passenger_turn_probe -- 127.0.0.1:25568 P08BreezeTurn57
execute as P08BreezeTurn57 at @s run summon minecraft:oak_boat ~ ~ ~ {Tags:["P08BETV57"],NoGravity:1b,Rotation:[0.0f,0.0f]}
execute as P08BreezeTurn57 at @s run summon minecraft:breeze ~ ~ ~ {Tags:["P08BETP57"],NoAI:1b,NoGravity:1b,PersistenceRequired:1b,Rotation:[170.0f,0.0f]}
ride @e[type=minecraft:breeze,tag=P08BETP57,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08BETV57,limit=1,sort=nearest]
```

Gözlenen sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
vehicle_id=14339
vehicle_type_id=89
vehicle_y_rot=0
passenger_id=14340
passenger_type_id=17
before_rotation=(168.75, 0.0, 168.75)
before_body_y_rotation=168.75
after_rotation=(105.0, 0.0, 105.0)
after_previous_rotation=(105.0, 0.0, 168.75)
after_body_y_rotation=0
passenger_position=(22.5, 69.0, -36.5)->(22.5, 69.0, -36.5)
passenger_observation_count=1
world_entity_count=77
world_chunk_count=157
received_packet_count=16298
```

Oak boat type `89`, Breeze type `17` ve tam relation alındı. Tick öncesi
`168.75` olan Breeze yaw/head state'i doğrudan turn sonrasında `105` oldu;
previous head yaw tarihsel `168.75` olarak, position değişmeden kaldı; body
yaw boat'ın `0` yaw'ına geldi. Bu gözlem Rust world kopyasının vanilla
sunucusundan alınan relation/entity state üzerinde source-derived callback
dalını uyguladığını gösterir; Java istemcisiyle tam parity iddiası değildir.

## Açık kapsam

Boat input/control kaynaklı nonzero `deltaRotation`, mouse input'un gerçek
pencere callback'inden `turn_entity` yoluna bağlanması, dismount/collision,
nested/lifecycle, render ve Java istemcisiyle `client-parity` karşılaştırması
açık kalır. Animal dışındaki living body/head state'i [P08.bf kaydında](client-world-entity-passenger-living-conformance.md)
izlenir.
P08.bb tick exemption [kendi kaydında](client-world-entity-passenger-boat-turn-exemption-conformance.md),
genel passenger clamp [P08.ba kaydında](client-world-entity-passenger-rotation-conformance.md),
Animal rotation [P08.bc kaydında](client-world-entity-passenger-animal-rotation-conformance.md)
ve Animal seat position [P08.bd kaydında](client-world-entity-passenger-animal-position-conformance.md)
izlenir.
