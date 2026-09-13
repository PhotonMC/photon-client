# P08.al — [server-conformance] Entity effect ambience semantic state

Bu kayıt, Java `LivingEntity` effect ambience alanının Rust `ClientWorld`
state'ine uygulanmasını kapsar. Kanıt sınıfı `server-conformance`'dır: Rust
packet akışı Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle
aynı girdide state karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`LivingEntity.java:176-186`, effect particle listesi ID `10` sonrasında effect
ambience alanını ID `11` ve `EntityDataSerializers.BOOLEAN` ile tanımlar.
`LivingEntity.java:322-329`, ambience değerini `false` ile başlatır.
`LivingEntity.java:902-909`, effect güncellemesinde görünür effect particle
listesiyle birlikte ambience değerini `areAllEffectsAmbient` sonucuna ayarlar;
`LivingEntity.java:960-965` görünür effect'lerden herhangi biri ambient değilse
sonucun `false` olduğunu tanımlar.

Client tick davranışı `LivingEntity.java:870-874` içinde particle listesi doluysa
ambience değerini particle sıklığına bağlar: ambient effect için çarpan `5`,
diğer durumda `1` kullanılır. Bu kayıt bu sıklık ve particle render yolunu
uygulamaz; yalnızca synchronized Boolean state'i taşır.

`EntityDataSerializers.java:53-54,164-172`, BOOLEAN serializer ID'sinin `8`
olduğunu; `ByteBufCodecs.java:54-66`, wire değerinin tek boolean byte'ı olduğunu
gösterir. Particle listesi serializer ID `17`'dir ve list count'u VarInt olarak
başlatır. `SynchedEntityData.java:74-85,108-125` dirty alanların yalnızca
değişen değerleri taşıdığını, `ClientboundSetEntityDataPacket.java:21-37` ise
metadata kayıtlarının `255` EOF marker'ı ile bittiğini doğrular.

## Sözleşme

- Geçerli metadata içinde ID `11` ve BOOLEAN serializer ID `8` varsa değer tek
  byte'tan Boolean olarak okunur ve `ClientEntity::is_effect_ambient` üzerinden
  sorgulanır.
- Yeni entity ve local player ambience değeri `false` ile başlar. Yalnız ID `11`
  içeren güncelleme `[11, 8, value, 255]` biçiminde uygulanabilir.
- Metadata yalnız gönderilen alanları günceller. ID `11` içermeyen sonraki
  packet önceki değeri korur; yanlış serializer veya eksik değer state'i
  değiştirmez.
- Java effect güncellemesi ID `10` particle listesi boşken ID `11` ambience
  alanını gönderebilir. Rust parser bu boş listeyi serializer ID `17` ve count
  `0` olarak atlar; dolu particle listesi ve particle option codec'leri bu
  dilimin dışındadır ve böyle bir packet ambience state'ini değiştirmez.
- Local player metadata'sı aynı state'i player world kopyasına uygular.
  Respawn `KEEP_ENTITY_DATA` maskesiyle entity data korunduğunda ambience
  korunur; boş keep maskesi yeni player varsayılanı olan `false` değerine döner.

## Rust doğrulaması

`effect_ambience_from_metadata`, mevcut BOOLEAN metadata okuyucusunu ID `11`
için kullanır. `skip_metadata_value`, boş `PARTICLES` listesi için count VarInt
`0` tüketir; bu, ID `10` boş alanından sonra ID `11` okumayı mümkün kılar.
`ClientEntity` ve `ClientPlayerState` ambience değerini ayrı Boolean alanında
tutar. Metadata uygulama yolu remote entity ve local player dallarında değeri
günceller; entity-data korumalı local player yeniden oluşturmasında değer
kopyalanır.

World unit testleri `false` başlangıcını, boş particle listesi öncesinden `true`
değerini, kısmi metadata korunmasını, yanlış serializer'ı ve dolu particle
listesinin desteklenmeyen dal olarak state'i değiştirmemesini kontrol eder.
Local player testi metadata değerini, `KEEP_ENTITY_DATA` ile korunmasını ve boş
keep maskesiyle `false` değerine dönmesini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken NoAI/no-gravity iron golem oluşturuldu. Ardından
parçacıkları gizlenmiş bir Speed effect vanilla komutuyla hedefe uygulandı.
`hideParticles=true` olduğundan Java effect update'i boş particle listesiyle
ambience Boolean state'ini güncelledi:

```text
cargo run --locked --offline --example world_entity_effect_ambience_probe -- 127.0.0.1:25568 P08EffectL1
summon minecraft:iron_golem 61 90 -36 {NoAI:1b,NoGravity:1b,Health:75.0f}
effect give @e[type=minecraft:iron_golem,x=61,y=90,z=-36,distance=..2,limit=1] minecraft:speed 30 0 true
```

Vanilla sunucusu iki komutu kabul etti; probe hedefe gelen metadata packet'ini
uygulayıp başarıyla tamamlandı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `565` |
| `entity_effect_ambient` | `true` |
| `entity_type_id` | `70` |
| `entity_position` | `(61.5, 90.0, -35.5)` |
| `entity_metadata_length` | `4` |
| `world_entity_count` | `58` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `16211` |

`entity_metadata_length=4`, ambience güncellemesinin ID `11`, BOOLEAN serializer,
value `true` ve `255` EOF alanlarından oluştuğunu doğrular. Bu sonuç Java
istemcisiyle parity iddiası değildir.

## Açık kapsam

Dolu effect particle listesi ve option codec'leri, ambience'e göre particle
sıklığı, particle render/seed, görünür effect listesi, effect lifecycle ve
client-parity karşılaştırması açık kalır.
