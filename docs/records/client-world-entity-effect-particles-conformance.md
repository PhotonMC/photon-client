# P08.am — [server-conformance] Entity effect particle type list semantic state

Bu kayıt, Java `LivingEntity` effect particle listesi alanının Rust
`ClientWorld` state'ine sınırlı ve sorgulanabilir biçimde uygulanmasını kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`LivingEntity.java:176-186`, effect particle listesini ID `10` ve
`EntityDataSerializers.PARTICLES` ile tanımlar. `LivingEntity.java:322-329`,
listeyi boş başlatır; `LivingEntity.java:902-909`, görünür effect'lerin particle
option'larını sırayla bu alana yazar.

`EntityDataSerializers.java:89-92,164-172`, `PARTICLE` serializer'ının ID `16`,
`PARTICLES` serializer'ının ID `17` olduğunu ve list codec'inin
`ParticleTypes.STREAM_CODEC` üzerine kurulduğunu gösterir. `ByteBufCodecs.java:446-455`,
list count'unun VarInt olarak yazıldığını ve sonra element codec'inin
her kayıt için çalıştığını tanımlar.

Bu dilimin vanilla effect dalı `MobEffect.java:46-60,187-188` içindeki renkli
effect particle factory'sidir. `ParticleTypes.java:54-56`,
`ENTITY_EFFECT` türünü built-in particle registry'sine ekler; sabitlenmiş
registration sırasındaki wire registry ID'si `28`'dir. `ColorParticleOption.java:18-20`,
bu türün particle type ID'sinden sonra ARGB rengini dört byte INT olarak taşıdığını
gösterir. Rust bu dört byte'ı tüketerek metadata sonundaki diğer
alanların parse edilmesini sağlar, rengi henüz world state'ine koymaz.

`SynchedEntityData.java:74-85,108-125`, yalnız değişen data item'larının dirty
metadata'ya girdiğini; `ClientboundSetEntityDataPacket.java:21-37`, metadata
kayıtlarının ID, serializer, değer ve `255` EOF sırasını taşıdığını doğrular.

## Sözleşme

- Geçerli metadata içinde ID `10` ve `PARTICLES` serializer ID `17` varsa list
  count'u ve desteklenen her option'ın type ID'si okunur. Bu dilimde desteklenen
  option `ENTITY_EFFECT` type ID `28` ve arkasındaki dört byte ARGB payload'ıdır.
- Type ID listesi packet sırasıyla `ClientEntity::effect_particle_type_ids`
  üzerinden sorgulanır. Boş liste geçerli bir değer olarak önceki listeyi
  temizler.
- Metadata yalnız gönderilen alanları günceller. ID `10` içermeyen sonraki
  packet önceki listeyi korur; yanlış serializer, eksik ARGB payload'ı veya
  bilinmeyen particle type state'i değiştirmez.
- Payload tüketilmesine rağmen ARGB renkleri saklanmaz. Bu nedenle bu dilim
  particle type/cardinality state'idir; renk, option alanları ve gerçek particle
  üretimi/render'ı kapsam dışıdır.
- Local player metadata'sı aynı listeyi player world kopyasına uygular.
  Respawn `KEEP_ENTITY_DATA` maskesi entity data'yı koruduğunda liste korunur;
  boş keep maskesi yeni player varsayılanı olan boş listeye döner.

## Rust doğrulaması

`read_effect_particle_type_ids`, list count'unu VarInt olarak okur, her destekli
`ENTITY_EFFECT` option için type ID'yi ve dört byte ARGB gövdesini tüketir.
`skip_metadata_value`, aynı codec'i kullanarak ID `10` alanını atlar; böylece
aynı packet içindeki ambience gibi daha sonraki alanlar okunabilir.

`ClientEntity` ve `ClientPlayerState` type ID listesini ayrı bir `Vec<i32>`
alanında tutar ve public slice getter sunar. World unit testleri boş başlangıcı,
bir renkli `ENTITY_EFFECT` kaydını, kısmi metadata korunmasını, yanlış
serializer'ı ve bilinmeyen type'ın state'i değiştirmemesini kontrol eder. Local
player testi liste değerini, `KEEP_ENTITY_DATA` ile korunmasını ve boş keep
maskesiyle temizlenmesini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken NoAI/no-gravity iron golem oluşturuldu. Ardından görünür
particle'ları olan Speed effect vanilla komutuyla hedefe uygulandı:

```text
cargo run --locked --offline --example world_entity_effect_particles_probe -- 127.0.0.1:25568 P08ParticlesL1
summon minecraft:iron_golem 67 90 -36 {NoAI:1b,NoGravity:1b,Health:75.0f}
effect give @e[type=minecraft:iron_golem,x=67,y=90,z=-36,distance=..2,limit=1] minecraft:speed 30 0 false
```

Vanilla sunucusu iki komutu kabul etti; probe hedefe gelen metadata packet'ini
uygulayıp başarıyla tamamlandı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `160` |
| `entity_effect_particle_count` | `1` |
| `entity_effect_particle_type_ids` | `[28]` |
| `entity_effect_ambient` | `false` |
| `entity_type_id` | `70` |
| `entity_position` | `(67.5, 90.0, -35.5)` |
| `entity_metadata_length` | `9` |
| `world_entity_count` | `54` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `11145` |

`entity_metadata_length=9`, list count `1`, `ENTITY_EFFECT` registry ID `28`,
ARGB INT payload'ı ve `255` EOF alanlarının tüketildiğini doğrular. Bu sonuç
Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Diğer particle type'ları ve option codec'leri, ARGB renginin state'e alınması,
birden fazla option listesi, particle sıklığı/seed/renderer, effect lifecycle
ve `client-parity` karşılaştırması açık kalır.
