# P08.bi-entity-passenger-baby-attachment — [server-conformance] Zombie baby `VEHICLE` attachment

Bu kayıt, Zombie'nin `isBaby` entity metadata state'inin boat passenger
attachment hesabına bağlanmasını kapsar. Kanıt sınıfı
`server-conformance`'dır: Rust world kopyası Mojang vanilla 26.2
sunucusundan gelen entity metadata ve passenger ilişkisi üzerinde çalıştırıldı.
Java istemcisiyle aynı oturum state'i karşılaştırılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`Zombie.java:82-84`, Zombie'nin `DATA_BABY_ID` alanını tanımlar;
`141-145` bu Boolean alanı `false` ile başlatır. `175-189` içindeki sorgu ve
setter state'i okur ve yazar. `440-442`, baby Zombie için adult Zombie'nin
normal `EntityType` dimensions'ı yerine `BABY_DIMENSIONS` döndürür.

`Zombie.java:90-93` içindeki baby dimensions `0.49F × 0.98F`, göz yüksekliği
`0.775F` ve `EntityAttachment.VEHICLE` noktası `(0,0.1875F,0)` içerir.
Adult Zombie'nin inherited dimensions'ında özel `VEHICLE` noktası yoktur;
`EntityAttachments` feet fallback'i `(0,0,0)` kullanılır.

`Entity.java:2404-2412` passenger attachment'ını mevcut pose dimensions'ından
okur. `LivingEntity.java:3737-3742` pose ve entity scale sonrasında bu
dimensions state'ini üretir. Boat `positionRider` yolunda passenger vehicle
attachment y'si riding position'dan çıkarılır. Bu nedenle tek passenger'lı oak
boat'ta `0.1875` ride height, adult Zombie için `boatY + 0.1875`, baby Zombie
için `boatY + 0.1875 - 0.1875 = boatY` üretir.

## Java oracle ve Rust sözleşmesi

Pinned client JAR'ındaki `EntityDimensions` ve `EntityAttachments` sınıflarını
doğrudan çağıran Java oracle tekrarlandı. Ayırt edici raw değerler:

| Değer | Java raw bitleri |
| --- | ---: |
| adult feet `VEHICLE` y (f64) | `0000000000000000` |
| baby width (f32) | `3efae148` |
| baby height (f32) | `3f7ae148` |
| baby eye height (f32) | `3f466666` |
| baby `VEHICLE` y (f32) | `3e400000` |

Rust `ClientEntity`, Zombie type ID `151` için metadata data ID `16` ve
Boolean serializer ID `8` değerini `is_baby` state'ine taşır. State `true`
olduğunda attachment hesabı `0.1875`; `false` olduğunda adult feet fallback'i
`0` kullanılır. Partial metadata başka alanları güncellerken önceki baby
state'ini korur; aynı alanın `false` değeri adult yoluna geri döner.

`passenger_tick_uses_zombie_baby_vehicle_attachment_metadata` tek world
senaryosunda adult → baby → adult geçişini kontrol eder. Başlangıç adult
konumu `boatY + 0.1875`, baby konumu `boatY`, reset sonrası adult konumu yine
`boatY + 0.1875` olur.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer oyuncusu yanında sıfır heading'li no-gravity oak boat ve
`IsBaby:1b`, `NoAI:1b`, `NoGravity:1b` özellikli Zombie oluşturuldu ve
vanilla `ride` komutuyla bağlandı:

```text
execute as P08BabyAttach85 at @s run summon minecraft:oak_boat ~5 ~ ~ {Tags:["P08BIV85"],NoGravity:1b,Rotation:[0.0f,0.0f]}
execute as P08BabyAttach85 at @s run summon minecraft:zombie ~5 ~ ~ {Tags:["P08BIP85"],IsBaby:1b,NoAI:1b,NoGravity:1b,PersistenceRequired:1b,Invulnerable:1b,Rotation:[0.0f,0.0f]}
ride @e[type=minecraft:zombie,tag=P08BIP85,limit=1,sort=nearest] dismount
ride @e[type=minecraft:zombie,tag=P08BIP85,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08BIV85,limit=1,sort=nearest]
```

Çalıştırma:

```text
cargo run --locked --offline --example world_entity_passenger_baby_attachment_probe -- 127.0.0.1:25568 P08BabyAttach85
```

Gözlenen sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
vehicle_id=19310
vehicle_type_id=89
vehicle_position=(29.5, 78.0, -30.5)
passenger_id=19311
passenger_type_id=151
passenger_is_baby=true
passenger_position=(29.5, 78.0, -30.5)->(29.5, 78.0, -30.5)
expected_position=(29.5, 78.0, -30.5)
world_entity_count=61
world_chunk_count=157
received_packet_count=18850
```

Vanilla metadata akışındaki Zombie baby state'i Rust world kopyasında
`true` olarak sorgulandı. Bir tick sonrasında passenger y'si boat y'sine eşit
kaldı; bu, baby attachment'ın oak boat ride height'ını tamamen düşürdüğünü
gösterir. Adult Zombie fallback'i P08.bg canlı akışı ve bu kaydın Rust
adult→baby→adult testiyle korunur.

## Açık kapsam

Baby Zombie dışındaki dinamik baby mob dimensions/attachment dalları,
`refreshDimensions()` collision ve pose yan etkileri, scale değişimleri,
entity-specific tick, dismount/collision, render ve Java istemcisiyle
`client-parity` karşılaştırması açık kalır.
