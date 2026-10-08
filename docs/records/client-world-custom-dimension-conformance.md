# P08.o — [server-conformance] Custom dimension key/type ve registry bounds

Bu dosya, datapack ile eklenen bir dimension key ve dimension type kaydının
configuration registry'den alınarak respawn sonrasında sorgulanabilir
`ClientWorld` state'ine uygulanmasını kapsar. Canlı kanıt sınıfı
`server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated
server'ına bağlanır ve server'ın gönderdiği custom dimension state'ini
tüketir. Aynı girdide Java istemcisiyle karşılaştırma yapılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`DimensionType.java:29-107`, dimension type'ın environment alanlarını,
registry codec'lerini ve `min_y`/`height`/`logical_height` sınırlarını tanımlar.
`LevelStem.java:11-18`, bir dimension key'ini dimension type holder ve chunk
generator ile eşler. `WorldDimensions.java:37-57`, built-in level stem'lerden
sonra custom key'lerin registry'de tutulduğunu gösterir.

`CommonPlayerSpawnInfo.java:15-53` dimension type holder ID'sini, dimension
key'ini, seed'i ve diğer spawn alanlarını taşır. `ClientboundRespawnPacket.java:8-35`
bu ortak bilgiyi `dataToKeep` maskesiyle paketler. `ClientPacketListener.java:1247-1337`
dimension key'i değiştiğinde yeni client level oluşturur, eski chunk/entity
depolarını bırakır ve registry'den çözülen dimension type'ı yeni level'a verir.

## Sözleşme

- Configuration aşamasındaki `minecraft:dimension_type` registry listesi
  numeric ID sırasını korur. Respawn packet'inin type ID `2` entry'si custom
  type'ın `min_y=-32` ve `height=192` bounds'unu verir.
- `p08:test` dimension key'i, built-in fallback kullanmadan respawn spawn
  bilgisinde taşınır ve key değişimi `ClientWorld` dimension replacement'ını
  tetikler.
- World replacement sonrasında Rust world key'i `p08:test`, dimension type ID'si
  `2` ve bounds'u `min_y=-32,height=192` olur. Bu bounds 12 section'lık bir
  custom world aralığıdır.
- Vanilla'nın custom dimension'da gönderdiği chunk'lar world kopyasına
  uygulanır; kontrollü senaryoda 157 chunk column ve 1884 section sorgulanır.
- Known pack seçimi boş tutulduğu için dimension type'ın network NBT'si
  configuration registry'den çözülür. Known pack sayısı ve seçilen liste ayrıca
  kaydedilir.

## Rust doğrulaması

Mevcut `connect_and_world_dimension_type_bounds_probe` yapılandırması known
pack seçimini kapatır, configuration registry bounds listesini tutar ve
externally triggered respawn'ı `ClientWorld::replace_dimension` yoluna bağlar.
Bu dilim yeni bir `ProbeAction` varyantı açmaz. Yeni
`examples/world_custom_dimension_probe.rs` mevcut yolu custom key, type ID,
registry entry/resolution sayısı, bounds ve final chunk/section state'i için
vanilla baseline'a karşı doğrular.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Senaryoda yalnızca bu test için `/tmp/photon-vanilla-26.2-world/world/datapacks`
altında tutulan sentetik `p08_custom_dimension` datapack'i kullanıldı. Pack
format `107.1`'dir. `p08:test` dimension type'ı `has_skylight=true`,
`coordinate_scale=1.0`, `min_y=-32`, `height=192`, `logical_height=192` ve
diğer gerekli environment alanlarıyla; dimension ise plains biome'lu flat
generator ile tanımlandı. Datapack repository'ye eklenmedi ve orijinal asset
taşımıyor.

Dimension registry world yüklenmeden önce okunabildiği için server datapack
kurulumundan sonra yeniden başlatıldı; yalnızca `/reload` custom dimension'ı
kaydetmedi. Her probe login olduktan sonra vanilla server konsolundan şu
komutla externally trigger edildi:

```text
execute in p08:test run tp <probe-user> 0 80 0
```

Probe komutu:

```text
cargo run --locked --offline --example world_custom_dimension_probe -- \
  127.0.0.1:25568 <probe-user>
```

Üç bağımsız geçerli vanilla oturumunda gözlenen sonuçlar:

| Gözlem | `P08oA` | `P08oB` | `P08oD` |
| --- | ---: | ---: | ---: |
| known pack sayısı | `1` | `1` | `1` |
| seçilen known pack sayısı | `0` | `0` | `0` |
| dimension type registry entry sayısı | `5` | `5` | `5` |
| çözülen bounds entry sayısı | `5` | `5` | `5` |
| login dimension | `minecraft:overworld` | `minecraft:overworld` | `minecraft:overworld` |
| respawn dimension | `p08:test` | `p08:test` | `p08:test` |
| respawn dimension type ID | `2` | `2` | `2` |
| registry/world bounds | `min_y=-32,height=192` | `min_y=-32,height=192` | `min_y=-32,height=192` |
| `data_to_keep` | `3` | `3` | `3` |
| dimension changed | `true` | `true` | `true` |
| cleared chunk/entity count | `157 / 65` | `157 / 55` | `157 / 46` |
| final world chunk/section count | `157 / 1884` | `157 / 1884` | `157 / 1884` |
| chunk packet count | `314` | `314` | `314` |

Her üç koşu custom key/type çözümünü, dimension replacement'ı ve custom
section bounds'lu final world state'ini geçti. Entity clear ve alınan packet
toplamı oturum zamanlamasına bağlı gözlemlerdir; probe'un kabul koşulu olan
custom key, registry type, bounds ve world chunk/section state'i üç koşuda da
aynı kaldı.

Bu sonuç Rust paket tüketimini ve vanilla sunucusunun custom dimension state'ini
kabul ettiğini gösterir. Java client eşitliği veya custom generator semantiği
kanıtı değildir.

## Açık kapsam

Dimension type'ın skybox, cardinal lighting, environment attributes, timelines,
clock ve portal scaling etkileri; custom generator semantiği; custom dimension
içindeki oyun davranışları ve `client-parity` karşılaştırması açık kalır.
