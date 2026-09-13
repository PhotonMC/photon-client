# P08.e-entity-ground — [server-conformance] ClientWorld entity on-ground state

Bu kayıt, vanilla sunucusundan gelen entity on-ground boolean alanlarının
parse edilip `ClientWorld` global entity map'indeki ham ground state'ine
uygulanmasını kapsar. Kanıt sınıfı `server-conformance`'dır: Rust packet akışı
Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle aynı girdide
state karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundMoveEntityPacket.java:12-187`, üç MoveEntity dalının sonunda
on-ground boolean taşır. `ClientboundEntityPositionSyncPacket.java:10-35` ve
`ClientboundTeleportEntityPacket.java:10-42` aynı alanı kendi packet
gövdelerinin sonuna koyar; AddEntity packet'inde on-ground alanı yoktur.

`ClientPacketListener.java:639-727,729-754,806-823`, entity konum/rotasyon
güncellemesinden sonra bu boolean'ı `Entity.setOnGround` ile yazar. `Entity.java:228,
667-710`, state'in entity üzerinde tutulduğunu ve setter'ın supporting-block
kontrolünü başlattığını gösterir. Bu dilim supporting block aramasını veya
çarpışma sonucunu uygulamaz.

## Sözleşme

- MoveEntity, PositionSync ve Teleport packet'lerinin son boolean değeri
  Java'nın sıfır dışı byte'ı true kabul eden boolean codec'iyle okunur.
- Mevcut entity bulunduğunda packet'in konum ve rotation uygulamasından sonra
  `onGround` state'i packet değeriyle değiştirilir.
- Bilinmeyen entity ID'si için packet observation tutulur, world uygulaması
  başarısız olur ve var olmayan entity oluşturulmaz.
- AddEntity yeni entity'yi Java entity alanının başlangıç değeri olan
  `onGround=false` ile oluşturur; AddEntity gövdesi bu alanı taşımadığı için
  packet'ten ayrıca okunmaz.
- Bu dilim yalnız ham boolean state'ini kapsar. Supporting block, fall damage,
  collision flags, movement integration, interpolation, passenger ve
  `client-parity` kapsam dışıdır.

## Rust doğrulaması

`parse_entity_position_sync_packet`, `parse_entity_teleport_packet` ve
MoveEntity parser'ları boolean alanını çözer. `ClientWorld` entity kayıtları
`on_ground` değerini saklar ve `ClientEntity::on_ground()` ile sorgulanabilir;
probe uygulama sonrasındaki state'i observation değeriyle eşleştirir.

Unit testleri başlangıç değerini, true/false geçişlerini ve bilinmeyen entity
sonucunu kontrol eder. `world_entity_ground_probe`, sync, teleport ve MoveEntity
yollarındaki her gözlemi world state'ine uygular ve hem true hem false canlı
değerlerin packet değeriyle eşleşmesini zorunlu tutar.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken bir normal gravity armor stand ve bir `NoGravity`
armor stand oluşturuldu:

```text
cargo run --locked --offline --example world_entity_ground_probe -- 127.0.0.1:25568 EntityGround
# vanilla server console, while the probe is connected
summon minecraft:armor_stand 25 81 -36 {NoGravity:0b}
summon minecraft:armor_stand 27 90 -36 {NoGravity:1b}
```

Vanilla konsolu iki kez `Summoned new Armor Stand` bildirdi. Rust probe
çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_position_sync_packet_count` | `51` |
| `entity_teleport_packet_count` | `0` |
| `entity_move_packet_count` | `4390` |
| `entity_ground_packet_count` | `4441` |
| `entity_ground_applied_count` | `4441` |
| `entity_ground_true_count` | `82` |
| `entity_ground_false_count` | `4359` |
| `first_ground_true_path` | `position_sync` |
| `first_ground_false_path` | `position_sync` |
| `world_entity_count` | `44` |
| `world_chunk_count / world_section_count` | `157 / 3768` |
| `received_packet_count` | `9732` |

`entity_ground_applied_count` toplamı, gözlenen sync, teleport ve MoveEntity
packet'lerinin tamamına world entity bulunduğunda aynı boolean'ın yazıldığını
gösterir. Hem true hem false yollarının görülmesi, yalnızca tek bir sabit
değerin kopyalanmadığını kontrol eder. Bu sonuç vanilla raw entity ground
state'inin Rust world kopyasına taşındığını gösterir; Java istemcisiyle parity
iddiası değildir.

## Açık kapsam

Supporting block konumu, `checkSupportingBlock` yan etkileri, fall damage,
collision flags, entity tick/fizik, movement integration/interpolation,
passenger ilişkisi, dimension/respawn, chunk unload temizliği, render state'i
ve oynanabilir istemci bağlantısı açık kalır. Bu raw state `server-conformance`
ilerlemesidir; P08'in `client-parity` kapanışı değildir.
