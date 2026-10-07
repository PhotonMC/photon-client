# P08.ac — [server-conformance] Entity silent semantic state

Bu kayıt, `ClientboundSetEntityDataPacket` içindeki Java entity silent alanının
Rust `ClientWorld` state'ine semantik olarak uygulanmasını kapsar. Kanıt sınıfı
`server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2 dedicated
server'a bağlanır. Java istemcisiyle aynı girdide state karşılaştırılmadığı için
bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:268,317`, `DATA_SILENT` alanını entity data dizisinde ID `4` olarak
tanımlar ve varsayılanını `false` yapar. `Entity.java:1502-1507`, silent alanının
okunup yazıldığını gösterir. `Entity.java:1490-1498`, entity'nin normal ses
çıkışında `isSilent()` true ise level'a ses gönderilmediğini belirtir;
`Entity.java:619-620` adım/itme gibi başka bir ses yolunun da aynı koşulu
kullandığını gösterir.

`EntityDataSerializers.java:53-54,164-172`, `BOOLEAN` serializer'ının kayıt
sırasındaki ID'sini `8` olarak verir. `ByteBufCodecs.java:56-66`, boolean wire
değerinin tek byte ile okunup yazıldığını gösterir. Metadata kayıtlarının ID,
serializer, değer ve `255` sonlandırıcısı biçimi
`ClientboundSetEntityDataPacket.java:21-37` içindedir; dirty kayıt sırasını ve
yalnız gönderilen alanların uygulanmasını `SynchedEntityData.java:74-85,108-125`
belirtir.

## Sözleşme

- Geçerli metadata içinde ID `4` ve `BOOLEAN` serializer ID `8` varsa byte
  değeri sıfırsa `false`, sıfır dışıysa `true` silent state'ine yazılır.
- ID `4` tek başına veya ID `0` shared flags, ID `1` air supply ve ID `5`
  no-gravity gibi daha önce desteklenen kayıtlarla birlikte gönderilebilir;
  gönderilmeyen alanlar önceki değerlerini korur.
- Yeni `ClientEntity` ve yeni `ClientPlayerState` silent değerini `false` ile
  başlatır.
- Local player yeniden kurulumunda `dataToKeep` bit 2 varsa silent mevcut
  entity data ile birlikte korunur; bit 2 yoksa yeni oyuncu `false` ile başlar.

## Rust doğrulaması

`metadata_boolean_value`, Java'nın ID sırasındaki BYTE, INT ve BOOLEAN
kayıtlarını tüketerek ID `4` BOOL alanını çözer. `ClientEntity::is_silent` ve
`ClientPlayerState::is_silent` değerleri world kopyasından sorgulanabilir.

World unit testi silent true/false geçişlerini, no-gravity ve air supply
state'lerinin korunmasını doğrular. Local player respawn testi `dataToKeep` bit
2 ile true değerinin korunmasını, boş maskeyle false değerine dönülmesini
kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken sabit bir konumda silent armor stand oluşturuldu:

```text
cargo run --locked --offline --example world_entity_silent_probe -- 127.0.0.1:25568 P08SilL1
summon minecraft:armor_stand 25 90 -36 {Silent:1b}
```

Vanilla sunucusu summon komutunu kabul etti. Probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `148` |
| `silent` | `true` |
| `entity_type_id` | `5` |
| `entity_position` | `(25.5, 81.0, -35.5)` |
| `entity_metadata_length` | `10` |
| `world_entity_count` | `46` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `8846` |

Silent entity'nin metadata üzerinden bulunması ve boolean state'inin world
map'inde sorgulanması, vanilla sunucunun ID `4` BOOL metadata akışının Rust
semantic state'e ulaştığını gösterir. Bu sonuç Java istemcisiyle parity iddiası
değildir.

## Açık kapsam

Silent state'inin istemci ses motoru ve tüm `EntityBoundSoundInstance` yollarına
uygulanması, ses seçimi/zamanlaması, diğer BOOLEAN alanları (`NoGravity`,
`CustomNameVisible`), component ve pose serializer'ları ile `client-parity`
karşılaştırması açık kalır.
