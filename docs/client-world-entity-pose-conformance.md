# P08.ad — [server-conformance] Entity pose semantic state

Bu kayıt, `ClientboundSetEntityDataPacket` içindeki Java entity pose alanının
Rust `ClientWorld` state'ine semantik olarak uygulanmasını kapsar. Kanıt sınıfı
`server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2 dedicated
server'a bağlanır. Java istemcisiyle aynı girdide state karşılaştırılmadığı için
bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Pose.java:9-37`, Java'nın entity pose enum'ünü ID `0` ile `17` arasında
tanımlar. `BY_ID`, geçersiz veya sınır dışı ID'leri `OutOfBoundsStrategy.ZERO`
ile `STANDING` değerine döndürür; stream codec pose ID'sini VarInt olarak
taşır.

`Entity.java:270-271,319`, `DATA_POSE` alanını entity data dizisinde ID `6`
olarak tanımlar ve varsayılanını `Pose.STANDING` yapar. `Entity.java:440-449`
pose okuma/yazma sorgularını, `Entity.java:3377-3380` ise pose değişiminde
`refreshDimensions()` çağrısını gösterir. Player'ın seçebileceği temel pose
dalları `player/Player.java:344-369` içindedir.

`EntityDataSerializers.java:117,184`, `POSE` serializer'ının kayıt sırasındaki
ID'sini `20` olarak verir. `ByteBufCodecs.java:546-558`, ID mapper'ın VarInt
okuyup yazdığını gösterir. Metadata kayıtlarının ID, serializer, değer ve `255`
sonlandırıcısı biçimi `ClientboundSetEntityDataPacket.java:21-37` içindedir;
dirty kayıt sırasını ve yalnız gönderilen alanların uygulanmasını
`SynchedEntityData.java:74-85,108-125` belirtir.

Canlı senaryo için `animal/camel/Camel.java:104-117`, `LastPoseTick` değeri
negatifse camel'ın `Pose.SITTING` yaptığını gösterir. Bu, entity türüne özel
bir state üretmeden pose metadata dalını vanilla sunucusunda gözlemlemeyi
sağlar.

## Sözleşme

- Geçerli metadata içinde ID `6` ve `POSE` serializer ID `20` varsa değer Java
  `Pose` ID tablosuna göre uygulanır ve `0`–`17` dışındaki değerler
  `STANDING` olur.
- Pose ID eşlemesi `0 STANDING`, `1 FALL_FLYING`, `2 SLEEPING`, `3 SWIMMING`,
  `4 SPIN_ATTACK`, `5 CROUCHING`, `6 LONG_JUMPING`, `7 DYING`, `8 CROAKING`,
  `9 USING_TONGUE`, `10 SITTING`, `11 ROARING`, `12 SNIFFING`, `13 EMERGING`,
  `14 DIGGING`, `15 SLIDING`, `16 SHOOTING`, `17 INHALING` biçimindedir.
- ID `6` tek başına veya daha önce desteklenen entity metadata kayıtlarıyla
  birlikte gönderilebilir; gönderilmeyen alanlar önceki değerlerini korur.
- Yeni `ClientEntity` ve yeni `ClientPlayerState` pose değerini `STANDING` ile
  başlatır.
- Local player yeniden kurulmasında `dataToKeep` bit 2 varsa pose mevcut entity
  data ile birlikte korunur; bit 2 yoksa yeni oyuncu `STANDING` ile başlar.

## Rust doğrulaması

`metadata_pose_value`, Java'nın ID sırasındaki BYTE, INT, BOOLEAN ve POSE
kayıtlarını tüketerek ID `6` POSE alanını çözer. `ClientEntity::pose` ve
`ClientPlayerState::pose` değerleri world kopyasından, Java enum adlarını
taşıyan Rust enum'ü ve wire ID'si ile sorgulanabilir.

World unit testi 18 pose ID'sinin tamamını tek parametrik akışta, combined
metadata uygulamasını, belirtilmeyen alanların korunmasını ve bilinmeyen ID'nin
`STANDING` fallback'ini doğrular. Local player respawn testi `dataToKeep` bit 2
ile `SITTING` değerinin korunmasını, boş maskeyle `STANDING` değerine dönülmesini
kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken sabit bir konumda `LastPoseTick` değeri negatif ve
`NoAI` açık bir camel oluşturuldu:

```text
cargo run --locked --offline --example world_entity_pose_probe -- 127.0.0.1:25568 P08PoseL1
summon minecraft:camel 25 90 -36 {LastPoseTick:-1L,NoAI:1b}
```

Vanilla sunucusu summon komutunu kabul etti. Probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `365` |
| `pose` | `Sitting` |
| `pose_id` | `10` |
| `entity_type_id` | `19` |
| `entity_position` | `(25.5, 90.0, -35.5)` |
| `entity_metadata_length` | `25` |
| `world_entity_count` | `48` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `10203` |

Camel entity'nin metadata üzerinden bulunması ve pose ID `10` değerinin world
map'inde `Sitting` olarak sorgulanması, vanilla sunucunun ID `6` POSE metadata
akışının Rust semantic state'e ulaştığını gösterir. Bu sonuç Java istemcisiyle
parity iddiası değildir.

## Açık kapsam

Pose değişiminde Java'nın `refreshDimensions()` yan etkisinin uygulanması,
pose'a bağlı collision/movement/camera/render davranışları, entity türüne özel
pose geçişleri, gerçek client tick pose güncellemeleri, diğer metadata
serializer'ları ve `client-parity` karşılaştırması açık kalır.
