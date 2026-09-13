# P08.f — [server-conformance] ClientWorld dimension ve respawn geçişi

Bu kayıt, vanilla sunucunun dimension değişiminde gönderdiği respawn paketinin
client world kopyasına uygulanmasını kapsar. Canlı kanıt sınıfı
`server-conformance`'dır: Rust bağlantısı Mojang vanilla 26.2 dedicated
server'ına bağlanır, sunucu konsolundan kontrollü dimension değişimi yapılır ve
gelen dünya state'i sorgulanır. Java istemcisiyle aynı girdide karşılaştırma
yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundLoginPacket.java:15-78`, ilk login packet'inin player ID, izin
verilen level listesi, view/simulation mesafeleri ve `CommonPlayerSpawnInfo`
alanlarını taşır. `CommonPlayerSpawnInfo.java:15-53`, dimension type registry
ID'si, dimension resource key'i, seed, oyun modları, debug/flat bayrakları,
nullable last-death global position, portal cooldown ve sea level sırasını
tanımlar.

`ClientboundRespawnPacket.java:8-38`, aynı ortak spawn alanlarının arkasından
`dataToKeep` byte'ını okur. Maskenin `1` biti attribute'ların, `2` biti entity
verisinin korunmasını ister; `3` ikisini birlikte belirtir.

`ClientPacketListener.java:1247-1334`, respawn geldiğinde dimension resource
key'ini önceki key ile karşılaştırır. Key değişirse yeni `ClientLevel` oluşturur,
eski chunk/entity depolarını yeni level'a taşımaz ve yeni oyuncuyu bu level'a
bağlar. Key aynıysa mevcut level korunur; oyuncu yine yeniden oluşturulur.

`DimensionTypes.java:45-113`, vanilla built-in dimension yüksekliklerini
tanımlar: Overworld `min_y=-64,height=384`, Nether ve End
`min_y=0,height=256`. `GameProtocols.java` clientbound dispatch sırası ve
`GamePacketTypes.java:83`, respawn packet'inin protokol 776'daki ID'sini
belirler; bundle delimiter ID `0` olduğundan bu packet ID `82`'dir.

## Sözleşme

- Login packet'indeki ortak spawn bilgisi başlangıç `ClientWorld` dimension
  metadata'sına yazılır.
- Respawn packet'inin ortak alanları ve keep maskesi packet sonuna kadar
  tüketilir; nullable last-death global position, resource key ve packed
  BlockPos olarak kaydedilir.
- Dimension key değiştiğinde chunk ve entity kopyası temizlenir, world game
  time sıfırlanır ve vanilla built-in dimension için section bounds değiştirilir.
- Dimension key aynı kaldığında mevcut chunk/entity depolaması korunur; bu,
  Java listener'ın yeni `ClientLevel` yaratmadığı dala karşılık gelir.
- `dataToKeep` kaydedilir. Local player'ın aynı ID ile yeniden kurulması ve bit
  2 entity state'inin taşınması [P08.h kaydında](client-world-player-respawn-conformance.md)
  uygulanmıştır; attribute/menu/inventory state'i ve bit 1'in gerçek değer
  aktarımı hâlâ açık kapsamdır.
- P08.f kapsamındaki built-in key çözümü, configuration registry'den gelen
  `DimensionType` bounds çözümüyle [P08.g kaydında](client-world-dimension-type-bounds-conformance.md)
  genişletilmiştir. P08.f'in tek başına ölçümü bilinmeyen key için varsayılan
  bounds kullanır.

## Rust doğrulaması

`parse_login_packet` ve `parse_respawn_packet`, Java ortak spawn alanlarının
sırasını ve packet sonunu kontrol eder. `ClientWorld` dimension key, dimension
type ID, seed, debug/flat ve sea level metadata'sını saklar. P08.f ölçümünde
built-in Nether ve End için `min_y=0,height=256`, Overworld için
`min_y=-64,height=384` bounds'u seçilir; registry öncelikli çözüm P08.g'de
ayrıca doğrulanır.

Local player ID, generation ve `dataToKeep` bit 2 entity state uygulaması
ayrıca [P08.h kaydında](client-world-player-respawn-conformance.md) doğrulanır.

Unit testler login/respawn ortak codec sırasını, optional global position'ı,
keep maskesini, packet trailing-byte reddini ve dimension değişiminde chunk,
entity ve game-time temizliğini doğrular. Aynı dimension key'de chunk
deposunun korunduğu dal da test edilir.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Probe çalışırken vanilla server konsolunda şu komut çalıştırıldı:

```text
execute as DimNether in minecraft:the_nether run tp @s 0 80 0
```

Rust probe:

```text
cargo run --locked --offline --example world_dimension_respawn_probe -- 127.0.0.1:25568 DimNether
```

Gözlenen sonuç:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| login dimension | `minecraft:overworld` |
| respawn packet count | `1` |
| respawn dimension | `minecraft:the_nether` |
| respawn dimension type ID | `3` |
| `data_to_keep` | `3` |
| dimension changed | `true` |
| cleared chunk count | `157` |
| cleared entity count | `39` |
| final world bounds | `min_y=0,height=256` |
| final world chunk count | `157` |
| final world section count | `2512` |
| total chunk packet count | `314` |
| received packet count | `16652` |

Respawn packet'i eski Overworld chunk/entity kopyası doluyken geldi; parser
packet ID `82` dalına girdi, eski kopyayı temizledi ve sonraki Nether chunk'ları
`0` tabanlı 16 section olarak world state'e yazdı. Bu sonuç wire kabulü ve
state uygulamasını gösterir; Java client eşitliği iddiası değildir.

## Açık kapsam

Datapack custom dimension için gerçek bounds çözümü, `dataToKeep` maskesinin
attribute/menu/inventory state'ine uygulanması, same-dimension ölüm respawn'ı,
portal coordinate scaling, dimension-specific
environment/render state'i ve `client-parity` karşılaştırması açık kalır.
