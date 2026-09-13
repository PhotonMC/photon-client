# P08.z — [server-conformance] Entity shared flags semantic state

Bu kayıt, `ClientboundSetEntityDataPacket` içindeki Java entity ortak flags
byte'ının Rust `ClientWorld` state'ine semantik olarak uygulanmasını kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:255-262,313`, bütün entity'lerin `DATA_SHARED_FLAGS_ID` alanını
`BYTE` serializer'ıyla ID `0` olarak tanımladığını ve varsayılan değerin sıfır
olduğunu gösterir. `Entity.java:2787-2797`, ortak byte içindeki tek bitin
`1 << flag` ile okunup yazıldığını belirtir. Java 26.2'de kullanılan bitler
şunlardır:

| Bit | Java anlamı |
| ---: | --- |
| 0 | on fire |
| 1 | shift key down |
| 3 | sprinting |
| 4 | swimming |
| 5 | invisible |
| 6 | glowing |
| 7 | fall flying |

`SynchedEntityData.java:74-85`, dirty değerleri ID dizisinin artan sırasıyla
paketlediği için ortak flags kaydı mevcutsa metadata gövdesinin ilk kaydıdır.
`SynchedEntityData.java:108-125` gelen kayıtların yalnızca listelenen alanları
güncellediğini, `ClientboundSetEntityDataPacket.java:21-37` ise kayıt biçimini
ID byte'ı, serializer VarInt'i, değer ve `255` sonlandırıcısı olarak verir.
`LivingEntity.java:891-916`, görünmezlik ve parlamanın effect durumundan ortak
flags alanına yazıldığı sunucu dalını gösterir.

## Sözleşme

- Metadata gövdesi ID `0` ve `BYTE` serializer ID `0` ile başlayan geçerli bir
  kayıt içeriyorsa üçüncü byte ortak flags state'idir.
- Ortak flags değeri mevcut entity ve local player kopyasında sorgulanabilir;
  yedi Java bitinin her biri named query ile okunur.
- ID `0` içermeyen sonraki metadata paketi ortak flags değerini değiştirmez;
  Java'nın yalnızca gönderilen `DataValue` alanını güncelleme davranışı korunur.
- Yeni entity ve `dataToKeep` bit 2 olmadan yeniden kurulan local player ortak
  flags'i sıfırdan başlatır. Bit 2 ile yeniden kurulum mevcut entity data ile
  birlikte ortak flags'i korur.
- Bilinmeyen entity, yanlış serializer veya eksik metadata değeri ortak flags
  state'ini değiştirmez; raw metadata kapsamı P08.e-entity-metadata kaydında
  tutulur.

## Rust doğrulaması

`shared_flags_from_metadata`, Java'nın ID sırasını kullanarak ortak BYTE
alanını çözer. `ClientEntity` ve `ClientPlayerState` ortak flags byte'ını ve
on-fire, shift, sprint, swimming, invisible, glowing ve fall-flying sorgularını
sağlar. World unit testi bütün tanımlı bitleri, belirtilmeyen metadata alanının
önceki değeri korumasını ve sıfırlama kaydını kontrol eder; local player
respawn testi bit 2 koruma maskesini aynı state ile doğrular.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Bağlı local player'a vanilla effect komutları uygulandı:

```text
cargo run --locked --offline --example world_entity_shared_flags_probe -- 127.0.0.1:25568 P08FlagsL2
effect give P08FlagsL2 minecraft:invisibility 30 0 true
effect give P08FlagsL2 minecraft:glowing 30 0 true
```

Sunucu iki effect'i kabul etti. Java bit sözleşmesine göre beklenen değer
`(1 << 5) | (1 << 6) = 96` idi. Probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `350` |
| `flagged_target` | `local_player` |
| `flagged_entity_type_id` | `player` |
| `shared_flags` | `96` |
| `on_fire` | `false` |
| `shift_key_down` | `false` |
| `sprinting` | `false` |
| `swimming` | `false` |
| `invisible` | `true` |
| `glowing` | `true` |
| `fall_flying` | `false` |
| `world_entity_count` | `42` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `10271` |

Bu sonuç vanilla sunucusunun effect kaynaklı metadata akışını ve Rust world
kopyasındaki shared flags sorgusunu gösterir; Java istemcisiyle parity iddiası
değildir.

## Açık kapsam

Diğer metadata serializer'larının semantic decode edilmesi, pose/custom name/
equipment alanları, entity türüne göre metadata ID doğrulaması,
`onSyncedDataUpdated` yan etkileri, living entity effect state'inin tam istemci
kopyası, render davranışı ve `client-parity` karşılaştırması açık kalır. Shared
flags'in entity-map yolu unit test ile kapsanır; canlı koşu local player
metadata yolu üzerindedir.
