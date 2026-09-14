# P08.av-entity-passengers — [server-conformance] ClientWorld araç ve yolcu ilişkisi

Bu kayıt, vanilla `SET_PASSENGERS` paketinin araç entity'sindeki yolcu sırasını
ve yolcu entity'lerinin araç ilişkisini `ClientWorld` kopyasına taşımasını
kapsar. Kanıt sınıfı `server-conformance`'dır: Rust probe'u Mojang vanilla
26.2 dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundSetPassengersPacket.java:12-39`, araç entity ID'sini VarInt olarak
ve sıralı yolcu ID'lerini VarInt array olarak taşır. Java 26.2 play dispatch
tablosunda paket ID'si `107`'dir (`GameProtocols.java:240`).

`ClientPacketListener.java:1146-1175`, araç bulunmuyorsa paketi yok sayar;
bulunan aracın mevcut yolcularını çıkarır ve packet'teki mevcut entity'leri
araç üzerinde sırayla `startRiding` yoluna verir. `Entity.java:2428-2465`
aynı araca tekrar bağlanmayı, self-riding'i ve ilişki döngülerini reddeder;
force yolunda mevcut araçtan ayrılma ve yeni araç listesine ekleme
`Entity.java:2472-2525` içindeki state değişimidir. `getPassengers` ve
`getControllingPassenger` erişimleri `Entity.java:3508-3553` ile ayrıdır;
boat'ın ilk yolcuyu kontrolcü kabul etmesi `AbstractBoat.java:760-761` içinde
tanımlıdır.

`ClientLevel.java:450-490`, passenger entity'lerinin üst seviye entity tick
listesinde ayrı yürütülmediğini ve araç tick'i sırasında özel passenger
transform/tick akışına girdiğini gösterir. Bu dilim ilişki listesini ve
vehicle ID'sini saklar; passenger konumlandırması, araç kontrolü ve gerçek
tick sıralaması açık kapsamdır.

## Sözleşme

- Paket, araç ID'sini ve yolcu ID'lerini packet sırasıyla tüketir.
- Araç world kopyasında yoksa observation tutulur fakat uygulama sonucu
  `false` olur.
- Araç varsa önce o aracın mevcut yolcu listesi temizlenir.
- Paket listesindeki world'de bulunan yolcular, listedeki sırayla araca
  bağlanır. Yolcunun eski araç listesi aynı anda temizlenir.
- Bilinmeyen yolcu ID'leri, self-riding, aynı araca tekrar ekleme ve ilişki
  döngüsü world kopyasına eklenmez.
- Boş yolcu listesi araçta kalan yolcu ilişkilerini temizler.
- LocalPlayer ID'si `ClientWorld` içinde bilinen entity gibi ilişki state'ine
  alınır; Java'nın HUD, narrator, yaw/head ve boat kontrolü yan etkileri bu
  dar state diliminin dışındadır.
- Entity kaldırıldığında parent araç listesinden ve kendi child yolcu
  ilişkilerinden temizlenir; yeni dimension world'ü ilişki state'ini bırakır.

## Rust doğrulaması

`parse_set_passengers_packet`, Java packet alan sırasını, VarInt array count
ve yolcu sırasını tüketir; negatif count ve trailing byte'ları reddeder.
`ClientWorld::apply_entity_passengers`, `ClientEntity` ve `ClientPlayerState`
içindeki vehicle/passenger state'lerini Java listener'ın replacement ve
detach sırasına göre günceller. `REMOVE_ENTITIES` ve entity ID replacement
yolları da ilişki state'ini temizler.

`set_passengers_parser_matches_java_vehicle_and_varint_array_order` unit testi
araç ID'si, çok byte'lı yolcu ID'si, signed VarInt alanı, boş liste, negatif
count ve trailing payload davranışını kontrol eder.

`entity_passenger_state_replaces_previous_vehicle_and_cleans_relationships`
unit testi eski araçtan ayrılmayı, packet sırasını, bilinmeyen ID atlamasını,
duplicate ID'yi, local player yolcusunu, boş listenin temizliğini ve araç
kaldırılınca child ilişkisinin bırakılmasını kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Rust probe'u açıkken gözlemcinin yakınında sabit bir oak boat ve armor stand
oluşturuldu. Armor stand boat'a bindirildi ve sonra indirildi:

```text
cargo run --locked --offline --example world_entity_passengers_probe -- 127.0.0.1:25568 P08PassObs26
summon minecraft:oak_boat 18.5 64 -30.5 {Tags:["P08V26"],NoGravity:1b}
summon minecraft:armor_stand 18.5 64 -30.5 {Tags:["P08P26"],NoGravity:1b}
ride @e[type=minecraft:armor_stand,tag=P08P26,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08V26,limit=1,sort=nearest]
ride @e[type=minecraft:armor_stand,tag=P08P26,limit=1,sort=nearest] dismount
```

Vanilla konsolu armor stand'ın boat'a bindiğini ve indiğini bildirdi. Rust
probe'u aynı araç için önce dolu, sonra boş iki `SET_PASSENGERS` packet'ini
uyguladı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `vehicle_id` | `6253` |
| `mounted_passengers` | `[6254]` |
| `mounted_world_passengers` | `[6254]` |
| `dismounted_passengers` | `[]` |
| `dismounted_world_passengers` | `Some([])` |
| `passenger_packet_count` | `2` |
| `world_entity_count` | `45` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `17541` |

Canlı packet'lerde araç ve yolcu entity'leri world kopyasında mevcut olduğu
için iki observation da uygulandı. Dolu packet'te sıra `[6254]` olarak
korundu; boş packet aynı araçta kalan ilişkiyi temizledi. Bu vanilla kabulü
wire tüketimini ve relation state uygulamasını gösterir; Java istemcisiyle
tam parity iddiası değildir.

## Açık kapsam

Passenger tick/transform sırası, araç kontrolü, boat'a özel yaw/head ve HUD
yan etkileri, passenger konumlandırması, vehicle physics, entity lifecycle'ın
chunk unload tarafı ve Java istemcisiyle `client-parity` karşılaştırması açık
kalır. İlişki listesinin `TELEPORT_ENTITY`, hareket ve gerçek `ClientLevel`
tick akışlarıyla birlikte kullanılması sonraki dünya dilimidir.
