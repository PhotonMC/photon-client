# P08.e-entity-rotation — [server-conformance] ClientWorld entity rotation state

Bu kayıt, vanilla sunucusundan gelen entity rotation alanlarının parse edilip
`ClientWorld` global entity map'indeki ham y/x/head rotation state'ine
uygulanmasını kapsar. Kanıt sınıfı `server-conformance`'dır: Rust packet akışı
Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle aynı girdide
state karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundAddEntityPacket.java:17-31,95-120,132-169`, AddEntity gövdesinde
packed movement'dan sonra x rotation, y rotation ve head y rotation için üç
byte taşır. `Mth.java:182-188`, byte'ı signed değer olarak açıp
`rotation * 360 / 256` ile dereceye çevirir. `Entity.java:3859-3869`, genel
entity add yolunda x/y rotation'ı snap eder; `LivingEntity.java:3961-3975`,
living entity için head/body rotation'ı da packet'teki head değeriyle başlatır.

`ClientboundMoveEntityPacket.java:12-187`, position+rotation ve rotation-only
dallarında y/x rotation byte'larını taşır. `ClientboundEntityPositionSyncPacket.java:10-35`
ve `ClientboundTeleportEntityPacket.java:10-42`, `PositionMoveRotation` içindeki
y/x float'larını taşır; `Relative.java:10-19`, teleport relative rotation
bitlerini Y_ROT=3 ve X_ROT=4 olarak tanımlar. `ClientboundRotateHeadPacket.java:12-46`
entity ID ile tek head rotation byte'ı taşır. Bu packet'in clientbound ID'si
`GameProtocols.java:216` dispatch sırasından `83`'tür.

`ClientPacketListener.java:639-727,729-754,806-823`, mevcut entity'ye sync,
teleport ve MoveEntity rotation'ını uygular; `handleRotateMob` head rotation'ı
günceller. `Entity.java:2539-2560`, uygun durumda rotation'ı interpolation
handler'a verir; bu kayıt yalnız packet değerinin world kopyasına uygulanmasını
ölçer.

## Sözleşme

- AddEntity'nin üç signed packed rotation byte'ı Java `Mth.unpackDegrees`
  alan sırasıyla x/y/head derece değerlerine çevrilir ve yeni entity'nin raw
  rotation state'inde saklanır.
- MoveEntity rotation içeren dallarda y/x state'i güncellenir; position-only
  dalı rotation'ı değiştirmez.
- PositionSync ve Teleport y/x rotation değerlerini mevcut entity'ye uygular.
  Teleport'ta relative Y_ROT/X_ROT bitleri set ise ilgili mevcut değere eklenir
  ve x rotation Java sınırına göre `[-90, 90]` aralığında tutulur.
- RotateHead packet'i yalnız y head rotation state'ini değiştirir. Bilinmeyen
  entity ID'si için observation tutulur fakat application sonucu false olur.
- Bu dilim raw current rotation sorgusunu sağlar. Interpolation history,
  partial-tick rotation, living entity body/head semantic'leri, finite-value
  hata yolu ve render state'i uygulanmaz.

## Rust doğrulaması

`parse_add_entity_packet` ve MoveEntity parser'ları signed packed byte'ları
dereceye çevirir. `parse_rotate_head_packet`, clientbound ID `83` gövdesini
tüketir ve trailing byte'ları reddeder. PositionSync ve Teleport uygulamaları
rotation değerlerini ilgili relative flag'lerle birlikte `ClientWorld`'e yazar.

`ClientEntity::rotation()` y, x ve head değerlerini sorgulanabilir kılar.
Unit testleri AddEntity/MoveEntity/RotateHead packed derece dönüşümünü, sync
ve teleport observation alanlarını, mutlak/göreli rotation uygulamasını ve
bilinmeyen entity sonucunu kontrol eder. `world_entity_rotation_probe`, canlı
vanilla oturumunda packet observation ile uygulama anındaki world rotation
state'ini eşleştirir.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken kontrollü bir armor stand, başlangıç rotation NBT'si
ile oluşturuldu:

```text
cargo run --locked --offline --example world_entity_rotation_probe -- 127.0.0.1:25568 EntityRotation
# vanilla server console, while the probe is connected
summon minecraft:armor_stand 25 81 -36 {Rotation:[90.0f,30.0f]}
```

Vanilla konsolu `Summoned new Armor Stand` bildirdi. Rust probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `add_entity_packet_count` | `49` |
| `add_entity_nonzero_rotation_count` | `44` |
| `entity_move_rotation_packet_count` | `1830` |
| `entity_move_rotation_applied_count` | `1830` |
| `entity_position_sync_packet_count` | `58` |
| `entity_position_sync_rotation_applied_count` | `58` |
| `entity_teleport_packet_count` | `0` |
| `entity_teleport_rotation_applied_count` | `0` |
| `entity_head_rotation_packet_count` | `2045` |
| `entity_head_rotation_applied_count` | `2045` |
| ilk başlangıç rotation entity | `id=1006`, `(y=-135,x=0,head=-135)` |
| world başlangıç rotation | aynı üç f32 bileşen |
| ilk MoveEntity rotation entity | `id=1005`, `(y=-45,x=0)`, world head `-40.78125` |
| ilk RotateHead entity | `id=1005`, `head=-43.59375`, world head `-43.59375` |
| `world_entity_count` | `47` |
| `world_chunk_count / world_section_count` | `157 / 3768` |
| `received_packet_count` | `8604` |

Probe, başlangıç rotation içeren AddEntity kayıtlarını, rotation içeren tüm
MoveEntity kayıtlarını ve 2.045 RotateHead kaydının tamamını mevcut world
entity'lerine uyguladı. Tablodaki örnekler ilk eşleşen observations'tır; sample
entity ID'si kontrollü armor stand ile eşleştirilmemiştir. Her uygulanan örnekte
packet değeri ile application anındaki `ClientEntity::rotation()` değeri
eşleşti. Sonuç vanilla raw entity rotation akışının Rust world kopyasına
taşındığını gösterir; Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Rotation interpolation history, partial tick ve `rotLerp`, living entity'nin
body/head ayrımının tick içindeki semantic davranışı, on-ground, motion
integration, passenger ilişkisi, metadata/equipment, remove/lifecycle, chunk
unload temizliği, dimension/respawn, render state'i ve oynanabilir istemci
bağlantısı açık kalır. Bu rotation state'i `server-conformance`
ilerlemesidir; P08'in `client-parity` kapanışı değildir.
