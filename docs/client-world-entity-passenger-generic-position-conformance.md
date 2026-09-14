# P08.bg-entity-passenger-generic-position — [server-conformance] Default vehicle attachment

Bu kayıt, boat `positionRider` yolunun Player, Snowball ve Animal dışındaki
passenger türlerinde Java'nın varsayılan `VEHICLE` attachment'ını uygulamasını
kapsar. Kanıt sınıfı `server-conformance`'dır: Rust probe'u Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:2380-2417`, passenger `rideTick` sonrasında vehicle'ın
`positionRider` yolunu çağırdığını; bu yolun vehicle passenger riding
position'ından passenger `VEHICLE` attachment'ını çıkardığını gösterir.
`EntityAttachment.java:6-26` içindeki `VEHICLE` fallback'i feet noktasıdır ve
`EntityAttachments.java:44-80` attachment tanımlı değilse bunu `(0,0,0)` olarak
üretir.

Bu default yol LivingEntity için de geçerlidir; `LivingEntity` genel bir
vehicle attachment override'ı yapmaz. Pinned registry'de Item type ID `71` ve
Zombie type ID `151` bu yolu iki farklı runtime sınıfıyla gösterir. `Player`
type ID `156` için `(0,0.6,0)` attachment'ı P08.ay'de ayrıca uygulanır.

`Spider.java:178-181` Spider'ın boat yeterince geniş değilse
`(0,0.3125,0)` attachment'ı kullandığını, `CaveSpider.java:59-62` ise kendi
daha küçük genişliği nedeniyle pinned boat ailesinde default feet yoluna
düştüğünü gösterir. `EntityTypes.java` içindeki pinned boat/raft kayıtlarının
genişliği `1.375F`, Spider'ın genişliği `1.4F` olduğundan Spider özel dalı
boat ailesinin tamamında çalışır. Bu hesap varsayılan scale ve standing pose
ile sınırlıdır.

## Sözleşme

- Pinned boat ailesindeki passenger için vehicle riding position hesaplanır ve
  passenger'ın `VEHICLE` attachment y bileşeni bundan çıkarılır.
- Default Entity attachment kullanan Item ve Zombie için attachment
  `(0,0,0)`'dır; oak boat tek passenger seat'i `(0,0.1875,0)` ekler.
- Spider type ID `124` için boat genişlik koşulu nedeniyle attachment y'si
  `0.3125`'tir ve oak boat tek passenger sonucu boat y'sinden `0.125` aşağıda
  kalır. CaveSpider pinned boat genişliğinde default feet davranışına düşer.
- Passenger'ın tick başındaki `previous_position` değeri korunur; yalnız
  current position `positionRider` sonucu ile değiştirilir.
- Player, Animal, chest boat/raft ve seat rotation yolları mevcut ilgili
  kayıtlarındaki özel dallarla aynı kalır.

## Java oracle ve Rust doğrulaması

P08.ax'te çalıştırılan pinned Java 26.2 oracle'ı, AbstractBoat'ın oak boat
ride height'ını `0.5625F / 3.0F = 0.1875` olarak verdi. Bu dilim yeni bir seat
arithmetic varyantı açmadı; yeni gözlem, aynı Java `positionRider` hesabının
default Entity attachment dispatch'iyle daha önce erken dönen passenger
türlerine uygulanmasıdır. Spider özel attachment değeri Java kaynak dalı ve
tek parametrik Rust testindeki type table ile karşılaştırıldı.

`passenger_tick_positions_generic_entity_vehicle_attachment` tek testte Item
`71`, Zombie `151` ve Spider `124` satırlarını aynı oak boat girdisiyle
kontrol eder. Beklenen current y değerleri sırasıyla `2.1875`, `2.1875` ve
`1.875`'tir; başlangıç `previous_position` her satırda `(2,12,2)` olarak
korunur.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer oyuncusundan beş blok uzakta no-AI/no-gravity/invulnerable Zombie ve
oak boat oluşturuldu. Summon sırasında oluşan ilk otomatik ilişki düşürülüp
yeniden kuruldu; böylece yeni relation packet'i Rust world kopyasına ulaştı:

```text
cargo run --locked --offline --example world_entity_passenger_generic_position_probe -- 127.0.0.1:25568 P08GenericPos75
execute as P08GenericPos75 at @s run summon minecraft:oak_boat ~5 ~ ~ {Tags:["P08BGV75"],NoGravity:1b,Rotation:[0.0f,0.0f]}
execute as P08GenericPos75 at @s run summon minecraft:zombie ~5 ~ ~ {Tags:["P08BGP75"],NoAI:1b,NoGravity:1b,PersistenceRequired:1b,Invulnerable:1b,Rotation:[170.0f,0.0f]}
ride @e[type=minecraft:zombie,tag=P08BGP75,limit=1,sort=nearest] dismount
ride @e[type=minecraft:zombie,tag=P08BGP75,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08BGV75,limit=1,sort=nearest]
```

Gözlenen sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
vehicle_id=16138
vehicle_type_id=89
vehicle_position=(28.5, 74, -25.5)
passenger_id=16139
passenger_type_id=151
passenger_position=(28.5, 74.0, -25.5)->(28.5, 74.1875, -25.5)
passenger_motion=(0.0, 0.0, 0.0)->(0.0, 0.0, 0.0)
passenger_position_observation_count=1
world_entity_count=62
world_chunk_count=157
received_packet_count=16950
```

Oak boat type `89` ve Zombie type `151` doğrulandı. Passenger current konumu
boat seat y'sine `0.1875` eklenerek güncellendi; tick başı konumu history'de
korundu. Bu sonuç Rust world kopyasının vanilla sunucusundan gelen
relation/entity state üzerinde source-derived default attachment dalını
uyguladığını gösterir; Java istemcisiyle tam parity iddiası değildir.

## Açık kapsam

Pose/scale/dimensions değişimleri, custom entity attachment'ları, nonzero boat
`deltaRotation`, vehicle control, collision/dismount, nested passenger ve
lifecycle, gerçek pencere input callback'i, render ve Java istemcisiyle
`client-parity` karşılaştırması açık kalır. Passenger living body/head state'i
[P08.bf kaydında](client-world-entity-passenger-living-conformance.md), boat
rotation clamp'i [P08.ba kaydında](client-world-entity-passenger-rotation-conformance.md),
seat family geometry'si [P08.az kaydında](client-world-entity-passenger-boat-family-conformance.md)
ve Player attachment'ı [P08.ay kaydında](client-world-entity-passenger-player-conformance.md)
izlenir.
