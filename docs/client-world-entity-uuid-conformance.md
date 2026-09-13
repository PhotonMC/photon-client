# P08.e-entity-uuid — [server-conformance] ClientWorld entity UUID state

Bu kayıt, vanilla sunucusundan gelen `ClientboundAddEntityPacket` içindeki UUID
alanının parse edilip `ClientWorld` global entity map'indeki entity state'ine
uygulanmasını kapsar. Kanıt sınıfı `server-conformance`'dır: Rust packet akışı
Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle aynı girdide
state karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundAddEntityPacket.java:17-31,95-120,132-155`, entity ID VarInt'inden
hemen sonra 16 byte UUID'yi iki big-endian long olarak okur; ardından entity type,
position, compressed movement, rotation ve data alanları gelir.

`FriendlyByteBuf.java:493-511`, UUID yazma/okuma sırasını most-significant ve
least-significant long olarak tanımlar. `Entity.java:3859-3869`, client add
akışında packet UUID'sini `setUUID` ile oluşturulan entity'ye uygular.
`ClientLevel.java:522-537`, entity'yi ID ile global depoya ekler ve aynı ID'nin
önceki kaydını kaldırır.

## Sözleşme

- `ADD_ENTITY` parser'ı entity ID'den sonra UUID'nin iki 64-bit yarısını Java
  byte sırasıyla tüketir ve kalan alanları aynı sırada okur.
- `ClientWorld` entity kaydı UUID'yi packet'ten gelen ham iki long olarak
  sorgulanabilir state'te saklar.
- Aynı entity ID yeniden eklendiğinde yeni UUID, position/type/data ile birlikte
  önceki kaydın yerine geçer.
- Bilinmeyen entity type veya UUID'nin semantic player/profile kullanımı bu
  raw state diliminin dışındadır; UUID başka paketlerde ayrıca kullanılmaz.

## Rust doğrulaması

`parse_add_entity_packet`, daha önce yalnızca sınırını atladığı UUID alanını
`binary::read_uuid` ile ayrıştırır. `EntityInput` UUID'yi `ClientEntity` içine
yazar; `ClientEntity::uuid` world sorgusundan okunabilir. Parser unit testi
UUID alanının Java add gövdesindeki yerini, world unit testi ise aynı ID ile
değiştirilen UUID'nin yeni kayda geçtiğini kontrol eder. Canlı add probe'u packet
UUID'si ile world UUID'sinin eşleşmesini zorunlu tutar.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Bağlantı açıkken oyuncunun yakınında armor stand oluşturuldu. Probe terminalinde
`cargo run` çalışırken vanilla server konsolundan `summon` komutu gönderildi:

```text
cargo run --locked --offline --example world_entity_add_probe -- 127.0.0.1:25568 EntityUuid
# vanilla server console, while the probe is connected
summon minecraft:armor_stand 25 81 -36
```

Rust probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `add_entity_packet_count` | `51` |
| `add_entity_applied_count` | `51` |
| ilk eşleşen entity ID | `775` |
| ilk eşleşen UUID | `most=5098573560207722201`, `least=10913791725331003435` |
| ilk eşleşen type/position/data | `type_id=10`, `(79.57099220824716,18.100000023841858,-4.25)`, `0` |
| world UUID/state | `Uuid { most: 5098573560207722201, least: 10913791725331003435 }`, aynı position/type/data |
| `world_entity_count` | `50` |
| `world_chunk_count / world_section_count` | `157 / 3768` |
| `received_packet_count` | `13750` |

Probe, uygulanan add gözlemlerinden packet UUID'si ile aynı entity ID'li world
kaydının UUID'si eşleşen bir kaydı seçerek başarı verdi. Paket sayısının world
entity sayısından yüksek olması ID tabanlı replacement akışının gözlendiğini
gösterir. Sonuç vanilla add entity UUID alanının Rust world kopyasına taşındığını
gösterir; Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Entity type registry'sinin semantic çözümü, player profile/seen-player map'i,
metadata/equipment/status, remove/lifecycle, chunk unload temizliği,
rotation/head rotation, motion tick integration/interpolation, passenger ilişkisi,
entity tick/fizik, dimension/respawn, render state'i ve oynanabilir istemci
bağlantısı açık kalır. Bu UUID state'i `server-conformance` ilerlemesidir;
P08'in `client-parity` kapanışı değildir.
