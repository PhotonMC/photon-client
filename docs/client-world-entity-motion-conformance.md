# P08.e-entity-motion — [server-conformance] ClientWorld entity motion state

Bu kayıt, vanilla sunucusundan gelen `AddEntity` içindeki başlangıç hareketi ile
`ClientboundSetEntityMotionPacket` delta movement alanının parse edilip
`ClientWorld` global entity map'indeki motion state'ine uygulanmasını kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundAddEntityPacket.java:17-31,95-120`, AddEntity gövdesinde entity
ID/UUID/type ve konumdan sonra `Vec3.LP_STREAM_CODEC` ile sıkıştırılmış
başlangıç hareketini taşır. `LpVec3.java:6-76`, üç bileşeni ortak ölçek, 15 bit
koordinat ve gerektiğinde VarInt continuation alanıyla kodlar.

`ClientboundSetEntityMotionPacket.java:11-30`, entity ID VarInt'i ile aynı
`LpVec3` vektörünü taşır. `GameProtocols.java:234` bu packet'i clientbound ID
`101` olarak kaydeder. `ClientPacketListener.java:621-627`, mevcut entity'yi
bulduğunda `lerpMotion` çağırır; `Entity.java:2644` bu yolu
`setDeltaMovement`'a yönlendirir.

## Sözleşme

- AddEntity'nin packed hareket vektörü Java `LpVec3` alan sırasıyla çözülür ve
  yeni entity'nin başlangıç motion state'i olarak saklanır.
- Set entity motion packet'i entity ID'sinden sonra packed hareket vektörünü
  tüketir; mevcut entity varsa üç f64 bileşeni world motion state'inin yerine
  yazar.
- Bilinmeyen entity ID'si için packet observation tutulur fakat application
  sonucu false olur.
- Bu dilim raw motion state'ini sorgulanabilir kılar. Type `120` snowball için
  loaded air-cell free-flight tick'i [P08.aq kaydında](client-world-entity-snowball-tick-conformance.md)
  ayrıca ele alınır; genel motion entegrasyonu, yerçekimi, çarpışma,
  interpolation, finite-value filtreleri ve render state'i bu kaydın dışındadır.
  Remote Player position interpolation [P08.ar kaydında](client-world-entity-player-interpolation-conformance.md)
  ayrı ve dar bir dal olarak tutulur.

## Rust doğrulaması

`parse_add_entity_packet`, AddEntity'nin başlangıç `LpVec3` alanını artık
atlamaz; `binary::read_lp_vec3` ile çözer. `parse_set_entity_motion_packet`
ID `101` gövdesini aynı codec ile tüketir ve trailing byte'ları reddeder.
`ClientWorld`, başlangıç movement değerini entity'ye yazar; SetEntityMotion
observation'ı mevcut entity'nin `motion()` sorgusunu günceller.

Unit testleri packet ID'sini, sıfır packed vektörün alan sırasını, AddEntity
başlangıç motion değerini, packet motion replacement'ını ve bilinmeyen ID
sonucunu kontrol eder. `world_entity_motion_probe`, canlı vanilla oturumunda
uygulanan packet motion ile world motion değerini eşleştirir.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Bağlantı açıkken server console'dan başlangıç hareketi olan bir arrow
oluşturuldu:

```text
cargo run --locked --offline --example world_entity_motion_probe -- 127.0.0.1:25568 EntityMotion
# vanilla server console, while the probe is connected
summon minecraft:arrow 25 81 -36 {Motion:[1.0d,0.0d,0.0d]}
```

Vanilla konsolu `Summoned new Arrow` bildirdi. Rust probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `add_entity_packet_count` | `59` |
| `add_entity_initial_motion_count` | `31` |
| `entity_motion_packet_count` | `2156` |
| `entity_motion_applied_count` | `2156` |
| ilk başlangıç motion entity | `id=888`, `(0.009522065555759074,-0.047060977842885876,-0.11505829213208818)` |
| world başlangıç motion | aynı üç f64 bileşen |
| ilk SET_ENTITY_MOTION entity | `id=874`, `(0,-0.0050051883049502255,0.0037233717878288886)` |
| world SET_ENTITY_MOTION | aynı üç f64 bileşen |
| `world_entity_count` | `46` |
| `world_chunk_count / world_section_count` | `157 / 3768` |
| `received_packet_count` | `9258` |

Probe, 31 non-zero AddEntity başlangıç motion kaydından birini ve canlı
sunucudan gelen 2.156 `SET_ENTITY_MOTION` gözleminin tamamını world map'ine
uyguladı. Her iki örnekte packet observation ile `ClientEntity::motion()`
aynı değerleri verdi. Sonuç vanilla raw entity motion akışının Rust world
kopyasına taşındığını gösterir; Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Genel motion'in tick entegrasyonu, yerçekimi ve collision, generic network
interpolation, rotation interpolation/head-body semantic state'i, on-ground,
passenger ilişkisi, metadata/equipment, remove/lifecycle, chunk unload temizliği,
dimension/respawn, render state'i ve oynanabilir istemci bağlantısı açık kalır.
Snowball free-flight alt dalı [P08.aq](client-world-entity-snowball-tick-conformance.md)
ile, remote Player position interpolation [P08.ar](client-world-entity-player-interpolation-conformance.md)
ile sınırlı olarak ilerletilmiştir. Bu motion state'i
`server-conformance` ilerlemesidir; P08'in `client-parity` kapanışı değildir.
