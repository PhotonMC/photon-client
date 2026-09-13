# P08.e-entity-position — [server-conformance] ClientWorld entity position synchronization

Bu kayıt, vanilla sunucusundan gelen entity position sync, relative move ve
entity teleport packet'lerinin `ClientWorld` global entity map'indeki konum
state'ine taşınmasını kapsar. Kanıt sınıfı `server-conformance`'dır: Rust
packet akışı Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle
aynı girdide state karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundEntityPositionSyncPacket.java:10-35`, entity ID VarInt'inden sonra
`PositionMoveRotation` içindeki üç konum double'ı, üç delta movement double'ı,
iki rotation float'ı ve `onGround` boolean'ını taşır. `GameProtocols.java:168`
bu packet'i clientbound ID `35` olarak kaydeder.

`ClientboundMoveEntityPacket.java:12-187`, position, position+rotation ve
rotation-only dallarını ayırır. Position dalları entity ID, üç signed short
delta ve `onGround` taşır; position+rotation dalı iki rotation byte'ı ekler.
Dispatch sırasındaki ID'ler sırasıyla `53`, `54` ve `56`'dır.
`VecDeltaCodec.java:6-42`, short delta'yı 4096 katlı quantized position base'e
ekleyerek çözer ve sıfır delta'da base'i korur.

`ClientboundTeleportEntityPacket.java:10-42` aynı position/movement/rotation
alanlarını ve `Relative.SET_STREAM_CODEC` ile dört byte'lık relative maskeyi
taşır; packet ID'si `125`'tir. `Relative.java:8-114`, X/Y/Z konum bitlerini
maskenin 0/1/2 bitlerine yerleştirir.

`ClientPacketListener.java:639-727`, position sync ve teleport alanlarını
mevcut entity'ye uygular; move handler da position codec'ini kullanarak delta
konumunu çözer. Java'nın interpolation, rotation, movement ve passenger yan
etkileri entity sınıfının ayrı state'lerine bağlıdır.

## Sözleşme

- Position sync konumu mutlak f64 state olarak okunur ve mevcut entity ID'sine
  uygulanır.
- Relative move packet'lerinde signed short delta, Java'nın `Math.round` ile
  4096 ölçekli quantized base hesabına eklenir. Rotation-only kayıt konumu
  değiştirmez.
- Teleport packet'inde X/Y/Z relative bitleri set ise ilgili değer mevcut
  konuma eklenir; bit set değilse değer mutlak kabul edilir. Bilinmeyen bitler
  konum çözümünü etkilemez.
- World'de bulunmayan entity ID'si için packet observation tutulur fakat
  application sonucu false olur.
- Bu dilim `ClientWorld` içinde konum state'ini günceller. Packet içindeki raw
  rotation alanlarının world state'ine uygulanması [rotation kaydında](client-world-entity-rotation-conformance.md)
  ele alınır. On-ground, generic interpolation base/history, passenger ve render state'leri semantic
  olarak uygulanmaz; raw delta movement state'i [motion kaydında](client-world-entity-motion-conformance.md)
  ele alınır.

## Rust doğrulaması

`parse_entity_position_sync_packet`, `parse_entity_teleport_packet` ve üç move
varyant parser'ı Java alan sırasını tüketir ve trailing byte'ları reddeder.
`ClientWorld`, mutlak/relative konum uygulamasını ve quantized move çözümünü
sorgulanabilir entity position olarak saklar. Her uygulanan observation, probe
sırasında uygulama anındaki konumu kaydeder; sonradan gelen entity tick'leri
ile karıştırılmaz. Player type ID `156` için network interpolation hedefi ve
üç tick sonucu [P08.ar kaydında](client-world-entity-player-interpolation-conformance.md)
ayrıca tutulur.

Unit testleri packet ID'lerini, iki `PositionMoveRotation` alan sırasını, üç
move wire varyantını, absolute/relative konum çözümünü ve rotation-only
konumun korunmasını kontrol eder. Probe, kontrollü armor stand konumunun
uygulama anında world sorgusundan okunmasını zorunlu tutar.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Oyuncunun yakınında sabit bir platform hazırlandı. Probe bağlantısı açıkken
platform üzerinde armor stand oluşturuldu ve başka bir platforma teleport
edildi:

```text
setblock 25 80 -36 minecraft:stone
setblock 27 80 -36 minecraft:stone
cargo run --locked --offline --example world_entity_position_probe -- 127.0.0.1:25568 EntityPosition5
summon minecraft:armor_stand 25 81 -36
tp @e[type=minecraft:armor_stand,limit=1,sort=nearest] 27 81 -36
```

Vanilla konsolu `Summoned new Armor Stand` ve `Teleported Armor Stand to
27.500000, 81.000000, -35.500000` bildirdi. Rust probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_position_sync_packet_count` | `105` |
| `entity_position_sync_applied_count` | `105` |
| `entity_move_packet_count` | `4467` |
| `entity_move_applied_count` | `4467` |
| `entity_teleport_packet_count` | `0` |
| `last_controlled_entity_position_kind` | `position_sync` |
| `applied_entity_position` | `id=737, (27.5,81.0,-35.5)` |
| `world_entity_final_position` | `id=737, (27.5,81.0,-35.5)` |
| `world_entity_count` | `53` |
| `world_chunk_count / world_section_count` | `157 / 3768` |
| `received_packet_count` | `10067` |

Bu oturumda vanilla'nın entity `tp` command yolu `TELEPORT_ENTITY` yerine
`ENTITY_POSITION_SYNC` state güncellemesi üretti; bu nedenle canlı tabloda
teleport packet sayısı sıfırdır. Position sync'in kontrollü hedef konuma
uygulanması ve aynı oturumdaki move paketlerinin tüketilmesi doğrulanmıştır;
`TELEPORT_ENTITY` relative çözümü yalnız parser ve unit test kapsamındadır.
Sonuç vanilla raw entity position akışının Rust world kopyasına taşındığını
gösterir; Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Quantized delta base'inin Java render/interpolation state'iyle ayrıştırılması,
rotation interpolation ve head/body semantic state'i, delta movement integration, on-ground, teleport
relative rotation/delta bitleri, generic living interpolation, passenger/vehicle ilişkisi, entity tick/fizik,
chunk unload temizliği, dimension/respawn, render state'i
ve oynanabilir istemci bağlantısı açık kalır. Bu konum state'i
`server-conformance` ilerlemesidir; P08'in `client-parity` kapanışı değildir.
