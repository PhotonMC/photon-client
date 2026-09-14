# P08.ba-entity-passenger-rotation — [server-conformance] Boat passenger rotation clamp

Bu kayıt, boat passenger tick'inde passenger yaw/head/body state'inin Java
`AbstractBoat.clampRotation` davranışına göre güncellenmesini kapsar. Kanıt
sınıfı `server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated
server'a bağlanır. Java istemcisiyle aynı girdide state karşılaştırılmadığı
için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`AbstractBoat.java:60-69` boat'ın `deltaRotation` state'ini tutar.
`586-620` içindeki `controlBoat`, input alan boat için bu state'i değiştirir;
remote authoritative server fixture'ında boat input'u gönderilmediği için
bu kayıtta `deltaRotation` sıfırdır.

`AbstractBoat.java:631-642`, passenger `CAN_TURN_IN_BOATS` tag'inde değilse
passenger yaw ve head yaw'ına `deltaRotation` ekler, sonra `clampRotation`ı
çağırır. `677-684` içindeki clamp, boat ile passenger arasındaki yaw farkını
wrapped olarak hesaplar, `[-105.0F,105.0F]` aralığına sıkıştırır ve aynı
düzeltmeyi passenger'ın current ve old yaw state'lerine uygular. Head yaw
current passenger yaw'a, living passenger body yaw'ı boat yaw'ına yazılır.
`687-688`, passenger doğrudan döndüğünde aynı clamp yolunun
`onPassengerTurned` üzerinden çağrıldığını gösterir.

`Entity.java:490-501`, `turn` sonrasında vehicle'ın `onPassengerTurned`
callback'ini çağırır. `2380-2405`, passenger tick'inde hareketin sıfırlanıp
entity tick'inin çalıştırıldığını ve ardından vehicle riding position'ının
uygulandığını gösterir. Genel `Entity` için `get/setYHeadRot` ve
`setYBodyRot` (`2969-2977`) no-op'tur; `LivingEntity.java:238-241,498-501`
head/body alanlarını ve old state snapshot'ını tutar, `3380-3392` setter'ları
bu alanları değiştirir.

## Sözleşme

- P08.az'daki 20 normal boat, chest boat, raft ve chest raft type ID'si aynı
  rotation clamp yolunu kullanır; bu dilim yeni boat varyantı açmaz.
- Tag istisnası dışındaki passenger için `deltaRotation` sonrası wrapped yaw
  farkı `[-105,105]` içinde kalır.
- Clamp düzeltmesi current ve previous yaw'a aynı f32 correction ile uygulanır.
- Living Player için current head yaw current yaw'a, body yaw boat yaw'ına
  eşitlenir. Base `Entity` passenger'da head/body setter'larının no-op
  olması korunur.
- Remote Player interpolation sonucu üç tick içinde sınırın altında kalırsa
  Java clamp yeni bir düzeltme yapmaz; sınır fixture'ı bu nedenle ayrı generic
  passenger ile ölçülür.
- Passenger seat position, nonzero boat `deltaRotation`, `onPassengerTurned`
  input callback'i, animal ikinci passenger rotation offset'i ve tag
  istisnası P08.ba'nın dışındadır; Breeze istisnası [P08.bb kaydındadır](client-world-entity-passenger-boat-turn-exemption-conformance.md).

## Rust doğrulaması

`ClientWorld::position_entity_passenger`, boat family tablosundaki araçlarda
seat geometry uygulanırken passenger rotation clamp'ını da çağırır.
Seat geometry'si henüz tanınmayan passenger için fallback yalnız boat rotation
state'ini uygular; konum hesabı yapılmaz. `ClientEntity` ve
`ClientPlayerState`, current/previous body yaw alanlarını taşır.

`passenger_tick_clamps_remote_player_rotation_to_boat_heading` testi Player
type ID `156` için `(140,0,140)` yaw state'ini oak boat yaw `0` ile karşılaştırır
ve `(105,0,105)` current rotation, `(105,0,140)` previous rotation, body yaw
`0` sonucunu kontrol eder.

`passenger_tick_clamps_boat_rotation_for_passenger_without_seat_geometry`
generic passenger'ın seat geometry olmadan yaw clamp'ını; mevcut
`passenger_tick_positions_*` testleri ise aynı tick sırasındaki position
uygulamasını korur.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Oak boat ve rotation'ı `170` olan no-gravity armor stand aynı observer
konumunda oluşturuldu. Armor stand boat seat geometry'si Rust'ta tanınmayan
generic passenger olarak yalnız rotation branch'ini ölçer:

```text
cargo run --locked --offline --example world_entity_passenger_rotation_clamp_probe -- 127.0.0.1:25568 P08BoatClamp44
execute as P08BoatClamp44 at @s run summon minecraft:oak_boat ~ ~ ~ {Tags:["P08BCV44"],NoGravity:1b,Rotation:[0.0f,0.0f]}
execute as P08BoatClamp44 at @s run summon minecraft:armor_stand ~ ~ ~ {Tags:["P08BCP44"],NoGravity:1b,Rotation:[170.0f,0.0f]}
ride @e[type=minecraft:armor_stand,tag=P08BCP44,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08BCV44,limit=1,sort=nearest]
```

Gözlenen sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
vehicle_id=11370
vehicle_type_id=89
vehicle_y_rot=0
passenger_id=11371
passenger_type_id=5
passenger_rotation=(168.75, 0.0, 168.75)->(105.0, 0.0, 168.75)
passenger_previous_rotation_after_tick=Some((105.0, 0.0, 168.75))
passenger_observation_count=1
world_entity_count=32
world_chunk_count=157
received_packet_count=15469
```

Oak boat type ID `89`, armor stand type ID `5`, current yaw `105` ve
previous yaw correction sonucu birlikte gözlendi. Head yaw'ın generic
passenger'da değişmemesi base `Entity` no-op setter davranışıyla uyumludur.

Remote Player probe'u da vanilla oturumunda Player type ID `156` için boat
ilişkisini, bounded relative yaw'ı ve head/body state'ini ölçtü:

```text
cargo run --locked --offline --example world_entity_passenger_rotation_probe -- 127.0.0.1:25568 P08BoatRotObs43 P08BoatRotMove43
passenger_type_id=156
passenger_rotation=(0.0, 0.0, 0.0)->(0.0, 0.0, 0.0)
passenger_body_y_rotation=0->0
passenger_observation_count=1
```

Bu fixture'da server packet sırası Player yaw'ını observer world state'ine
ulaştırmadan passenger tick'i çalıştırdığı için over-boundary Player clamp'ı
bu canlı çıktıda tetiklenmedi; sınır sonucu Java kaynaklı unit test ve generic
canlı fixture ile ayrı ayrı doğrulandı.

## Açık kapsam

Boat input'u ile nonzero `deltaRotation`, remote/local `onPassengerTurned`
callback'i, animal ikinci passenger body/head rotation offset'i, diğer
LivingEntity türlerinin body/head state'i, generic passenger seat attachment'ı,
dismount/collision, render ve Java istemcisiyle `client-parity` karşılaştırması
açık kalır. Chest boat/raft geometri [P08.az kaydında](client-world-entity-passenger-boat-family-conformance.md), passenger tick sırası [P08.aw kaydında](client-world-entity-passenger-tick-conformance.md) izlenir.
