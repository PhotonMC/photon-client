# P08.ah — [server-conformance] Entity living health semantic state

Bu kayıt, `ClientboundSetEntityDataPacket` içindeki Java `LivingEntity` sağlık
alanının Rust `ClientWorld` state'ine semantik olarak uygulanmasını kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`LivingEntity.java:176-180`, base `Entity` alanlarından sonra yaşayan
entity'lere living flags ID `8` BYTE ve health ID `9` FLOAT alanlarını ekler.
`LivingEntity.java:322-329`, flags alanını `0` ve health alanını `1.0F` ile
tanımlar; constructor `LivingEntity.java:286-290` içinde health'i türün
`getMaxHealth()` değerine ayarlar. Böylece ağ metadata'sında gelen ID `9`
değeri yaşayan entity'nin o anki synchronized health state'idir.

`EntityDataSerializers.java:53-56,164-172`, FLOAT serializer'ının ID `3`
olduğunu ve registration sırasını verir. `ByteBufCodecs.java:132-139`, FLOAT
değerinin ByteBuf `readFloat`/`writeFloat` ile dört byte taşındığını gösterir;
Rust parser bu baytları Java/Netty big-endian wire sırasıyla okur.

`LivingEntity.java:744-749,798-813`, health değerinin NBT save/load alanının
`Health` olduğunu; `LivingEntity.java:1163-1168`, getter'ın synchronized
değeri döndürdüğünü ve setter'ın maksimum health'e göre clamp uyguladığını;
`LivingEntity.java:1750-1752`, canlılık kararının health değerini kullandığını
gösterir. Bu dilim client tarafındaki attribute, clamp, hasar ve ölüm
mekaniklerini uygulamaz; metadata'dan gelen gözlenebilir health state'ini
taşır.

## Sözleşme

- Geçerli metadata içinde ID `9` ve FLOAT serializer ID `3` varsa sonraki dört
  byte signed IEEE-754 `f32` bitleri olarak okunur ve `ClientEntity::health`
  üzerinden sorgulanır.
- ID `8` living flags gibi önceki alanlar atlanır; ID `9` bulunmadan gelen
  bozuk veya eksik FLOAT kaydı health state'ini değiştirmez.
- Yeni genel `ClientEntity` health değerini `None` ile başlatır. ID `9`
  metadata'sı alınca değer `Some(...)` olur; bu, non-living entity'lerin
  varsayılan bir health değeriyle yanlış sınıflandırılmasını önler.
- Metadata yalnızca gönderilen alanları günceller. ID `9` içermeyen sonraki
  metadata packet'i önceki entity health değerini korur; `0.0` da wire'dan
  gelen geçerli bir değer olarak saklanır.
- Local player metadata'sındaki ID `9`, mevcut player health state'ini
  günceller. Player health packet'i (`SetHealth`) ayrıca food ve saturation ile
  birlikte uygulanmaya devam eder; respawn'da yeni local player Java
  `LocalPlayer.respawn` akışındaki max-health başlangıcına döner.

## Rust doğrulaması

`metadata_float_value`, metadata ID `8` BYTE alanını atlayıp ID `9` FLOAT
alanını dört byte olarak çözer. `skip_metadata_value` FLOAT serializer'ını
tanıdığı için diğer metadata semantic parser'ları da sağlık alanı içeren tam
packet'lerde sonraki bilinen alanlara ilerleyebilir. `ClientEntity` raw bitleri
`Option<u32>` olarak saklar; public getter bunları `Option<f32>` olarak sunar.

World unit testi yeni entity'de `None` başlangıcını, ID `8` öncesinden geçerek
`7.5` değerini almayı, `0.0` güncellemesini ve yalnız no-gravity metadata'sı
geldiğinde önceki health değerinin korunmasını kontrol eder. Local player unit
testi metadata health güncellemesini ve entity-data korunarak respawn sonrasında
yeni player'ın `20.0` başlangıç health'ine dönmesini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken, tracking alanı içinde konumu sabitlenen bir iron
golem oluşturuldu. `Health:75.0f` NBT değeri sunucunun entity data sorgusunda
da `75.0f` olarak doğrulandı; `NoAI` ve `NoGravity` fixture'ın tick sırasında
konum değiştirmemesini sağladı:

```text
cargo run --locked --offline --example world_entity_health_probe -- 127.0.0.1:25568 P08HealthL3
summon minecraft:iron_golem 37 90 -36 {Health:75.0f,NoAI:1b,NoGravity:1b}
```

Vanilla sunucusu summon komutunu kabul etti. Probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `131` |
| `entity_health` | `75` |
| `entity_type_id` | `70` |
| `entity_position` | `(37.5, 90.0, -35.5)` |
| `entity_metadata_length` | `13` |
| `world_entity_count` | `42` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `8837` |

Vanilla entity metadata akışındaki iron golem ID `9` FLOAT değeri Rust
`ClientWorld` map'inde `75.0` olarak sorgulandı. Bu sonuç Java istemcisiyle
parity iddiası değildir.

## Açık kapsam

Entity türlerinin max-health attribute çözümü ve setter clamp'i, hasar/iyileşme
ve ölüm lifecycle'ı, local health HUD/renderer, living flags ve diğer living
metadata alanları, health'in tick davranışı ve `client-parity` karşılaştırması
açık kalır.
