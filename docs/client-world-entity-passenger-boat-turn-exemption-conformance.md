# P08.bb-entity-passenger-boat-turn-exemption — [server-conformance] Breeze `CAN_TURN_IN_BOATS`

Bu kayıt, Breeze passenger'ının boat `positionRider` içindeki yaw/head/body
düzeltme dalından çıkarılmasını kapsar. Kanıt sınıfı
`server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated server'a
bağlanır. Java istemcisiyle aynı girdide state karşılaştırılmadığı için bu
kayıt `client-parity` değildir.

## Java kaynak incelemesi

`AbstractBoat.java:631-642`, `passenger.is(EntityTypeTags.CAN_TURN_IN_BOATS)`
koşuluyla yaw'a `deltaRotation` ekleyen, head yaw'ı değiştiren ve
`clampRotation` çağıran bloğu koşullu çalıştırır. Tag'deki passenger bu bloğu
tamamen atlar; genel `positionRider` seat/attachment konumu ayrı olarak
uygulanır.

`EntityTypeTags.java:25-27`, `CAN_TURN_IN_BOATS` tag anahtarını tanımlar.
Pinned data provider (`data/tags/EntityTypeTagsProvider.java:178-184`) bu
tag'e yalnız `EntityTypeIds.BREEZE` ekler. `EntityTypes.java:246-249` Breeze
kaydını, `EntityTypeIds.java:25` resource key'ini ve protocol registry kayıt
sırasındaki type ID `17` karşılığını verir.

## Sözleşme

- Oak boat ve P08.az boat family araçlarında Breeze passenger rotation'ı
  `positionRider` clamp dalına girmez.
- Boat `deltaRotation` Breeze yaw/head state'ine eklenmez; current ve previous
  yaw, passenger tick'in kendi entity davranışı dışında korunur.
- Bu pinned sürümde `CAN_TURN_IN_BOATS` tablosunda ölçülen tek type ID `17`
  Breeze'dir. Yeni entity type varyantı açılmaz; değişiklik aynı tag tablosuna
  satır olarak eklenir.
- Seat riding position ve Breeze vehicle attachment hesabı bu rotation
  istisnasından bağımsızdır ve ayrı kapsamdır.

## Rust doğrulaması

`VANILLA_BREEZE_ENTITY_TYPE_ID` `17` olarak tanımlanır.
`position_entity_passenger`, Breeze için rotation clamp fallback'ini atlar;
diğer tanınmayan passenger türleri için P08.ba'nın generic rotation yolu
korunur. Breeze seat geometry'si henüz ayrı bir attachment row'u olarak
uygulanmaz.

`passenger_tick_leaves_breeze_rotation_unclamped` testi oak boat type `89` ve
Breeze type `17` için `(170,0,170)` current/previous rotation state'ini bir
passenger tick sonrasında korur.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

No-AI/no-gravity Breeze ve oak boat observer oyuncusunun konumunda oluşturuldu.
Hostile entity'nin peaceful world tarafından temizlenmemesi için fixture öncesi
difficulty `normal` yapıldı ve Breeze boat'a bindirildi:

```text
cargo run --locked --offline --example world_entity_passenger_rotation_exemption_probe -- 127.0.0.1:25568 P08Breeze46
difficulty normal
execute as P08Breeze46 at @s run summon minecraft:oak_boat ~ ~ ~ {Tags:["P08BEV46"],NoGravity:1b,Rotation:[0.0f,0.0f]}
execute as P08Breeze46 at @s run summon minecraft:breeze ~ ~ ~ {Tags:["P08BEP46"],NoAI:1b,NoGravity:1b,PersistenceRequired:1b,Rotation:[170.0f,0.0f]}
ride @e[type=minecraft:breeze,tag=P08BEP46,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08BEV46,limit=1,sort=nearest]
```

Gözlenen sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
vehicle_id=11703
vehicle_type_id=89
vehicle_y_rot=0
passenger_id=11704
passenger_type_id=17
passenger_rotation=(168.75, 0.0, 168.75)->(168.75, 0.0, 168.75)
passenger_body_y_rotation=168.75->168.75
passenger_observation_count=1
world_entity_count=84
world_chunk_count=157
received_packet_count=20281
```

Rust world state'i, vanilla AddEntity rotation quantization'ı olan `168.75`
değerini korudu; Breeze type ID `17` ve oak boat type ID `89` doğrulandı.
Bu sonuç server'ın probe paketini ve bağlı entity akışını kabul ettiğini
gösterir; Java istemcisiyle tam parity iddiası değildir.

## Açık kapsam

Breeze vehicle attachment/seat position, Breeze AI ve boat dışı turn
davranışı, dinamik tag çözümü, diğer LivingEntity türleri, nonzero boat
`deltaRotation`, collision, dismount, render ve Java istemcisiyle
`client-parity` karşılaştırması açık kalır. Breeze'in doğrudan turn callback'i
[P08.be kaydında](client-world-entity-passenger-turn-conformance.md), genel
boat rotation clamp [P08.ba kaydında](client-world-entity-passenger-rotation-conformance.md),
seat geometry'si [P08.az kaydında](client-world-entity-passenger-boat-family-conformance.md)
izlenir.
