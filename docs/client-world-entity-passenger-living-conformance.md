# P08.bf-entity-passenger-living — [server-conformance] Boat passenger `LivingEntity` body/head state

Bu kayıt, boat passenger rotation clamp'inin Animal, Breeze ve Player dışındaki
living entity türlerinde de Java'nın dinamik setter dispatch'iyle çalışmasını
kapsar. Kanıt sınıfı `server-conformance`'dır: Rust probe'u Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`AbstractBoat.java:677-684`, `clampRotation` içinde body yaw'ı her passenger
entity'sine boat yaw'ıyla yazar, passenger yaw farkını wrapped olarak
`[-105.0F,105.0F]` aralığına indirir, correction'ı current ve old yaw'a ekler
ve head yaw'ı current yaw'a yazar. `Entity.java:2969-2977` temel Entity için
head/body setter'larının no-op olduğunu gösterir; `LivingEntity.java:238-241`
state alanlarını, `3380-3392` ise bu setter'ların gerçek alanları değiştirdiğini
gösterir. Bu nedenle clamp'teki sonuç passenger'ın runtime sınıfına bağlıdır.

`Zombie.java:71`, Zombie'nin Monster'dan; `Mob.java:99-103`, Monster'ın
LivingEntity yolundan geldiğini gösterir. `ArmorStand.java:53` ArmorStand'ın
doğrudan LivingEntity olduğunu, `Avatar.java:13` ise Player ve Mannequin gibi
avatar türlerinin aynı living state'i kullandığını doğrular. Bu yüzden önceki
P08.ba ArmorStand fixture'ı generic Entity kanıtı değildir; düzeltilmiş canlı
çıktı P08.ba kaydında, genel tablo ve test bu kayıtta tutulur.

Pinned `EntityTypeIds.java:7-164` kayıt sırası ile `EntityTypes.java:165-1172`
factory sınıfları ve superclass zincirleri incelendiğinde 26.2'deki living
entity type ID tablosu şöyledir:

`2, 4, 5, 7, 10, 11, 14, 16, 17, 19, 20, 21, 22, 26, 27, 28, 30, 31, 32,
35, 36, 38, 40, 41, 42, 43, 46, 54, 55, 57, 58, 59, 61, 62, 63, 64, 66,
67, 68, 70, 78, 80, 83, 86, 87, 88, 91, 96, 97, 98, 99, 100, 101, 102,
103, 104, 107, 108, 109, 110, 111, 112, 114, 115, 116, 117, 119, 121, 124,
127, 128, 129, 130, 131, 135, 137, 138, 139, 140, 141, 142, 143, 145, 146,
147, 149, 150, 151, 152, 153, 154, 155, 156`.

Zombie type ID `151` bu tablonun Animal ve Breeze dışındaki Mob witness'ıdır.
Non-living `Item` type ID `71` ise aynı clamp yolunda base Entity no-op head/body
davranışını koruyan karşılaştırma türüdür.

## Sözleşme

- Pinned 93 living type ID'sinin boat clamp'i body yaw'ını boat yaw'ına ve head
  yaw'ını clamp edilmiş current yaw'a yazar.
- Current ve previous yaw correction'ı aynı Java f32 işlemiyle uygulanır;
  wrapped passenger/boat yaw farkı `[-105,105]` sınırında kalır.
- Tick içindeki `positionRider` yolu ile doğrudan `Entity.turn` callback'i aynı
  living head/body dispatch'ini kullanır.
- Animal kapasite parite offset'i yalnız P08.bc'deki `positionRider` dalında
  bulunur; doğrudan turn callback'ine eklenmez.
- Breeze'in tick sırasında `CAN_TURN_IN_BOATS` istisnası P08.bb'de kalır;
  doğrudan turn callback'i yine living clamp uygular.
- Non-living passenger'da head/body setter no-op davranışı korunur.

## Java oracle ve Rust doğrulaması

P08.be'de pinned Java 26.2 JAR'ındaki `Mth.wrapDegrees` ve `Mth.clamp` ile
çalıştırılan `xo=-100`, `yo=0`, initial yaw `168.75F`, boat yaw `0F` oracle'ı
aynı clamp arithmetic dalının sayısal sonucunu verdi:

```text
x_delta=0.0
y_delta=-15.000001
after_rotation=105.0
after_previous_rotation=105.0
after_head=105.0
after_body=0.0
```

Bu dilim yeni bir sabit arithmetic varyantı açmadı. Yeni bilgi, aynı Java
dalının Zombie gibi Animal dışı bir `LivingEntity` sınıfında dispatch edilmesi
ve ArmorStand type `5`'in de living olmasıdır. Rust'ta
`VANILLA_LIVING_ENTITY_TYPE_IDS` tek tablo olarak kullanılır; tick ve direct
turn clamp'i `vanilla_passenger_is_living` üzerinden aynı gövde/head state'ini
günceller.

`passenger_turn_clamps_every_vanilla_living_entity_body_and_head` tek
parametrik unit testte tablodaki 93 type ID'sini aynı boat callback girdisiyle
kontrol eder. Test, Animal offset'ini bu callback'e taşımadan living dispatch'i
doğrular.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer oyuncusundan beş blok uzakta no-AI/no-gravity/invulnerable Zombie ve
oak boat oluşturuldu. Zombie, summon sırasında boat'a otomatik bağlandığı için
önce düşürülüp yeniden bindirildi; böylece observer'ın world kopyasına yeni
relation paketi ulaştı:

```text
cargo run --locked --offline --example world_entity_passenger_living_turn_probe -- 127.0.0.1:25568 P08ZombieTurn78
execute as P08ZombieTurn78 at @s run summon minecraft:oak_boat ~5 ~ ~ {Tags:["P08BFV78"],NoGravity:1b,Rotation:[0.0f,0.0f]}
execute as P08ZombieTurn78 at @s run summon minecraft:zombie ~5 ~ ~ {Tags:["P08BFP78"],NoAI:1b,NoGravity:1b,PersistenceRequired:1b,Invulnerable:1b,Rotation:[170.0f,0.0f]}
ride @e[type=minecraft:zombie,tag=P08BFP78,limit=1,sort=nearest] dismount
ride @e[type=minecraft:zombie,tag=P08BFP78,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08BFV78,limit=1,sort=nearest]
```

Gözlenen sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
vehicle_id=17080
vehicle_type_id=89
vehicle_y_rot=0
passenger_id=17081
passenger_type_id=151
before_rotation=(104.0625, 0.0, 104.0625)
before_previous_rotation=(105.0, 0.0, 168.75)
before_body_y_rotation=0
after_rotation=(105.0, 0.0, 105.0)
after_previous_rotation=(105.9375, 0.0, 168.75)
after_body_y_rotation=0
passenger_position=(29.5, 74.1875, -35.5)->(29.5, 74.1875, -35.5)
passenger_observation_count=1
world_entity_count=69
world_chunk_count=157
received_packet_count=24309
```

Oak boat type `89`, Zombie type `151` ve relation state'i alındı. Bu koşuda
remote interpolation nedeniyle tick sonrası current/head yaw `104.0625` olarak
gözlendi; doğrudan `turn_entity` callback'i sonrasında head ve current yaw
`105`'e, body yaw boat yaw'ı `0`'a geldi. Callback sonrası passenger konumu
seat konumunda sabit kaldı; default vehicle attachment hesabı [P08.bg kaydında](client-world-entity-passenger-generic-position-conformance.md)
ayrıca kapatıldı. Bu gözlem Rust world kopyasının vanilla
sunucusundan alınan relation/entity state üzerinde Java kaynaklı living clamp
dalını uyguladığını gösterir; Java istemcisiyle tam parity iddiası değildir.

## Açık kapsam

Pose/scale/dimensions'a bağlı entity attachment'ları, nonzero boat
`deltaRotation`, gerçek pencere input callback'i, dismount/collision, nested
passenger/lifecycle, entity-specific tick/AI, render ve Java istemcisiyle
`client-parity` karşılaştırması açık kalır. Generic vehicle attachment [P08.bg kaydında](client-world-entity-passenger-generic-position-conformance.md),
genel clamp [P08.ba kaydında](client-world-entity-passenger-rotation-conformance.md),
Breeze tick istisnası [P08.bb kaydında](client-world-entity-passenger-boat-turn-exemption-conformance.md),
Animal rotation [P08.bc kaydında](client-world-entity-passenger-animal-rotation-conformance.md),
Animal seat attachment [P08.bd kaydında](client-world-entity-passenger-animal-position-conformance.md)
ve doğrudan turn callback'i [P08.be kaydında](client-world-entity-passenger-turn-conformance.md)
izlenir.
