# P08.q — [server-conformance] Custom dimension environment alanları

Bu kayıt, custom dimension type'ın istemci world kopyasında kullanılan
environment alanlarını kapsar. P08.o yalnız key/type registry ve dikey bounds'u
doğruluyordu; bu dilim ayrı bir Java dalı olan environment değerlerinin NBT'den
çözülüp world state'ine uygulanmasını ölçer. Canlı kanıt sınıfı
`server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated server'ına
bağlanır. Aynı girdide Java istemcisiyle karşılaştırma yapılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`DimensionType.java:30-45,82-103`, dimension type record'ının fixed-time,
skylight, ceiling, ender-dragon-fight, coordinate scale, vertical bounds,
logical height, ambient light, skybox ve cardinal-light alanlarını tanımlar.
`has_fixed_time`, `skybox` ve `cardinal_light` optional codec alanları için Java
default'ları sırasıyla `false`, `overworld` ve `default`'tur.

`CardinalLighting.java:22-44`, `default` ve `nether` preset'lerini çözer.
`ClientLevel.java:235-259,1003-1006`, dimension type'ı yeni client level'a
bağlar ve cardinal lighting'i oradan seçer. `ClientChunkCache.java:38-42`
skylight bayrağını light engine kurulmasına verir. `SkyRenderer.java:266-280`
skybox seçimini, `Lightmap.java:92-96` ambient light değerini kullanır.

## Sözleşme

- Dimension type registry entry'si `min_y` ve `height` yanında client tarafından
  tüketilen environment alanlarını taşır. Required alanların NBT türü korunur:
  boolean'lar byte, coordinate scale double, logical height int ve ambient
  light float'tır.
- `has_fixed_time`, `has_skylight`, `has_ceiling` ve
  `has_ender_dragon_fight` boolean değerleri; coordinate scale, logical height
  ve ambient light sayısal değerleriyle birlikte sorgulanabilir state'e yazılır.
- Optional `skybox` ve `cardinal_light` alanları yoksa Java default'ları
  kullanılır; tanınmayan enum değeri veya geçersiz range kaydı reddedilir.
- Respawn sonrasında `ClientWorld::dimension_environment`, registry'den
  çözülen aynı environment değerlerini döndürür. Dimension type ID `2` olan
  `p08:test` entry'si bu eşlemenin canlı senaryosudur.

Bu dilim monster spawn provider'ını, infiniburn holder set'ini, environment
attribute/timeline/default clock registry'lerini veya custom generator'ı
semantic olarak yürütmez. Bunlar sonraki kapsam olarak açık bırakılır.

## Rust doğrulaması

`PackedRegistryEntry::dimension_type_environment`, DimensionType NBT'sindeki
client environment alanlarını çözer. `PlayConnectionTrace`, dimension type
registry sırasındaki environment entry'lerini bounds ile birlikte tutar.
`ClientWorld::replace_dimension_with_environment`, respawn key/type seçimini
registry environment'ı ile world kopyasına uygular; built-in key'ler için Java
baseline değerlerine dayalı fallback bulunur.

`examples/world_custom_dimension_environment_probe.rs`, mevcut no-known-pack
custom dimension trigger'ını yeniden kullanır ve yeni bir `ProbeAction`
varyantı açmaz. Registry entry'si ile final world state'ini alan alan
karşılaştırır.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

P08.o'da kullanılan sentetik `p08_custom_dimension` datapack'i geçici olarak
değiştirilerek `p08:test` type'ına şu alanlar verildi:

```text
has_fixed_time=true
has_skylight=false
has_ceiling=true
has_ender_dragon_fight=true
coordinate_scale=2.5
min_y=-32
height=192
logical_height=128
ambient_light=0.35
skybox=end
cardinal_light=nether
```

Datapack `/tmp/voxel-vanilla-26.2-world/world/datapacks` altında kaldı;
repository'ye eklenmedi ve orijinal asset taşımıyor. Her probe login olduktan
sonra şu server console komutuyla respawn tetiklendi:

```text
execute in p08:test run tp <probe-user> 0 80 0
```

Probe komutu:

```text
cargo run --locked --offline --example world_custom_dimension_environment_probe -- \
  127.0.0.1:25568 <probe-user>
```

İki bağımsız vanilla oturumunda aynı registry/world state'i gözlendi:

| Gözlem | `P08qA` | `P08qB` |
| --- | ---: | ---: |
| bağlantı | `ok` | `ok` |
| respawn dimension | `p08:test` | `p08:test` |
| respawn dimension type ID | `2` | `2` |
| registry entry/resolved environment sayısı | `5 / 5` | `5 / 5` |
| `has_fixed_time` | `true` | `true` |
| `has_skylight` | `false` | `false` |
| `has_ceiling` | `true` | `true` |
| `has_ender_dragon_fight` | `true` | `true` |
| `coordinate_scale` | `2.5` | `2.5` |
| `logical_height` | `128` | `128` |
| `ambient_light` | `0.35` | `0.35` |
| `skybox` | `end` | `end` |
| `cardinal_light` | `nether` | `nether` |
| final world dimension | `p08:test` | `p08:test` |
| final chunk/section sayısı | `157 / 1884` | `157 / 1884` |

Her iki koşu da registry entry'si ile final `ClientWorld` environment state'ini
alan alan eşleştirdi. Toplam packet sayısı `P08qA` için `9839`, `P08qB` için
`6406` oldu; bu oturum zamanlamasına bağlı olduğu için kabul ölçütü değildir.

Bu sonuç vanilla sunucusunun custom dimension environment taşıyan registry
state'ini Rust'ın kabul ettiğini ve sorgulanabilir world state'ine yazdığını
gösterir. Java client eşitliği, renderer çıktısı, light propagation veya
environment gameplay etkileri kanıtı değildir.

## Açık kapsam

Monster spawn light provider'ı ve block-light limiti, infiniburn tag semantiği,
environment attribute map'i, timeline/default clock, portal coordinate-scale
uygulaması, skybox/cardinal lighting renderer etkileri, custom generator ve
`client-parity` karşılaştırması açık kalır.
