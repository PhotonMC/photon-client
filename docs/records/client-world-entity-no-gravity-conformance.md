# P08.ab — [server-conformance] Entity no-gravity semantic state

Bu kayıt, `ClientboundSetEntityDataPacket` içindeki Java entity no-gravity
alanının Rust `ClientWorld` state'ine semantik olarak uygulanmasını kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:270-271,318`, `DATA_NO_GRAVITY_ID` alanını entity data dizisinde ID
`5` olarak tanımlar ve varsayılanını `false` yapar. `Entity.java:1510-1523`,
bu alanın okunup yazıldığını ve no-gravity açıkken gravity değerinin `0.0`
olduğunu gösterir.

`EntityDataSerializers.java:53-54,164-172`, `BOOLEAN` serializer'ının kayıt
sırasındaki ID'sini `8` olarak verir. `ByteBufCodecs.java:56-66`, boolean wire
değerinin tek byte ile okunup yazıldığını gösterir. Metadata kayıtlarının ID,
serializer, değer ve `255` sonlandırıcısı biçimi
`ClientboundSetEntityDataPacket.java:21-37` içindedir; dirty kayıt sırasını ve
yalnız gönderilen alanların uygulanmasını `SynchedEntityData.java:74-85,108-125`
belirtir.

## Sözleşme

- Geçerli metadata içinde ID `5` ve `BOOLEAN` serializer ID `8` varsa byte
  değeri sıfırsa `false`, sıfır dışıysa `true` no-gravity state'ine yazılır.
- ID `5` tek başına veya ID `0` shared flags / ID `1` air supply gibi daha önce
  desteklenen kayıtlarla birlikte gönderilebilir; gönderilmeyen alanlar önceki
  değerlerini korur.
- Yeni `ClientEntity` ve yeni `ClientPlayerState` no-gravity değerini
  `false` ile başlatır.
- Local player yeniden kurulumunda `dataToKeep` bit 2 varsa no-gravity mevcut
  entity data ile birlikte korunur; bit 2 yoksa yeni oyuncu `false` ile başlar.

## Rust doğrulaması

`metadata_boolean_value`, Java'nın ID sırasındaki BYTE, INT ve BOOLEAN
kayıtlarını tüketerek ID `5` BOOL alanını çözer. `ClientEntity::is_no_gravity`
ve `ClientPlayerState::is_no_gravity` değerleri world kopyasından
sorgulanabilir.

World unit testi ID `5` BOOL metadata'sını true ve false değerleriyle, shared
flags ile aynı gövdede, air supply güncellemesiyle birlikte ve belirtilmeyen
alanların korunmasıyla doğrular. Local player respawn testi `dataToKeep` bit 2
ile true değerinin korunmasını, boş maskeyle false değerine dönülmesini kontrol
eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken sabit bir konumda no-gravity armor stand oluşturuldu:

```text
cargo run --locked --offline --example world_entity_no_gravity_probe -- 127.0.0.1:25568 P08GravL1
summon minecraft:armor_stand 25 90 -36 {NoGravity:1b}
```

Vanilla sunucusu summon komutunu kabul etti. Probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `261` |
| `no_gravity` | `true` |
| `entity_type_id` | `5` |
| `entity_position` | `(25.5, 90.0, -35.5)` |
| `entity_metadata_length` | `10` |
| `world_entity_count` | `36` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `9471` |

No-gravity entity'nin metadata üzerinden bulunması ve boolean state'inin
world map'inde sorgulanması, vanilla sunucunun ID `5` BOOL metadata akışının
Rust semantic state'e ulaştığını gösterir. Bu sonuç Java istemcisiyle parity
iddiası değildir.

## Açık kapsam

No-gravity state'inin client tick içinde velocity/gravity entegrasyonu, düşen
entity hareketi, pose ve dimension etkileri, local player'ın gerçek movement
davranışı, diğer BOOLEAN alanları (`Silent`, `CustomNameVisible`), component ve
pose serializer'ları ile `client-parity` karşılaştırması açık kalır.
