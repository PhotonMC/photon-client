# P08.aj — [server-conformance] Entity mob flags semantic state

Bu kayıt, `Mob` sınıfının entity metadata içindeki mob flags BYTE alanının Rust
`ClientWorld` state'ine semantik olarak uygulanmasını kapsar. Kanıt sınıfı
`server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2 dedicated
server'a bağlanır. Java istemcisiyle aynı girdide state karşılaştırılmadığı için
bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Mob.java:99-103`, `Mob` sınıfına living entity alanlarından sonra ID `15` BYTE
alanını ekler ve üç maskeyi tanımlar: `0x01` NoAI, `0x02` left-handed ve `0x04`
aggressive. `Mob.java:269-272`, alanı `(byte)0` ile başlatır.

`Mob.java:1312-1340`, `isEffectiveAi()` sonucunu NoAI bitiyle sınırlar; üç bitin
set/clear işlemlerini ve `isNoAi()`, `isLeftHanded()`, `isAggressive()`
getter'larını tanımlar. `Mob.java:1346-1348`, left-handed bitinin main-arm
seçimine aktarıldığını gösterir. `Mob.java:382-390`, NoAI ve LeftHanded
durumlarının entity NBT'sinden okunup synced state'e yazıldığı server dalıdır.

`SynchedEntityData.java:74-85,108-125` yalnız gönderilen dirty alanları
uygular; `ClientboundSetEntityDataPacket.java:21-37` ID, serializer ve değeri
taşır. BYTE serializer ID `0`'dır (`EntityDataSerializers.java:53-54,164-172`).
`Mob` player sınıfının üst sınıfı değildir; bu alan local player için
uygulanmaz.

## Sözleşme

- Geçerli metadata içinde ID `15` ve BYTE serializer ID `0` varsa değer ham
  `u8` olarak saklanır ve `ClientEntity::mob_flags` ile sorgulanır.
- `0x01` `is_no_ai()`, `0x02` `is_left_handed()` ve `0x04`
  `is_aggressive()` sonuçlarını üretir. Diğer bitler korunur fakat bu dilimde
  semantik anlamlandırılmaz.
- Yeni takip edilen entity mob flags değerini Java default'u olan `0` ile
  başlatır. ID `15` içermeyen sonraki metadata packet'i önceki değeri korur;
  yanlış serializer veya eksik BYTE değeri state'i değiştirmez.
- NoAI'nin gerçek AI/tick yürütmesi, left-handed main-arm item kullanımı,
  aggressive combat davranışı ve renderer/animation yan etkileri bu state taşıma
  diliminin dışındadır.

## Rust doğrulaması

`mob_flags_from_metadata`, ID `15` BYTE alanını ortak metadata parser'ından alır.
`ClientEntity` ham byte'ı saklar ve üç public getter Java maskelerini aynı bit
konumlarıyla sorgulatır. Local player state'ine mob flags alanı eklenmez; Java
kalıtım sınırı korunur.

World unit testi default `0` değerini, üç bitin birlikte ve NoAI tekil maskesini,
başka bir metadata alanı geldiğinde değer korunmasını ve yanlış serializer'ın
reddedilmesini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken konumu sabit bir iron golem oluşturuldu. Entity ilk
oluşturulurken NoAI `true` idi; ardından aynı entity üzerinde NoAI `false` ve
`true` NBT güncellemeleri uygulanarak server'ın ID `15` dirty BYTE metadata
paketi üretmesi sağlandı:

```text
cargo run --locked --offline --example world_entity_mob_flags_probe -- 127.0.0.1:25568 P08AjL1
summon minecraft:iron_golem 49 90 -36 {Health:75.0f,NoAI:1b,NoGravity:1b}
execute in minecraft:overworld positioned 49 90 -36 run data merge entity @e[type=minecraft:iron_golem,distance=..2,limit=1] {NoAI:0b}
execute in minecraft:overworld positioned 49 90 -36 run data merge entity @e[type=minecraft:iron_golem,distance=..2,limit=1] {NoAI:1b}
```

Vanilla sunucusu üç komutu da kabul etti. Probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `512` |
| `entity_mob_flags` | `1` |
| `is_no_ai` | `true` |
| `is_left_handed` | `false` |
| `is_aggressive` | `false` |
| `entity_type_id` | `70` |
| `entity_position` | `(49.5, 90.0, -35.5)` |
| `entity_metadata_length` | `13` |
| `world_entity_count` | `68` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `14929` |

Vanilla iron golem metadata akışındaki ID `15` BYTE state'i Rust
`ClientWorld` map'inde `1` olarak sorgulandı. Pozitif left-handed ve
aggressive maskeleri unit testlerde sentetik metadata ile doğrulandı; canlı
server koşusu NoAI dirty update'ini doğrular. Bu sonuç Java istemcisiyle parity
iddiası değildir.

## Açık kapsam

Gerçek mob AI/tick yürütmesi, main-arm item/hand state'i, combat/aggressive
animasyonları, mob equipment metadata'sı, tür özel mob alanları ve
`client-parity` karşılaştırması açık kalır.
