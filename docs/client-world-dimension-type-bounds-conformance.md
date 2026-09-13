# P08.g — [server-conformance] Registry’den DimensionType bounds çözümü

Bu kayıt, configuration aşamasındaki `minecraft:dimension_type` registry
verisinin login ve respawn sırasında `ClientWorld` section bounds seçimine
bağlanmasını kapsar. Canlı kanıt sınıfı `server-conformance`'dır: Rust probe'u
Mojang vanilla 26.2 dedicated server'ının gönderdiği registry ve respawn
paketlerini tüketir. Java istemcisiyle aynı girdide state karşılaştırması
yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`RegistrySynchronization.java:27-69,91-98`, registry listesini numeric ID
sırasıyla paketler ve içeriği known pack ile karşılanmıyorsa optional NBT olarak
gönderir. `MappedRegistry.java:35-36,115-117,232-234`, ID'nin kayıt sırasından
geldiğini ve `listElements()` akışının aynı `byId` listesini kullandığını
gösterir. `RegistryDataCollector.java:80-82,162-186`, bu kayıtları client'ın
remote registry katmanında toplar.

`DimensionType.java:30-93`, network codec'in `min_y` ve `height` alanlarını
taşıdığını, yüksekliğin en az 16 ve 16'nın katı olması gerektiğini, `min_y`'nin
16'ya hizalandığını ve Y sınırlarını tanımlar. `ClientPacketListener.java:490-517`
ile `1247-1274`, login ve dimension değiştiren respawn sırasında bu registry
`Holder<DimensionType>` değerini `ClientLevel` kurulumuna verir.

## Sözleşme

- `minecraft:dimension_type` registry packet'inin entry listesi korunur; her
  entry'nin list index'i CommonPlayerSpawnInfo içindeki dimension type ID'sine
  karşılık gelir.
- Dolu entry NBT'sinden yalnız Java codec'inin `IntTag` olarak taşıdığı `min_y`
  ve `height` alanları alınır. Java codec/constructor sınırlarını geçmeyen
  kayıtlar `DimensionBounds` olur; eksik veya bozuk dolu kayıt codec hatasıdır.
- Known pack nedeniyle `data` boş olan kayıtlar çözülmüş sayılmaz. Built-in key
  için önceki sabit fallback korunur; registry verisi mevcutsa ID çözümü
  fallback'in önüne geçer.
- Login ve respawn world replacement aynı bounds çözümünü kullanır. Böylece
  chunk section index'i dimension key adına göre tahmin edilmez.

## Rust doğrulaması

`PackedRegistryEntry::dimension_type_bounds`, generic NBT compound içinden
`min_y`/`height` alanlarını ve Java sınırlarını doğrular. `PlayConnectionTrace`
dimension type kayıtlarını list index'iyle saklar; no-known-pack probe modu,
vanilla'nın sunduğu known pack'i seçmeden registry içeriğinin gönderilmesini
sağlar. Login ve respawn parser'ları bu kayıtları `ClientWorld` bounds'una
bağlar.

Unit testler geçerli custom bounds'u, boş known-pack entry'sini ve eksik alanlı
bozuk entry'yi ayırır. Önceki login/respawn ve world replacement testleri de
aynı suite içinde çalıştırılır.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Probe çalışırken vanilla server konsolunda şu komut çalıştırıldı:

```text
execute as <probe-user> in minecraft:the_nether run tp @s 0 80 0
```

Rust probe:

```text
cargo run --locked --offline --example world_dimension_type_bounds_probe -- 127.0.0.1:25568 <probe-user>
```

İki ayrı vanilla oturumunda (`DimTypeProbe`, `DimTypeProbe2`) sabit kalan
sonuçlar:

| Gözlem | Sonuç |
| --- | ---: |
| server'ın sunduğu known pack sayısı | `1` |
| seçilen known pack sayısı | `0` |
| dimension type registry entry sayısı | `4` |
| çözülen bounds entry sayısı | `4` |
| login dimension | `minecraft:overworld` |
| respawn dimension | `minecraft:the_nether` |
| respawn dimension type ID | `3` |
| registry bounds | `min_y=0,height=256` |
| world bounds | `min_y=0,height=256` |
| `data_to_keep` | `3` |
| dimension changed | `true` |

Registry ID `3` için bounds, dimension key fallback'inden değil configuration
registry NBT'sinden çözüldü ve Nether respawn sonrasında `ClientWorld` aynı
section aralığıyla chunk'ları kabul etti. Bu sonuç wire kabulü ve state
uygulamasını gösterir; Java client eşitliği iddiası değildir.

## Açık kapsam

Vanilla built-in kayıtlarla registry çözümü doğrulandı. Datapack ile yeni bir
dimension key/type oluşturulan canlı senaryo, dimension type'ın diğer
environment alanları, attribute/menu/inventory respawn state'i,
`dataToKeep` bit 1 etkisi ve `client-parity` karşılaştırması açık kalır.
Local player identity ve bit 2 entity state uygulaması [P08.h kaydında](client-world-player-respawn-conformance.md),
same-dimension ölüm respawn'ı ve `dataToKeep=0` akışı [P08.i kaydında](client-world-player-death-respawn-conformance.md)
ele alınmıştır.
